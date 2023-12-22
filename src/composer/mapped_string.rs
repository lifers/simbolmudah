use std::{ffi::OsString, fmt::Display};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MappedString {
    Basic(char),
    Extra(String),
}

impl Into<u64> for MappedString {
    fn into(self) -> u64 {
        match self {
            MappedString::Basic(c) => c as u64,
            MappedString::Extra(_) => 1_000_000_000_000,
        }
    }
}

impl From<u64> for MappedString {
    fn from(value: u64) -> Self {
        if value < 1_000_000_000_000 {
            MappedString::Basic(char::from_u32(value as u32).unwrap())
        } else {
            MappedString::Extra("peradaban".into())
        }
    }
}

impl Into<OsString> for MappedString {
    fn into(self) -> OsString {
        match self {
            MappedString::Basic(c) => OsString::from(c.to_string()),
            MappedString::Extra(s) => OsString::from(s),
        }
    }
}

impl Display for MappedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MappedString::Basic(c) => write!(f, "{}", c),
            MappedString::Extra(s) => write!(f, "{}", s),
        }
    }
}
