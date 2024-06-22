use crate::bindings;
use std::{cell::RefCell, collections::HashMap};
use windows::{
    core::{implement, Error, IInspectable, Result, HSTRING},
    Win32::{
        Foundation::E_INVALIDARG,
        System::WinRT::{IActivationFactory, IActivationFactory_Impl},
    },
};

#[implement(bindings::SequenceTranslator)]
struct SequenceTranslator {
    keysymdef: RefCell<HashMap<String, String>>,
}

impl bindings::ISequenceTranslator_Impl for SequenceTranslator {
    fn Translate(&self, value: &HSTRING) -> Result<HSTRING> {
        let value = value.to_string();
        if let Some(result) = self.keysymdef.borrow().get(&value) {
            Ok(result.into())
        } else {
            Err(Error::new(E_INVALIDARG, "value not found"))
        }
    }

    fn BuildDictionary(&self) -> Result<()> {
        let mut map = self.keysymdef.borrow_mut();
        map.insert(">=".into(), "≥".into());
        map.insert("fm".into(), "👨🏿‍👩🏻‍👧🏿‍👦🏼".into());
        Ok(())
    }
}

#[implement(IActivationFactory)]
pub(super) struct SequenceTranslatorFactory;

// Default constructor for SequenceTranslator WinRT class
impl IActivationFactory_Impl for SequenceTranslatorFactory {
    fn ActivateInstance(&self) -> Result<IInspectable> {
        let instance: bindings::SequenceTranslator = SequenceTranslator {
            keysymdef: RefCell::new(HashMap::new()),
        }
        .into();
        Ok(instance.into())
    }
}
