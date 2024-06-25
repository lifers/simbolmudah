mod sequence_translator;

use crate::{bindings, delegate_storage::DelegateStorage, thread_handler::ThreadHandler};
use sequence_translator::{SequenceTranslator, SequenceTranslatorError};
use std::{
    collections::HashMap,
    ffi::c_void,
    hash::{DefaultHasher, Hash, Hasher},
    sync::{
        atomic::{
            AtomicIsize,
            Ordering::{Acquire, Release},
        },
        Arc, RwLock,
    },
};
use windows::{
    core::{implement, AgileReference, Error, IInspectable, Interface, Result, HRESULT, HSTRING},
    Foundation::{EventRegistrationToken, TypedEventHandler},
    Win32::{
        Foundation::{ERROR_NO_UNICODE_TRANSLATION, E_ACCESSDENIED, E_FAIL, E_INVALIDARG},
        System::WinRT::{IActivationFactory, IActivationFactory_Impl},
        UI::{
            Input::KeyboardAndMouse::{
                GetKeyboardLayout, ToUnicodeEx, VK_CAPITAL, VK_CONTROL, VK_MENU, VK_SHIFT, VK_SPACE,
            },
            TextServices::HKL,
            WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
        },
    },
};

#[derive(Clone)]
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
#[derive(Clone)]
struct KeyboardTranslator {
    thread_controller: Arc<ThreadHandler>,
    keyboard_layout: Arc<AtomicIsize>,
    report_invalid: Arc<RwLock<DelegateStorage<bindings::KeyboardTranslator, HSTRING>>>,
    report_translated: Arc<RwLock<DelegateStorage<bindings::KeyboardTranslator, HSTRING>>>,
    sequence_translator: Arc<RwLock<SequenceTranslator>>,
    possible_altgr: Arc<RwLock<HashMap<String, String>>>,
    possible_dead: Arc<RwLock<HashMap<String, u16>>>,
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
        let self_clone = (*self).clone();

