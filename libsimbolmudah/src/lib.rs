mod bindings;
mod keyboard_hook;
mod keyboard_translator;
mod notify_icon;
mod sequence_definition;
mod utils;

use keyboard_hook::KeyboardHookFactory;
use keyboard_translator::KeyboardTranslatorFactory;
use notify_icon::NotifyIconFactory;
use sequence_definition::SequenceDefinitionFactory;
use utils::sender::Sender;
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
            "LibSimbolMudah.Sender" => result.write(Some(Sender.into())).into(),
            _ => {
                let _ = result.write(None);
                CLASS_E_CLASSNOTAVAILABLE
            }
        }
    }
}
