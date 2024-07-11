mod bindings;
mod delegate_storage;
mod keyboard_hook;
mod keyboard_translator;
mod sender;
mod sequence_searcher;
mod thread_handler;

use keyboard_hook::KeyboardHookFactory;
use keyboard_translator::KeyboardTranslatorFactory;
use sequence_searcher::SequenceSearcherFactory;
use windows::{
    core::{OutRef, Ref, HRESULT, HSTRING},
    Win32::{
        Foundation::{CLASS_E_CLASSNOTAVAILABLE, E_POINTER},
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
    } else {
        let _ = result.write(None);
        CLASS_E_CLASSNOTAVAILABLE
    }
}
