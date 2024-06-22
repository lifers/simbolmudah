use std::{collections::HashMap, sync::Arc};

use windows::{
    core::{AgileReference, IInspectable, Result, RuntimeType},
    Foundation::EventHandler,
};

pub(crate) struct DelegateStorage<T: RuntimeType + 'static> {
    delegates: HashMap<i64, AgileReference<EventHandler<T>>>,
}

impl<T: RuntimeType + 'static> DelegateStorage<T> {
    pub(crate) fn new() -> Self {
        Self {
            delegates: HashMap::new(),
        }
    }

    pub(crate) fn insert(&mut self, token: i64, handler: Arc<AgileReference<EventHandler<T>>>) {
        self.delegates.insert(token, (*handler).clone());
    }

    pub(crate) fn remove(&mut self, token: i64) {
        self.delegates.remove(&token);
    }

    pub(crate) fn invoke_all(&mut self, sender: &IInspectable, args: Option<&T>) -> Result<()> {
        self.delegates
            .retain(|_, handler| handler.resolve().is_ok());

        for (_, handler) in self.delegates.iter() {
            handler.resolve()?.Invoke(sender, args)?;
        }
        Ok(())
    }
}
