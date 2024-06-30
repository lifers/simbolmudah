use std::mem::size_of;

use windows::{
    core::{Error, Result},
    Foundation::{AsyncStatus, IAsyncAction},
    System::Threading::{ThreadPool, WorkItemHandler},
    Win32::{
        Foundation::{GetLastError, E_ABORT, E_POINTER},
        UI::Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT},
    },
};

fn send(sent: &[INPUT]) -> Result<()> {
    unsafe {
        if SendInput(&sent, size_of::<INPUT>() as i32) != sent.len() as u32 {
            GetLastError().ok()
        } else {
            Ok(())
        }
    }
}

pub(crate) fn send_keybdinput(sent: Vec<KEYBDINPUT>) -> Result<IAsyncAction> {
    ThreadPool::RunAsync(&WorkItemHandler::new(move |a| {
        if let Some(a) = a {
            if a.Status()? == AsyncStatus::Canceled {
                return Err(Error::new(E_ABORT, "Operation canceled"));
            }

            let mut inputs = Vec::new();
            for ki in &sent {
                inputs.push(INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 { ki: *ki },
                });
            }
            
            send(&inputs)
        } else {
            Err(Error::new(E_POINTER, "Null pointer"))
        }
    }))
}
