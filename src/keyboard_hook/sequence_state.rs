use std::ffi::OsString;

use windows::Win32::UI::{
    Input::KeyboardAndMouse::VIRTUAL_KEY, WindowsAndMessaging::KBDLLHOOKSTRUCT,
};

use crate::{
    composer::{ComposeError, Composer},
    key::Key,
    key_sequence::KeySequence,
};

use super::keyboard_layout::{KeyboardLayout, ParseVKError};

pub(super) struct SequenceState {
    state: KeySequence,
    composer: Composer,
}

impl SequenceState {
    pub(super) fn new() -> Self {
        Self {
            state: KeySequence::new(),
            composer: Composer::new(),
        }
    }

    pub(super) fn compose_sequence(
        &mut self,
        event: &KBDLLHOOKSTRUCT,
        keystate: &[u8; 256],
        layout: &KeyboardLayout,
    ) -> Result<OsString, ComposeError> {
        match layout.vk_to_unicode(
            VIRTUAL_KEY(event.vkCode as u16),
            event.scanCode,
            keystate,
            4,
        ) {
            Ok(s) => self.state.push(Key::Char(s)),
            Err(ParseVKError::DeadKey(s)) => {
                self.state.push(Key::Char(s));
            }
            Err(ParseVKError::NoTranslation) => {
                return Err(ComposeError::Incomplete);
                // CONVERTED_SEQUENCE.with_borrow_mut(|v| v.push(Key::VirtualKey(VIRTUAL_KEY(vk))))
            }
            Err(ParseVKError::InvalidUnicode) => {
                panic!("invalid unicode");
            }
        };

        println!("{:?}", &self.state);
        let res = self
            .composer
            .search(&self.state.clone().try_into().unwrap());
        if res == Err(ComposeError::NotFound) || res.is_ok() {
            self.state.clear();
        }
        dbg!(res)
    }
}
