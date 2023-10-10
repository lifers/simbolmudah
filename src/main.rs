mod composer;
mod key_event_sequence;
mod keyboard_hook;
mod keyboard_layout;
mod sequence;
mod settings;

use windows::{
    core::Result,
    Win32::UI::WindowsAndMessaging::{DispatchMessageA, GetMessageA, TranslateMessage, MSG},
};

fn run() -> Result<()> {
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
