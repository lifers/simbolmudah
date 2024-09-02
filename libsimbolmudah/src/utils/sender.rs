use std::mem::size_of;

use windows::{
    core::{h, implement, Error, IInspectable, Result, HSTRING},
    Foundation::{AsyncStatus, IAsyncAction},
    System::Threading::{ThreadPool, WorkItemHandler},
    Win32::{
        Foundation::{GetLastError, E_ABORT, E_POINTER, HWND, LPARAM, LRESULT, WPARAM},
        System::WinRT::{IActivationFactory, IActivationFactory_Impl},
        UI::{
            Input::KeyboardAndMouse::{
                SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_CONTROL,
                VK_V,
            },
            WindowsAndMessaging::DefWindowProcW,
        },
    },
};

use super::{clipboard::Clipboard, message_window::MessageWindow};

use crate::bindings;

#[implement(IActivationFactory, bindings::ISenderStatics)]
pub(crate) struct Sender;

impl IActivationFactory_Impl for Sender_Impl {
    fn ActivateInstance(&self) -> Result<IInspectable> {
        Ok(Sender.into())
    }
}

impl bindings::ISenderStatics_Impl for Sender_Impl {
    fn SendTextClipboard(&self, message: &HSTRING) -> Result<IAsyncAction> {
        send_text_clipboard(message)
    }
}

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

pub(crate) fn send_text_clipboard(message: &HSTRING) -> Result<IAsyncAction> {
    let text = message.clone();
    ThreadPool::RunAsync(&WorkItemHandler::new(move |a| {
        if let Some(a) = a {
            if a.Status()? == AsyncStatus::Canceled {
                return Err(Error::new(E_ABORT, "Operation canceled"));
            }

            {
                unsafe extern "system" fn wnd_proc(
                    hwnd: HWND,
                    msg: u32,
                    w_param: WPARAM,
                    l_param: LPARAM,
                ) -> LRESULT {
                    DefWindowProcW(hwnd, msg, w_param, l_param)
                }

                // Create message only window
                let h_wnd = MessageWindow::new(h!("LibSimbolMudah.Clipboard"), Some(wnd_proc))?;

                // Get clipboard access
                let clipboard = Clipboard::new(h_wnd.handle())?;

                // Copy text to clipboard
                clipboard.set_text(&text)?;
            }

            // Simulate Ctrl+V
            let ctrl_down = KEYBDINPUT {
                wVk: VK_CONTROL,
                ..Default::default()
            };
            let v_down = KEYBDINPUT {
                wVk: VK_V,
                ..Default::default()
            };
            let v_up = KEYBDINPUT {
                wVk: VK_V,
                dwFlags: KEYEVENTF_KEYUP,
                ..Default::default()
            };
            let ctrl_up = KEYBDINPUT {
                wVk: VK_CONTROL,
                dwFlags: KEYEVENTF_KEYUP,
                ..Default::default()
            };
            send(&[
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 { ki: ctrl_down },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 { ki: v_down },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 { ki: v_up },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 { ki: ctrl_up },
                },
            ])
        } else {
            Err(Error::new(E_POINTER, "Null pointer"))
        }
    }))
}
