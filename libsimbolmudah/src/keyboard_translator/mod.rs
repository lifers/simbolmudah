mod sequence_translator;

use crate::{bindings, delegate_storage::DelegateStorage};
use sequence_translator::SequenceTranslator;
use std::{
    ffi::c_void,
    hash::{DefaultHasher, Hash, Hasher},
    sync::{
        atomic::{AtomicIsize, Ordering::Acquire},
        Arc, RwLock,
    },
};
use windows::{
    core::{implement, AgileReference, Error, IInspectable, Interface, Result, HRESULT, HSTRING},
    Foundation::{EventRegistrationToken, TypedEventHandler},
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

impl Into<String> for StringVariant {
    fn into(self) -> String {
        match self {
            StringVariant::LiveString(s) => s,
            StringVariant::DeadString(s) => s,
        }
    }
}

enum VKToUnicodeError {
    InvalidReturn,
    NoTranslation,
}

enum TranslateError {
    ValueNotFound,
    Incomplete,
    InvalidDestination,
    DeadTranslator,
}

#[implement(bindings::KeyboardTranslator)]
struct KeyboardTranslator {
    thread_controller: DispatcherQueueController,
    keyboard_layout: AtomicIsize,
    report_invalid: Arc<RwLock<DelegateStorage<bindings::KeyboardTranslator, HSTRING>>>,
    report_translated: Arc<RwLock<DelegateStorage<bindings::KeyboardTranslator, HSTRING>>>,
    sequence_translator: Arc<RwLock<SequenceTranslator>>,
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
        let report_invalid = self.report_invalid.clone();
        let report_translated = self.report_translated.clone();
        let sequence_translator = self.sequence_translator.clone();
        let self_ref = Arc::new(AgileReference::new(&unsafe {
            self.cast::<bindings::KeyboardTranslator>()
        }?)?);

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
                    Ok(s) => Ok(s.into()),
                    Err(e) => {
                        report_invalid
                            .write()
                            .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?
                            .invoke_all(
                                self_ref.clone(),
                                Some(&HSTRING::from("Invalid VK code")),
                            )?;

                        match e {
                            VKToUnicodeError::NoTranslation => Err(E_INVALIDARG.into()),
                            VKToUnicodeError::InvalidReturn => Err(Error::new(
                                HRESULT::from_win32(ERROR_NO_UNICODE_TRANSLATION.0),
                                "Invalid return from ToUnicodeEx",
                            )),
                        }
                    }
                }?;

                // TODO: Forward to SequenceTranslator (destination 0) or UnicodeTranslator (destination 1)
                match forward(destination, to_forward, sequence_translator.clone()) {
                    Ok(s) => {
                        report_translated
                            .write()
                            .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?
                            .invoke_all(self_ref.clone(), Some(&HSTRING::from(s)))?;
                    }
                    Err(TranslateError::ValueNotFound) => {
                        report_invalid
                            .write()
                            .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?
                            .invoke_all(self_ref.clone(), Some(&HSTRING::from("Value not found")))?;
                    }
                    Err(TranslateError::InvalidDestination) => {
                        report_invalid
                            .write()
                            .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?
                            .invoke_all(
                                self_ref.clone(),
                                Some(&HSTRING::from("Invalid destination")),
                            )?;
                    }
                    Err(TranslateError::DeadTranslator) => {
                        report_invalid
                            .write()
                            .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?
                            .invoke_all(self_ref.clone(), Some(&HSTRING::from("Dead translator")))?;
                    }
                    Err(TranslateError::Incomplete) => {
                        // Do nothing
                    }
                }
                Err(E_NOTIMPL.into())
            }))?;
        Ok(())
    }

    fn CheckLayoutAndUpdate(&self) -> Result<()> {
        Err(E_NOTIMPL.into())
    }

    fn OnInvalid(
        &self,
        handler: Option<&TypedEventHandler<bindings::KeyboardTranslator, HSTRING>>,
    ) -> Result<EventRegistrationToken> {
        if let Some(handler) = handler {
            let token = get_token(handler.as_raw());
            let delegate_storage = self.report_invalid.clone();
            let handler_ref = Arc::new(AgileReference::new(handler)?);

            self.thread_controller
                .DispatcherQueue()?
                .TryEnqueue(&DispatcherQueueHandler::new(move || -> Result<()> {
                    Ok(delegate_storage
                        .write()
                        .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?
                        .insert(token, handler_ref.clone()))
                }))?;

            Ok(EventRegistrationToken { Value: token })
        } else {
            Err(E_INVALIDARG.into())
        }
    }

    fn OnTranslated(
        &self,
        handler: Option<&TypedEventHandler<bindings::KeyboardTranslator, HSTRING>>,
    ) -> Result<EventRegistrationToken> {
        if let Some(handler) = handler {
            let token = get_token(handler.as_raw());
            let delegate_storage = self.report_translated.clone();
            let handler_ref = Arc::new(AgileReference::new(handler)?);

            self.thread_controller
                .DispatcherQueue()?
                .TryEnqueue(&DispatcherQueueHandler::new(move || -> Result<()> {
                    Ok(delegate_storage
                        .write()
                        .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?
                        .insert(token, handler_ref.clone()))
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
                Ok(delegate_storage
                    .write()
                    .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?
                    .remove(value))
            }))?;
        Ok(())
    }

    fn RemoveOnTranslated(&self, token: &EventRegistrationToken) -> Result<()> {
        let delegate_storage = self.report_translated.clone();
        let value = token.Value;
        self.thread_controller
            .DispatcherQueue()?
            .TryEnqueue(&DispatcherQueueHandler::new(move || -> Result<()> {
                Ok(delegate_storage
                    .write()
                    .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?
                    .remove(value))
            }))?;
        Ok(())
    }
}

impl Drop for KeyboardTranslator {
    fn drop(&mut self) {
        let _ = self.thread_controller.ShutdownQueueAsync();
    }
}

fn forward(
    destination: u8,
    value: String,
    sequence_translator: Arc<RwLock<SequenceTranslator>>,
) -> std::result::Result<String, TranslateError> {
    match destination {
        0 => {
            // Forward to SequenceTranslator
            sequence_translator
                .read()
                .map_err(|_| TranslateError::DeadTranslator)?
                .translate(&value)
        }
        1 => {
            // Forward to UnicodeTranslator
            Err(TranslateError::ValueNotFound)
        }
        _ => Err(TranslateError::InvalidDestination),
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
        let instance = KeyboardTranslator {
            thread_controller: DispatcherQueueController::CreateOnDedicatedThread()?,
            keyboard_layout: AtomicIsize::new(0),
            report_invalid: Arc::new(RwLock::new(DelegateStorage::new())),
            report_translated: Arc::new(RwLock::new(DelegateStorage::new())),
            sequence_translator: Arc::new(RwLock::new(SequenceTranslator::new())),
        };
        let instance: bindings::KeyboardTranslator = instance.into();
        Ok(instance.into())
    }
}
