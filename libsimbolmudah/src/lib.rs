mod bindings;
mod keyboard_translator;
mod delegate_storage;

use crate::keyboard_translator::KeyboardTranslatorFactory;
use windows::{
    core::{Interface, Ref, HRESULT, HSTRING},
    Win32::{
        Foundation::{CLASS_E_CLASSNOTAVAILABLE, E_POINTER, S_OK},
        System::WinRT::IActivationFactory,
    },
};

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
        factory = Some(KeyboardTranslatorFactory.into());
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
