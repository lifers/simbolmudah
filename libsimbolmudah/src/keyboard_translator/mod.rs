mod internal;

use crate::{bindings, delegate_storage::get_token, fail, thread_handler::ThreadHandler};
use internal::KeyboardTranslatorInternal;
use std::sync::{
    atomic::Ordering::{Acquire, Release},
    Arc, RwLock,
};
use windows::{
    core::{implement, AgileReference, Error, IInspectable, Interface, Result, Weak, HSTRING},
    Foundation::{EventRegistrationToken, TypedEventHandler},
    Win32::{
        Foundation::{E_ACCESSDENIED, E_INVALIDARG, E_NOTIMPL, E_POINTER},
        System::WinRT::{IActivationFactory, IActivationFactory_Impl},
        UI::{
            Input::KeyboardAndMouse::{
                GetKeyboardLayout, VK_CAPITAL, VK_CONTROL, VK_MENU, VK_SHIFT,
            },
            WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
        },
    },
};

#[implement(bindings::KeyboardTranslator)]
#[derive(Debug)]
pub(crate) struct KeyboardTranslator {
    internal: Arc<RwLock<KeyboardTranslatorInternal>>,
    thread_controller: ThreadHandler,
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

#[implement(IActivationFactory, bindings::IKeyboardTranslatorFactory)]
pub(super) struct KeyboardTranslatorFactory;

// Default constructor for KeyboardTranslator WinRT class
impl IActivationFactory_Impl for KeyboardTranslatorFactory_Impl {
    fn ActivateInstance(&self) -> Result<IInspectable> {
        Err(E_NOTIMPL.into())
    }
}

impl bindings::IKeyboardTranslatorFactory_Impl for KeyboardTranslatorFactory_Impl {
    fn CreateInstance(
        &self,
        definition: Option<&bindings::SequenceDefinition>,
    ) -> Result<bindings::KeyboardTranslator> {
        let definition = definition.ok_or(Error::new(E_POINTER, "Null pointer"))?;
        let instance: bindings::KeyboardTranslator = KeyboardTranslator {
            internal: Arc::new(RwLock::new(KeyboardTranslatorInternal::new(
                Weak::new(),
                Weak::new(),
            ))),
            thread_controller: ThreadHandler::new().expect("Thread controller should be created"),
        }
        .into();

        {
            let mut internal = instance
                .cast_object_ref::<KeyboardTranslator>()?
                .internal
                .try_write()
                .map_err(fail)?;
            internal.sequence_definition = definition.downgrade()?;
            internal.parent = instance.downgrade()?;
        }

        Ok(instance)
    }
}

#[cfg(test)]
mod tests {
    use bindings::IKeyboardTranslator_Impl;
    use windows::System::DispatcherQueue;
    use windows_core::AsImpl;

    use crate::sequence_definition::SequenceDefinitionError;

    use super::*;
    use std::sync::{
        atomic::{
            AtomicBool,
            Ordering::{AcqRel, Acquire},
        },
        Arc,
    };

    const KEYSYMDEF: &str = "tests/keysymdef.txt";
    const COMPOSEDEF: &str = "tests/Compose.pre";

    #[test]
    fn test_activate_instance() {
        // Create an empty SequenceDefinition
        let seqdef =
            bindings::SequenceDefinition::new().expect("SequenceDefinition should be created");

        // Create a new instance of KeyboardTranslator
        let instance = bindings::KeyboardTranslator::CreateInstance(&seqdef)
            .expect("Instance should be created");
        let instance = unsafe { instance.as_impl() };
        assert!(instance.internal.read().is_ok());
    }

    #[test]
    fn test_on_invalid() {
        // Create and build a SequenceDefinition
        let seqdef =
            bindings::SequenceDefinition::new().expect("SequenceDefinition should be created");
        seqdef
            .Build(&HSTRING::from(KEYSYMDEF), &HSTRING::from(COMPOSEDEF))
            .expect("Should be built");

        // Create a new instance of KeyboardTranslator
        let instance = bindings::KeyboardTranslator::CreateInstance(&seqdef)
            .expect("Instance should be created");
        let instance = instance
            .cast_object_ref::<KeyboardTranslator>()
            .expect("Should be castable");

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

    #[test]
    fn test_state_clear_after_translation() {
        // Create and build the SequenceDefinition
        let seqdef =
            bindings::SequenceDefinition::new().expect("SequenceDefinition should be created");
        seqdef
            .Build(&HSTRING::from(KEYSYMDEF), &HSTRING::from(COMPOSEDEF))
            .expect("SequenceDefinition should be built");

        // Create a new instance of KeyboardTranslator
        let instance = bindings::KeyboardTranslator::CreateInstance(&seqdef)
            .expect("Instance should be created");

        // Cast KeyboardTranslator to its object
        let instance = instance
            .cast_object_ref::<KeyboardTranslator>()
            .expect("Should be castable");

        // Assuming "omg" is an invalid sequence for this test
        assert_eq!(
            instance
                .internal
                .try_write()
                .unwrap()
                .forward(0, "omg".to_string()),
            Err(SequenceDefinitionError::ValueNotFound)
        );
        assert!(instance.internal.try_read().unwrap().state.is_empty());
    }

    #[test]
    fn test_state_accumulation() {
        // Create and build the SequenceDefinition
        let seqdef =
            bindings::SequenceDefinition::new().expect("SequenceDefinition should be created");
        seqdef
            .Build(&HSTRING::from(KEYSYMDEF), &HSTRING::from(COMPOSEDEF))
            .expect("SequenceDefinition should be built");

        // Create a new instance of KeyboardTranslator
        let instance = bindings::KeyboardTranslator::CreateInstance(&seqdef)
            .expect("Instance should be created");

        // Cast KeyboardTranslator to its object
        let instance = instance
            .cast_object_ref::<KeyboardTranslator>()
            .expect("Should be castable");

        let result = instance
            .internal
            .try_write()
            .unwrap()
            .forward(0, "/".to_string());
        assert_eq!(result, Err(SequenceDefinitionError::Incomplete));
        let result = instance
            .internal
            .try_write()
            .unwrap()
            .forward(0, "=".to_string());
        assert_eq!(result, Ok("≠".to_string()));
        assert!(instance.internal.try_read().unwrap().state.is_empty());
    }
}
