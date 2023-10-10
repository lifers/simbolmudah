use std::sync::atomic::{
    AtomicU8,
    Ordering::{Acquire, Release},
};

use once_cell::sync::Lazy;
use windows::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleA,
    UI::{
        Input::KeyboardAndMouse::VK_RMENU,
        WindowsAndMessaging::{
            CallNextHookEx, SetWindowsHookExA, UnhookWindowsHookEx, HC_ACTION, HHOOK,
            KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
        },
    },
};

use crate::key_event_sequence::STORED_KEY_EVENTS;

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
/// Intercept and send the event to the search engine. The search engine will call compose fail
/// upon failure.
///
static STAGE: AtomicU8 = AtomicU8::new(0);

static GLOBAL_HOOK: Lazy<KeyboardHook> = Lazy::new(|| {
    let h_instance =
        unsafe { GetModuleHandleA(None).expect("We don't need a module handle for this") };
    let h_hook = unsafe {
        SetWindowsHookExA(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), h_instance, 0)
            .expect("This is the only way for the program to work")
    };

    KeyboardHook { h_hook }
});

struct KeyboardHook {
    h_hook: HHOOK,
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

    if ncode == HC_ACTION as i32 && is_key {
        let compose_stage = STAGE.load(Acquire);
        let kb_hook = lparam.0 as *const KBDLLHOOKSTRUCT;
        let mut events = STORED_KEY_EVENTS.lock().unwrap();

        match compose_stage {
            0 => {
                if wparamu == WM_SYSKEYDOWN && (*kb_hook).vkCode == VK_RMENU.0.into() {
                    events.push(&(*kb_hook));
                    STAGE.store(1, Release);
                    return LRESULT(1);
                } else {
                    // Do nothing
                }
            }
            1 => {
                if wparamu == WM_KEYUP && (*kb_hook).vkCode == VK_RMENU.0.into() {
                    events.push(&(*kb_hook));
                    STAGE.store(2, Release);
                } else {
                    events.push(&(*kb_hook));
                    events.send(0);
                    STAGE.store(0, Release);
                }
                return LRESULT(1);
            }
            2 => {
                events.push(&(*kb_hook));
                if wparamu == WM_SYSKEYDOWN && (*kb_hook).vkCode == VK_RMENU.0.into() {
                    STAGE.store(3, Release);
                } else {
                    STAGE.store(254, Release);
                    // send to sequence tree
                }
                return LRESULT(1);
            }
            3 => {
                events.push(&(*kb_hook));
                if wparamu == WM_KEYUP && (*kb_hook).vkCode == VK_RMENU.0.into() {
                    STAGE.store(255, Release);
                    // send to search engine
                } else {
                    STAGE.store(254, Release);
                    // send to sequence tree
                }
                return LRESULT(1);
            }
            254 => {
                // send to sequence tree
            }
            255 => {
                //send to search engine
            }
            _ => {}
        }
    }

    CallNextHookEx(None, ncode, wparam, lparam)
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        unsafe { UnhookWindowsHookEx(self.h_hook).expect("cannot unhook!") };
        println!("hook successfully dropped");
    }
}
