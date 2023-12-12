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
    composer::{search, ComposeError},
    keyboard_layout::{vk_to_unicode, ParseVKError},
    sequence::{key::Key, key_sequence::KeySequence},
};

pub struct KeyboardController {
    stored_sequence: Vec<INPUT>,
    converted_sequence: KeySequence,
}

impl KeyboardController {
    pub fn new() -> Self {
        Self {
            stored_sequence: Vec::new(),
            converted_sequence: Vec::new(),
        }
    }

    pub fn push_input(&mut self, event: &KBDLLHOOKSTRUCT) {
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

    pub fn clear_input(&mut self) {
        self.stored_sequence.clear();
    }

    pub fn abort_control(&mut self, skip: usize) -> windows::core::Result<()> {
        let out = self.stored_sequence.split_off(skip);
        Self::send(&out)
    }

    pub fn compose_sequence(
        &mut self,
        event: &KBDLLHOOKSTRUCT,
        has_shift: bool,
        has_altgr: bool,
        has_capslock: bool,
    ) -> Result<char, ComposeError> {
        let mut keystate = [0; 256];
        unsafe { GetKeyboardState(&mut keystate).unwrap() };

        keystate[VK_SHIFT.0 as usize] = if has_shift { 0x80 } else { 0 };
        keystate[VK_CONTROL.0 as usize] = if has_altgr { 0x80 } else { 0 };
        keystate[VK_MENU.0 as usize] = if has_altgr { 0x80 } else { 0 };
        keystate[VK_CAPITAL.0 as usize] = if has_capslock { 1 } else { 0 };

        let vk = event.vkCode as u16;
        match vk_to_unicode(VIRTUAL_KEY(vk), event.scanCode, &keystate, 4) {
            Ok(s) => self.converted_sequence.push(Key::Char(s)),
            Err(ParseVKError::DeadKey(s)) => {
                self.converted_sequence.push(Key::Char(s));
            }
            Err(ParseVKError::NoTranslation) => {
                return Err(ComposeError::Incomplete);
                // CONVERTED_SEQUENCE.with_borrow_mut(|v| v.push(Key::VirtualKey(VIRTUAL_KEY(vk))))
            }
            Err(ParseVKError::InvalidUnicode) => {
                panic!("invalid unicode");
            }
        };

        dbg!(&self.converted_sequence);
        let res = search(&self.converted_sequence);
        if res == Err(ComposeError::NotFound) || res.is_ok() {
            self.converted_sequence.clear();
        }
        dbg!(res)
    }

    pub fn search_sequence(&mut self) -> windows::core::Result<()> {
        Ok(())
    }

    pub fn send(out: &[INPUT]) -> windows::core::Result<()> {
        unsafe {
            if SendInput(&out, size_of::<INPUT>() as i32) != out.len() as u32 {
                GetLastError()
            } else {
                Ok(())
            }
        }
    }

    pub fn send_string(&mut self, str: OsString) -> windows::core::Result<()> {
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
