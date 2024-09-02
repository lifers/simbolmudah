use std::fmt::Debug;

use windows::{
    core::{Owned, Result, Weak, HSTRING},
    Foundation::{EventRegistrationToken, TypedEventHandler},
    Win32::{
        Foundation::{LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::KeyboardAndMouse::{
                GetKeyState, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP,
                KEYEVENTF_SCANCODE, VIRTUAL_KEY, VK_CAPITAL, VK_LSHIFT, VK_RMENU, VK_RSHIFT,
                VK_SHIFT, VK_U,
            },
            WindowsAndMessaging::{
                CallNextHookEx, SetWindowsHookExW, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT,
                LLKHF_EXTENDED, LLKHF_INJECTED, LLKHF_UP, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP,
                WM_SYSKEYDOWN, WM_SYSKEYUP,
            },
        },
    },
};

use crate::{
    bindings,
    utils::{
        delegate_storage::DelegateStorage,
        functions::{fail_message, get_strong_ref},
        sender::send_keybdinput,
        single_threaded::{single_threaded, SingleThreaded},
    },
};

/// Stage enum controls how low_level_keyboard_proc behave.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
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

pub(super) static INTERNAL: SingleThreaded<KeyboardHookInternal> =
    single_threaded!(KeyboardHookInternal);

#[allow(non_snake_case)]
pub(super) struct KeyboardHookInternal {
    pub(super) OnStateChanged: DelegateStorage<TypedEventHandler<bindings::KeyboardHook, u8>>,
    pub(super) OnKeyEvent: DelegateStorage<TypedEventHandler<bindings::KeyboardHook, HSTRING>>,
    pub(super) keyboard_translator: Weak<bindings::KeyboardTranslator>,
    pub(super) on_invalid_token: EventRegistrationToken,
    pub(super) on_translated_token: EventRegistrationToken,
    input_buffer: Vec<KEYBDINPUT>,
    has_capslock: bool,
    has_shift: bool,
    has_altgr: bool,
    stage: Stage,
    // will be automatically freed byb windows_core::Free
    #[allow(dead_code)]
    h_hook: Owned<HHOOK>,
    parent: Weak<bindings::KeyboardHook>,
}

impl Debug for KeyboardHookInternal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyboardHookInternal")
            .field("has_capslock", &self.has_capslock)
            .field("has_shift", &self.has_shift)
            .field("has_altgr", &self.has_altgr)
            .field("stage", &self.stage)
            .finish()
    }
}

impl KeyboardHookInternal {
    pub(super) fn new(
        keyboard_translator: Weak<bindings::KeyboardTranslator>,
        parent: Weak<bindings::KeyboardHook>,
    ) -> Result<Self> {
        let h_hook = unsafe {
            Owned::new(SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(keyboard_procedure),
                GetModuleHandleW(None)?,
                0,
            )?)
        };

        let reset_handler = TypedEventHandler::new(|_, _| {
            INTERNAL.with_borrow_mut(|internal| internal.reset_state())
        });
        let translator_ref = get_strong_ref(&keyboard_translator)?;

