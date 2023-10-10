use std::{mem::size_of, sync::Mutex};

use once_cell::sync::Lazy;
use windows::Win32::{
    Foundation::GetLastError,
    UI::{
        Input::KeyboardAndMouse::{
            SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
            KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, VIRTUAL_KEY,
        },
        WindowsAndMessaging::{KBDLLHOOKSTRUCT, KBDLLHOOKSTRUCT_FLAGS, LLKHF_EXTENDED, LLKHF_UP},
    },
};

pub static STORED_KEY_EVENTS: Lazy<Mutex<KeyEventSeq>> = Lazy::new(|| {
    let mut k = KeyEventSeq { data: Vec::new() };
    Mutex::new(k)
});

pub struct KeyEventSeq {
    data: Vec<INPUT>,
}

impl KeyEventSeq {
    pub fn push(&mut self, event: &KBDLLHOOKSTRUCT) {
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
        self.data.push(input);
    }

    pub fn send(&mut self, skip: usize) {
        unsafe {
            if SendInput(&self.data[skip..], size_of::<INPUT>() as i32) != self.data.len() as u32 {
                if let Err(e) = GetLastError() {
                    println!("{}", e.to_string());
                }
            }
        };
        self.data.clear();
    }
}
