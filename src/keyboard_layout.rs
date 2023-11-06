use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    error::Error,
    ffi::OsString,
    fmt::Display,
    os::windows::prelude::OsStringExt,
};

use itertools::iproduct;
use windows::Win32::UI::{
    Input::KeyboardAndMouse::{
        GetKeyboardLayout, ToUnicodeEx, VIRTUAL_KEY, VK_CONTROL, VK_MENU, VK_SHIFT, VK_SPACE,
    },
    TextServices::HKL,
};

thread_local! {
    static POSSIBLE_DEAD_KEYS: RefCell<HashMap<OsString, u16>> = RefCell::new(HashMap::new());
    static POSSIBLE_ALTGR_KEYS: RefCell<HashMap<OsString, OsString>> = RefCell::new(HashMap::new());
    static SAVED_DEAD_KEY: Cell<u16> = Cell::new(Default::default());
    static CURRENT_LAYOUT: Cell<HKL> = Cell::new(Default::default());
}

#[derive(Debug)]
pub enum ParseVKError {
    DeadKey(OsString),
    NoTranslation,
    InvalidUnicode,
}

impl Display for ParseVKError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseVKError::DeadKey(x) => write!(f, "Dead key is {:?}", x),
            ParseVKError::NoTranslation => write!(f, "No VK translation available"),
            ParseVKError::InvalidUnicode => write!(f, "Invalid Unicode produced by ToUnicodeEx"),
        }
    }
}

impl Error for ParseVKError {}

pub fn vk_to_unicode(
    virtual_key: VIRTUAL_KEY,
    scan_code: u32,
    keystate: &[u8; 256],
    flags: u32,
) -> Result<OsString, ParseVKError> {
    let mut buffer = [0; 8];
    let status = unsafe {
        ToUnicodeEx(
            virtual_key.0.into(),
            scan_code,
            keystate,
            &mut buffer,
            flags,
            CURRENT_LAYOUT.get(),
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
                CURRENT_LAYOUT.get(),
            )
        };
    }

    if status == 0 {
        Err(ParseVKError::NoTranslation)
    } else if let Ok(ret) = OsString::from_wide(&buffer).into_string() {
        if status < 0 {
            Err(ParseVKError::DeadKey(ret.into()))
        } else {
            Ok(ret.into())
        }
    } else {
        Err(ParseVKError::InvalidUnicode)
    }
}

fn to_unicode_ex_clear_buffer() {
    let _ = vk_to_unicode(VK_SPACE, 0, &[0; 256], 0);
    let _ = vk_to_unicode(VK_SPACE, 0, &[0; 256], 0);
}

pub fn analyze_layout() {
    CURRENT_LAYOUT.set(unsafe { GetKeyboardLayout(0) });

    let mut no_altgr = vec![OsString::new(); 0x200];
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

        let curr = vk_to_unicode(VIRTUAL_KEY(codepoint), 0, &state, 0);
        to_unicode_ex_clear_buffer();

        let altgr_codepoint = match *has_shift {
            true => (codepoint + 0x100) as usize,
            false => codepoint as usize,
        };

        if let Ok(x) = curr {
            if *has_altgr {
                if no_altgr[altgr_codepoint] != "" && no_altgr[altgr_codepoint] != x {
                    POSSIBLE_ALTGR_KEYS
                        .with_borrow_mut(|m| m.insert(no_altgr[altgr_codepoint].clone(), x));
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
            POSSIBLE_DEAD_KEYS.with_borrow_mut(|m| m.insert(ret, dead_codepoint));
        }
    }
}
