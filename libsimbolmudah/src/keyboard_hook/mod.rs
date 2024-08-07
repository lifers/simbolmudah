mod internal;

use crate::{
    bindings,
    utils::{
        delegate_storage::get_token, functions::get_strong_ref, thread_handler::ThreadHandler,
    },
};
use internal::KeyboardHookInternal;
use std::{
    cell::Cell,
    fmt::Debug,
    sync::{mpsc::sync_channel, Arc, OnceLock, RwLock, RwLockWriteGuard},
    usize,
};
use windows::{
    core::{implement, AgileReference, Error, IInspectable, Interface, Result, Weak, HSTRING},
    Foundation::{EventRegistrationToken, TypedEventHandler},
    Win32::{
        Foundation::{E_ACCESSDENIED, E_NOTIMPL, E_POINTER, LPARAM, LRESULT, WPARAM},
        System::{
            LibraryLoader::GetModuleHandleW,
            WinRT::{IActivationFactory, IActivationFactory_Impl},
        },
        UI::{
            Input::KeyboardAndMouse::{
                KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP,
                KEYEVENTF_SCANCODE, VIRTUAL_KEY,
            },
            WindowsAndMessaging::{
                CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HC_ACTION, HHOOK,
                KBDLLHOOKSTRUCT, LLKHF_EXTENDED, LLKHF_INJECTED, LLKHF_UP, WH_KEYBOARD_LL,
                WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
            },
        },
    },
};

static GLOBAL_INSTANCE: RwLock<OnceLock<KeyboardHookInternal>> = RwLock::new(OnceLock::new());

thread_local!(static THREAD_HOOK: Cell<HHOOK> = panic!("Uninitialized thread hook"));

#[derive(Debug, Clone)]
enum Reporter {
    OnStateChanged(
        (
            AgileReference<TypedEventHandler<bindings::KeyboardHook, u8>>,
            i64,
        ),
    ),
    OnKeyEvent(
        (
            AgileReference<TypedEventHandler<bindings::KeyboardHook, HSTRING>>,
            i64,
        ),
    ),
}

#[implement(bindings::KeyboardHook)]
#[derive(Debug)]
struct KeyboardHook {
    thread_controller: Arc<ThreadHandler>,
}

impl KeyboardHook {
    fn activate(
        &self,
        keyboard_translator: Weak<bindings::KeyboardTranslator>,
        parent: Weak<bindings::KeyboardHook>,
    ) -> Result<()> {
        let controller_clone = self.thread_controller.clone();
        let (tx, rx) = sync_channel(16);
        tx.send((keyboard_translator, parent)).unwrap();

        self.thread_controller.try_enqueue_high(move || {
            let (keyboard_translator_ref, parent_ref) = rx.recv().unwrap();
            let hmod = unsafe { GetModuleHandleW(None) }?;
            let h_hook =
                unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_procedure), hmod, 0) }?;
            THREAD_HOOK.set(h_hook);

            let internal = KeyboardHookInternal::new(keyboard_translator_ref, parent_ref);
            let mut write_lock = global_write()?;

            write_lock.set(internal).expect(
                "GLOBAL_INSTANCE has to be empty or emptied by the previous deactivate call",
            );

            let another_clone = controller_clone.clone();
            let event_handler =
                TypedEventHandler::new(move |_, _| another_clone.try_enqueue_high(reset_state));
            let write_lock = write_lock.get_mut().expect("GLOBAL_INSTANCE should be set");

            // Reset the stage when we met an invalid sequence or a complete sequence
            write_lock.on_invalid_token =
                get_strong_ref(&write_lock.keyboard_translator)?.OnInvalid(Some(&event_handler))?;
            write_lock.on_translated_token = get_strong_ref(&write_lock.keyboard_translator)?
                .OnTranslated(Some(&event_handler))?;

