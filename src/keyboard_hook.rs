use std::cell::RefCell;

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
    composer::ComposeError, keyboard_controller::KeyboardController,
    keyboard_layout::analyze_layout,
};

thread_local! {
    pub static GLOBAL_HOOK: RefCell<KeyboardHook> = panic!("GLOBAL_HOOK not initialized");
}

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
///
/// has_shift, has_altgr, has_capslock: indicator whether the previous message was a modifier key
pub struct KeyboardHook {
    h_hook: HHOOK,
    stage: u8,
    has_shift: bool,
    has_altgr: bool,
    has_capslock: bool,
    controller: KeyboardController,
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
        let stage = 0;
        let has_shift = false;
        let has_altgr = false;
        let has_capslock = false;
        let controller = KeyboardController::new();

        Self {
            h_hook,
            stage,
            has_shift,
            has_altgr,
            has_capslock,
            controller,
        }
    }

    fn process_event(&mut self, event: KBDLLHOOKSTRUCT, message: u32) -> Option<LRESULT> {
        self.controller.push_input(&event);

        match self.stage {
            0 => {
                if message == WM_SYSKEYDOWN && event.vkCode == VK_RMENU.0.into() {
                    self.stage = 1;
                    return Some(LRESULT(1));
                } else {
                    // Do nothing
                    self.controller.clear_input();
                }
            }
            1 => {
                if message == WM_KEYUP && event.vkCode == VK_RMENU.0.into() {
                    self.stage = 2;
                } else {
                    self.controller.abort_control(0).unwrap();
                    self.stage = 0;
                }
                return Some(LRESULT(1));
            }
            2 => {
                if message == WM_SYSKEYDOWN && event.vkCode == VK_RMENU.0.into() {
                    self.stage = 3;
                } else {
                    self.stage = 254;
                    // send to sequence tree
                    self.sequence_tree(true, &event);
                }
                return Some(LRESULT(1));
            }
            3 => {
                if message == WM_KEYUP && event.vkCode == VK_RMENU.0.into() {
                    self.stage = 255;
                } else {
                    self.stage = 254;
                    // send to sequence tree
                    self.sequence_tree(true, &event);
                }
                return Some(LRESULT(1));
            }
            254 => {
                // send to sequence tree
                if message == WM_KEYDOWN || message == WM_SYSKEYDOWN {
                    self.sequence_tree(false, &event);
                }

                return Some(LRESULT(1));
            }
            255 => {
                //send to search engine
                self.controller.search_sequence().unwrap();
            }
            _ => {}
        }

        None
    }

    fn sequence_tree(&mut self, new: bool, event: &KBDLLHOOKSTRUCT) {
        if new {
            analyze_layout();
        }

        // If current event are modifier key (e.g. shift keydown),
        // take notes and do not send to composer.
        // Otherwise send everything to composer and reset notes.
        match VIRTUAL_KEY(event.vkCode as u16) {
            VK_SHIFT | VK_LSHIFT | VK_RSHIFT => {
                self.has_shift = true;
            }
            VK_RMENU => {
                self.has_altgr = true;
            }
            VK_CAPITAL => {
                self.has_capslock = true;
            }
            _ => {
                match self.controller.compose_sequence(
                    event,
                    self.has_shift,
                    self.has_altgr,
                    self.has_capslock,
                ) {
                    Ok(c) => {
                        self.controller.send_string(c.to_string().into()).unwrap();
                        self.stage = 0;
                    }
                    Err(ComposeError::NotFound) => {
                        self.controller.abort_control(2).unwrap();
                        self.stage = 0;
                    }
                    Err(ComposeError::Incomplete) => {}
                };

                // reset to default state
                self.has_shift = false;
                self.has_altgr = false;
                self.has_capslock = false;
            }
        }
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
            if let Some(result) = GLOBAL_HOOK.with_borrow_mut(|hook: &mut KeyboardHook| {
                hook.process_event(*kb_hook, wparam.0 as u32)
            }) {
                return result;
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
