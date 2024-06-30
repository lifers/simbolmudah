mod bindings;
mod delegate_storage;
mod keyboard_hook;
mod keyboard_translator;
mod sender;
mod thread_handler;

use crate::{keyboard_translator::KeyboardTranslatorFactory, keyboard_hook::KeyboardHookFactory};
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

    let factory: Option<IActivationFactory> = if *name == "LibSimbolMudah.KeyboardTranslator" {
        Some(KeyboardTranslatorFactory.into())
    } else if *name == "LibSimbolMudah.KeyboardHook" {
        Some(KeyboardHookFactory.into())
    } else {
        None
    };

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
