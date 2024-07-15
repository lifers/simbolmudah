use std::fmt::Display;

use smol_str::SmolStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum MappedString {
    Basic(SmolStr),
    Extra(String),
}

impl From<SmolStr> for MappedString {
    fn from(value: SmolStr) -> Self {
        MappedString::Basic(value)
    }
}

impl From<String> for MappedString {
    fn from(value: String) -> Self {
        MappedString::Extra(value)
    }
}

impl Into<String> for MappedString {
    fn into(self) -> String {
        match self {
            MappedString::Basic(c) => c.into(),
            MappedString::Extra(s) => s,
        }
    }
}

impl Display for MappedString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MappedString::Basic(c) => write!(f, "{}", c),
            MappedString::Extra(s) => write!(f, "{}", s),
        }
    }
}
