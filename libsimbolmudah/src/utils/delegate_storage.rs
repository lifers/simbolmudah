use std::{
    collections::HashMap,
    ffi::c_void,
    hash::{DefaultHasher, Hash, Hasher},
};
use windows::{
    core::{AgileReference, Interface, Result, RuntimeType},
    Foundation::TypedEventHandler,
};

#[derive(Debug, Default)]
pub(crate) struct DelegateStorage<T: RuntimeType + Interface + 'static, U: RuntimeType + 'static> {
    delegates: HashMap<i64, AgileReference<TypedEventHandler<T, U>>>,
}

impl<T: RuntimeType + Interface + 'static, U: RuntimeType + 'static> DelegateStorage<T, U> {
    pub(crate) fn new() -> Self {
        Self {
            delegates: HashMap::new(),
        }
    }

    pub(crate) fn insert(&mut self, token: i64, handler: AgileReference<TypedEventHandler<T, U>>) {
        self.delegates.insert(token, handler);
    }

    pub(crate) fn remove(&mut self, token: i64) {
        self.delegates.remove(&token);
    }

    pub(crate) fn invoke_all(&mut self, sender: &T, args: Option<&U>) -> Result<()> {
        self.delegates
            .retain(|_, handler| handler.resolve().is_ok());

        for (_, handler) in self.delegates.iter() {
            handler.resolve().unwrap().Invoke(Some(sender), args)?;
        }
        Ok(())
    }
}

pub(crate) fn get_token(handler: *mut c_void) -> i64 {
    // Generate a unique token
    let mut hasher = DefaultHasher::new();
    handler.hash(&mut hasher);
    hasher.finish() as i64
}

macro_rules! event_registration {
    ($name:ident, $source_type:ty, $result_type:ty) => {
        fn $name(
            &self,
            handler: Option<&windows::Foundation::TypedEventHandler<$source_type, $result_type>>,
        ) -> windows_core::Result<windows::Foundation::EventRegistrationToken> {
            if let Some(handler) = handler {
                let handler_ref = windows_core::AgileReference::new(handler)?;
                let token = crate::utils::delegate_storage::get_token(handler.as_raw());
                internal::INTERNAL.with_borrow_mut(move |internal| {
                    internal.$name.insert(token, handler_ref);
                    Ok(())
                })?;
                Ok(windows::Foundation::EventRegistrationToken { Value: token })
            } else {
                Err(windows_core::Error::new(windows::Win32::Foundation::E_POINTER, "delegate is null"))
            }
        }

        concat_idents::concat_idents!(fn_name = Remove, $name {
            fn fn_name(&self, token: &windows::Foundation::EventRegistrationToken) -> windows_core::Result<()> {
                let value = token.Value;
                internal::INTERNAL.with_borrow_mut(move |internal| {
                    internal.$name.remove(value);
                    Ok(())
                })
            }
        });
    };
}
pub(crate) use event_registration;
