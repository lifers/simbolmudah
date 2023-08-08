use std::{
    mem::transmute,
    sync::atomic::{AtomicU8, Ordering::Relaxed},
};

use windows::{
    core::Result,
    Win32::{
        Foundation::{GetLastError, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::KeyboardAndMouse::VK_RMENU,
            WindowsAndMessaging::{
                CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT,
                WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN,
            },
        },
    },
};

use crate::composer;

pub struct KeyboardHook {
    h_hook: HHOOK,
}

impl KeyboardHook {
    pub fn new() -> Result<Box<Self>> {
        let h_instance = unsafe { GetModuleHandleW(None)? };
        let h_hook = unsafe {
            SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(Self::low_level_keyboard_proc),
                h_instance,
                0,
            )?
        };

        let result = Box::new(Self { h_hook });

        Ok(result)
    }

    unsafe extern "system" fn low_level_keyboard_proc(
        ncode: i32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        // println!("{}", wparam.0);
        if ncode == 0 {
            static PRESSNO: AtomicU8 = AtomicU8::new(0);
            let times_pressed = PRESSNO.load(Relaxed);
            dbg!(times_pressed);
            let kb_hook = transmute::<isize, &KBDLLHOOKSTRUCT>(lparam.0);
            dbg!(kb_hook.vkCode);
            dbg!(wparam);

            if times_pressed == 0 && wparam.0 as u32 == WM_SYSKEYDOWN {
                if kb_hook.vkCode == VK_RMENU.0.into() {
                    PRESSNO.store(1, Relaxed);
                    return LRESULT(1);
                }
            } else if times_pressed == 1 {
                if wparam.0 as u32 == WM_KEYUP && kb_hook.vkCode == VK_RMENU.0.into() {
                    // right alt keyup, enter sequence
                    PRESSNO.store(2, Relaxed);
                    return LRESULT(1);
                } else {
                    // other key after right alt down, return input to system
                    let mut input = composer::InputSequence::default();
                    input.add_input(
                        composer::EventType::KeyDown,
                        composer::InputType::VirtualKey(VK_RMENU),
                    );
                    input.send();
                    PRESSNO.store(0, Relaxed);
                }
            } else if times_pressed > 1 && wparam.0 as u32 == WM_KEYDOWN {
                if times_pressed == 2 && kb_hook.vkCode == 0x31 {
                    PRESSNO.store(3, Relaxed);
                    return LRESULT(1);
                } else if times_pressed == 3 && kb_hook.vkCode == 0x32 {
                    let mut input = composer::InputSequence::default();
                    input.add_input(
                        composer::EventType::KeyUpDown,
                        composer::InputType::Unicode(0x00BD),
                    );
                    input.send();
                    println!("compose successful 🎉🎉");
                    PRESSNO.store(0, Relaxed);
                    return LRESULT(1);
                }
            }
            dbg!(PRESSNO.load(Relaxed));
        }

        CallNextHookEx(None, ncode, wparam, lparam)
    }

    fn unhook(&mut self) -> Result<()> {
        unsafe {
            if !UnhookWindowsHookEx(self.h_hook).as_bool() {
                return GetLastError().ok();
            }
        }

        Ok(())
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        self.unhook().expect("hook cannot be unhooked");
        println!("hook successfully dropped");
    }
}
