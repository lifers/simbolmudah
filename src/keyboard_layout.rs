use std::{
    collections::HashMap, error::Error, ffi::OsString, fmt::Display,
    os::windows::prelude::OsStringExt,
};

use itertools::iproduct;
use windows::Win32::UI::{
    Input::KeyboardAndMouse::{
        GetKeyboardLayout, ToUnicodeEx, VIRTUAL_KEY, VK_CONTROL, VK_MENU, VK_SHIFT, VK_SPACE,
    },
    TextServices::HKL,
};

pub struct KeyboardLayout {
    possible_dead_keys: HashMap<String, u16>,
    possible_altgr_keys: HashMap<String, String>,
    saved_dead_key: u16,
    current_layout: HKL,
    transformed_hkl: HKL,
}

#[derive(Debug)]
enum ParseVKError {
    DeadKey(String),
    NoTranslation,
    InvalidUnicode,
}

// impl Display for ParseVKError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             ParseVKError::DeadKey(x) => write!(f, "Dead key is {x}"),
//             ParseVKError::NoTranslation => write!(f, "No VK translation available"),
//             ParseVKError::InvalidUnicode => write!(f, "Invalid Unicode produced by ToUnicodeEx"),
//         }
//     }
// }

// impl Error for ParseVKError {}

impl KeyboardLayout {
    fn vk_to_unicode(
        &self,
        virtual_key: VIRTUAL_KEY,
        scan_code: u32,
        keystate: &[u8; 256],
        flags: u32,
    ) -> Result<String, ParseVKError> {
        let mut buffer = Vec::<u16>::new();
        let status = unsafe {
            ToUnicodeEx(
                virtual_key.0.into(),
                scan_code,
                keystate,
                &mut buffer,
                flags,
                self.transformed_hkl,
            )
        };

        if status < 0 {
            let _ = unsafe {
                ToUnicodeEx(
                    VK_SPACE.0.into(),
                    scan_code,
                    keystate,
                    &mut buffer,
                    flags,
                    self.transformed_hkl,
                )
            };
        }

        if status == 0 {
            Err(ParseVKError::NoTranslation)
        } else if let Ok(ret) = OsString::from_wide(&buffer).into_string() {
            if status < 0 {
                Err(ParseVKError::DeadKey(ret))
            } else {
                Ok(ret)
            }
        } else {
            Err(ParseVKError::InvalidUnicode)
        }
    }

    fn to_unicode_ex_clear_buffer(&self) {
        let _ = self.vk_to_unicode(VK_SPACE, 0, &[0; 256], 0);
        let _ = self.vk_to_unicode(VK_SPACE, 0, &[0; 256], 0);
    }

    fn analyze_layout(&mut self) {
        self.current_layout = unsafe { GetKeyboardLayout(0) };

        let mut no_altgr = vec![String::new(); 0x200];
        let mut state = [0u8; 256];
        const FT: &[bool; 2] = &[true, false];

        for (has_altgr, has_shift, codepoint) in iproduct!(FT, FT, 0..0x100u16) {
            if *has_altgr {
                state[VK_MENU.0 as usize] = 0x80;
                state[VK_CONTROL.0 as usize] = 0x80;
            } else {
                state[VK_MENU.0 as usize] = 0x00;
                state[VK_CONTROL.0 as usize] = 0x00;
            }

            if *has_shift {
                state[VK_SHIFT.0 as usize] = 0x80;
            } else {
                state[VK_SHIFT.0 as usize] = 0x00;
            }

            let curr = self.vk_to_unicode(VIRTUAL_KEY(codepoint), 0, &state, 0);
            self.to_unicode_ex_clear_buffer();

            let altgr_codepoint = match *has_shift {
                true => (codepoint + 0x100) as usize,
                false => codepoint as usize,
            };

            if let Ok(x) = curr {
                if *has_altgr {
                    if no_altgr[altgr_codepoint] != "" && no_altgr[altgr_codepoint] != x {
                        self.possible_altgr_keys
                            .insert(no_altgr[altgr_codepoint].clone(), x);
                    }
                } else {
                    no_altgr[altgr_codepoint] = x;
                }
            } else if let Err(ParseVKError::DeadKey(ret)) = curr {
                let mut dead_codepoint = codepoint;
                if *has_altgr {
                    dead_codepoint += 0x200;
                }
                if *has_shift {
                    dead_codepoint += 0x100;
                }
                self.possible_dead_keys.insert(ret, dead_codepoint);
            }
        }
    }
}
