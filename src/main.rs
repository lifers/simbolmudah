mod composer;
mod keyboard_hook;

use windows::{
    core::Result,
    Win32::UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, TranslateMessage, MSG},
};

use keyboard_hook::KeyboardHook;

fn run() -> Result<()> {
    let kb_hook = KeyboardHook::new()?;

    let mut msg = MSG::default();
    unsafe {
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
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
