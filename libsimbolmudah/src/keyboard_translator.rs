use std::{
    ffi::c_void,
    hash::{DefaultHasher, Hash, Hasher},
    sync::{
        atomic::{AtomicIsize, Ordering::Acquire},
        Arc, RwLock,
    },
};

use crate::{bindings, delegate_storage::DelegateStorage};
use windows::{
    core::{implement, AgileReference, Error, IInspectable, Interface, Result, HRESULT, HSTRING},
    Foundation::{EventHandler, EventRegistrationToken},
    System::{DispatcherQueueController, DispatcherQueueHandler},
    Win32::{
        Foundation::{ERROR_NO_UNICODE_TRANSLATION, E_ACCESSDENIED, E_INVALIDARG, E_NOTIMPL},
        System::WinRT::{IActivationFactory, IActivationFactory_Impl},
        UI::{
            Input::KeyboardAndMouse::{ToUnicodeEx, VK_CAPITAL, VK_CONTROL, VK_MENU, VK_SHIFT},
            TextServices::HKL,
        },
    },
};

enum StringVariant {
    LiveString(String),
    DeadString(String),
}

enum VKToUnicodeError {
    InvalidReturn,
    NoTranslation,
}

#[implement(bindings::KeyboardTranslator)]
struct KeyboardTranslator {
    thread_controller: DispatcherQueueController,
    keyboard_layout: AtomicIsize,
    report_invalid: Arc<RwLock<DelegateStorage<HSTRING>>>,
    report_translated: Arc<RwLock<DelegateStorage<HSTRING>>>,
}

impl bindings::IKeyboardTranslator_Impl for KeyboardTranslator {
    fn TranslateAndForward(
        &self,
        vkcode: u32,
        scancode: u32,
        hascapslock: bool,
        hasshift: bool,
        hasaltgr: bool,
        destination: u8,
    ) -> Result<()> {
        let current_layout = HKL(self.keyboard_layout.load(Acquire));
        self.thread_controller
            .DispatcherQueue()?
            .TryEnqueue(&DispatcherQueueHandler::new(move || -> Result<()> {
                let mut keystate = [0; 256];
                if hascapslock {
                    keystate[VK_CAPITAL.0 as usize] = 1;
                }
                if hasshift {
                    keystate[VK_SHIFT.0 as usize] = 0x80;
                }
                if hasaltgr {
                    keystate[VK_CONTROL.0 as usize] = 0x80;
                    keystate[VK_MENU.0 as usize] = 0x80;
                }

                let to_forward = match vk_to_unicode(vkcode, scancode, &keystate, current_layout) {
                    Ok(StringVariant::LiveString(s)) => Ok(s),
                    Ok(StringVariant::DeadString(s)) => Ok(s),
                    Err(VKToUnicodeError::NoTranslation) => Err(E_INVALIDARG.into()),
                    Err(VKToUnicodeError::InvalidReturn) => Err(Error::new(
                        HRESULT::from_win32(ERROR_NO_UNICODE_TRANSLATION.0),
                        "Invalid return from ToUnicodeEx",
                    )),
                }?;

                // TODO: Forward to SequenceTranslator (destination 0) or UnicodeTranslator (destination 1)
                Err(E_NOTIMPL.into())
            }))?;
        Ok(())
    }

    fn CheckLayoutAndUpdate(&self) -> Result<()> {
        Err(E_NOTIMPL.into())
    }

    fn OnInvalid(&self, handler: Option<&EventHandler<HSTRING>>) -> Result<EventRegistrationToken> {
        if let Some(handler) = handler {
            let token = get_token(handler.as_raw());
            let delegate_storage = self.report_invalid.clone();
            let handler_ref = Arc::new(AgileReference::new(handler)?);

            self.thread_controller
                .DispatcherQueue()?
                .TryEnqueue(&DispatcherQueueHandler::new(move || -> Result<()> {
                    delegate_storage
                        .write()
                        .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?
                        .insert(token, handler_ref.clone());
                    Ok(())
                }))?;

            Ok(EventRegistrationToken { Value: token })
        } else {
            Err(E_INVALIDARG.into())
        }
    }

