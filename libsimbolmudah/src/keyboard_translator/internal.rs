use std::{collections::HashMap, fmt::Debug, ptr::null_mut};

use windows::{
    core::{h, Error, Interface, Result, Weak, HRESULT, HSTRING},
    Foundation::TypedEventHandler,
    Win32::{
        Foundation::{ERROR_NO_UNICODE_TRANSLATION, E_INVALIDARG, E_POINTER},
        UI::Input::KeyboardAndMouse::{ToUnicodeEx, HKL, VK_CONTROL, VK_MENU, VK_SHIFT, VK_SPACE},
    },
};

use crate::{
    bindings,
    sequence_definition::{SequenceDefinition, SequenceDefinitionError},
    utils::{
        delegate_storage::DelegateStorage,
        functions::get_strong_ref,
        sender::send_text_clipboard,
        single_threaded::{single_threaded, SingleThreaded},
    },
};

pub(super) static INTERNAL: SingleThreaded<KeyboardTranslatorInternal> =
    single_threaded!(KeyboardTranslatorInternal);

enum VKToUnicodeError {
    InvalidReturn,
    NoTranslation,
}

#[derive(Clone)]
enum StringVariant {
    LiveString(String),
    DeadString(String),
}

impl Into<String> for StringVariant {
    fn into(self) -> String {
        match self {
            StringVariant::LiveString(s) => s,
            StringVariant::DeadString(s) => s,
        }
    }
}

#[allow(non_snake_case)]
pub(super) struct KeyboardTranslatorInternal {
    pub(super) keyboard_layout: HKL,
    pub(super) OnInvalid: DelegateStorage<TypedEventHandler<bindings::KeyboardTranslator, HSTRING>>,
    pub(super) OnTranslated:
        DelegateStorage<TypedEventHandler<bindings::KeyboardTranslator, HSTRING>>,
    pub(super) OnKeyTranslated:
        DelegateStorage<TypedEventHandler<bindings::KeyboardTranslator, HSTRING>>,
    possible_altgr: HashMap<String, String>,
    possible_dead: HashMap<String, u16>,
    pub(super) state: String,
    pub(super) sequence_definition: Weak<bindings::SequenceDefinition>,
    pub(super) parent: Weak<bindings::KeyboardTranslator>,
}

impl KeyboardTranslatorInternal {
    pub(super) fn new(
        sequence_definition: Weak<bindings::SequenceDefinition>,
        parent: Weak<bindings::KeyboardTranslator>,
    ) -> Self {
        Self {
            keyboard_layout: HKL(null_mut()),
            OnInvalid: DelegateStorage::new(),
            OnTranslated: DelegateStorage::new(),
            OnKeyTranslated: DelegateStorage::new(),
            possible_altgr: HashMap::new(),
            possible_dead: HashMap::new(),
            state: String::new(),
            sequence_definition,
            parent,
        }
    }

    fn report_invalid(&mut self, message: &HSTRING) -> Result<()> {
        self.OnInvalid
            .invoke_all(|d| d.Invoke(&get_strong_ref(&self.parent)?, message))
    }

    pub(super) fn translate(
        &mut self,
        vkcode: u32,
        scancode: u32,
        keystate: &[u8; 256],
    ) -> Result<String> {
        match vk_to_unicode(vkcode, scancode, &keystate, self.keyboard_layout) {
            Ok(s) => Ok(s.into()),
            Err(e) => {
                self.report_invalid(h!("Invalid VK code"))?;

                match e {
                    VKToUnicodeError::NoTranslation => Err(E_INVALIDARG.into()),
                    VKToUnicodeError::InvalidReturn => Err(Error::new(
                        HRESULT::from_win32(ERROR_NO_UNICODE_TRANSLATION.0),
                        "Invalid return from ToUnicodeEx",
                    )),
                }
            }
        }
    }

    pub(super) fn forward(
        &mut self,
        destination: u8,
        value: String,
    ) -> std::result::Result<String, SequenceDefinitionError> {
        match destination {
            0 => {
                // Forward to SequenceTranslator
                self.state.push_str(&value);
                match self
                    .get_seqdef_ref()?
                    .cast_object_ref::<SequenceDefinition>()?
                    .translate_sequence(&self.state)
                {
                    Ok(s) => {
                        self.state.clear();
                        Ok(s)
                    }
                    Err(SequenceDefinitionError::Incomplete) => {
                        Err(SequenceDefinitionError::Incomplete)
                    }
                    Err(e) => {
                        self.state.clear();
                        Err(e)
                    }
                }
            }
            1 => {
                // Forward to UnicodeTranslator
                self.state.push_str(&value);
                match self.parse_as_unicode() {
                    Ok(s) => {
                        self.state.clear();
                        Ok(s)
                    }
                    Err(SequenceDefinitionError::Incomplete) => {
                        Err(SequenceDefinitionError::Incomplete)
                    }
                    Err(e) => {
                        self.state.clear();
                        Err(e)
                    }
                }
            }
            _ => Err(SequenceDefinitionError::Failure(E_INVALIDARG.into())),
        }
    }

