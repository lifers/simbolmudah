use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
};

use regex::Regex;

use crate::{key::Key, key_sequence::KeySequence};

use super::{keysym_reader::KeySymDef, mapped_string::MappedString};

const COMPOSEDEF: &str = "3rd-party/libx11/nls/en_US.UTF-8/Compose.pre";
const COMPOSE_REGEX_2_STR: &str =
    r#"^<Multi_key> <([a-zA-Z0-9_]+)> <([a-zA-Z0-9_]+)>\s+: "(.*)".*$"#;
const COMPOSE_REGEX_3_STR: &str =
    r#"^<Multi_key> <([a-zA-Z0-9_]+)> <([a-zA-Z0-9_]+)> <([a-zA-Z0-9_]+)>\s+: "(.*)".*$"#;
const COMPOSE_REGEX_4_STR: &str = r#"^<Multi_key> <([a-zA-Z0-9_]+)> <([a-zA-Z0-9_]+)> <([a-zA-Z0-9_]+)> <([a-zA-Z0-9_]+)>\s+: "(.*)".*$"#;

pub(super) struct ComposeDef {
    content: BTreeMap<String, MappedString>,
}

impl ComposeDef {
    pub(super) fn build(keysym: &KeySymDef) -> Self {
        let content = Self::get_compose_def(keysym);
        Self { content }
    }

    fn get_compose_def(keysym: &KeySymDef) -> BTreeMap<String, MappedString> {
        let file = File::open(COMPOSEDEF).unwrap();
        let reader = BufReader::new(file);
        let output = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("resource/composelist.txt")
            .unwrap();
        let mut writer = BufWriter::new(output);

        let mut result = BTreeMap::new();
        let regex2 = Regex::new(COMPOSE_REGEX_2_STR).unwrap();
        let regex3 = Regex::new(COMPOSE_REGEX_3_STR).unwrap();
        let regex4 = Regex::new(COMPOSE_REGEX_4_STR).unwrap();

        for line in reader.lines() {
            let line = line.unwrap();
            if let Some((keys, value)) =
                Self::decode_entry(&line, &regex2, &regex3, &regex4, keysym)
            {
                writeln!(writer, "{:?} {}", keys, value).unwrap();
                result.insert(
                    keys.try_into().unwrap(),
                    MappedString::Basic(value.chars().next().unwrap()),
                );
            }
        }

        // result.insert(">=".into(), MappedString::Basic('≥'));
        // result.insert("oe".into(), MappedString::Basic('œ'));
        result.insert("wkwk".into(), MappedString::Basic('🤣'));
        result.insert("pr".into(), MappedString::Extra("peradaban".into()));

        result
    }

    fn decode_entry(
        line: &str,
        regex2: &Regex,
        regex3: &Regex,
        regex4: &Regex,
        keysymdef: &KeySymDef,
    ) -> Option<(KeySequence, String)> {
        if let Some(caps) = regex2.captures(line) {
            let key1 = Self::str_to_key(caps.get(1).unwrap().as_str(), keysymdef)?;
            let key2 = Self::str_to_key(caps.get(2).unwrap().as_str(), keysymdef)?;
            let value = caps.get(3).unwrap().as_str().into();
            Some((vec![key1, key2].into(), value))
        } else if let Some(caps) = regex3.captures(line) {
            let key1 = Self::str_to_key(caps.get(1).unwrap().as_str(), keysymdef)?;
            let key2 = Self::str_to_key(caps.get(2).unwrap().as_str(), keysymdef)?;
            let key3 = Self::str_to_key(caps.get(3).unwrap().as_str(), keysymdef)?;
            let value = caps.get(4).unwrap().as_str().into();
            Some((vec![key1, key2, key3].into(), value))
        } else if let Some(caps) = regex4.captures(line) {
            let key1 = Self::str_to_key(caps.get(1).unwrap().as_str(), keysymdef)?;
            let key2 = Self::str_to_key(caps.get(2).unwrap().as_str(), keysymdef)?;
            let key3 = Self::str_to_key(caps.get(3).unwrap().as_str(), keysymdef)?;
            let key4 = Self::str_to_key(caps.get(4).unwrap().as_str(), keysymdef)?;
            let value = caps.get(5).unwrap().as_str().into();
            Some((vec![key1, key2, key3, key4].into(), value))
        } else {
            return None;
        }
    }

    fn str_to_key(string: &str, keysymdef: &KeySymDef) -> Option<Key> {
        if let Some(key) = keysymdef.get_key(string) {
            return Some(key);
        }
        Key::from_unicode_string(string)
    }
}

impl IntoIterator for ComposeDef {
    type Item = (String, MappedString);
    type IntoIter = std::collections::btree_map::IntoIter<String, MappedString>;

    fn into_iter(self) -> Self::IntoIter {
        self.content.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_regex() {
        let regex = Regex::new(COMPOSE_REGEX_2_STR).unwrap();
        let line =
            "<Multi_key> <y> <quote_dbl>		: \"ÿ\"	ydiaeresis # LATIN SMALL LETTER Y WITH DIAERESIS";
        let caps = regex.captures(&line).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "y");
        assert_eq!(caps.get(2).unwrap().as_str(), "quote_dbl");
        assert_eq!(caps.get(3).unwrap().as_str(), "ÿ");
    }
}
