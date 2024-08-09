mod internal;

use crate::{bindings, utils::delegate_storage::get_token};
use internal::{KeyboardTranslatorInternal, INTERNAL};
use windows::{
    core::{implement, AgileReference, Error, IInspectable, Interface, Result, HSTRING},
    Foundation::{EventRegistrationToken, TypedEventHandler},
    Win32::{
        Foundation::{E_NOTIMPL, E_POINTER},
        System::WinRT::{IActivationFactory, IActivationFactory_Impl},
        UI::{
            Input::KeyboardAndMouse::{
                GetKeyboardLayout, VK_CAPITAL, VK_CONTROL, VK_MENU, VK_SHIFT,
            },
            WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
        },
    },
};

type H = TypedEventHandler<bindings::KeyboardTranslator, HSTRING>;

enum Reporter {
    Invalid,
    Translated,
    KeyTranslated,
}

#[implement(bindings::KeyboardTranslator)]
#[derive(Debug)]
pub(crate) struct KeyboardTranslator;

impl KeyboardTranslator {
    fn register_reporter(
        &self,
        handler: Option<&H>,
        reporter: Reporter,
    ) -> Result<EventRegistrationToken> {
        if let Some(handler) = handler {
            let token = get_token(handler.as_raw());
            let handler_ref = AgileReference::new(handler)?;

            INTERNAL.with_borrow_mut(move |internal| {
                Ok(match reporter {
                    Reporter::Invalid => &mut internal.report_invalid,
                    Reporter::Translated => &mut internal.report_translated,
                    Reporter::KeyTranslated => &mut internal.report_key_translated,
                }
                .insert(token, handler_ref.clone()))
            })?;

            Ok(EventRegistrationToken { Value: token })
        } else {
            Err(E_POINTER.into())
        }
    }

    fn unregister_reporter(
        &self,
        reporter: Reporter,
        token: &EventRegistrationToken,
    ) -> Result<()> {
        let value = token.Value;
        INTERNAL.with_borrow_mut(move |internal| {
            Ok(match reporter {
                Reporter::Invalid => &mut internal.report_invalid,
                Reporter::Translated => &mut internal.report_translated,
                Reporter::KeyTranslated => &mut internal.report_key_translated,
            }
            .remove(value))
        })
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
        INTERNAL.with_borrow_mut(move |internal| {
            let keystate = calculate_bg_keystate(hascapslock, hasshift, hasaltgr);
            let value = internal.translate(vkcode, scancode, &keystate)?;
            internal.report_key(&value)?;
            let result = internal.forward(destination, value);
            internal.report(result)
        })
    }

    fn CheckLayoutAndUpdate(&self) -> Result<()> {
        INTERNAL.with_borrow_mut(|internal| {
            let foreground_window = unsafe { GetForegroundWindow() };
            let tid = unsafe { GetWindowThreadProcessId(foreground_window, None) };
            let active_layout = unsafe { GetKeyboardLayout(tid) };

            if internal.keyboard_layout == active_layout {
                return Ok(());
            }

            internal.keyboard_layout = active_layout;
            internal.analyze_layout()
        })
    }

    fn OnInvalid(&self, handler: Option<&H>) -> Result<EventRegistrationToken> {
        self.register_reporter(handler, Reporter::Invalid)
    }

    fn OnTranslated(&self, handler: Option<&H>) -> Result<EventRegistrationToken> {
        self.register_reporter(handler, Reporter::Translated)
    }

    fn OnKeyTranslated(&self, handler: Option<&H>) -> Result<EventRegistrationToken> {
        self.register_reporter(handler, Reporter::KeyTranslated)
    }

    fn RemoveOnInvalid(&self, token: &EventRegistrationToken) -> Result<()> {
        self.unregister_reporter(Reporter::Invalid, token)
    }

    fn RemoveOnTranslated(&self, token: &EventRegistrationToken) -> Result<()> {
        self.unregister_reporter(Reporter::Translated, token)
    }

    fn RemoveOnKeyTranslated(&self, token: &EventRegistrationToken) -> Result<()> {
        self.unregister_reporter(Reporter::KeyTranslated, token)
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
        let definition = definition
            .ok_or(Error::new(E_POINTER, "Null pointer"))?
            .downgrade()?;
        let instance: bindings::KeyboardTranslator = KeyboardTranslator.into();
        let instance_weak = instance.downgrade()?;

        INTERNAL
            .initialize(move || Ok(KeyboardTranslatorInternal::new(definition, instance_weak)))?;

        Ok(instance)
    }
}