            Ok(())
        })
    }

    fn register_reporter(&self, reporter: Reporter) -> Result<()> {
        // making sure this callback runs after the hook is activated
        self.thread_controller.try_enqueue_high(move || {
            let mut write_lock = global_write()?;
            let internal = write_lock.get_mut().expect("GLOBAL_INSTANCE should be set");

            Ok(match &reporter {
                Reporter::OnStateChanged((h, token)) => {
                    internal.state_changed.insert(*token, h.clone())
                }
                Reporter::OnKeyEvent((h, token)) => internal.key_event.insert(*token, h.clone()),
            })
        })
    }

    fn unregister_reporter(&self, reporter: Reporter) -> Result<()> {
        // making sure this callback runs after the reporter is registered
        self.thread_controller.try_enqueue_high(move || {
            let mut write_lock = global_write()?;
            let internal = write_lock.get_mut().expect("GLOBAL_INSTANCE should be set");

            Ok(match reporter {
                Reporter::OnStateChanged((_, token)) => internal.state_changed.remove(token),
                Reporter::OnKeyEvent((_, token)) => internal.key_event.remove(token),
            })
        })?;
        Ok(())
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        self.thread_controller
            .try_enqueue_high(|| {
                let mut write_lock = global_write()?;
                let instance_ref = write_lock.get_mut().expect("GLOBAL_INSTANCE should be set");

                // Unregister the event handlers
                get_strong_ref(&instance_ref.keyboard_translator)
                    .unwrap()
                    .RemoveOnInvalid(instance_ref.on_invalid_token)
                    .unwrap();
                get_strong_ref(&instance_ref.keyboard_translator)
                    .unwrap()
                    .RemoveOnTranslated(instance_ref.on_translated_token)
                    .unwrap();

                unsafe { UnhookWindowsHookEx(THREAD_HOOK.take()) }.expect("Unhooking must succeed");
                let _ = write_lock.take().expect("GLOBAL_INSTANCE should be set");
                Ok(())
            })
            .unwrap();
    }
}

impl bindings::IKeyboardHook_Impl for KeyboardHook_Impl {
    fn ResetStage(&self) -> Result<()> {
        self.thread_controller.try_enqueue_high(reset_state)
    }

    fn OnStateChanged(
        &self,
        handler: Option<&TypedEventHandler<bindings::KeyboardHook, u8>>,
    ) -> Result<EventRegistrationToken> {
        if let Some(handler) = handler {
            let token = get_token(handler.as_raw());
            self.register_reporter(Reporter::OnStateChanged((
                AgileReference::new(handler)?,
                token,
            )))?;
            Ok(EventRegistrationToken { Value: token })
        } else {
            Err(Error::new(E_POINTER, "Null OnStateChanged handle pointer"))
        }
    }

    fn RemoveOnStateChanged(&self, token: &EventRegistrationToken) -> Result<()> {
        self.unregister_reporter(Reporter::OnStateChanged((
            AgileReference::new(&TypedEventHandler::new(|_, _| Err(E_NOTIMPL.into())))?,
            token.Value,
        )))
    }

    fn OnKeyEvent(
        &self,
        handler: Option<&TypedEventHandler<bindings::KeyboardHook, HSTRING>>,
    ) -> Result<EventRegistrationToken> {
        if let Some(handler) = handler {
            let token = get_token(handler.as_raw());
            self.register_reporter(Reporter::OnKeyEvent((AgileReference::new(handler)?, token)))?;
            Ok(EventRegistrationToken { Value: token })
        } else {
            Err(Error::new(E_POINTER, "Null OnKeyEvent handle pointer"))
        }
    }

    fn RemoveOnKeyEvent(&self, token: &EventRegistrationToken) -> Result<()> {
        self.unregister_reporter(Reporter::OnKeyEvent((
            AgileReference::new(&TypedEventHandler::new(|_, _| Err(E_NOTIMPL.into())))?,
            token.Value,
        )))
    }
}

/// Stage enum controls how low_level_keyboard_proc behave.
#[derive(Debug, Default, Clone, Copy)]
#[repr(u8)]
enum Stage {
    /// No compose key pressed.
    /// If receives compose keydown, intercept the event and go to stage 1.
    /// Otherwise, ignore the event and stay at state 0.
    /// The compose fail function will return all events to system and set stage to 0 when called.
    #[default]
    Idle = 0,
    /// Compose keydown pressed for the first time.
    /// If receives compose keyup, intercept the event and go to stage 2. Assume consecutive compose
    /// keydown-keyup to be an intentional compose key press by the user.
    /// Otherwise, intercept the event and call compose fail. Assume the user is using the compose
    /// key to do something outside the scope of `simbolmudah`, so we return their inputs in the
    /// correct order.
    ComposeKeydownFirst = 1,
    /// Compose key pressed once, compose mode on.
    /// Whatever happens, intercept the event.
    /// If receives compose keydown, go to stage 3.
    /// Else if receives keydown, send the event to the sequence tree, go to stage 254.
    ComposeKeyupFirst = 2,
    /// Compose keydown pressed for the second time.
    /// Whatever happens, intercept the event.
    /// If receives compose keyup, intercept the event and go to stage 255. Assume consecutive compose
    /// keydown-keyup to be an intentional compose key press by the user.
    /// Otherwise, send the event to the sequence tree. Assume the user is using the compose key to
    /// insert a character for sequence mode. Go to stage 254.
    ComposeKeydownSecond = 3,
    /// Sequence mode.
    /// Intercept and send the event to the sequence tree. The sequence tree will call compose fail
    /// upon failure.
    SequenceMode = 4,
    /// Search mode. Not implemented yet.
    SearchMode = 5,
    /// Unicode mode. If receives a hexadecimal keydown, push the key to the unicode state.
    UnicodeMode = 6,
}