        let success = self.thread_controller.try_enqueue(move || -> Result<()> {
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

            let to_forward = match vk_to_unicode(
                vkcode,
                scancode,
                &keystate,
                HKL(self_clone.keyboard_layout.load(Acquire)),
            ) {
                Ok(s) => Ok(s.into()),
                Err(e) => {
                    self_clone
                        .report_invalid
                        .write()
                        .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?
                        .invoke_all(
                            &self_clone.clone().into(),
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
            match forward(
                destination,
                to_forward,
                self_clone.sequence_translator.clone(),
            ) {
                Ok(s) => {
                    self_clone
                        .report_translated
                        .write()
                        .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?
                        .invoke_all(&self_clone.clone().into(), Some(&HSTRING::from(s)))
                }
                Err(TranslateError::ValueNotFound) => {
                    self_clone
                        .report_invalid
                        .write()
                        .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?
                        .invoke_all(
                            &self_clone.clone().into(),
                            Some(&HSTRING::from("Value not found")),
                        )
                }
                Err(TranslateError::InvalidDestination) => {
                    self_clone
                        .report_invalid
                        .write()
                        .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?
                        .invoke_all(
                            &self_clone.clone().into(),
                            Some(&HSTRING::from("Invalid destination")),
                        )
                }
                Err(TranslateError::DeadTranslator) => {
                    self_clone
                        .report_invalid
                        .write()
                        .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?
                        .invoke_all(
                            &self_clone.clone().into(),
                            Some(&HSTRING::from("Dead translator")),
                        )
                }
                Err(TranslateError::Incomplete) => {
                    // Do nothing
                    Ok(())
                }
            }
        })?;

        if success {
            Ok(())
        } else {
            Err(Error::new(E_FAIL, "Failed to enqueue task"))
        }
    }

    fn CheckLayoutAndUpdate(&self) -> Result<()> {
        let self_clone = (*self).clone();

        let success = self.thread_controller.try_enqueue(move || -> Result<()> {
            let foreground_window = unsafe { GetForegroundWindow() };
            let tid = unsafe { GetWindowThreadProcessId(foreground_window, None) };
            let active_layout = unsafe { GetKeyboardLayout(tid) };

            if active_layout.0 != self_clone.keyboard_layout.load(Acquire) {
                self_clone
                    .keyboard_layout
                    .store(active_layout.0 as isize, Release);
                analyze_layout(
                    self_clone.possible_altgr.clone(),
                    self_clone.possible_dead.clone(),
                    active_layout,
                )?;
            }
            Ok(())
        })?;

        if success {
            Ok(())
        } else {
            Err(Error::new(E_FAIL, "Failed to enqueue task"))
        }
    }

    fn BuildTranslator(&self) -> Result<()> {
        let sequence_translator = self.sequence_translator.clone();

        let success = self.thread_controller.try_enqueue(move || -> Result<()> {
            sequence_translator
                .write()
                .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?
                .build()
                .map_err(|e| <SequenceTranslatorError as Into<Error>>::into(e))
        })?;

        if success {
            Ok(())
        } else {
            Err(Error::new(E_FAIL, "Failed to enqueue task"))
        }
    }

    fn OnInvalid(
        &self,
        handler: Option<&TypedEventHandler<bindings::KeyboardTranslator, HSTRING>>,
    ) -> Result<EventRegistrationToken> {
        if let Some(handler) = handler {
            let token = get_token(handler.as_raw());
            let delegate_storage = self.report_invalid.clone();
            let handler_ref = Arc::new(AgileReference::new(handler)?);

            let success = self.thread_controller.try_enqueue(move || -> Result<()> {
                Ok(delegate_storage
                    .write()
                    .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?
                    .insert(token, handler_ref.clone()))
            })?;

            if success {
                Ok(EventRegistrationToken { Value: token })
            } else {
                Err(Error::new(E_FAIL, "Failed to enqueue task"))
            }
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

            let success = self.thread_controller.try_enqueue(move || -> Result<()> {
                Ok(delegate_storage
                    .write()
                    .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?
                    .insert(token, handler_ref.clone()))
            })?;

            if success {
                Ok(EventRegistrationToken { Value: token })
            } else {
                Err(Error::new(E_FAIL, "Failed to enqueue task"))
            }
        } else {
            Err(E_INVALIDARG.into())
        }
    }

    fn RemoveOnInvalid(&self, token: &EventRegistrationToken) -> Result<()> {
        let delegate_storage = self.report_invalid.clone();
        let value = token.Value;
        let success = self.thread_controller.try_enqueue(move || -> Result<()> {
            Ok(delegate_storage
                .write()
                .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?
                .remove(value))
        })?;

        if success {
            Ok(())
        } else {
            Err(Error::new(E_FAIL, "Failed to enqueue task"))
        }
    }

    fn RemoveOnTranslated(&self, token: &EventRegistrationToken) -> Result<()> {
        let delegate_storage = self.report_translated.clone();
        let value = token.Value;
        let success = self.thread_controller.try_enqueue(move || -> Result<()> {
            Ok(delegate_storage
                .write()
                .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?
                .remove(value))
        })?;

        if success {
            Ok(())
        } else {
            Err(Error::new(E_FAIL, "Failed to enqueue task"))
        }
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
                .write()
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

const EMPTY_KEYSTATE: [u8; 256] = [0; 256];

fn to_unicode_ex_clear_state() {
    let mut buffer = [0; 8];
    unsafe {
        ToUnicodeEx(
            VK_SPACE.0.into(),
            0,
            &EMPTY_KEYSTATE,
            &mut buffer,
            0,
            HKL(0),
        );
        ToUnicodeEx(
            VK_SPACE.0.into(),
            0,
            &EMPTY_KEYSTATE,
            &mut buffer,
            0,
            HKL(0),
        );
    }
}

fn analyze_layout(
    possible_altgr: Arc<RwLock<HashMap<String, String>>>,
    possible_dead: Arc<RwLock<HashMap<String, u16>>>,
    keyboard_layout: HKL,
) -> Result<()> {
    let mut possible_altgr = possible_altgr
        .write()
        .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?;
    let mut possible_dead = possible_dead
        .write()
        .map_err(|_| Error::new(E_ACCESSDENIED, "poisoned lock"))?;

    let mut no_altgr = vec![String::new(); 512];
    let mut keystate = [0; 256];

    for i in 0..0x400 {
        let vk_code = i & 0xFF;
        let has_shift = (i & 0x100) != 0;
        let has_altgr = (i & 0x200) != 0;

        if has_shift {
            keystate[VK_SHIFT.0 as usize] = 0x80;
        } else {
            keystate[VK_SHIFT.0 as usize] = 0;
        }

        if has_altgr {
            keystate[VK_CONTROL.0 as usize] = 0x80;
            keystate[VK_MENU.0 as usize] = 0x80;
        } else {
            keystate[VK_CONTROL.0 as usize] = 0;
            keystate[VK_MENU.0 as usize] = 0;
        }

        if let Ok(s) = vk_to_unicode(vk_code, 0, &keystate, keyboard_layout) {
            if has_altgr {
                possible_altgr.insert(s.clone().into(), no_altgr[i as usize - 0x200].clone());
            } else {
                no_altgr[i as usize] = s.clone().into();
            }

            if let StringVariant::DeadString(s) = s {
                possible_dead.insert(s, i as u16);
            }
        }

        to_unicode_ex_clear_state();
    }

    Ok(())
}

#[implement(IActivationFactory)]
pub(super) struct KeyboardTranslatorFactory;

// Default constructor for KeyboardTranslator WinRT class
impl IActivationFactory_Impl for KeyboardTranslatorFactory {
    fn ActivateInstance(&self) -> Result<IInspectable> {
        let instance = KeyboardTranslator {
            thread_controller: Arc::new(ThreadHandler::new()?),
            keyboard_layout: Arc::new(AtomicIsize::new(0)),
            report_invalid: Arc::new(RwLock::new(DelegateStorage::new())),
            report_translated: Arc::new(RwLock::new(DelegateStorage::new())),
            sequence_translator: Arc::new(RwLock::new(SequenceTranslator::default())),
            possible_altgr: Arc::new(RwLock::new(HashMap::new())),
            possible_dead: Arc::new(RwLock::new(HashMap::new())),
        };
        let instance: bindings::KeyboardTranslator = instance.into();
        Ok(instance.into())
    }
}
