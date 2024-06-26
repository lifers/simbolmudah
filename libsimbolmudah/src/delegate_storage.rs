use std::{collections::HashMap, sync::Arc};

use windows::{
    core::{AgileReference, Interface, Result, RuntimeType},
    Foundation::TypedEventHandler,
};

#[derive(Debug)]
pub(crate) struct DelegateStorage<T: RuntimeType + Interface + 'static, U: RuntimeType + 'static> {
    delegates: HashMap<i64, AgileReference<TypedEventHandler<T, U>>>,
}

impl<T: RuntimeType + Interface + 'static, U: RuntimeType + 'static> DelegateStorage<T, U> {
    pub(crate) fn new() -> Self {
        Self {
            delegates: HashMap::new(),
        }
    }

    pub(crate) fn insert(
        &mut self,
        token: i64,
        handler: Arc<AgileReference<TypedEventHandler<T, U>>>,
    ) {
        self.delegates.insert(token, (*handler).clone());
    }

    pub(crate) fn remove(&mut self, token: i64) {
        self.delegates.remove(&token);
    }

    pub(crate) fn invoke_all(
        &mut self,
        sender: &T,
        args: Option<&U>,
    ) -> Result<()> {
        self.delegates
            .retain(|_, handler| handler.resolve().is_ok());

        for (_, handler) in self.delegates.iter() {
            handler.resolve()?.Invoke(Some(sender), args)?;
        }
        Ok(())
    }
}
