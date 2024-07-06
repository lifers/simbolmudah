use crate::bindings::{self, ISequenceDescription_Impl};
use windows::{
    core::{h, implement, Result, HSTRING, IInspectable},
    Foundation::Collections::IVectorView,
    Win32::System::WinRT::{IActivationFactory, IActivationFactory_Impl},
};

#[implement(bindings::SequenceDescription)]
struct SequenceDescription;

impl ISequenceDescription_Impl for SequenceDescription_Impl {
    fn description(&self) -> Result<HSTRING> {
        Ok(h!("This is a sequence description").to_owned())
    }

    fn result(&self) -> Result<HSTRING> {
        Ok(h!("😎").to_owned())
    }

    fn sequence(&self) -> Result<IVectorView<u32>> {
        IVectorView::try_from(vec![0x65, 0x66, 0x67])
    }
}

#[implement(IActivationFactory)]
struct SequenceDescriptionFactory;

impl IActivationFactory_Impl for SequenceDescriptionFactory_Impl {
    fn ActivateInstance(&self) -> Result<IInspectable> {
        Ok(SequenceDescription {}.into())
    }
}