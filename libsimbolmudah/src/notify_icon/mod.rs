mod counter;
mod internal;
mod menu;

use internal::{NotifyIconInternal, INTERNAL};
use windows::{
    core::{implement, Error, IInspectable, Interface, Result, HSTRING},
    Foundation::TypedEventHandler,
    Win32::{
        Foundation::{E_NOTIMPL, E_POINTER},
        System::WinRT::{IActivationFactory, IActivationFactory_Impl},
    },
};

use crate::{bindings, utils::delegate_storage::event_registration};

#[implement(bindings::NotifyIcon)]
struct NotifyIcon;

impl Drop for NotifyIcon {
    fn drop(&mut self) {
        INTERNAL
            .destroy()
            .expect("NotifyIconInternal should be destroyed");
    }
}

impl bindings::INotifyIcon_Impl for NotifyIcon_Impl {
    fn SubscribeStateChanged(&self, hook: Option<&bindings::KeyboardHook>) -> Result<()> {
        if let Some(hook) = hook {
            let hook_clone = hook.clone();
            INTERNAL.with_borrow_mut(move |internal| {
                let event_handler = TypedEventHandler::new(|_, _state| {
                    // modify icon according to state
                    Ok(())
                });
                internal.on_state_changed_token = hook_clone.OnStateChanged(&event_handler)?;

                Ok(())
            })
        } else {
            Err(Error::new(E_POINTER, "hook is null"))
        }
    }

    fn GetHookEnabled(&self, enabled: bool) -> Result<()> {
        INTERNAL.with_borrow_mut(move |internal| internal.update_listening_check(enabled))
    }

    event_registration!(OnOpenSettings, bindings::NotifyIcon, bool);
    event_registration!(OnExitApp, bindings::NotifyIcon, bool);
    event_registration!(OnSetHookEnabled, bindings::NotifyIcon, bool);
}

#[implement(IActivationFactory, bindings::INotifyIconFactory)]
pub(super) struct NotifyIconFactory;

impl IActivationFactory_Impl for NotifyIconFactory_Impl {
    fn ActivateInstance(&self) -> Result<IInspectable> {
        Err(E_NOTIMPL.into())
    }
}

impl bindings::INotifyIconFactory_Impl for NotifyIconFactory_Impl {
    fn CreateInstance(
        &self,
        iconpath: &HSTRING,
        hookenabled: bool,
    ) -> Result<bindings::NotifyIcon> {
        let res: bindings::NotifyIcon = NotifyIcon.into();
        let res_weak = res.downgrade()?;
        let iconpath = iconpath.to_owned();

        INTERNAL.initialize(move || NotifyIconInternal::new(iconpath, hookenabled, res_weak))?;

        Ok(res)
    }
}
