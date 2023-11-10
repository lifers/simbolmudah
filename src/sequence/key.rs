use std::{cell::RefCell, collections::HashMap};

use windows::Win32::UI::Input::KeyboardAndMouse::{
    VIRTUAL_KEY, VK_BACK, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END, VK_HOME, VK_LCONTROL, VK_LEFT,
    VK_LMENU, VK_LSHIFT, VK_MENU, VK_RCONTROL, VK_RIGHT, VK_RMENU, VK_RSHIFT, VK_SHIFT, VK_TAB,
    VK_UP,
};

pub const VK_COMPOSE: VIRTUAL_KEY = VIRTUAL_KEY(0x100);
pub const VK_NONE: VIRTUAL_KEY = VIRTUAL_KEY(0x00);

thread_local! {
    static KEY_LABELS: RefCell<HashMap<u16, &'static str>> = RefCell::new({
        HashMap::from([
            (VK_COMPOSE.0, "♦"),
            (VK_UP.0, "▲"),
            (VK_DOWN.0, "▼"),
            (VK_LEFT.0, "◀"),
            (VK_RIGHT.0, "▶"),
            (VK_HOME.0, "Home"),
            (VK_END.0, "End"),
            (VK_BACK.0, "⌫"),
            (VK_DELETE.0, "␡"),
            (VK_TAB.0, "↹"),
        ])
    });
}

#[derive(PartialEq, Eq)]
pub enum Key {
    VirtualKey(VIRTUAL_KEY),
    Char(char),
}

impl Key {
    pub fn is_modifier(&self) -> bool {
        match self {
            Key::VirtualKey(x) => match *x {
                VK_SHIFT | VK_CONTROL | VK_MENU | VK_LSHIFT | VK_RSHIFT | VK_LCONTROL
                | VK_RCONTROL | VK_LMENU | VK_RMENU => true,
                _ => false,
            },
            _ => false,
        }
    }
}