#[cfg(test)]
mod tests {
    use bindings::{IKeyboardTranslatorFactory_Impl, ISequenceDefinition_Impl};

    use crate::sequence_definition::{
        SequenceDefinition, SequenceDefinitionError, SequenceDefinitionFactory,
    };

    use super::*;
    use std::thread::sleep;

    const KEYSYMDEF: &str = "tests/keysymdef.txt";
    const COMPOSEDEF: &str = "tests/Compose.pre";

    #[test]
    fn test_activate_instance() {
        // Create an empty SequenceDefinition
        let seqdef =
            bindings::SequenceDefinition::new().expect("SequenceDefinition should be created");

        let factory: IActivationFactory = KeyboardTranslatorFactory.into();

        // Create a new instance of KeyboardTranslator
        let instance = factory
            .cast_object_ref::<KeyboardTranslatorFactory>()
            .unwrap()
            .CreateInstance(Some(&seqdef))
            .expect("Instance should be created");

        INTERNAL
            .with_borrow(move |internal| {
                assert_eq!(
                    internal.sequence_definition.upgrade().expect("must be set"),
                    seqdef
                );
                assert_eq!(internal.parent.upgrade().expect("must be set"), instance);
                Ok(())
            })
            .expect("Internal should be borrowed");
    }

    #[test]
    fn test_state_clear_after_translation() {
        // Create and build the SequenceDefinition
        let factory: IActivationFactory = SequenceDefinitionFactory.into();
        let seqdef = factory
            .cast_object_ref::<SequenceDefinitionFactory>()
            .unwrap()
            .ActivateInstance()
            .expect("SequenceDefinition should be created");

        seqdef
            .cast_object_ref::<SequenceDefinition>()
            .expect("Should be castable")
            .Build(&KEYSYMDEF.into(), &COMPOSEDEF.into())
            .expect("SequenceDefinition should be built");

        let factory: IActivationFactory = KeyboardTranslatorFactory.into();
        // Create a new instance of KeyboardTranslator
        let instance = factory
            .cast_object_ref::<KeyboardTranslatorFactory>()
            .unwrap()
            .CreateInstance(Some(
                &seqdef.cast::<bindings::SequenceDefinition>().unwrap(),
            ))
            .expect("Instance should be created");

        // Cast KeyboardTranslator to its object
        let instance = instance
            .cast_object_ref::<KeyboardTranslator>()
            .expect("Should be castable");

        // Assuming "omg" is an invalid sequence for this test
        INTERNAL
            .with_borrow_mut(|internal| {
                assert_eq!(
                    internal.forward(0, "omg".to_string()),
                    Err(SequenceDefinitionError::ValueNotFound)
                );
                assert!(internal.state.is_empty());
                Ok(())
            })
            .expect("Internal should be borrowed");

        sleep(std::time::Duration::from_secs(5));
    }

    #[test]
    fn test_state_accumulation() {
        // Create and build the SequenceDefinition
        let factory: IActivationFactory = SequenceDefinitionFactory.into();
        let seqdef = factory
            .cast_object_ref::<SequenceDefinitionFactory>()
            .unwrap()
            .ActivateInstance()
            .expect("SequenceDefinition should be created");
        seqdef
            .cast_object_ref::<SequenceDefinition>()
            .expect("Should be castable")
            .Build(&KEYSYMDEF.into(), &COMPOSEDEF.into())
            .expect("SequenceDefinition should be built");

        // Create a new instance of KeyboardTranslator
        let factory: IActivationFactory = KeyboardTranslatorFactory.into();
        let instance = factory
            .cast_object_ref::<KeyboardTranslatorFactory>()
            .unwrap()
            .CreateInstance(Some(
                &seqdef.cast::<bindings::SequenceDefinition>().unwrap(),
            ))
            .expect("Instance should be created");

        // Cast KeyboardTranslator to its object
        let instance = instance
            .cast_object_ref::<KeyboardTranslator>()
            .expect("Should be castable");

        INTERNAL
            .with_borrow_mut(|internal| {
                assert_eq!(
                    internal.forward(0, "/".to_string()),
                    Err(SequenceDefinitionError::Incomplete)
                );
                assert_eq!(internal.forward(0, "=".to_string()), Ok("≠".to_string()));
                assert!(internal.state.is_empty());
                Ok(())
            })
            .expect("Internal should be borrowed");

        sleep(std::time::Duration::from_secs(5));
    }
}
