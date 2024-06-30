// mod input_dispatcher;

use crate::{
    bindings::{self, IKeyboardTranslator_Impl},
    delegate_storage::{get_token, DelegateStorage},
    keyboard_translator::KeyboardTranslator,
    sender::send_keybdinput,
    thread_handler::ThreadHandler,
};
use std::{
    fmt::Debug,
    sync::{mpsc::sync_channel, Arc, OnceLock, RwLock, RwLockWriteGuard},
    usize,
};
use windows::{
    core::{
        implement, AgileReference, ComObject, Error, IInspectable, Interface, Result, Weak, HSTRING,
    },
    Foundation::{EventRegistrationToken, TypedEventHandler},
    Win32::{
        Foundation::{E_ACCESSDENIED, E_INVALIDARG, E_NOTIMPL, E_POINTER, LPARAM, LRESULT, WPARAM},
        System::{
            LibraryLoader::GetModuleHandleW,
            WinRT::{IActivationFactory, IActivationFactory_Impl},
        },
        UI::{
            Input::KeyboardAndMouse::{
                GetKeyState, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP,
                KEYEVENTF_SCANCODE, VIRTUAL_KEY, VK_CAPITAL, VK_LSHIFT, VK_RMENU, VK_RSHIFT,
                VK_SHIFT, VK_U,
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

enum Reporter {
    StateChanged,
    KeyEvent,
}

struct WeakWrapper(pub Weak<bindings::KeyboardHook>);
unsafe impl Send for WeakWrapper {}
unsafe impl Sync for WeakWrapper {}

#[implement(bindings::KeyboardHook)]
#[derive(Clone, Debug)]
struct KeyboardHook {
    thread_controller: Arc<ThreadHandler>,
}

impl KeyboardHook {
    fn activate(
        &self,
        keyboard_translator: ComObject<KeyboardTranslator>,
        parent: WeakWrapper,
    ) -> Result<()> {
        let (tx, rx) = sync_channel(16);
        tx.send((keyboard_translator, parent))
            .expect("message should be sent before enqueue");

        self.thread_controller.try_enqueue_high(move || {
            let (keyboard_translator, parent) =
                rx.recv().expect("message should be sent before enqueue");
            let hmod = unsafe { GetModuleHandleW(None) }?;
            let h_hook =
                unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_procedure), hmod, 0) }?;

            let internal = KeyboardHookInternal::new(keyboard_translator, parent.0, h_hook);
            let mut write_lock = Self::global_write()?;

            write_lock.set(internal).expect(
                "GLOBAL_INSTANCE has to be empty or emptied by the previous deactivate call",
            );
            let event_handler = TypedEventHandler::new(|_, _| {
                Ok(Self::global_write()?
                    .get_mut()
                    .expect("GLOBAL_INSTANCE should be set")
                    .stage = Stage::Idle)
            });
            let write_lock = write_lock.get_mut().expect("GLOBAL_INSTANCE should be set");
            write_lock.on_invalid_token = write_lock
                .keyboard_translator
                .OnInvalid(Some(&event_handler))?;
            Ok(())
        })
    }

    fn global_write() -> Result<RwLockWriteGuard<'static, OnceLock<KeyboardHookInternal>>> {
        GLOBAL_INSTANCE
            .write()
            .map_err(|e| Error::new(E_ACCESSDENIED, format!("{:?}", e)))
    }

    fn register_reporter(
        &self,
        reporter: Reporter,
        handler: Option<&TypedEventHandler<bindings::KeyboardHook, HSTRING>>,
    ) -> Result<EventRegistrationToken> {
        if let Some(handler) = handler {
            let token = get_token(handler.as_raw());
            let handler_ref = AgileReference::new(handler)?;

            // making sure this callback runs after the hook is activated
            self.thread_controller.try_enqueue_high(move || {
                let mut write_lock = Self::global_write()?;
                let internal = write_lock.get_mut().expect("GLOBAL_INSTANCE should be set");

                Ok(match reporter {
                    Reporter::StateChanged => internal
                        .debug_state_changed
                        .insert(token, handler_ref.clone()),
                    Reporter::KeyEvent => {
                        internal.debug_key_event.insert(token, handler_ref.clone())
                    }
                })
            })?;

            Ok(EventRegistrationToken { Value: token })
        } else {
            Err(Error::new(E_POINTER, "Null handler pointer"))
        }
    }

    fn unregister_reporter(&self, reporter: Reporter, token: i64) -> Result<()> {
        // making sure this callback runs after the reporter is registered
        self.thread_controller.try_enqueue_high(move || {
            let mut write_lock = Self::global_write()?;
            let internal = write_lock.get_mut().expect("GLOBAL_INSTANCE should be set");

            Ok(match reporter {
                Reporter::StateChanged => internal.debug_state_changed.remove(token),
                Reporter::KeyEvent => internal.debug_key_event.remove(token),
            })
        })?;
        Ok(())
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        self.thread_controller
            .try_enqueue_high(|| {
                let mut write_lock = Self::global_write()?;
                let instance_ref = write_lock.get_mut().expect("GLOBAL_INSTANCE should be set");
                instance_ref
                    .keyboard_translator
                    .RemoveOnInvalid(&instance_ref.on_invalid_token)
                    .unwrap();
                unsafe { UnhookWindowsHookEx(instance_ref.h_hook) }
                    .expect("Unhooking must succeed");
                let _ = write_lock.take().expect("GLOBAL_INSTANCE should be set");
                Ok(())
            })
            .unwrap();
    }
}

