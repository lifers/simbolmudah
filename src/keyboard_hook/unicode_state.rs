use std::ffi::OsString;

use windows::Win32::UI::{
    Input::KeyboardAndMouse::VIRTUAL_KEY, WindowsAndMessaging::KBDLLHOOKSTRUCT,
};

use crate::key::Key;

use super::keyboard_layout::{KeyboardLayout, ParseVKError};

pub(super) struct UnicodeError;

pub(super) struct UnicodeState {
    state: String,
}

impl UnicodeState {
    pub(super) fn new() -> Self {
        Self {
            state: "U".to_string(),
        }
    }

    pub(super) fn build_unicode(&mut self) -> Result<OsString, UnicodeError> {
        println!("{:?}", &self.state);
        let res = Key::from_unicode_string(&self.state);
        self.state.truncate(1);

        if let Some(Key::Char(c)) = res {
            Ok(c.to_string().into())
        } else {
            Err(UnicodeError)
        }
    }

    pub(super) fn push(
        &mut self,
        event: &KBDLLHOOKSTRUCT,
        keystate: &[u8; 256],
        layout: &KeyboardLayout,
    ) {
        match layout.vk_to_unicode(
            VIRTUAL_KEY(event.vkCode as u16),
            event.scanCode,
            keystate,
            4,
        ) {
            Ok(s) => self.state.push(s.to_ascii_uppercase()),
            Err(ParseVKError::DeadKey(s)) => {
                self.state.push(s.to_ascii_uppercase());
            }
            Err(ParseVKError::NoTranslation) => {
                panic!("invalid input");
            }
            Err(ParseVKError::InvalidUnicode) => {
                panic!("invalid unicode");
            }
        };
    }
}
