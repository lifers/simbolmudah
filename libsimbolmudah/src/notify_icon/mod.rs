mod counter;
mod internal;
mod menu;

use internal::{NotifyIconInternal, INTERNAL_NOTIFYICON};
use windows::{
    core::{implement, AgileReference, Error, IInspectable, Interface, Result, HSTRING},
    Foundation::{EventRegistrationToken, TypedEventHandler},
    Win32::{
        Foundation::{E_NOTIMPL, E_POINTER},
        System::WinRT::{IActivationFactory, IActivationFactory_Impl},
    },
};

use crate::{
    bindings,
    utils::{delegate_storage::get_token, thread_handler::ThreadHandler},
};

#[implement(bindings::NotifyIcon)]
struct NotifyIcon {
    thread_controller: ThreadHandler,
}

impl Drop for NotifyIcon {
    fn drop(&mut self) {
        self.thread_controller
            .try_enqueue(|| {
                let _ = INTERNAL_NOTIFYICON.take();
                Ok(())
            })
            .expect("internal should be destroyed");
    }
}

impl bindings::INotifyIcon_Impl for NotifyIcon_Impl {
    fn SubscribeStateChanged(&self, hook: Option<&bindings::KeyboardHook>) -> Result<()> {
        if let Some(hook) = hook {
            let hook_clone = hook.clone();
            self.thread_controller.try_enqueue(move || {
                INTERNAL_NOTIFYICON.with_borrow_mut(|internal: &mut Option<NotifyIconInternal>| {
                    let event_handler = TypedEventHandler::new(|_, _state| {
                        // modify icon according to state
                        Ok(())
                    });
                    internal
                        .as_mut()
                        .expect("internal should be initialised")
                        .on_state_changed_token = hook_clone.OnStateChanged(&event_handler)?;

                    Ok(())
                })
            })
        } else {
            Err(Error::new(E_POINTER, "hook is null"))
        }
    }

    fn GetHookEnabled(&self, enabled: bool) -> Result<()> {
        self.thread_controller.try_enqueue(move || {
            INTERNAL_NOTIFYICON.with_borrow_mut(|internal: &mut Option<NotifyIconInternal>| {
                internal
                    .as_mut()
                    .expect("internal should be initialised")
                    .update_listening_check(enabled)
            })
        })
    }

    fn OnOpenSettings(
        &self,
        handler: Option<&TypedEventHandler<bindings::NotifyIcon, bool>>,
    ) -> Result<EventRegistrationToken> {
        if let Some(handler) = handler {
            let handler_ref = AgileReference::new(handler)?;
            let token = get_token(handler.as_raw());
            self.thread_controller.try_enqueue(move || {
                INTERNAL_NOTIFYICON.with_borrow_mut(|internal: &mut Option<NotifyIconInternal>| {
                    internal
                        .as_mut()
                        .expect("internal should be initialised")
                        .report_open_settings
                        .insert(token, handler_ref.clone());

                    Ok(())
                })
            })?;

            Ok(EventRegistrationToken { Value: token })
        } else {
            Err(Error::new(E_POINTER, "delegate is null"))
        }
    }

    fn RemoveOnOpenSettings(&self, token: &EventRegistrationToken) -> Result<()> {
        let value = token.Value;
        self.thread_controller.try_enqueue(move || {
            INTERNAL_NOTIFYICON.with_borrow_mut(|internal: &mut Option<NotifyIconInternal>| {
                internal
                    .as_mut()
                    .expect("internal should be initialised")
                    .report_open_settings
                    .remove(value);

                Ok(())
            })
        })
    }

    fn OnExitApp(
        &self,
        handler: Option<&TypedEventHandler<bindings::NotifyIcon, bool>>,
    ) -> Result<EventRegistrationToken> {
        if let Some(handler) = handler {
            let handler_ref = AgileReference::new(handler)?;
            let token = get_token(handler.as_raw());
            self.thread_controller.try_enqueue(move || {
                INTERNAL_NOTIFYICON.with_borrow_mut(|internal: &mut Option<NotifyIconInternal>| {
                    internal
                        .as_mut()
                        .expect("internal should be initialised")
                        .report_exit_app
                        .insert(token, handler_ref.clone());

                    Ok(())
                })
            })?;

            Ok(EventRegistrationToken { Value: token })
        } else {
            Err(Error::new(E_POINTER, "delegate is null"))
        }
    }

    fn RemoveOnExitApp(&self, token: &EventRegistrationToken) -> Result<()> {
        let value = token.Value;
        self.thread_controller.try_enqueue(move || {
            INTERNAL_NOTIFYICON.with_borrow_mut(|internal: &mut Option<NotifyIconInternal>| {
                internal
                    .as_mut()
                    .expect("internal should be initialised")
                    .report_exit_app
                    .remove(value);

                Ok(())
            })
        })
    }

    fn OnSetHookEnabled(
        &self,
        handler: Option<&TypedEventHandler<bindings::NotifyIcon, bool>>,
    ) -> Result<EventRegistrationToken> {
        if let Some(handler) = handler {
            let handler_ref = AgileReference::new(handler)?;
            let token = get_token(handler.as_raw());
            self.thread_controller.try_enqueue(move || {
                INTERNAL_NOTIFYICON.with_borrow_mut(|internal: &mut Option<NotifyIconInternal>| {
                    internal
                        .as_mut()
                        .expect("internal should be initialised")
                        .report_set_listening
                        .insert(token, handler_ref.clone());

                    Ok(())
                })
            })?;

            Ok(EventRegistrationToken { Value: token })
        } else {
            Err(Error::new(E_POINTER, "delegate is null"))
        }
    }

    fn RemoveOnSetHookEnabled(&self, token: &EventRegistrationToken) -> Result<()> {
        let value = token.Value;
        self.thread_controller.try_enqueue(move || {
            INTERNAL_NOTIFYICON.with_borrow_mut(|internal: &mut Option<NotifyIconInternal>| {
                internal
                    .as_mut()
                    .expect("internal should be initialised")
                    .report_set_listening
                    .remove(value);

                Ok(())
            })
        })
    }
}

#[implement(IActivationFactory, bindings::INotifyIconFactory)]
pub(super) struct NotifyIconFactory;

impl IActivationFactory_Impl for NotifyIconFactory_Impl {
    fn ActivateInstance(&self) -> Result<IInspectable> {
        Err(E_NOTIMPL.into())
    }
}

impl bindings::INotifyIconFactory_Impl for NotifyIconFactory_Impl {
    fn CreateInstance(&self, iconpath: &HSTRING, hookenabled: bool) -> Result<bindings::NotifyIcon> {
        let res: bindings::NotifyIcon = NotifyIcon {
            thread_controller: ThreadHandler::new()?,
        }
        .into();

        let res_clone = res.clone();
        let iconpath = iconpath.to_owned();

        res.cast_object_ref::<NotifyIcon>()?
            .thread_controller
            .try_enqueue(move || {
                NotifyIconInternal::create_for_thread(iconpath.clone(), hookenabled, res_clone.downgrade()?)
            })?;

        Ok(res)
    }
}