unsafe extern "system" fn keyboard_procedure(
    ncode: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if ncode == HC_ACTION as i32 {
        let kb_hook = lparam.0 as *const KBDLLHOOKSTRUCT;
        let is_key = wparam.0 == WM_KEYDOWN as usize
            || wparam.0 == WM_SYSKEYDOWN as usize
            || wparam.0 == WM_KEYUP as usize
            || wparam.0 == WM_SYSKEYUP as usize;
        let is_injected = (*kb_hook).flags.0 & LLKHF_INJECTED.0 != 0;

        if is_key && !is_injected {
            // Ignore input if we cannot write to the global instance. This means the stage
            // is being reset.
            if let Ok(mut lock) = GLOBAL_INSTANCE.try_write() {
                // Ignore input if the global instance is not set. This means the hook is not
                // activated.
                if let Some(instance) = lock.get_mut() {
                    let input = kbdllhookstruct_to_keybdinput(*kb_hook);
                    instance.report_key_event(input);

                    if instance.process_input(input) {
                        return LRESULT(1);
                    }
                }
            }
        }
    }

    return CallNextHookEx(None, ncode, wparam, lparam);
}

const fn kbdllhookstruct_to_keybdinput(event: KBDLLHOOKSTRUCT) -> KEYBDINPUT {
    let mut flags = KEYEVENTF_SCANCODE.0;
    if event.flags.0 & LLKHF_EXTENDED.0 != 0 {
        flags |= KEYEVENTF_EXTENDEDKEY.0;
    }
    if event.flags.0 & LLKHF_UP.0 != 0 {
        flags |= KEYEVENTF_KEYUP.0;
    }

    KEYBDINPUT {
        wVk: VIRTUAL_KEY(event.vkCode as u16),
        wScan: event.scanCode as u16,
        dwFlags: KEYBD_EVENT_FLAGS(flags),
        time: event.time,
        dwExtraInfo: event.dwExtraInfo,
    }
}

fn global_write() -> Result<RwLockWriteGuard<'static, OnceLock<KeyboardHookInternal>>> {
    GLOBAL_INSTANCE
        .try_write()
        .map_err(|e| Error::new(E_ACCESSDENIED, format!("{:?}", e)))
}

fn reset_state() -> Result<()> {
    let mut lock = global_write()?;
    let internal = lock.get_mut().expect("GLOBAL_INSTANCE should be set");

    internal.stage = Stage::Idle;
    internal.input_buffer.clear();
    internal.report_state();
    Ok(())
}

#[implement(IActivationFactory, bindings::IKeyboardHookFactory)]
pub(super) struct KeyboardHookFactory;

impl IActivationFactory_Impl for KeyboardHookFactory_Impl {
    fn ActivateInstance(&self) -> Result<IInspectable> {
        Err(E_NOTIMPL.into())
    }
}

impl bindings::IKeyboardHookFactory_Impl for KeyboardHookFactory_Impl {
    fn CreateInstance(
        &self,
        translator: Option<&bindings::KeyboardTranslator>,
    ) -> Result<bindings::KeyboardHook> {
        let translator = translator.ok_or_else(|| Error::new(E_POINTER, "translator is null"))?;
        let instance = KeyboardHook {
            thread_controller: Arc::new(
                ThreadHandler::new().expect("Thread handler should be created"),
            ),
        };
        let binding: bindings::KeyboardHook = instance.into();
        binding
            .cast_object_ref::<KeyboardHook>()?
            .activate(translator.downgrade()?, binding.downgrade()?)?;

        Ok(binding)
    }
}
