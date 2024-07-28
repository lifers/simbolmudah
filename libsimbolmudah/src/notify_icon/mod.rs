mod counter;
mod internal;
mod menu;

use internal::{NotifyIconInternal, INTERNAL_NOTIFYICON};
use windows::{
    core::{implement, AgileReference, Error, IInspectable, Interface, Result},
    Foundation::{EventRegistrationToken, TypedEventHandler},
    Graphics::PointInt32,
    Win32::{
        Foundation::E_POINTER,
        System::WinRT::{IActivationFactory, IActivationFactory_Impl},
    },
};

use crate::delegate_storage::get_token;
use crate::{bindings, thread_handler::ThreadHandler};

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

    fn OnSelected(
        &self,
        handler: Option<&TypedEventHandler<bindings::NotifyIcon, PointInt32>>,
    ) -> Result<EventRegistrationToken> {
        if let Some(handler) = handler {
            let handler_ref = AgileReference::new(handler)?;
            let token = get_token(handler.as_raw());
            self.thread_controller.try_enqueue(move || {
                INTERNAL_NOTIFYICON.with_borrow_mut(|internal: &mut Option<NotifyIconInternal>| {
                    internal
                        .as_mut()
                        .expect("internal should be initialised")
                        .report_selected
                        .insert(token, handler_ref.clone());

                    Ok(())
                })
            })?;

            Ok(EventRegistrationToken { Value: token })
        } else {
            Err(Error::new(E_POINTER, "hook is null"))
        }
    }

    fn RemoveOnSelected(&self, token: &EventRegistrationToken) -> Result<()> {
        let value = token.Value;
        self.thread_controller.try_enqueue(move || {
            INTERNAL_NOTIFYICON.with_borrow_mut(|internal: &mut Option<NotifyIconInternal>| {
                internal
                    .as_mut()
                    .expect("internal should be initialised")
                    .report_selected
                    .remove(value);

                Ok(())
            })
        })
    }
}

#[implement(IActivationFactory)]
pub(super) struct NotifyIconFactory;

impl IActivationFactory_Impl for NotifyIconFactory_Impl {
    fn ActivateInstance(&self) -> Result<IInspectable> {
        let res: bindings::NotifyIcon = NotifyIcon {
            thread_controller: ThreadHandler::new()?,
        }
        .into();

        let res_clone = res.clone();

        res.cast_object_ref::<NotifyIcon>()?
            .thread_controller
            .try_enqueue(move || NotifyIconInternal::create_for_thread(res_clone.downgrade()?))?;

        Ok(res.into())
    }
}
