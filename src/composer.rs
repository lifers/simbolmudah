use std::ffi::OsString;

use windows::Win32::UI::Input::KeyboardAndMouse::VK_MENU;

use crate::sequence::{key::Key, key_sequence::KeySequence};

pub enum ComposeError {
    Incomplete,
    NotFound,
}

pub fn search(seq: KeySequence) -> Result<char, ComposeError> {
    let ans = vec![Key::VirtualKey(VK_MENU), Key::Char('o'), Key::Char('e')];
    let ans2 = vec![Key::VirtualKey(VK_MENU), Key::Char('>'), Key::Char('=')];

    if seq == ans {
        Ok('œ')
    } else if seq == ans2 {
        Ok('≥')
    } else {
        Err(ComposeError::NotFound)
    }
}
