use std::cell::Cell;

use windows::Win32::{
    Foundation::{HMODULE, LPARAM, LRESULT, WPARAM},
    UI::{
        Input::KeyboardAndMouse::{
            VIRTUAL_KEY, VK_CAPITAL, VK_LSHIFT, VK_RMENU, VK_RSHIFT, VK_SHIFT,
        },
        WindowsAndMessaging::{
            CallNextHookEx, SetWindowsHookExA, UnhookWindowsHookEx, HC_ACTION, HHOOK,
            KBDLLHOOKSTRUCT, LLKHF_INJECTED, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN,
            WM_SYSKEYUP,
        },
    },
};

use crate::{
    character_sender::send_string,
    composer::ComposeError,
    keyboard_controller::{
        abort_control, clear_input, compose_sequence, push_input, search_sequence,
    },
    keyboard_layout::analyze_layout,
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

    // Indicator whether the previous message was a modifier
    static HAS_SHIFT: Cell<bool> = Cell::new(false);
    static HAS_ALTGR: Cell<bool> = Cell::new(false);
    static HAS_CAPSLOCK: Cell<bool> = Cell::new(false);
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

        // detect whether event is injected
        let is_injected = (*kb_hook).flags.0 & LLKHF_INJECTED.0 != 0;

        // TODO: might allow injected for testing purposes
        if ncode == HC_ACTION as i32 && is_key && !is_injected {
            let compose_stage = STAGE.get();
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
                        abort_control(0).unwrap();
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
                        sequence_tree(true, &(*kb_hook));
                    }
                    return LRESULT(1);
                }
                3 => {
                    if wparamu == WM_KEYUP && (*kb_hook).vkCode == VK_RMENU.0.into() {
                        STAGE.set(255);
                    } else {
                        STAGE.set(254);
                        // send to sequence tree
                        sequence_tree(true, &(*kb_hook));
                    }
                    return LRESULT(1);
                }
                254 => {
                    // send to sequence tree
                    if wparamu == WM_KEYDOWN || wparamu == WM_SYSKEYDOWN {
                        sequence_tree(false, &(*kb_hook));
                    }

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

fn sequence_tree(new: bool, event: &KBDLLHOOKSTRUCT) {
    if new {
        analyze_layout();
    }

    // If current event are modifier key (e.g. shift keydown),
    // take notes and do not send to composer.
    // Otherwise send everything to composer and reset notes.
    match VIRTUAL_KEY(event.vkCode as u16) {
        VK_SHIFT | VK_LSHIFT | VK_RSHIFT => {
            HAS_SHIFT.replace(true);
        }
        VK_RMENU => {
            HAS_ALTGR.replace(true);
        }
        VK_CAPITAL => {
            HAS_CAPSLOCK.replace(true);
        }
        _ => {
            match compose_sequence(
                event,
                HAS_SHIFT.take(),
                HAS_ALTGR.take(),
                HAS_CAPSLOCK.take(),
            ) {
                Ok(c) => {
                    send_string(c.to_string().into()).unwrap();
                    STAGE.set(0);
                }
                Err(ComposeError::NotFound) => {
                    abort_control(2).unwrap();
                    STAGE.set(0);
                }
                Err(ComposeError::Incomplete) => {}
            };
        }
    }
}