struct KeyboardHookInternal {
    debug_state_changed: DelegateStorage<bindings::KeyboardHook, HSTRING>,
    debug_key_event: DelegateStorage<bindings::KeyboardHook, HSTRING>,
    h_hook: HHOOK,
    keyboard_translator: ComObject<KeyboardTranslator>,
    on_invalid_token: EventRegistrationToken,
    input_buffer: Vec<KEYBDINPUT>,
    has_capslock: bool,
    has_shift: bool,
    has_altgr: bool,
    stage: Stage,
    parent: Weak<bindings::KeyboardHook>,
}

unsafe impl Send for KeyboardHookInternal {}
unsafe impl Sync for KeyboardHookInternal {}

impl Debug for KeyboardHookInternal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyboardHookInternal")
            .field("h_hook", &self.h_hook)
            .field("has_capslock", &self.has_capslock)
            .field("has_shift", &self.has_shift)
            .field("has_altgr", &self.has_altgr)
            .field("stage", &self.stage)
            .finish()
    }
}

impl KeyboardHookInternal {
    fn new(
        keyboard_translator: ComObject<KeyboardTranslator>,
        parent: Weak<bindings::KeyboardHook>,
        h_hook: HHOOK,
    ) -> Self {
        Self {
            debug_state_changed: DelegateStorage::new(),
            debug_key_event: DelegateStorage::new(),
            h_hook,
            keyboard_translator,
            on_invalid_token: EventRegistrationToken::default(),
            input_buffer: Vec::new(),
            has_capslock: unsafe { GetKeyState(VK_CAPITAL.0.into()) } & 0x0001 != 0,
            has_shift: false,
            has_altgr: false,
            stage: Stage::Idle,
            parent,
        }
    }

    fn process_input(&mut self, input: KEYBDINPUT) -> bool {
        let is_keydown = input.dwFlags & KEYEVENTF_KEYUP == KEYBD_EVENT_FLAGS(0);
        match input.wVk {
            VK_SHIFT | VK_LSHIFT | VK_RSHIFT => {
                self.has_shift = is_keydown;
                return false;
            }
            VK_RMENU => {
                self.has_altgr = is_keydown;
            }
            VK_CAPITAL => {
                self.has_capslock = !self.has_capslock;
                return false;
            }
            _ => {}
        }

        self.input_buffer.push(input);

        match self.stage {
            Stage::Idle => {
                if is_keydown && input.wVk == VK_RMENU {
                    self.stage = Stage::ComposeKeydownFirst;
                    self.report_state();
                    true
                } else {
                    self.input_buffer.clear();
                    self.report_state();
                    false
                }
            }
            Stage::ComposeKeydownFirst => {
                if !is_keydown && input.wVk == VK_RMENU {
                    self.stage = Stage::ComposeKeyupFirst;
                } else {
                    send_keybdinput(self.input_buffer.drain(..).collect()).unwrap();
                    self.stage = Stage::Idle;
                }
                self.report_state();
                true
            }
            Stage::ComposeKeyupFirst => {
                if is_keydown && input.wVk == VK_RMENU {
                    self.stage = Stage::ComposeKeydownSecond;
                } else if is_keydown && input.wVk == VK_U {
                    self.stage = Stage::UnicodeMode;
                    self.keyboard_translator.CheckLayoutAndUpdate().unwrap();
                    self.translate_and_forward(input);
                } else {
                    self.stage = Stage::SequenceMode;
                    self.keyboard_translator.CheckLayoutAndUpdate().unwrap();
                    self.translate_and_forward(input);
                }
                self.report_state();
                true
            }
            Stage::ComposeKeydownSecond => {
                if !is_keydown && input.wVk == VK_RMENU {
                    self.stage = Stage::SearchMode;
                    self.input_buffer.clear();
                    // TODO: yield control to search engine
                } else {
                    self.stage = Stage::SequenceMode;
                    self.keyboard_translator.CheckLayoutAndUpdate().unwrap();
                    self.translate_and_forward(input);
                }
                self.report_state();
                true
            }
            Stage::SequenceMode => {
                if is_keydown {
                    self.translate_and_forward(input);
                }
                true
            }
            Stage::UnicodeMode => {
                if is_keydown {
                    self.translate_and_forward(input);
                }
                true
            }
            _ => panic!("Invalid stage"),
        }
    }

    fn report_state(&mut self) {
        self.debug_state_changed
            .invoke_all(
                &self.parent.upgrade().unwrap(),
                Some(&HSTRING::from(format!("{:?}", self.stage))),
            )
            .unwrap();
    }