    fn OnTranslated(
        &self,
        handler: Option<&EventHandler<HSTRING>>,
    ) -> Result<EventRegistrationToken> {
        if let Some(handler) = handler {
            let token = get_token(handler.as_raw());
            let delegate_storage = self.report_translated.clone();
            let handler_ref = Arc::new(AgileReference::new(handler)?);

            self.thread_controller
                .DispatcherQueue()?
                .TryEnqueue(&DispatcherQueueHandler::new(move || -> Result<()> {
                    delegate_storage
                        .write()
                        .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?
                        .insert(token, handler_ref.clone());
                    Ok(())
                }))?;

            Ok(EventRegistrationToken { Value: token })
        } else {
            Err(E_INVALIDARG.into())
        }
    }

    fn RemoveOnInvalid(&self, token: &EventRegistrationToken) -> Result<()> {
        let delegate_storage = self.report_invalid.clone();
        let value = token.Value;
        self.thread_controller
            .DispatcherQueue()?
            .TryEnqueue(&DispatcherQueueHandler::new(move || -> Result<()> {
                delegate_storage
                    .write()
                    .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?
                    .remove(value);
                Ok(())
            }))?;
        Ok(())
    }

    fn RemoveOnTranslated(&self, token: &EventRegistrationToken) -> Result<()> {
        let delegate_storage = self.report_translated.clone();
        let value = token.Value;
        self.thread_controller
            .DispatcherQueue()?
            .TryEnqueue(&DispatcherQueueHandler::new(move || -> Result<()> {
                delegate_storage
                    .write()
                    .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?
                    .remove(value);
                Ok(())
            }))?;
        Ok(())
    }
}

impl Drop for KeyboardTranslator {
    fn drop(&mut self) {
        let _ = self.thread_controller.ShutdownQueueAsync();
    }
}

fn get_token(handler: *mut c_void) -> i64 {
    // Generate a unique token
    let mut hasher = DefaultHasher::new();
    handler.hash(&mut hasher);
    hasher.finish() as i64
}

fn vk_to_unicode(
    vkcode: u32,
    scancode: u32,
    keystate: &[u8; 256],
    keyboard_layout: HKL,
) -> std::result::Result<StringVariant, VKToUnicodeError> {
    let mut buffer = [0; 8];
    let status =
        unsafe { ToUnicodeEx(vkcode, scancode, keystate, &mut buffer, 4, keyboard_layout) };

    if status > 0 {
        Ok(StringVariant::LiveString(
            String::from_utf16(&buffer[..status as usize])
                .map_err(|_| VKToUnicodeError::InvalidReturn)?,
        ))
    } else if status < 0 {
        let status =
            unsafe { ToUnicodeEx(vkcode, scancode, keystate, &mut buffer, 4, keyboard_layout) };

        if status > 0 {
            Ok(StringVariant::DeadString(
                String::from_utf16(&buffer[..status as usize])
                    .map_err(|_| VKToUnicodeError::InvalidReturn)?,
            ))
        } else {
            Err(VKToUnicodeError::NoTranslation)
        }
    } else {
        Err(VKToUnicodeError::NoTranslation)
    }
}

#[implement(IActivationFactory)]
pub(super) struct KeyboardTranslatorFactory;

// Default constructor for KeyboardTranslator WinRT class
impl IActivationFactory_Impl for KeyboardTranslatorFactory {
    fn ActivateInstance(&self) -> Result<IInspectable> {
        let instance: bindings::KeyboardTranslator = KeyboardTranslator {
            thread_controller: DispatcherQueueController::CreateOnDedicatedThread()?,
            keyboard_layout: AtomicIsize::new(0),
            report_invalid: Arc::new(RwLock::new(DelegateStorage::new())),
            report_translated: Arc::new(RwLock::new(DelegateStorage::new())),
        }
        .into();
        Ok(instance.into())
    }
}
