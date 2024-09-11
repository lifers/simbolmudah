mod internal;

use crate::{bindings, utils::delegate_storage::event_registration};
use internal::{KeyboardTranslatorInternal, INTERNAL};
use windows::{
    core::{implement, Error, IInspectable, Interface, Result, HSTRING},
    Foundation::TypedEventHandler,
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

#[implement(bindings::KeyboardTranslator)]
#[derive(Debug)]
pub(crate) struct KeyboardTranslator;

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
            if let Ok(value) = internal.translate(vkcode, scancode, &keystate) {
                internal.report_key(&value)?;
                let result = internal.forward(destination, value);
                internal.report(result)
            } else {
                // Even though the translation failed, the state might be stored inside ToUnicodeEx's internal buffer.
                // Do not reset the hook state.
                Ok(())
            }
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

    event_registration!(OnInvalid, TypedEventHandler<bindings::KeyboardTranslator, HSTRING>);
    event_registration!(OnTranslated, TypedEventHandler<bindings::KeyboardTranslator, HSTRING>);
    event_registration!(OnKeyTranslated, TypedEventHandler<bindings::KeyboardTranslator, HSTRING>);
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
    use bindings::IKeyboardTranslatorFactory_Impl;

    use crate::sequence_definition::{SequenceDefinitionError, SequenceDefinitionFactory};

    use super::*;
    use std::thread::sleep;
    use windows_core::{ComObjectInner, Result};

    const KEYSYMDEF: &str = "x11-defs/keysymdef.h.br";
    const COMPOSEDEF: &str = "x11-defs/Compose.pre.br";

    #[test]
    fn test_activate_instance() -> Result<()> {
        // Create an empty SequenceDefinition
        let seqdef = SequenceDefinitionFactory
            .into_object()
            .ActivateInstance()?
            .cast::<bindings::SequenceDefinition>()?;

        // Create a new instance of KeyboardTranslator
        let instance = KeyboardTranslatorFactory
            .into_object()
            .CreateInstance(Some(&seqdef))?;

        INTERNAL.with_borrow(move |internal| {
            assert_eq!(
                internal.sequence_definition.upgrade().expect("must be set"),
                seqdef
            );
            assert_eq!(internal.parent.upgrade().expect("must be set"), instance);
            Ok(())
        })?;

        INTERNAL.destroy()
    }

    #[test]
    fn test_state_clear_after_translation() -> Result<()> {
        // Create and build the SequenceDefinition
        let seqdef = SequenceDefinitionFactory
            .into_object()
            .ActivateInstance()?
            .cast::<bindings::SequenceDefinition>()?;

        seqdef.Rebuild(&KEYSYMDEF.into(), &COMPOSEDEF.into())?;

        // Create a new instance of KeyboardTranslator
        let _instance = KeyboardTranslatorFactory
            .into_object()
            .CreateInstance(Some(&seqdef))?;

        // Assuming "omg" is an invalid sequence for this test
        INTERNAL.with_borrow_mut(|internal| {
            assert_eq!(
                internal.forward(0, "omg".to_string()),
                Err(SequenceDefinitionError::ValueNotFound)
            );
            assert!(internal.state.is_empty());
            Ok(())
        })?;

        INTERNAL.destroy()?;

        sleep(std::time::Duration::from_secs(5));
        Ok(())
    }

    #[test]
    fn test_state_accumulation() -> Result<()> {
        // Create and build the SequenceDefinition
        let seqdef = SequenceDefinitionFactory
            .into_object()
            .ActivateInstance()?
            .cast::<bindings::SequenceDefinition>()?;

        seqdef.Rebuild(&KEYSYMDEF.into(), &COMPOSEDEF.into())?;

        // Create a new instance of KeyboardTranslator
        let factory: IActivationFactory = KeyboardTranslatorFactory.into();
        let _instance = factory
            .cast_object_ref::<KeyboardTranslatorFactory>()?
            .CreateInstance(Some(&seqdef))?;

        INTERNAL.with_borrow_mut(|internal| {
            assert_eq!(
                internal.forward(0, "/".to_string()),
                Err(SequenceDefinitionError::Incomplete)
            );
            assert_eq!(internal.forward(0, "=".to_string()), Ok("≠".to_string()));
            assert!(internal.state.is_empty());
            Ok(())
        })?;

        INTERNAL.destroy()?;

        sleep(std::time::Duration::from_secs(5));
        Ok(())
    }
}
