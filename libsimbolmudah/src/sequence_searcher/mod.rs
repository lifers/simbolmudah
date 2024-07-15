mod internal;

use std::sync::RwLock;

use crate::{bindings, fail, get_strong_ref, sequence_definition::SequenceDefinition};
use internal::SequenceSearcherInternal;
use windows::{
    core::{implement, Error, IInspectable, Interface, Result, Weak, HSTRING},
    Foundation::Collections::IVectorView,
    Win32::{
        Foundation::{E_NOTIMPL, E_POINTER},
        System::WinRT::{IActivationFactory, IActivationFactory_Impl},
    },
};

type SS = bindings::SequenceSearcher;

#[implement(SS)]
struct SequenceSearcher {
    internal: RwLock<SequenceSearcherInternal>,
}

impl SequenceSearcher {
    fn tokenize(&self, keyword: &HSTRING) -> Vec<String> {
        keyword
            .to_string()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }
}

impl bindings::ISequenceSearcher_Impl for SequenceSearcher_Impl {
    fn Search(&self, keyword: &HSTRING) -> Result<IVectorView<bindings::SequenceDescription>> {
        get_strong_ref(&self.internal.read().map_err(fail)?.sequence_definition)?
            .cast_object_ref::<SequenceDefinition>()?
            .filter_sequence(self.tokenize(keyword))?
            .try_into()
    }
}

#[implement(IActivationFactory, bindings::ISequenceSearcherFactory)]
pub(crate) struct SequenceSearcherFactory;

impl IActivationFactory_Impl for SequenceSearcherFactory_Impl {
    fn ActivateInstance(&self) -> Result<IInspectable> {
        Err(E_NOTIMPL.into())
    }
}

impl bindings::ISequenceSearcherFactory_Impl for SequenceSearcherFactory_Impl {
    fn CreateInstance(
        &self,
        definition: Option<&bindings::SequenceDefinition>,
    ) -> Result<bindings::SequenceSearcher> {
        let definition = definition.ok_or_else(|| Error::new(E_POINTER, "definition is null"))?;
        let internal = SequenceSearcherInternal::new(definition.downgrade()?, Weak::new());
        let instance: SS = SequenceSearcher {
            internal: RwLock::new(internal),
        }
        .into();

        instance
            .cast_object::<SequenceSearcher>()?
            .internal
            .try_write()
            .unwrap()
            .parent = instance.downgrade()?;

        Ok(instance)
    }
}
