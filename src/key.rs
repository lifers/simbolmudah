use std::{
    cell::RefCell,
    collections::HashMap,
    fmt,
    hash::{Hash, Hasher},
};

use windows::Win32::UI::Input::KeyboardAndMouse::{
    VIRTUAL_KEY, VK_BACK, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END, VK_HOME, VK_LCONTROL, VK_LEFT,
    VK_LMENU, VK_LSHIFT, VK_MENU, VK_RCONTROL, VK_RIGHT, VK_RMENU, VK_RSHIFT, VK_SHIFT, VK_TAB,
    VK_UP,
};

pub(super) const VK_COMPOSE: VIRTUAL_KEY = VIRTUAL_KEY(0x100);
pub(super) const VK_NONE: VIRTUAL_KEY = VIRTUAL_KEY(0x00);

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

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub(super) enum Key {
    VirtualKey(VIRTUAL_KEY),
    Char(char),
}

impl Hash for Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            // Assuming VIRTUAL_KEY is a tuple struct with one field
            Key::VirtualKey(vk) => vk.0.hash(state),
            Key::Char(c) => c.hash(state),
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Key::VirtualKey(vk) => write!(f, "{}", vk.0),
            Key::Char(c) => write!(f, "{}", c),
        }
    }
}

impl From<u32> for Key {
    fn from(value: u32) -> Key {
        Key::Char(
            char::from_u32(value).expect(format!("Invalid Unicode value: {}", value).as_str()),
        )
    }
}

impl Key {
    pub(super) fn is_modifier(&self) -> bool {
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
