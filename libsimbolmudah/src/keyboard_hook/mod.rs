mod internal;

use crate::{
    bindings,
    utils::{delegate_storage::get_token, functions::get_strong_ref},
};
use internal::{KeyboardHookInternal, INTERNAL};
use std::{fmt::Debug, usize};
use windows::{
    core::{implement, AgileReference, Error, IInspectable, Interface, Result, HSTRING},
    Foundation::{EventRegistrationToken, TypedEventHandler},
    Win32::{
        Foundation::{E_NOTIMPL, E_POINTER},
        System::WinRT::{IActivationFactory, IActivationFactory_Impl},
    },
};

#[derive(Debug, Clone)]
enum Reporter {
    OnStateChanged(
        (
            AgileReference<TypedEventHandler<bindings::KeyboardHook, u8>>,
            i64,
        ),
    ),
    OnKeyEvent(
        (
            AgileReference<TypedEventHandler<bindings::KeyboardHook, HSTRING>>,
            i64,
        ),
    ),
}

#[implement(bindings::KeyboardHook)]
#[derive(Debug)]
struct KeyboardHook;

impl KeyboardHook {
    fn register_reporter(&self, reporter: Reporter) -> Result<()> {
        // making sure this callback runs after the hook is activated
        INTERNAL.with_borrow_mut(move |internal| {
            Ok(match &reporter {
                Reporter::OnStateChanged((h, token)) => {
                    internal.state_changed.insert(*token, h.clone())
                }
                Reporter::OnKeyEvent((h, token)) => internal.key_event.insert(*token, h.clone()),
            })
        })
    }

    fn unregister_reporter(&self, reporter: Reporter) -> Result<()> {
        // making sure this callback runs after the reporter is registered
        INTERNAL.with_borrow_mut(move |internal| {
            Ok(match reporter {
                Reporter::OnStateChanged((_, token)) => internal.state_changed.remove(token),
                Reporter::OnKeyEvent((_, token)) => internal.key_event.remove(token),
            })
        })
    }
}

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

    fn OnStateChanged(
        &self,
        handler: Option<&TypedEventHandler<bindings::KeyboardHook, u8>>,
    ) -> Result<EventRegistrationToken> {
        if let Some(handler) = handler {
            let token = get_token(handler.as_raw());
            self.register_reporter(Reporter::OnStateChanged((
                AgileReference::new(handler)?,
                token,
            )))?;
            Ok(EventRegistrationToken { Value: token })
        } else {
            Err(Error::new(E_POINTER, "Null OnStateChanged handle pointer"))
        }
    }

    fn RemoveOnStateChanged(&self, token: &EventRegistrationToken) -> Result<()> {
        self.unregister_reporter(Reporter::OnStateChanged((
            AgileReference::new(&TypedEventHandler::new(|_, _| Err(E_NOTIMPL.into())))?,
            token.Value,
        )))
    }

    fn OnKeyEvent(
        &self,
        handler: Option<&TypedEventHandler<bindings::KeyboardHook, HSTRING>>,
    ) -> Result<EventRegistrationToken> {
        if let Some(handler) = handler {
            let token = get_token(handler.as_raw());
            self.register_reporter(Reporter::OnKeyEvent((AgileReference::new(handler)?, token)))?;
            Ok(EventRegistrationToken { Value: token })
        } else {
            Err(Error::new(E_POINTER, "Null OnKeyEvent handle pointer"))
        }
    }

    fn RemoveOnKeyEvent(&self, token: &EventRegistrationToken) -> Result<()> {
        self.unregister_reporter(Reporter::OnKeyEvent((
            AgileReference::new(&TypedEventHandler::new(|_, _| Err(E_NOTIMPL.into())))?,
            token.Value,
        )))
    }
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
