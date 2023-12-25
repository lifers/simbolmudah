mod keyboard_controller;
mod keyboard_layout;

use std::cell::RefCell;

use windows::Win32::{
    Foundation::{HMODULE, LPARAM, LRESULT, WPARAM},
    UI::{
        Input::KeyboardAndMouse::{
            VIRTUAL_KEY, VK_CAPITAL, VK_LSHIFT, VK_RMENU, VK_RSHIFT, VK_SHIFT, GetKeyState,
        },
        WindowsAndMessaging::{
            CallNextHookEx, SetWindowsHookExA, UnhookWindowsHookEx, HC_ACTION, HHOOK,
            KBDLLHOOKSTRUCT, LLKHF_INJECTED, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN,
            WM_SYSKEYUP,
        },
    },
};

use crate::composer::ComposeError;

use self::{keyboard_controller::KeyboardController, keyboard_layout::KeyboardLayout};

thread_local! {
    pub(super) static GLOBAL_HOOK: RefCell<Option<KeyboardHook>> = RefCell::new(None);
}

enum Stage {
    Neutral,
    ComposeKeydownFirst,
    ComposeKeyupFirst,
    ComposeKeydownSecond,
    SequenceMode,
    SearchMode,
    UnicodeMode,
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
pub(super) struct KeyboardHook {
    h_hook: HHOOK,
    stage: Stage,
    has_shift: bool,
    has_altgr: bool,
    has_capslock: bool,
    controller: KeyboardController,
    layout: KeyboardLayout,
}

impl KeyboardHook {
    pub(super) fn new(h_instance: HMODULE) -> Self {
        let h_hook = unsafe {
            SetWindowsHookExA(
                WH_KEYBOARD_LL,
                Some(Self::low_level_keyboard_proc),
                h_instance,
                0,
            )
            .expect("This is the only way for the program to work")
        };

        let has_capslock = unsafe { GetKeyState(VK_CAPITAL.0.into()) } & 0x0001 != 0;

        Self {
            h_hook,
            stage: Stage::Neutral,
            has_shift: false,
            has_altgr: false,
            has_capslock,
            controller: KeyboardController::new(),
            layout: KeyboardLayout::new(),
        }
    }

    fn process_event(&mut self, event: KBDLLHOOKSTRUCT, message: u32) -> Option<LRESULT> {
        // Update modifier key state
        match VIRTUAL_KEY(event.vkCode as u16) {
            VK_SHIFT | VK_LSHIFT | VK_RSHIFT => {
                self.has_shift = message == WM_KEYDOWN || message == WM_SYSKEYDOWN;
            }
            VK_RMENU => {
                self.has_altgr = message == WM_KEYDOWN || message == WM_SYSKEYDOWN;
            }
            VK_CAPITAL => {
                if message == WM_KEYDOWN || message == WM_SYSKEYDOWN {
                    println!("switching to {}", !self.has_capslock);
                    self.has_capslock = !self.has_capslock;
                    return None;
                }
            }
            _ => {}
        };

        self.controller.push_input(&event);

        match self.stage {
            Stage::Neutral => {
                if message == WM_SYSKEYDOWN && event.vkCode == VK_RMENU.0.into() {
                    self.stage = Stage::ComposeKeydownFirst;
                    return Some(LRESULT(1));
                } else {
                    // Do nothing
                    self.controller.clear_input();
                }
            }
            Stage::ComposeKeydownFirst => {
                if message == WM_KEYUP && event.vkCode == VK_RMENU.0.into() {
                    self.stage = Stage::ComposeKeyupFirst;
                } else {
                    self.controller.abort_control(0).unwrap();
                    self.stage = Stage::Neutral;
                }
                return Some(LRESULT(1));
            }
            Stage::ComposeKeyupFirst => {
                if message == WM_SYSKEYDOWN && event.vkCode == VK_RMENU.0.into() {
                    self.stage = Stage::ComposeKeydownSecond;
                } else {
                    self.stage = Stage::SequenceMode;
                    // send to sequence tree
                    self.sequence_tree(true, &event);
                }
                return Some(LRESULT(1));
            }
            Stage::ComposeKeydownSecond => {
                if message == WM_KEYUP && event.vkCode == VK_RMENU.0.into() {
                    self.stage = Stage::SearchMode;
                } else {
                    self.stage = Stage::SequenceMode;
                    // send to sequence tree
                    self.sequence_tree(true, &event);
                }
                return Some(LRESULT(1));
            }
            Stage::SequenceMode => {
                // send to sequence tree
                if message == WM_KEYDOWN || message == WM_SYSKEYDOWN {
                    self.sequence_tree(false, &event);
                }

                return Some(LRESULT(1));
            }
            Stage::SearchMode => {
                //send to search engine
                self.controller.search_sequence().unwrap();
            }
            Stage::UnicodeMode => {
                todo!()
            }
        }

        None
    }

    fn sequence_tree(&mut self, new: bool, event: &KBDLLHOOKSTRUCT) {
        if new {
            self.layout.analyze_layout();
        }

        match self.controller.compose_sequence(
            event,
            self.has_shift,
            self.has_altgr,
            self.has_capslock,
            &self.layout,
        ) {
            Ok(c) => {
                self.controller.send_string(c).unwrap();
                self.stage = Stage::Neutral;
            }
            Err(ComposeError::NotFound) => {
                self.controller.abort_control(2).unwrap();
                self.stage = Stage::Neutral;
            }
            Err(ComposeError::Incomplete) => {}
        };
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
            if let Some(result) = GLOBAL_HOOK.with_borrow_mut(|hook: &mut Option<KeyboardHook>| {
                hook.as_mut()
                    .expect("GLOBAL_HOOK uninitialised")
                    .process_event(*kb_hook, wparam.0 as u32)
            }) {
                return result;
            }
        }

        if (*kb_hook).vkCode == VK_CAPITAL.0.into() && (wparamu == WM_KEYDOWN || wparamu == WM_SYSKEYDOWN) {
            println!("Caps Lock toggled");
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
