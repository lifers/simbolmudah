use std::mem::size_of;

use windows::Win32::{
    Foundation::GetLastError,
    UI::{
        Input::KeyboardAndMouse::{
            GetKeyState, MapVirtualKeyA, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT,
            KEYBD_EVENT_FLAGS, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
            MAPVK_VK_TO_VSC, VIRTUAL_KEY, VK_CAPITAL, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_RMENU,
            VK_SHIFT,
        },
        WindowsAndMessaging::{
            KBDLLHOOKSTRUCT_FLAGS, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
        },
    },
};

use crate::sequence::{
    key::{Key, VK_NONE},
    key_sequence::KeySequence,
};

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
                MapVirtualKeyA(x.0.into(), MAPVK_VK_TO_VSC) as u16
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
            dbg!(unsafe { input.Anonymous.ki });
        }
        unsafe {
            if SendInput(&self.data, size_of::<INPUT>() as i32) != self.data.len() as u32 {
                if let Err(e) = GetLastError() {
                    println!("{}", e.to_string());
                }
            }
        };
    }
}

#[derive(PartialEq)]
pub enum State {
    Idle,
    Sequence,
    KeyCombination,
}

pub struct Composer {
    /// The sequence being currently typed
    sequence: KeySequence,
    /// The list of keys that have been pressed but not released yet
    pressed: KeySequence,
    last_key: Key,
    compose_count: u8,
    ctrl_was_altgr: bool,
    compose_key: Key,
    state: State,
    changed: bool,
    magic_pos: u8,
    magic_sequence: [(VIRTUAL_KEY, bool); 8],
}

impl Composer {
    // pub fn new() -> Self {
    //     Self {
    //         compose_key: Key::VirtualKey(VK_NONE),
    //         magic_pos: 0,
    //         magic_sequence: [
    //             (VK_LSHIFT, true),
    //             (VK_LMENU, true),
    //             (VK_LCONTROL, true),
    //             (VK_LMENU, false),
    //             (VK_LMENU, true),
    //             (VK_LSHIFT, false),
    //             (VK_LCONTROL, false),
    //             (VK_LMENU, false),
    //         ],
    //         ..Default::default()
    //     }
    // }

    pub fn see_state(&self) -> &State {
        &self.state
    }

    pub fn set_state(&mut self, value: State) {
        self.changed = self.state != value;
        self.state = value;
    }

    pub fn see_compose_key(&self) -> &Key {
        &self.compose_key
    }

    // pub fn on_key(window_message: u32, virtual_key: VIRTUAL_KEY, scan_code: u32, flags: KBDLLHOOKSTRUCT_FLAGS) -> bool {
    //     let event = match window_message {
    //         WM_KEYDOWN | WM_SYSKEYDOWN => EventType::KeyDown,
    //         WM_KEYUP | WM_SYSKEYUP => EventType::KeyUp,
    //         _ => {return false;}
    //     };
    //     // save dead key
    //     let ret = Self::on_key_internal(event, virtual_key, scan_code, flags);
    //     // restore dead key
    //     ret
    // }

    // fn on_key_internal(event: EventType, virtual_key: VIRTUAL_KEY, scan_code: u32, flags: KBDLLHOOKSTRUCT_FLAGS) -> bool {
    //     let has_shift = unsafe { GetKeyState(VK_SHIFT.0 as i32) } & 0x80 != 0;
    //     let has_altgr = unsafe { GetKeyState((VK_LCONTROL.0 & VK_RMENU.0) as i32) } & 0x80 != 0;
    //     let has_capslock = unsafe { GetKeyState(VK_CAPITAL.0 as i32) } != 0;

    // }
}
