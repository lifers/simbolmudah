mod sequence_translator;

use crate::{
    bindings,
    delegate_storage::{get_token, DelegateStorage},
    thread_handler::ThreadHandler,
};
use sequence_translator::SequenceTranslator;
use std::{
    collections::HashMap,
    ffi::c_void,
    ptr::null_mut,
    sync::{
        atomic::{
            AtomicPtr,
            Ordering::{Acquire, Release},
        },
        Arc, RwLock,
    },
};
use windows::{
    core::{implement, AgileReference, Error, IInspectable, Interface, Result, HRESULT, HSTRING},
    Foundation::{EventRegistrationToken, TypedEventHandler},
    Win32::{
        Foundation::{ERROR_NO_UNICODE_TRANSLATION, E_ACCESSDENIED, E_INVALIDARG},
        System::WinRT::{IActivationFactory, IActivationFactory_Impl},
        UI::{
            Input::KeyboardAndMouse::{
                GetKeyboardLayout, ToUnicodeEx, HKL, VK_CAPITAL, VK_CONTROL, VK_MENU, VK_SHIFT,
                VK_SPACE,
            },
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

#[derive(Debug, PartialEq)]
enum TranslateError {
    ValueNotFound,
    Incomplete,
    InvalidDestination,
}

#[implement(bindings::KeyboardTranslator)]
#[derive(Debug)]
pub(crate) struct KeyboardTranslator {
    internal: Arc<RwLock<KeyboardTranslatorInternal>>,
    thread_controller: ThreadHandler,
}

#[derive(Debug)]
struct KeyboardTranslatorInternal {
    keyboard_layout: AtomicPtr<c_void>,
    report_invalid: DelegateStorage<bindings::KeyboardTranslator, HSTRING>,
    report_translated: DelegateStorage<bindings::KeyboardTranslator, HSTRING>,
    sequence_translator: SequenceTranslator,
    possible_altgr: HashMap<String, String>,
    possible_dead: HashMap<String, u16>,
    parent: Option<bindings::KeyboardTranslator>,
}

impl KeyboardTranslatorInternal {
    fn new() -> Self {
        Self {
            keyboard_layout: AtomicPtr::new(null_mut()),
            report_invalid: DelegateStorage::new(),
            report_translated: DelegateStorage::new(),
            sequence_translator: SequenceTranslator::default(),
            possible_altgr: HashMap::new(),
            possible_dead: HashMap::new(),
            parent: None,
        }
    }

    fn translate(&mut self, vkcode: u32, scancode: u32, keystate: &[u8; 256]) -> Result<String> {
        match vk_to_unicode(
            vkcode,
            scancode,
            &keystate,
            HKL(self.keyboard_layout.load(Acquire)),
        ) {
            Ok(s) => Ok(s.into()),
            Err(e) => {
                self.report_invalid.invoke_all(
                    &self.parent.as_ref().unwrap(),
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
        }
    }

    fn forward(
        &mut self,
        destination: u8,
        value: String,
    ) -> std::result::Result<String, TranslateError> {
        match destination {
            0 => {
                // Forward to SequenceTranslator
                self.sequence_translator.translate(&value)
            }
            1 => {
                // Forward to UnicodeTranslator
                Err(TranslateError::ValueNotFound)
            }
            _ => Err(TranslateError::InvalidDestination),
        }
    }

    fn report(&mut self, result: std::result::Result<String, TranslateError>) -> Result<()> {
        match result {
            Ok(s) => self
                .report_translated
                .invoke_all(&self.parent.as_ref().unwrap(), Some(&HSTRING::from(s))),
            Err(TranslateError::ValueNotFound) => self.report_invalid.invoke_all(
                &self.parent.as_ref().unwrap(),
                Some(&HSTRING::from("Value not found")),
            ),
            Err(TranslateError::InvalidDestination) => self.report_invalid.invoke_all(
                &self.parent.as_ref().unwrap(),
                Some(&HSTRING::from("Invalid destination")),
            ),
            Err(TranslateError::Incomplete) => {
                // Do nothing
                Ok(())
            }
        }
    }

    fn analyze_layout(&mut self) -> Result<()> {
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

            if let Ok(s) = vk_to_unicode(
                vk_code,
                0,
                &keystate,
                HKL(self.keyboard_layout.load(Acquire)),
            ) {
                if has_altgr {
                    self.possible_altgr
                        .insert(s.clone().into(), no_altgr[i as usize - 0x200].clone());
                } else {
                    no_altgr[i as usize] = s.clone().into();
                }

                if let StringVariant::DeadString(s) = s {
                    self.possible_dead.insert(s, i as u16);
                }
            }

            to_unicode_ex_clear_state();
        }

        Ok(())
    }
}

impl bindings::IKeyboardTranslator_Impl for KeyboardTranslator_Impl {
    fn TranslateAndForward(
        &self,
        vkcode: u32,
        scancode: u32,
        hascapslock: bool,
        hasshift: bool,
        hasaltgr: bool,
        destination: u8,
    ) -> Result<()> {
        let internal = self.internal.clone();
        self.thread_controller.try_enqueue(move || {
            let mut internal = internal
                .try_write()
                .map_err(|e| Error::new(E_ACCESSDENIED, format!("{:?}", e)))?;

            let keystate = calculate_bg_keystate(hascapslock, hasshift, hasaltgr);
            let value = internal.translate(vkcode, scancode, &keystate)?;
            let result = internal.forward(destination, value);
            internal.report(result)
        })
    }

    fn CheckLayoutAndUpdate(&self) -> Result<()> {
        let internal = self.internal.clone();
        self.thread_controller.try_enqueue_high(move || {
            {
                let lock = internal
                    .try_read()
                    .map_err(|e| Error::new(E_ACCESSDENIED, format!("{:?}", e)))?;

                let foreground_window = unsafe { GetForegroundWindow() };
                let tid = unsafe { GetWindowThreadProcessId(foreground_window, None) };
                let active_layout = unsafe { GetKeyboardLayout(tid) };

                if active_layout.0 == lock.keyboard_layout.load(Acquire) {
                    return Ok(());
                }

                lock.keyboard_layout.store(active_layout.0, Release);
            }

            internal
                .try_write()
                .map_err(|e| Error::new(E_ACCESSDENIED, format!("{:?}", e)))?
                .analyze_layout()
        })
    }

    fn BuildTranslator(&self, keysymdef_path: &HSTRING, composedef_path: &HSTRING) -> Result<()> {
        let internal = self.internal.clone();
        let keysymdef_path = keysymdef_path.to_string();
        let composedef_path = composedef_path.to_string();
        self.thread_controller.try_enqueue(move || {
            let mut lock = internal
                .try_write()
                .map_err(|e| Error::new(E_ACCESSDENIED, format!("{:?}", e)))?;

            lock.sequence_translator
                .build(&keysymdef_path, &composedef_path)?;

            let parent = lock.parent.clone();
            lock.report_translated.invoke_all(
                &parent.as_ref().unwrap(),
                Some(&HSTRING::from("Build Successful! 🎉")),
            )
        })
    }

    fn OnInvalid(
        &self,
        handler: Option<&TypedEventHandler<bindings::KeyboardTranslator, HSTRING>>,
    ) -> Result<EventRegistrationToken> {
        if let Some(handler) = handler {
            let token = get_token(handler.as_raw());
            let internal = self.internal.clone();
            let handler_ref = AgileReference::new(handler)?;

            self.thread_controller.try_enqueue(move || -> Result<()> {
                Ok(internal
                    .try_write()
                    .map_err(|e| Error::new(E_ACCESSDENIED, format!("{:?}", e)))?
                    .report_invalid
                    .insert(token, handler_ref.clone()))
            })?;

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
            let internal = self.internal.clone();
            let handler_ref = AgileReference::new(handler)?;

            self.thread_controller.try_enqueue(move || -> Result<()> {
                Ok(internal
                    .try_write()
                    .map_err(|e| Error::new(E_ACCESSDENIED, format!("{:?}", e)))?
                    .report_translated
                    .insert(token, handler_ref.clone()))
            })?;

            Ok(EventRegistrationToken { Value: token })
        } else {
            Err(E_INVALIDARG.into())
        }
    }

    fn RemoveOnInvalid(&self, token: &EventRegistrationToken) -> Result<()> {
        let internal = self.internal.clone();
        let value = token.Value;
        self.thread_controller.try_enqueue(move || -> Result<()> {
            Ok(internal
                .try_write()
                .map_err(|e| Error::new(E_ACCESSDENIED, format!("{:?}", e)))?
                .report_invalid
                .remove(value))
        })
    }

    fn RemoveOnTranslated(&self, token: &EventRegistrationToken) -> Result<()> {
        let internal = self.internal.clone();
        let value = token.Value;
        self.thread_controller.try_enqueue(move || -> Result<()> {
            Ok(internal
                .try_write()
                .map_err(|e| Error::new(E_ACCESSDENIED, format!("{:?}", e)))?
                .report_translated
                .remove(value))
        })
    }
}

const fn calculate_bg_keystate(has_capslock: bool, has_shift: bool, has_altgr: bool) -> [u8; 256] {
    let mut keystate = [0; 256];
    if has_capslock {
        keystate[VK_CAPITAL.0 as usize] = 1;
    }
    if has_shift {
        keystate[VK_SHIFT.0 as usize] = 0x80;
    }
    if has_altgr {
        keystate[VK_CONTROL.0 as usize] = 0x80;
        keystate[VK_MENU.0 as usize] = 0x80;
    }

    keystate
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
            HKL(null_mut()),
        );
        ToUnicodeEx(
            VK_SPACE.0.into(),
            0,
            &EMPTY_KEYSTATE,
            &mut buffer,
            0,
            HKL(null_mut()),
        );
    }
}

#[implement(IActivationFactory)]
pub(super) struct KeyboardTranslatorFactory;

// Default constructor for KeyboardTranslator WinRT class
impl IActivationFactory_Impl for KeyboardTranslatorFactory_Impl {
    fn ActivateInstance(&self) -> Result<IInspectable> {
        let instance = KeyboardTranslator {
            internal: Arc::new(RwLock::new(KeyboardTranslatorInternal::new())),
            thread_controller: ThreadHandler::new().expect("Thread controller should be created"),
        };
        let binding: bindings::KeyboardTranslator = instance.into();
        let casted_binding = binding.cast_object::<KeyboardTranslator>()?;
        casted_binding.internal.try_write().unwrap().parent = Some(binding.clone());
        Ok(binding.into())
    }
}

#[cfg(test)]
mod tests {
    use bindings::IKeyboardTranslator_Impl;
    use windows::System::DispatcherQueue;
    use windows_core::ComObject;

    use super::*;
    use std::sync::{
        atomic::{
            AtomicBool,
            Ordering::{AcqRel, Acquire},
        },
        Arc,
    };

    #[test]
    fn test_activate_instance() {
        // Create a new instance of KeyboardTranslator
        let instance = bindings::KeyboardTranslator::new().expect("Instance should be created");
        let instance = instance
            .cast_object::<KeyboardTranslator>()
            .expect("Should be castable");
        assert!(instance.internal.read().is_ok());
    }

    #[test]
    fn test_on_invalid() {
        // Create a new instance of KeyboardTranslator
        let instance = bindings::KeyboardTranslator::new().expect("Instance should be created");
        let instance: ComObject<KeyboardTranslator> =
            instance.cast_object().expect("Should be castable");

        // Create a flag to check if the event handler was called
        let flag = Arc::new(AtomicBool::new(false));
        let clone = flag.clone();

        // Create a mock event handler
        let handler = TypedEventHandler::new(
            move |sender: &Option<bindings::KeyboardTranslator>, result| {
                assert!(sender.is_some());
                assert_eq!(result, &HSTRING::from("Value not found"));
                assert!(!clone.fetch_or(true, AcqRel));
                Ok(())
            },
        );

        // Register the event handler
        let _token = instance
            .OnInvalid(Some(&handler))
            .expect("Event handler should be registered");

        // Check flag on ShutdownCompleted event
        let complete_handler =
            TypedEventHandler::new(move |sender: &Option<DispatcherQueue>, _| {
                assert!(sender.is_some());
                assert!(flag.load(Acquire));
                Ok(())
            });

        // Register the event handler
        let complete_token = instance
            .thread_controller
            .register_shutdown_complete_callback(Some(&complete_handler))
            .expect("Event handler should be registered");

        // Trigger the event
        instance
            .TranslateAndForward(0x41, 0, false, false, false, 1)
            .expect("Should be successful");

        // Wait until event handler is finished
        instance
            .thread_controller
            .disable()
            .expect("Thread handler should be disabled");

        // Unregister the event handler
        // instance
        //     .RemoveOnInvalid(&token)
        //     .expect("Event handler should be unregistered");
        instance
            .thread_controller
            .unregister_shutdown_complete_callback(complete_token)
            .expect("Event handler should be unregistered");
    }

    #[test]
    fn test_enqueue_task() {
        // Create a flag to check if the task was executed
        let flag = Arc::new(AtomicBool::new(false));
        let clone = flag.clone();

        // Create a thread handler
        let thread_handler =
            Arc::new(ThreadHandler::new().expect("Thread handler should be created"));

        // Enqueue a task that sets the flag to true
        thread_handler
            .try_enqueue(move || {
                assert!(!clone.fetch_or(true, AcqRel));
                Ok(())
            })
            .expect("Task should be enqueued");

        let clone = flag.clone();

        let complete_handler =
            TypedEventHandler::new(move |sender: &Option<DispatcherQueue>, _| {
                assert!(sender.is_some());
                assert!(clone.load(Acquire));
                Ok(())
            });

        // Register the event handler
        let complete_token = thread_handler
            .register_shutdown_complete_callback(Some(&complete_handler))
            .expect("Event handler should be registered");

        // Wait for the task to complete
        thread_handler
            .disable()
            .expect("Thread handler should be disabled, all tasks should be completed");

        // Unregister the event handler
        thread_handler
            .unregister_shutdown_complete_callback(complete_token)
            .expect("Event handler should be unregistered");
    }
}
