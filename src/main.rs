mod composer;
mod keyboard_controller;
mod keyboard_hook;
mod keyboard_layout;
mod sequence;
mod settings;

use keyboard_hook::KeyboardHook;
use windows::{
    core::Result,
    Win32::{
        System::LibraryLoader::GetModuleHandleA,
        UI::WindowsAndMessaging::{DispatchMessageA, GetMessageA, TranslateMessage, MSG},
    },
};

fn run() -> Result<()> {
    let h_instance = unsafe { GetModuleHandleA(None).unwrap() };
    let kb_hook = KeyboardHook::new(h_instance);
    let mut msg = MSG::default();
    unsafe {
        while GetMessageA(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }
    }

    Ok(())
}

fn main() {
    println!("Hello, world!");
    let result = run();
    if let Err(error) = result {
        println!("{}", error.to_string());
    }
}
