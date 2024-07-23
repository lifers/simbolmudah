use std::fmt::Debug;

use windows::{core::{Weak, HSTRING}, Foundation::EventRegistrationToken, Win32::UI::Input::KeyboardAndMouse::{GetKeyState, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VK_CAPITAL, VK_LSHIFT, VK_RMENU, VK_RSHIFT, VK_SHIFT, VK_U}};

use crate::{bindings, delegate_storage::DelegateStorage, get_strong_ref, sender::send_keybdinput};

use super::Stage;

pub(super) struct KeyboardHookInternal {
    pub(super) state_changed: DelegateStorage<bindings::KeyboardHook, u8>,
    pub(super) key_event: DelegateStorage<bindings::KeyboardHook, HSTRING>,
    pub(super) keyboard_translator: Weak<bindings::KeyboardTranslator>,
    pub(super) on_invalid_token: EventRegistrationToken,
    pub(super) on_translated_token: EventRegistrationToken,
    pub(super) input_buffer: Vec<KEYBDINPUT>,
    has_capslock: bool,
    has_shift: bool,
    has_altgr: bool,
    pub(super) stage: Stage,
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
    ) -> Self {
        Self {
            state_changed: DelegateStorage::new(),
            key_event: DelegateStorage::new(),
            keyboard_translator,
            on_invalid_token: EventRegistrationToken::default(),
            on_translated_token: EventRegistrationToken::default(),
            input_buffer: Vec::new(),
            has_capslock: unsafe { GetKeyState(VK_CAPITAL.0.into()) } & 0x0001 != 0,
            has_shift: false,
            has_altgr: false,
            stage: Stage::Idle,
            parent,
        }
    }

    pub(super) fn process_input(&mut self, input: KEYBDINPUT) -> bool {
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
                if is_keydown {
                    self.has_capslock = !self.has_capslock;
                }
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
                    get_strong_ref(&self.keyboard_translator)
                        .unwrap()
                        .CheckLayoutAndUpdate()
                        .unwrap();
                    self.translate_and_forward(input);
                } else {
                    self.stage = Stage::SequenceMode;
                    get_strong_ref(&self.keyboard_translator)
                        .unwrap()
                        .CheckLayoutAndUpdate()
                        .unwrap();
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
                    get_strong_ref(&self.keyboard_translator)
                        .unwrap()
                        .CheckLayoutAndUpdate()
                        .unwrap();
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
            Stage::SearchMode => false
        }
    }

    pub(super) fn report_state(&mut self) {
        self.state_changed
            .invoke_all(
                &get_strong_ref(&self.parent).unwrap(),
                Some(&(self.stage as u8)),
            )
            .unwrap();
    }

    pub(super) fn report_key_event(&mut self, input: KEYBDINPUT) {
        self.key_event
            .invoke_all(
                &get_strong_ref(&self.parent).unwrap(),
                Some(&keybdinput_to_hstring(input)),
            )
            .unwrap();
    }

    fn translate_and_forward(&self, input: KEYBDINPUT) {
        get_strong_ref(&self.keyboard_translator)
            .unwrap()
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

fn keybdinput_to_hstring(input: KEYBDINPUT) -> HSTRING {
    format!("KEYBDINPUT {{\n\twVk: {},\n\twScan: {},\n\tdwFlags: {},\n\ttime: {},\n\tdwExtraInfo: {}\n}}",
        input.wVk.0, input.wScan, input.dwFlags.0, input.time, input.dwExtraInfo).into()
}