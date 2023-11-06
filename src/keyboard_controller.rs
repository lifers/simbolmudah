use std::{cell::RefCell, ffi::OsString, mem::size_of, os::windows::prelude::OsStrExt};

use windows::{
    core::Result,
    Win32::{
        Foundation::GetLastError,
        UI::{
            Input::KeyboardAndMouse::{
                GetKeyboardState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT,
                KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, KEYEVENTF_UNICODE,
                VIRTUAL_KEY,
            },
            WindowsAndMessaging::{
                KBDLLHOOKSTRUCT, KBDLLHOOKSTRUCT_FLAGS, LLKHF_EXTENDED, LLKHF_UP,
            },
        },
    },
};

use crate::{
    composer::{search, ComposeError},
    keyboard_layout::{analyze_layout, vk_to_unicode, ParseVKError},
    sequence::{key::Key, key_sequence::KeySequence},
};

thread_local! {
    static STORED_SEQUENCE: RefCell<Vec<INPUT>> = RefCell::new(Vec::new());
    static CONVERTED_SEQUENCE: RefCell<KeySequence> = RefCell::new(Vec::new());
}

pub fn push_input(event: &KBDLLHOOKSTRUCT) {
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
    STORED_SEQUENCE.with_borrow_mut(|v| v.push(input));
}

fn send(out: &[INPUT]) -> Result<()> {
    unsafe {
        if SendInput(&out, size_of::<INPUT>() as i32) != out.len() as u32 {
            GetLastError()
        } else {
            Ok(())
        }
    }
}

pub fn send_back(skip: usize) -> Result<()> {
    let out = STORED_SEQUENCE.take();
    send(&out[skip..])
}

fn send_string(str: OsString) -> Result<()> {
    let out: Vec<_> = str
        .encode_wide()
        .flat_map(|c| {
            vec![
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(c),
                            wScan: 0,
                            dwFlags: KEYEVENTF_UNICODE,
                            ..Default::default()
                        },
                    },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(c),
                            wScan: 0,
                            dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                            ..Default::default()
                        },
                    },
                },
            ]
        })
        .collect();
    send(&out)
}

pub fn compose_sequence(new: bool, event: &KBDLLHOOKSTRUCT) {
    if new {
        analyze_layout();
    }
    let mut keystate = [0; 256];
    unsafe { GetKeyboardState(&mut keystate).unwrap() };

    let vk = event.vkCode as u16;
    match vk_to_unicode(VIRTUAL_KEY(vk), event.scanCode, &mut keystate, 0) {
        Ok(s) => CONVERTED_SEQUENCE.with_borrow_mut(|v| v.push(Key::String(s))),
        Err(ParseVKError::DeadKey(s)) => {
            CONVERTED_SEQUENCE.with_borrow_mut(|v| v.push(Key::String(s)))
        }
        Err(ParseVKError::NoTranslation) => {
            CONVERTED_SEQUENCE.with_borrow_mut(|v| v.push(Key::VirtualKey(VIRTUAL_KEY(vk))))
        }
        Err(ParseVKError::InvalidUnicode) => {}
    };
    match search(CONVERTED_SEQUENCE.take()) {
        Ok(s) => send_string(s).unwrap(),
        Err(ComposeError::NotFound) => send_back(0).unwrap(),
        Err(ComposeError::Incomplete) => {}
    };
}

pub fn search_sequence() -> Result<()> {
    Ok(())
}
