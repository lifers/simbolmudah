use std::{
    collections::HashMap,
    ffi::c_void,
    hash::{DefaultHasher, Hash, Hasher},
    mem::transmute_copy,
};
use windows::{
    core::{AgileReference, Interface, Result, HRESULT},
    Foundation::EventRegistrationToken,
    Win32::{Foundation::{JSCRIPT_E_CANTEXECUTE, RPC_E_DISCONNECTED}, System::{Com::IAgileObject, Diagnostics::Debug::EncodePointer}},
};

#[derive(Clone, Debug)]
enum Delegate<T> {
    Direct(T),
    Indirect(AgileReference<T>),
}

impl<T: Interface> Delegate<T> {
    fn new(delegate: &T) -> Result<Self> {
        if delegate.cast::<IAgileObject>().is_ok() {
            Ok(Self::Direct(delegate.clone()))
        } else {
            Ok(Self::Indirect(AgileReference::new(delegate)?))
        }
    }

    fn to_token(&self) -> i64 {
        match self {
            Self::Direct(delegate) => unsafe {
                EncodePointer(Some(transmute_copy(delegate))) as i64
            },
            Self::Indirect(delegate) => unsafe {
                EncodePointer(Some(transmute_copy(delegate))) as i64
            },
        }
    }

    fn call<F: FnMut(&T) -> Result<()>>(&self, mut callback: F) -> Result<()> {
        match self {
            Self::Direct(delegate) => callback(delegate),
            Self::Indirect(delegate) => callback(&delegate.resolve()?),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct DelegateStorage<D: Interface> {
    delegates: HashMap<i64, Delegate<D>>,
}

impl<D: Interface> DelegateStorage<D> {
    pub(crate) fn new() -> Self {
        Self {
            delegates: HashMap::new(),
        }
    }

    pub(crate) fn insert(&mut self, handler: &D) -> Result<EventRegistrationToken> {
        let delegate = Delegate::new(handler)?;
        let token = delegate.to_token();
        self.delegates.insert(token, delegate);
        Ok(EventRegistrationToken { Value: token })
    }

    pub(crate) fn remove(&mut self, token: EventRegistrationToken) {
        self.delegates.remove(&token.Value);
    }

    pub(crate) fn invoke_all<F: FnMut(&D) -> Result<()>>(&mut self, mut callback: F) -> Result<()> {
        let delegates = self.delegates.clone();
        for (_, handler) in delegates.iter() {
            if let Err(e) = handler.call(&mut callback) {
                const RPC_E_SERVER_UNAVAILABLE: HRESULT = HRESULT(-2147023174); // HRESULT_FROM_WIN32(RPC_S_SERVER_UNAVAILABLE)
                if matches!(e.code(), RPC_E_DISCONNECTED | JSCRIPT_E_CANTEXECUTE | RPC_E_SERVER_UNAVAILABLE) {
                    self.remove(EventRegistrationToken { Value: handler.to_token() });
                }
            }
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
    ($name:ident, $delegate_type:ty) => {
        fn $name(
            &self,
            handler: Option<&$delegate_type>,
        ) -> windows_core::Result<windows::Foundation::EventRegistrationToken> {
            if let Some(handler) = handler {
                let handler_ref = windows_core::AgileReference::new(handler)?;
                let token = crate::utils::delegate_storage::get_token(handler.as_raw());
                internal::INTERNAL.with_borrow_mut(move |internal| {
                    let handler = handler_ref.resolve()?;
                    internal.$name.insert(&handler)?;
                    Ok(())
                })?;
                Ok(windows::Foundation::EventRegistrationToken { Value: token })
            } else {
                Err(windows_core::Error::new(windows::Win32::Foundation::E_POINTER, "delegate is null"))
            }
        }

        concat_idents::concat_idents!(fn_name = Remove, $name {
            fn fn_name(&self, token: &windows::Foundation::EventRegistrationToken) -> windows_core::Result<()> {
                let token_clone = token.clone();
                internal::INTERNAL.with_borrow_mut(move |internal| {
                    internal.$name.remove(token_clone);
                    Ok(())
                })
            }
        });
    };
}
pub(crate) use event_registration;
