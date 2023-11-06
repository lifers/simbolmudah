use std::ffi::OsString;

use windows::Win32::UI::Input::KeyboardAndMouse::VK_MENU;

use crate::sequence::{key::Key, key_sequence::KeySequence};

pub enum ComposeError {
    Incomplete,
    NotFound,
}

pub fn search(seq: KeySequence) -> Result<OsString, ComposeError> {
    let ans = vec![
        Key::VirtualKey(VK_MENU),
        Key::String("o".into()),
        Key::String("e".into()),
    ];
    let ans2 = vec![
        Key::VirtualKey(VK_MENU),
        Key::String(">".into()),
        Key::String("=".into()),
    ];

    if seq == ans {
        Ok("œ".into())
    } else if seq == ans2 {
        Ok("≥".into())
    } else {
        Err(ComposeError::NotFound)
    }
}