        Ok(Self {
            OnStateChanged: DelegateStorage::new(),
            OnKeyEvent: DelegateStorage::new(),
            keyboard_translator,
            on_invalid_token: translator_ref.OnInvalid(&reset_handler)?,
            on_translated_token: translator_ref.OnTranslated(&reset_handler)?,
            input_buffer: Vec::new(),
            has_capslock: unsafe { GetKeyState(VK_CAPITAL.0.into()) } & 0x0001 != 0,
            has_shift: false,
            has_altgr: false,
            stage: Stage::Idle,
            h_hook,
            parent,
        })
    }

    pub(super) fn process_input(&mut self, input: KEYBDINPUT) -> Result<()> {
        let is_keydown = input.dwFlags & KEYEVENTF_KEYUP == KEYBD_EVENT_FLAGS(0);
        match input.wVk {
            VK_SHIFT | VK_LSHIFT | VK_RSHIFT => {
                self.has_shift = is_keydown;
                return Err(fail_message("do not capture shift key"));
            }
            VK_RMENU => {
                self.has_altgr = is_keydown;
            }
            VK_CAPITAL => {
                if is_keydown {
                    self.has_capslock = !self.has_capslock;
                }
                return Err(fail_message("do not capture capslock key"));
            }
            _ => {}
        }

        self.input_buffer.push(input);

        match self.stage {
            Stage::Idle => {
                if is_keydown && input.wVk == VK_RMENU {
                    self.stage = Stage::ComposeKeydownFirst;
                    self.report_state()
                } else {
                    self.input_buffer.clear();
                    self.report_state()?;
                    Err(fail_message("let key pass through"))
                }
            }
            Stage::ComposeKeydownFirst => {
                if !is_keydown && input.wVk == VK_RMENU {
                    self.stage = Stage::ComposeKeyupFirst;
                } else {
                    send_keybdinput(self.input_buffer.drain(..).collect())?;
                    self.stage = Stage::Idle;
                }
                self.report_state()
            }
            Stage::ComposeKeyupFirst => {
                if is_keydown && input.wVk == VK_RMENU {
                    self.stage = Stage::ComposeKeydownSecond;
                } else if is_keydown && input.wVk == VK_U {
                    self.stage = Stage::UnicodeMode;
                    get_strong_ref(&self.keyboard_translator)?.CheckLayoutAndUpdate()?;
                    self.translate_and_forward(input)?;
                } else {
                    self.stage = Stage::SequenceMode;
                    get_strong_ref(&self.keyboard_translator)?.CheckLayoutAndUpdate()?;
                    self.translate_and_forward(input)?;
                }
                self.report_state()
            }
            Stage::ComposeKeydownSecond => {
                if !is_keydown && input.wVk == VK_RMENU {
                    self.stage = Stage::SearchMode;
                    self.input_buffer.clear();
                    // TODO: yield control to search engine
                } else {
                    self.stage = Stage::SequenceMode;
                    get_strong_ref(&self.keyboard_translator)?.CheckLayoutAndUpdate()?;
                    self.translate_and_forward(input)?;
                }
                self.report_state()
            }
            Stage::SequenceMode => {
                if is_keydown {
                    self.translate_and_forward(input)?;
                }
                Ok(())
            }
            Stage::UnicodeMode => {
                if is_keydown {
                    self.translate_and_forward(input)?;
                }
                Ok(())
            }
            Stage::SearchMode => Err(fail_message("shouldn't reach here")),
        }
    }

    pub(super) fn report_state(&mut self) -> Result<()> {
        self.OnStateChanged
            .invoke_all(|d| d.Invoke(&get_strong_ref(&self.parent)?, Some(&(self.stage as u8))))
    }

    pub(super) fn report_key_event(&mut self, input: KEYBDINPUT) -> Result<()> {
        self.OnKeyEvent.invoke_all(|d| {
            d.Invoke(
                &get_strong_ref(&self.parent)?,
                Some(&keybdinput_to_hstring(input)),
            )
        })
    }

    fn translate_and_forward(&self, input: KEYBDINPUT) -> Result<()> {
        get_strong_ref(&self.keyboard_translator)?.TranslateAndForward(
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
    }

    pub(super) fn reset_state(&mut self) -> Result<()> {
        self.stage = Stage::Idle;
        self.input_buffer.clear();
        self.report_state()
    }
}

fn keybdinput_to_hstring(input: KEYBDINPUT) -> HSTRING {
    format!("KEYBDINPUT {{\n\twVk: {},\n\twScan: {},\n\tdwFlags: {},\n\ttime: {},\n\tdwExtraInfo: {}\n}}",
        input.wVk.0, input.wScan, input.dwFlags.0, input.time, input.dwExtraInfo).into()
}

#[no_mangle]
extern "system" fn keyboard_procedure(ncode: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if ncode == HC_ACTION as i32 {
        let kb_hook = lparam.0 as *const KBDLLHOOKSTRUCT;
        let is_key = matches!(
            wparam.0 as u32,
            WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP
        );
        let is_injected = unsafe { *kb_hook }.flags.0 & LLKHF_INJECTED.0 != 0;

        if is_key && !is_injected {
            if let Some(res) = unsafe {
                INTERNAL.in_thread_borrow_mut(|internal| {
                    if internal.stage == Stage::SearchMode {
                        return None;
                    }

                    let input = kbdllhookstruct_to_keybdinput(*kb_hook);
                    internal
                        .report_key_event(input)
                        .expect("report_key_event should succeed");

                    if internal.process_input(input).is_ok() {
                        Some(LRESULT(1))
                    } else {
                        None
                    }
                })
            } {
                return res;
            }
        }
    }

    unsafe { CallNextHookEx(None, ncode, wparam, lparam) }
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
