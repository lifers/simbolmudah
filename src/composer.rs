use std::mem::size_of;

use windows::Win32::{UI::Input::KeyboardAndMouse::{
    MapVirtualKeyW, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, MAPVK_VK_TO_VSC, VIRTUAL_KEY,
}, Foundation::GetLastError};

#[derive(PartialEq)]
pub enum EventType {
    KeyUp,
    KeyDown,
    KeyUpDown,
}

pub enum InputType {
    Unicode(u16),
    VirtualKey(VIRTUAL_KEY),
}

#[derive(Default)]
pub struct InputSequence {
    data: Vec<INPUT>,
}

impl InputSequence {
    pub fn add_input(&mut self, event: EventType, input: InputType) {
        let w_vk = match input {
            InputType::Unicode(_) => VIRTUAL_KEY(0),
            InputType::VirtualKey(x) => x,
        };

        let w_scan = match input {
            InputType::Unicode(x) => x,
            InputType::VirtualKey(x) => unsafe {
                MapVirtualKeyW(x.0.into(), MAPVK_VK_TO_VSC) as u16
            },
        };

        let dw_flags = match input {
            InputType::Unicode(_) => KEYEVENTF_UNICODE,
            InputType::VirtualKey(_) => {
                if w_scan | (0xE << 12) != 0 {
                    KEYEVENTF_EXTENDEDKEY
                } else {
                    KEYBD_EVENT_FLAGS(0)
                }
            }
        };

        if event != EventType::KeyUp {
            let input = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: w_vk,
                        wScan: w_scan,
                        dwFlags: dw_flags,
                        ..Default::default()
                    },
                },
            };
            self.data.push(input);
        }

        if event != EventType::KeyDown {
            let input = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: w_vk,
                        wScan: w_scan,
                        dwFlags: dw_flags | KEYEVENTF_KEYUP,
                        ..Default::default()
                    },
                },
            };
            self.data.push(input);
        }
    }

    pub fn send(&self) {
        for input in self.data.iter() {
            dbg!(unsafe {input.Anonymous.ki });
        }
        unsafe {
            if SendInput(&self.data, size_of::<INPUT>() as i32) != self.data.len() as u32 {
                if let Err(e) = GetLastError().ok() {
                    println!("{}", e.to_string());
                }
            }
        };
    }
}
