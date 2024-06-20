mod bindings;
use std::{cell::RefCell, collections::HashMap};
use windows::{
    core::*,
    Win32::{Foundation::*, System::WinRT::*},
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
struct SequenceTranslatorFactory;

impl IActivationFactory_Impl for SequenceTranslatorFactory {
    fn ActivateInstance(&self) -> Result<IInspectable> {
        let instance: bindings::SequenceTranslator = SequenceTranslator {
            keysymdef: RefCell::new(HashMap::new()),
        }
        .into();
        Ok(instance.into())
    }
}

#[no_mangle]
unsafe extern "system" fn DllGetActivationFactory(
    name: Ref<HSTRING>,
    result: *mut *mut std::ffi::c_void,
) -> HRESULT {
    if result.is_null() {
        return E_POINTER;
    }

    let mut factory: Option<IActivationFactory> = None;
    if *name == "LibSimbolMudah.SequenceTranslator" {
        factory = Some(SequenceTranslatorFactory.into());
    }

    unsafe {
        if let Some(factory) = factory {
            *result = factory.into_raw();
            S_OK
        } else {
            *result = std::ptr::null_mut();
            CLASS_E_CLASSNOTAVAILABLE
        }
    }
}
