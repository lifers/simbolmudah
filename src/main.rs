mod composer;
mod key;
mod key_sequence;
mod keyboard_hook;
mod settings;

use windows::{
    core::Result,
    Win32::{
        System::LibraryLoader::GetModuleHandleA,
        UI::WindowsAndMessaging::{
            DispatchMessageA, GetMessageA, PostQuitMessage, TranslateMessage, MSG,
        },
    },
};

use crate::keyboard_hook::{KeyboardHook, GLOBAL_HOOK};

fn run() -> Result<()> {
    let h_instance = unsafe { GetModuleHandleA(None).unwrap() };
    GLOBAL_HOOK.set(Some(KeyboardHook::new(h_instance)));

    ctrlc::set_handler(move || {
        println!("Ctrl-C received, exiting...");
        let _ = GLOBAL_HOOK.take();
        unsafe { PostQuitMessage(0) }
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

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
