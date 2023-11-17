use std::{ffi::OsString, mem::size_of, os::windows::ffi::OsStrExt};

use windows::Win32::{
    Foundation::GetLastError,
    UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
        VIRTUAL_KEY,
    },
};

use crate::keyboard_controller::clear_input;

pub fn send(out: &[INPUT]) -> windows::core::Result<()> {
    unsafe {
        if SendInput(&out, size_of::<INPUT>() as i32) != out.len() as u32 {
            GetLastError()
        } else {
            Ok(())
        }
    }
}

pub fn send_string(str: OsString) -> windows::core::Result<()> {
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

    clear_input();
    send(&out)
}