    fn report_key_event(&mut self, input: KEYBDINPUT) {
        self.debug_key_event
            .invoke_all(
                &self.parent.upgrade().unwrap(),
                Some(&keybdinput_to_hstring(input)),
            )
            .unwrap();
    }

    fn translate_and_forward(&self, input: KEYBDINPUT) {
        self.keyboard_translator
            .TranslateAndForward(
                input.wVk.0.into(),
                input.wScan.into(),
                self.has_capslock,
                self.has_shift,
                self.has_altgr,
                match self.stage {
                    Stage::SequenceMode => 0,
                    Stage::UnicodeMode => 1,
                    _ => panic!("Invalid stage"),
                },
            )
            .unwrap();
    }
}

impl bindings::IKeyboardHook_Impl for KeyboardHook {
    fn DebugStateChanged(
        &self,
        handler: Option<&TypedEventHandler<bindings::KeyboardHook, HSTRING>>,
    ) -> Result<EventRegistrationToken> {
        self.register_reporter(Reporter::StateChanged, handler)
    }

    fn RemoveDebugStateChanged(&self, token: &EventRegistrationToken) -> Result<()> {
        self.unregister_reporter(Reporter::StateChanged, token.Value)
    }

    fn DebugKeyEvent(
        &self,
        handler: Option<&TypedEventHandler<bindings::KeyboardHook, HSTRING>>,
    ) -> Result<EventRegistrationToken> {
        self.register_reporter(Reporter::KeyEvent, handler)
    }

    fn RemoveDebugKeyEvent(&self, token: &EventRegistrationToken) -> Result<()> {
        self.unregister_reporter(Reporter::KeyEvent, token.Value)
    }
}

/// Stage enum controls how low_level_keyboard_proc behave.
#[derive(Debug, Default)]
enum Stage {
    /// No compose key pressed.
    /// If receives compose keydown, intercept the event and go to stage 1.
    /// Otherwise, ignore the event and stay at state 0.
    /// The compose fail function will return all events to system and set stage to 0 when called.
    #[default]
    Idle,
    /// Compose keydown pressed for the first time.
    /// If receives compose keyup, intercept the event and go to stage 2. Assume consecutive compose
    /// keydown-keyup to be an intentional compose key press by the user.
    /// Otherwise, intercept the event and call compose fail. Assume the user is using the compose
    /// key to do something outside the scope of `simbolmudah`, so we return their inputs in the
    /// correct order.
    ComposeKeydownFirst,
    /// Compose key pressed once, compose mode on.
    /// Whatever happens, intercept the event.
    /// If receives compose keydown, go to stage 3.
    /// Else if receives keydown, send the event to the sequence tree, go to stage 254.
    ComposeKeyupFirst,
    /// Compose keydown pressed for the second time.
    /// Whatever happens, intercept the event.
    /// If receives compose keyup, intercept the event and go to stage 255. Assume consecutive compose
    /// keydown-keyup to be an intentional compose key press by the user.
    /// Otherwise, send the event to the sequence tree. Assume the user is using the compose key to
    /// insert a character for sequence mode. Go to stage 254.
    ComposeKeydownSecond,
    /// Sequence mode.
    /// Intercept and send the event to the sequence tree. The sequence tree will call compose fail
    /// upon failure.
    SequenceMode,
    /// Search mode. Not implemented yet.
    SearchMode,
    /// Unicode mode. If receives a hexadecimal keydown, push the key to the unicode state.
    UnicodeMode,
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

fn keybdinput_to_hstring(input: KEYBDINPUT) -> HSTRING {
    HSTRING::from(format!("KEYBDINPUT {{\n\twVk: {},\n\twScan: {},\n\tdwFlags: {},\n\ttime: {},\n\tdwExtraInfo: {}\n}}",
        input.wVk.0, input.wScan, input.dwFlags.0, input.time, input.dwExtraInfo))
}

#[implement(IActivationFactory, bindings::IKeyboardHookFactory)]
pub(super) struct KeyboardHookFactory;

impl IActivationFactory_Impl for KeyboardHookFactory {
    fn ActivateInstance(&self) -> Result<IInspectable> {
        Err(E_NOTIMPL.into())
    }
}

impl bindings::IKeyboardHookFactory_Impl for KeyboardHookFactory {
    fn CreateInstance(
        &self,
        translator: Option<&bindings::KeyboardTranslator>,
    ) -> Result<bindings::KeyboardHook> {
        let translator = translator
            .ok_or_else(|| Error::new(E_INVALIDARG, "No KeyboardTranslator passed"))?
            .cast_object::<KeyboardTranslator>()?;
        let instance = KeyboardHook {
            thread_controller: Arc::new(
                ThreadHandler::new().expect("Thread handler should be created"),
            ),
        };
        let binding: bindings::KeyboardHook = instance.into();
        let binding_ref = binding.downgrade()?;
        binding
            .cast_object::<KeyboardHook>()?
            .activate(translator, WeakWrapper(binding_ref))?;

        Ok(binding)
    }
}
