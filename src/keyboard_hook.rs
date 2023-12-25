mod keyboard_controller;
mod keyboard_layout;
mod sequence_state;
mod unicode_state;

use std::cell::RefCell;

use windows::Win32::{
    Foundation::{HMODULE, LPARAM, LRESULT, WPARAM},
    UI::{
        Input::KeyboardAndMouse::{
            GetKeyState, GetKeyboardState, VIRTUAL_KEY, VK_0, VK_9, VK_A, VK_CAPITAL, VK_CONTROL,
            VK_F, VK_LSHIFT, VK_MENU, VK_NUMPAD0, VK_NUMPAD9, VK_RETURN, VK_RMENU, VK_RSHIFT,
            VK_SHIFT, VK_U,
        },
        WindowsAndMessaging::{
            CallNextHookEx, SetWindowsHookExA, UnhookWindowsHookEx, HC_ACTION, HHOOK,
            KBDLLHOOKSTRUCT, LLKHF_INJECTED, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN,
            WM_SYSKEYUP,
        },
    },
};

use crate::composer::ComposeError;

use self::{
    keyboard_controller::KeyboardController, keyboard_layout::KeyboardLayout,
    sequence_state::SequenceState, unicode_state::UnicodeState,
};

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
    sequence_state: SequenceState,
    unicode_state: UnicodeState,
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
            sequence_state: SequenceState::new(),
            unicode_state: UnicodeState::new(),
            layout: KeyboardLayout::new(),
        }
    }

    fn process_event(&mut self, event: KBDLLHOOKSTRUCT, message: u32) -> Option<LRESULT> {
        let is_keydown = message == WM_KEYDOWN || message == WM_SYSKEYDOWN;

        // Update modifier key state
        match VIRTUAL_KEY(event.vkCode as u16) {
            VK_SHIFT | VK_LSHIFT | VK_RSHIFT => {
                self.has_shift = is_keydown;
                return None;
            }
            VK_RMENU => {
                self.has_altgr = is_keydown;
            }
            VK_CAPITAL => {
                if is_keydown {
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
                } else if message == WM_KEYDOWN && event.vkCode == VK_U.0.into() {
                    self.stage = Stage::UnicodeMode;
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
                if is_keydown {
                    self.sequence_tree(false, &event);
                }

                return Some(LRESULT(1));
            }
            Stage::SearchMode => {
                //send to search engine
                self.controller.search_sequence().unwrap();
            }
            Stage::UnicodeMode => {
                if is_keydown && Self::is_hexadecimal(event.vkCode as u16) {
                    let keystate = self.calculate_keystate();
                    self.unicode_state.push(&event, &keystate, &self.layout);
                } else if !is_keydown && event.vkCode == VK_RETURN.0.into() {
                    if let Ok(s) = self.unicode_state.build_unicode() {
                        self.controller
                            .send_string(s)
                            .expect("string should be sent successfully");
                    } else {
                        self.controller
                            .abort_control(2)
                            .expect("print all entered keystrokes");
                    }
                    self.stage = Stage::Neutral;
                }

                return Some(LRESULT(1));
            }
        }

        None
    }

    fn sequence_tree(&mut self, new: bool, event: &KBDLLHOOKSTRUCT) {
        if new {
            self.layout.analyze_layout();
        }

        let keystate = self.calculate_keystate();

        match self
            .sequence_state
            .compose_sequence(event, &keystate, &self.layout)
        {
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

    fn calculate_keystate(&self) -> [u8; 256] {
        let mut keystate = [0; 256];
        unsafe { GetKeyboardState(&mut keystate).unwrap() };

        keystate[VK_SHIFT.0 as usize] = if self.has_shift { 0x80 } else { 0 };
        keystate[VK_CONTROL.0 as usize] = if self.has_altgr { 0x80 } else { 0 };
        keystate[VK_MENU.0 as usize] = if self.has_altgr { 0x80 } else { 0 };
        keystate[VK_CAPITAL.0 as usize] = if self.has_capslock { 1 } else { 0 };

        keystate
    }

    fn is_hexadecimal(vkcode: u16) -> bool {
        (VK_0.0 <= vkcode && vkcode <= VK_9.0)
            || (VK_NUMPAD0.0 <= vkcode && vkcode <= VK_NUMPAD9.0)
            || (VK_A.0 <= vkcode && vkcode <= VK_F.0)
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

        if (*kb_hook).vkCode == VK_CAPITAL.0.into()
            && (wparamu == WM_KEYDOWN || wparamu == WM_SYSKEYDOWN)
        {
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
