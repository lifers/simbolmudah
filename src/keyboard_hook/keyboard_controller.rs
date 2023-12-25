use std::{ffi::OsString, mem::size_of, os::windows::ffi::OsStrExt};

use windows::Win32::{
    Foundation::GetLastError,
    UI::{
        Input::KeyboardAndMouse::{
            GetKeyboardState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT,
            KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, KEYEVENTF_UNICODE,
            VIRTUAL_KEY, VK_CAPITAL, VK_CONTROL, VK_MENU, VK_SHIFT,
        },
        WindowsAndMessaging::{KBDLLHOOKSTRUCT, KBDLLHOOKSTRUCT_FLAGS, LLKHF_EXTENDED, LLKHF_UP},
    },
};

use crate::{
    composer::{ComposeError, Composer},
    key::Key,
};

use super::{
    keyboard_layout::{KeyboardLayout, ParseVKError},
    sequence_state::SequenceState, unicode_state::UnicodeState,
};

pub(super) struct KeyboardController {
    stored_sequence: Vec<INPUT>,
    sequence_state: SequenceState,
    unicode_state: UnicodeState,
    composer: Composer,
}

impl KeyboardController {
    pub(super) fn new() -> Self {
        Self {
            stored_sequence: Vec::new(),
            sequence_state: SequenceState::new(),
            unicode_state: UnicodeState::new(),
            composer: Composer::new(),
        }
    }

    pub(super) fn push_input(&mut self, event: &KBDLLHOOKSTRUCT) {
        let mut dwflags = KEYEVENTF_SCANCODE;
        if event.flags & LLKHF_EXTENDED != KBDLLHOOKSTRUCT_FLAGS(0) {
            dwflags |= KEYEVENTF_EXTENDEDKEY;
        }
        if event.flags & LLKHF_UP != KBDLLHOOKSTRUCT_FLAGS(0) {
            dwflags |= KEYEVENTF_KEYUP;
        }
        let input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(event.vkCode as u16),
                    wScan: event.scanCode as u16,
                    dwFlags: dwflags,
                    time: event.time,
                    dwExtraInfo: event.dwExtraInfo,
                },
            },
        };
        self.stored_sequence.push(input);
    }

    pub(super) fn clear_input(&mut self) {
        self.stored_sequence.clear();
    }

    pub(super) fn abort_control(&mut self, skip: usize) -> windows::core::Result<()> {
        let out = self.stored_sequence.split_off(skip);
        Self::send(&out)
    }

    pub(super) fn compose_sequence(
        &mut self,
        event: &KBDLLHOOKSTRUCT,
        has_shift: bool,
        has_altgr: bool,
        has_capslock: bool,
        layout: &KeyboardLayout,
    ) -> Result<OsString, ComposeError> {
        let mut keystate = [0; 256];
        unsafe { GetKeyboardState(&mut keystate).unwrap() };

        keystate[VK_SHIFT.0 as usize] = if has_shift { 0x80 } else { 0 };
        keystate[VK_CONTROL.0 as usize] = if has_altgr { 0x80 } else { 0 };
        keystate[VK_MENU.0 as usize] = if has_altgr { 0x80 } else { 0 };
        keystate[VK_CAPITAL.0 as usize] = if has_capslock { 1 } else { 0 };

        let vk = event.vkCode as u16;
        match layout.vk_to_unicode(VIRTUAL_KEY(vk), event.scanCode, &keystate, 4) {
            Ok(s) => self.sequence_state.push(Key::Char(s)),
            Err(ParseVKError::DeadKey(s)) => {
                self.sequence_state.push(Key::Char(s));
            }
            Err(ParseVKError::NoTranslation) => {
                return Err(ComposeError::Incomplete);
                // CONVERTED_SEQUENCE.with_borrow_mut(|v| v.push(Key::VirtualKey(VIRTUAL_KEY(vk))))
            }
            Err(ParseVKError::InvalidUnicode) => {
                panic!("invalid unicode");
            }
        };

        self.sequence_state.submit(&self.composer)
    }

    pub(super) fn search_sequence(&mut self) -> windows::core::Result<()> {
        Ok(())
    }

    fn send(out: &[INPUT]) -> windows::core::Result<()> {
        unsafe {
            if SendInput(&out, size_of::<INPUT>() as i32) != out.len() as u32 {
                GetLastError()
            } else {
                Ok(())
            }
        }
    }

    pub(super) fn send_string(&mut self, str: OsString) -> windows::core::Result<()> {
        let out: Vec<_> = str
            .encode_wide()
            .flat_map(|c| {
                vec![
                    INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VIRTUAL_KEY(0),
                                wScan: c,
                                dwFlags: KEYEVENTF_UNICODE,
                                ..Default::default()
                            },
                        },
                    },
                    INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VIRTUAL_KEY(0),
                                wScan: c,
                                dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                                ..Default::default()
                            },
                        },
                    },
                ]
            })
            .collect();

        self.clear_input();
        Self::send(&out)
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, os::windows::ffi::OsStrExt};

    #[test]
    fn char_to_os_string() {
        let c = 'œ';
        let s: OsString = c.to_string().into();
        let s: Vec<_> = s.encode_wide().collect();
        let ans = [339];
        assert_eq!(s, ans);
    }
}
