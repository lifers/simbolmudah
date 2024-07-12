mod bindings;
mod delegate_storage;
mod keyboard_hook;
mod keyboard_translator;
mod sender;
mod sequence_definition;
mod sequence_searcher;
mod thread_handler;

use keyboard_hook::KeyboardHookFactory;
use keyboard_translator::KeyboardTranslatorFactory;
use sequence_definition::SequenceDefinitionFactory;
use sequence_searcher::SequenceSearcherFactory;
use windows::{
    core::{Error, Interface, OutRef, Ref, Result, Weak, HRESULT, HSTRING},
    Win32::{
        Foundation::{CLASS_E_CLASSNOTAVAILABLE, E_FAIL, E_POINTER},
        System::WinRT::IActivationFactory,
    },
};

#[no_mangle]
extern "system" fn DllGetActivationFactory(
    name: Ref<HSTRING>,
    result: OutRef<IActivationFactory>,
) -> HRESULT {
    if result.is_null() {
        E_POINTER
    } else if *name == "LibSimbolMudah.KeyboardTranslator" {
        result.write(Some(KeyboardTranslatorFactory.into())).into()
    } else if *name == "LibSimbolMudah.KeyboardHook" {
        result.write(Some(KeyboardHookFactory.into())).into()
    } else if *name == "LibSimbolMudah.SequenceSearcher" {
        result.write(Some(SequenceSearcherFactory.into())).into()
    } else if *name == "LibSimbolMudah.SequenceDefinition" {
        result.write(Some(SequenceDefinitionFactory.into())).into()
    } else {
        let _ = result.write(None);
        CLASS_E_CLASSNOTAVAILABLE
    }
}

pub(crate) fn fail(error: impl std::error::Error) -> Error {
    Error::new(E_FAIL, format!("{:?}", error))
}

pub(crate) fn fail_message(message: &str) -> Error {
    Error::new(E_FAIL, message)
}

pub(crate) fn get_strong_ref<T>(weak: &Weak<T>) -> Result<T>
where
    T: Interface,
{
    weak.upgrade()
        .ok_or_else(|| Error::new(E_POINTER, "Weak pointer died"))
}
