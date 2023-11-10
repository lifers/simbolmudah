use std::cell::Cell;

use windows::Win32::{
    Foundation::{HMODULE, LPARAM, LRESULT, WPARAM},
    UI::{
        Input::KeyboardAndMouse::VK_RMENU,
        WindowsAndMessaging::{
            CallNextHookEx, SetWindowsHookExA, UnhookWindowsHookEx, HC_ACTION, HHOOK,
            KBDLLHOOKSTRUCT, LLKHF_INJECTED, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN,
            WM_SYSKEYUP,
        },
    },
};

use crate::{
    composer::ComposeError,
    keyboard_controller::{compose_sequence, push_input, search_sequence, send_back, send_string, clear_input},
};

thread_local! {
    /// STAGE variable controls how low_level_keyboard_proc behave.
    ///
    /// 0: neutral state.
    /// If receives compose keydown, intercept the event and go to stage 1.
    /// Otherwise, ignore the event and stay at state 0.
    /// The compose fail function will return all events to system and set stage to 0 when called.
    ///
    /// 1: compose keydown pressed for the first time.
    /// If receives compose keyup, intercept the event and go to stage 2. Assume consecutive compose
    /// keydown-keyup to be an intentional compose key press by the user.
    /// Otherwise, intercept the event and call compose fail. Assume the user is using the compose
    /// key to do something outside the scope of `simbolmudah`, so we return their inputs in the
    /// correct order.
    ///
    /// 2: compose key pressed once, compose mode on.
    /// Whatever happens, intercept the event.
    /// If receives compose keydown, go to stage 3.
    /// Else if receives keydown, send the event to the sequence tree, go to stage 254.
    ///
    /// 3: compose keydown pressed for the second time.
    /// Whatever happens, intercept the event.
    /// If receives compose keyup, intercept the event and go to stage 255. Assume consecutive compose
    /// keydown-keyup to be an intentional compose key press by the user.
    /// Otherwise, send the event to the sequence tree. Assume the user is using the compose key to
    /// insert a character for sequence mode. Go to stage 254.
    ///
    /// 254: sequence mode.
    /// Intercept and send the event to the sequence tree. The sequence tree will call compose fail
    /// upon failure.
    ///
    /// 255: search mode.
    /// same as 254
    static STAGE: Cell<u8> = Cell::new(0);
}

pub struct KeyboardHook {
    h_hook: HHOOK,
}

impl KeyboardHook {
    pub fn new(h_instance: HMODULE) -> Self {
        let h_hook = unsafe {
            SetWindowsHookExA(
                WH_KEYBOARD_LL,
                Some(Self::low_level_keyboard_proc),
                h_instance,
                0,
            )
            .expect("This is the only way for the program to work")
        };

        Self { h_hook }
    }

    unsafe extern "system" fn low_level_keyboard_proc(
        ncode: i32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        let wparamu = wparam.0 as u32;
        let is_key = match wparamu {
            WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP => true,
            _ => false,
        };
        let kb_hook = lparam.0 as *const KBDLLHOOKSTRUCT;
        let is_injected = (*kb_hook).flags.0 & LLKHF_INJECTED.0 != 0;
        // dbg!((*kb_hook).vkCode);
        // dbg!(is_key);
        // dbg!((*kb_hook).flags.0 & LLKHF_INJECTED.0);

        if ncode == HC_ACTION as i32 && is_key && !is_injected {
            let compose_stage = STAGE.get();

            dbg!(compose_stage);
            dbg!(wparamu);
            dbg!((*kb_hook).vkCode);
            push_input(&(*kb_hook));

            match compose_stage {
                0 => {
                    if wparamu == WM_SYSKEYDOWN && (*kb_hook).vkCode == VK_RMENU.0.into() {
                        STAGE.set(1);
                        return LRESULT(1);
                    } else {
                        // Do nothing
                        clear_input();
                    }
                }
                1 => {
                    if wparamu == WM_KEYUP && (*kb_hook).vkCode == VK_RMENU.0.into() {
                        STAGE.set(2);
                    } else {
                        send_back(0).unwrap();
                        STAGE.set(0);
                    }
                    return LRESULT(1);
                }
                2 => {
                    if wparamu == WM_SYSKEYDOWN && (*kb_hook).vkCode == VK_RMENU.0.into() {
                        STAGE.set(3);
                    } else {
                        STAGE.set(254);
                        // send to sequence tree
                        match compose_sequence(true, &(*kb_hook)) {
                            Ok(c) => {
                                send_string(c.to_string().into()).unwrap();
                            }
                            Err(ComposeError::NotFound) => {
                                send_back(0).unwrap();
                                STAGE.set(0);
                            }
                            Err(ComposeError::Incomplete) => {}
                        };
                    }
                    return LRESULT(1);
                }
                3 => {
                    if wparamu == WM_KEYUP && (*kb_hook).vkCode == VK_RMENU.0.into() {
                        STAGE.set(255);
                    } else {
                        STAGE.set(254);
                        // send to sequence tree
                        match compose_sequence(true, &(*kb_hook)) {
                            Ok(c) => {
                                send_string(c.to_string().into()).unwrap();
                            }
                            Err(ComposeError::NotFound) => {
                                send_back(0).unwrap();
                                STAGE.set(0);
                            }
                            Err(ComposeError::Incomplete) => {}
                        };
                    }
                    return LRESULT(1);
                }
                254 => {
                    // send to sequence tree
                    match compose_sequence(true, &(*kb_hook)) {
                        Ok(c) => {
                            send_string(c.to_string().into()).unwrap();
                        }
                        Err(ComposeError::NotFound) => {
                            send_back(0).unwrap();
                            STAGE.set(0);
                        }
                        Err(ComposeError::Incomplete) => {}
                    };
                    return LRESULT(1);
                }
                255 => {
                    //send to search engine
                    search_sequence().unwrap();
                }
                _ => {}
            }
        }

        CallNextHookEx(None, ncode, wparam, lparam)
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        unsafe { UnhookWindowsHookEx(self.h_hook).expect("cannot unhook!") };
        println!("hook successfully dropped");
    }
}
