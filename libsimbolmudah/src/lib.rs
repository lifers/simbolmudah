mod bindings;
mod delegate_storage;
mod keyboard_hook;
mod keyboard_translator;
mod notify_icon;
mod sender;
mod sequence_definition;
mod thread_handler;

use keyboard_hook::KeyboardHookFactory;
use keyboard_translator::KeyboardTranslatorFactory;
use notify_icon::NotifyIconFactory;
use sequence_definition::SequenceDefinitionFactory;
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
    } else {
        match name.to_string().as_str() {
            "LibSimbolMudah.KeyboardTranslator" => {
                result.write(Some(KeyboardTranslatorFactory.into())).into()
            }
            "LibSimbolMudah.KeyboardHook" => result.write(Some(KeyboardHookFactory.into())).into(),
            "LibSimbolMudah.NotifyIcon" => result.write(Some(NotifyIconFactory.into())).into(),
            "LibSimbolMudah.SequenceDefinition" => {
                result.write(Some(SequenceDefinitionFactory.into())).into()
            }
            _ => {
                let _ = result.write(None);
                CLASS_E_CLASSNOTAVAILABLE
            }
        }
    }
}

fn fail(error: impl std::error::Error) -> Error {
    Error::new(E_FAIL, format!("{:?}", error))
}

fn fail_message(message: &str) -> Error {
    Error::new(E_FAIL, message)
}

fn get_strong_ref<T>(weak: &Weak<T>) -> Result<T>
where
    T: Interface,
{
    weak.upgrade()
        .ok_or_else(|| Error::new(E_POINTER, "Weak pointer died"))
}
