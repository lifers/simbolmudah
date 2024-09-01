use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum MappedString {
    Basic(Box<str>),
    Extra(Box<str>),
}

impl Into<String> for MappedString {
    fn into(self) -> String {
        match self {
            MappedString::Basic(c) => c.into(),
            MappedString::Extra(s) => s.into(),
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