    pub(super) fn report(
        &mut self,
        result: std::result::Result<String, SequenceDefinitionError>,
    ) -> Result<()> {
        match result {
            Ok(s) => {
                let _ = send_text_clipboard(&s.clone().into())?;
                self.OnTranslated
                    .invoke_all(|d| d.Invoke(&get_strong_ref(&self.parent)?, &(&s).into()))?;
                Ok(())
            }
            Err(SequenceDefinitionError::ValueNotFound) => {
                self.report_invalid(h!("Value not found"))
            }
            Err(SequenceDefinitionError::Incomplete) => {
                // Do nothing
                Ok(())
            }
            Err(SequenceDefinitionError::Failure(e)) => Err(e),
        }
    }

    pub(super) fn report_key(&mut self, key: &str) -> Result<()> {
        self.OnKeyTranslated
            .invoke_all(|d| d.Invoke(&get_strong_ref(&self.parent)?, &(key).into()))
    }

    pub(super) fn analyze_layout(&mut self) -> Result<()> {
        let mut no_altgr = vec![String::new(); 512];
        let mut keystate = [0; 256];

        for i in 0..0x400 {
            let vk_code = i & 0xFF;
            let has_shift = (i & 0x100) != 0;
            let has_altgr = (i & 0x200) != 0;

            if has_shift {
                keystate[VK_SHIFT.0 as usize] = 0x80;
            } else {
                keystate[VK_SHIFT.0 as usize] = 0;
            }

            if has_altgr {
                keystate[VK_CONTROL.0 as usize] = 0x80;
                keystate[VK_MENU.0 as usize] = 0x80;
            } else {
                keystate[VK_CONTROL.0 as usize] = 0;
                keystate[VK_MENU.0 as usize] = 0;
            }

            if let Ok(s) = vk_to_unicode(vk_code, 0, &keystate, self.keyboard_layout) {
                if has_altgr {
                    self.possible_altgr
                        .insert(s.clone().into(), no_altgr[i as usize - 0x200].clone());
                } else {
                    no_altgr[i as usize] = s.clone().into();
                }

                if let StringVariant::DeadString(s) = s {
                    self.possible_dead.insert(s, i as u16);
                }
            }

            to_unicode_ex_clear_state();
        }

        Ok(())
    }

    fn get_seqdef_ref(&self) -> Result<bindings::SequenceDefinition> {
        self.sequence_definition
            .upgrade()
            .ok_or_else(|| Error::new(E_POINTER, "Weak pointer died"))
    }

    fn parse_as_unicode(&self) -> std::result::Result<String, SequenceDefinitionError> {
        if self.state.ends_with(char::is_whitespace) {
            Ok(char::from_u32(
                u32::from_str_radix(&self.state.trim_end(), 16)
                    .map_err(|_| SequenceDefinitionError::ValueNotFound)?,
            )
            .ok_or(SequenceDefinitionError::ValueNotFound)?
            .to_string())
        } else {
            Err(SequenceDefinitionError::Incomplete)
        }
    }
}

impl Debug for KeyboardTranslatorInternal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyboardTranslatorInternal")
            .field("keyboard_layout", &self.keyboard_layout)
            .field("report_invalid", &self.OnInvalid)
            .field("report_translated", &self.OnTranslated)
            .field("possible_altgr", &self.possible_altgr)
            .field("possible_dead", &self.possible_dead)
            .field("state", &self.state)
            .finish()
    }
}

fn vk_to_unicode(
    vkcode: u32,
    scancode: u32,
    keystate: &[u8; 256],
    keyboard_layout: HKL,
) -> std::result::Result<StringVariant, VKToUnicodeError> {
    let mut buffer = [0; 8];
    let status =
        unsafe { ToUnicodeEx(vkcode, scancode, keystate, &mut buffer, 4, keyboard_layout) };

    if status > 0 {
        Ok(StringVariant::LiveString(
            String::from_utf16(&buffer[..status as usize])
                .map_err(|_| VKToUnicodeError::InvalidReturn)?,
        ))
    } else if status < 0 {
        let status =
            unsafe { ToUnicodeEx(vkcode, scancode, keystate, &mut buffer, 4, keyboard_layout) };

        if status > 0 {
            Ok(StringVariant::DeadString(
                String::from_utf16(&buffer[..status as usize])
                    .map_err(|_| VKToUnicodeError::InvalidReturn)?,
            ))
        } else {
            Err(VKToUnicodeError::NoTranslation)
        }
    } else {
        Err(VKToUnicodeError::NoTranslation)
    }
}

const EMPTY_KEYSTATE: [u8; 256] = [0; 256];

fn to_unicode_ex_clear_state() {
    let mut buffer = [0; 8];
    unsafe {
        ToUnicodeEx(
            VK_SPACE.0.into(),
            0,
            &EMPTY_KEYSTATE,
            &mut buffer,
            0,
            HKL(null_mut()),
        );
        ToUnicodeEx(
            VK_SPACE.0.into(),
            0,
            &EMPTY_KEYSTATE,
            &mut buffer,
            0,
            HKL(null_mut()),
        );
    }
}
