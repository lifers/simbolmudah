mod internal;

use crate::{
    bindings,
    utils::{delegate_storage::event_registration, functions::get_strong_ref},
};
use internal::{KeyboardHookInternal, INTERNAL};
use std::{fmt::Debug, usize};
use windows::{
    core::{implement, Error, IInspectable, Interface, Result, HSTRING},
    Foundation::TypedEventHandler,
    Win32::{
        Foundation::{E_NOTIMPL, E_POINTER},
        System::WinRT::{IActivationFactory, IActivationFactory_Impl},
    },
};

#[implement(bindings::KeyboardHook)]
#[derive(Debug)]
struct KeyboardHook;

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        INTERNAL
            .with_borrow(|internal| {
                get_strong_ref(&internal.keyboard_translator)?
                    .RemoveOnInvalid(internal.on_invalid_token)?;
                get_strong_ref(&internal.keyboard_translator)?
                    .RemoveOnTranslated(internal.on_translated_token)
            })
            .expect("Cleanup must succeed");
        INTERNAL.destroy().expect("INTERNAL should be destroyed");
    }
}

impl bindings::IKeyboardHook_Impl for KeyboardHook_Impl {
    fn ResetStage(&self) -> Result<()> {
        INTERNAL.with_borrow_mut(|internal| internal.reset_state())
    }

    event_registration!(OnStateChanged, TypedEventHandler<bindings::KeyboardHook, u8>);
    event_registration!(OnKeyEvent, TypedEventHandler<bindings::KeyboardHook, HSTRING>);
}

#[implement(IActivationFactory, bindings::IKeyboardHookFactory)]
pub(super) struct KeyboardHookFactory;

impl IActivationFactory_Impl for KeyboardHookFactory_Impl {
    fn ActivateInstance(&self) -> Result<IInspectable> {
        Err(E_NOTIMPL.into())
    }
}

impl bindings::IKeyboardHookFactory_Impl for KeyboardHookFactory_Impl {
    fn CreateInstance(
        &self,
        translator: Option<&bindings::KeyboardTranslator>,
    ) -> Result<bindings::KeyboardHook> {
        let translator = translator
            .ok_or_else(|| Error::new(E_POINTER, "translator is null"))?
            .downgrade()?;
        let res: bindings::KeyboardHook = KeyboardHook.into();
        let res_weak = res.downgrade()?;

        INTERNAL.initialize(move || KeyboardHookInternal::new(translator, res_weak))?;

        Ok(res)
    }
}
