mod internal;

use std::sync::{Arc, RwLock};

use crate::bindings;
use internal::SequenceSearcherInternal;
use windows::{
    core::{h, implement, Array, IInspectable, Interface, Result, Weak, HSTRING},
    Foundation::Collections::IVectorView,
    Win32::System::WinRT::{IActivationFactory, IActivationFactory_Impl},
};

type SS = bindings::SequenceSearcher;

#[implement(SS)]
struct SequenceSearcher {
    internal: Arc<RwLock<SequenceSearcherInternal>>,
}

impl bindings::ISequenceSearcher_Impl for SequenceSearcher_Impl {
    fn Search(
        &self,
        keyword: &HSTRING,
        sequence: &mut Array<IVectorView<u32>>,
        result: &mut Array<HSTRING>,
        description: &mut Array<HSTRING>,
    ) -> Result<()> {
        let default_seq = IVectorView::try_from(vec![0x65, 0x66, 0x67])?;
        *sequence = Array::from_slice(&[Some(default_seq.clone()), Some(default_seq.clone())]);
        *result = Array::from_slice(&[h!("😎").to_owned(), h!("😎").to_owned()]);
        *description = Array::from_slice(&[keyword.clone(), keyword.clone()]);
        Ok(())
    }
}

#[implement(IActivationFactory)]
pub(crate) struct SequenceSearcherFactory;

impl IActivationFactory_Impl for SequenceSearcherFactory_Impl {
    fn ActivateInstance(&self) -> Result<IInspectable> {
        let internal = SequenceSearcherInternal::new(Weak::new());
        let instance: SS = SequenceSearcher {
            internal: Arc::new(RwLock::new(internal)),
        }
        .into();

        instance
            .cast_object::<SequenceSearcher>()?
            .internal
            .try_write()
            .unwrap()
            .parent = instance.downgrade()?;

        Ok(instance.into())
    }
}
