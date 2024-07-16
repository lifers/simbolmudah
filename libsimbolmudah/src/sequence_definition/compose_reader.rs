use super::{
    internal::SequenceDefinitionInternal, keysym_reader::KeySymDef, mapped_string::MappedString,
};
use crate::{fail, fail_message};
use regex::Regex;
use smol_str::ToSmolStr;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
};
use windows::core::Result;

const COMPOSE_REGEX_2_STR: &str =
    r#"^<Multi_key> <([a-zA-Z0-9_]+)> <([a-zA-Z0-9_]+)>\s+: "(.*)".*$"#;
const COMPOSE_REGEX_3_STR: &str =
    r#"^<Multi_key> <([a-zA-Z0-9_]+)> <([a-zA-Z0-9_]+)> <([a-zA-Z0-9_]+)>\s+: "(.*)".*$"#;
const COMPOSE_REGEX_4_STR: &str = r#"^<Multi_key> <([a-zA-Z0-9_]+)> <([a-zA-Z0-9_]+)> <([a-zA-Z0-9_]+)> <([a-zA-Z0-9_]+)>\s+: "(.*)".*$"#;

pub(super) struct ComposeDef {
    content: BTreeMap<String, MappedString>,
}

impl ComposeDef {
    pub(super) fn build(
        keysym: &KeySymDef,
        path: &str,
        definition: &mut SequenceDefinitionInternal,
    ) -> Result<Self> {
        let content = get_compose_def(keysym, path, definition)?;
        Ok(Self { content })
    }
}

impl IntoIterator for ComposeDef {
    type Item = (String, MappedString);
    type IntoIter = std::collections::btree_map::IntoIter<String, MappedString>;

    fn into_iter(self) -> Self::IntoIter {
        self.content.into_iter()
    }
}

fn get_compose_def(
    keysym: &KeySymDef,
    path: &str,
    definition: &mut SequenceDefinitionInternal,
) -> Result<BTreeMap<String, MappedString>> {
    let file = File::open(path).map_err(fail)?;
    let reader = BufReader::new(file);

    let mut result = BTreeMap::new();
    let regex2 = Regex::new(COMPOSE_REGEX_2_STR).map_err(fail)?;
    let regex3 = Regex::new(COMPOSE_REGEX_3_STR).map_err(fail)?;
    let regex4 = Regex::new(COMPOSE_REGEX_4_STR).map_err(fail)?;

    for line in reader.lines() {
        let line = line.map_err(fail)?;
        if let Ok((key, value)) = decode_entry(&line, &regex2, &regex3, &regex4, keysym) {
            if let MappedString::Basic(ref s) = value {
                definition.index_char(s.chars().next().unwrap())?;
            }
            result.insert(key, value);
        }
    }

    // result.insert(">=".into(), MappedString::Basic('≥'));
    // result.insert("oe".into(), MappedString::Basic('œ'));
    result.insert("wkwk".into(), "🤣".to_string().into());
    result.insert("pr".into(), "peradaban".to_string().into());

    Ok(result)
}

fn decode_entry(
    line: &str,
    regex2: &Regex,
    regex3: &Regex,
    regex4: &Regex,
    keysymdef: &KeySymDef,
) -> Result<(String, MappedString)> {
    if let Some(caps) = regex2.captures(line) {
        let key1 = keysymdef.get_key(
            caps.get(1)
                .ok_or_else(|| fail_message("Regex parse"))?
                .as_str(),
        )?;
        let key2 = keysymdef.get_key(
            caps.get(2)
                .ok_or_else(|| fail_message("Regex parse"))?
                .as_str(),
        )?;
        let value = caps
            .get(3)
            .ok_or_else(|| fail_message("Regex parse"))?
            .as_str();

        Ok((
            [key1, key2].into_iter().collect(),
            value.to_smolstr().into(),
        ))
    } else if let Some(caps) = regex3.captures(line) {
        let key1 = keysymdef.get_key(
            caps.get(1)
                .ok_or_else(|| fail_message("Regex parse"))?
                .as_str(),
        )?;
        let key2 = keysymdef.get_key(
            caps.get(2)
                .ok_or_else(|| fail_message("Regex parse"))?
                .as_str(),
        )?;
        let key3 = keysymdef.get_key(
            caps.get(3)
                .ok_or_else(|| fail_message("Regex parse"))?
                .as_str(),
        )?;
        let value = caps
            .get(4)
            .ok_or_else(|| fail_message("Regex parse"))?
            .as_str();

        Ok((
            [key1, key2, key3].into_iter().collect(),
            value.to_smolstr().into(),
        ))
    } else if let Some(caps) = regex4.captures(line) {
        let key1 = keysymdef.get_key(
            caps.get(1)
                .ok_or_else(|| fail_message("Regex parse"))?
                .as_str(),
        )?;
        let key2 = keysymdef.get_key(
            caps.get(2)
                .ok_or_else(|| fail_message("Regex parse"))?
                .as_str(),
        )?;
        let key3 = keysymdef.get_key(
            caps.get(3)
                .ok_or_else(|| fail_message("Regex parse"))?
                .as_str(),
        )?;
        let key4 = keysymdef.get_key(
            caps.get(4)
                .ok_or_else(|| fail_message("Regex parse"))?
                .as_str(),
        )?;
        let value = caps
            .get(5)
            .ok_or_else(|| fail_message("Regex parse"))?
            .as_str();

        Ok((
            [key1, key2, key3, key4].into_iter().collect(),
            value.to_smolstr().into(),
        ))
    } else {
        Err(fail_message("Regex parse"))
    }
}

#[cfg(test)]
mod tests {
    use windows_core::Weak;

    use super::*;

    const KEYSYMDEF: &str = "tests/keysymdef.txt";
    const COMPOSEDEF: &str = "tests/Compose.pre";

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

    #[test]
    fn test_decode_entry_two_keys() {
        let mut seqdef = SequenceDefinitionInternal::new(Weak::new());
        let regex2 = Regex::new(COMPOSE_REGEX_2_STR).unwrap();
        let regex3 = Regex::new(COMPOSE_REGEX_3_STR).unwrap();
        let regex4 = Regex::new(COMPOSE_REGEX_4_STR).unwrap();
        let keysymdef = KeySymDef::new(KEYSYMDEF, &mut seqdef).unwrap(); // Assuming a constructor for simplicity

        let line = "<Multi_key> <A> <B> : \"C\".";
        let expected = Ok((
            "AB".to_string(),
            MappedString::Basic("C".to_smolstr().into()),
        ));
        let result = decode_entry(line, &regex2, &regex3, &regex4, &keysymdef);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_decode_entry_three_keys() {
        let mut seqdef = SequenceDefinitionInternal::new(Weak::new());
        let regex2 = Regex::new(COMPOSE_REGEX_2_STR).unwrap();
        let regex3 = Regex::new(COMPOSE_REGEX_3_STR).unwrap();
        let regex4 = Regex::new(COMPOSE_REGEX_4_STR).unwrap();
        let keysymdef = KeySymDef::new(KEYSYMDEF, &mut seqdef).unwrap(); // Assuming a constructor for simplicity

        let line = "<Multi_key> <A> <B> <C> : \"D\".";
        let expected = Ok((
            "ABC".to_string(),
            MappedString::Basic("D".to_smolstr().into()),
        ));
        let result = decode_entry(line, &regex2, &regex3, &regex4, &keysymdef);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_decode_entry_four_keys() {
        let mut seqdef = SequenceDefinitionInternal::new(Weak::new());
        let regex2 = Regex::new(COMPOSE_REGEX_2_STR).unwrap();
        let regex3 = Regex::new(COMPOSE_REGEX_3_STR).unwrap();
        let regex4 = Regex::new(COMPOSE_REGEX_4_STR).unwrap();
        let keysymdef = KeySymDef::new(KEYSYMDEF, &mut seqdef).unwrap(); // Assuming a constructor for simplicity

        let line = "<Multi_key> <A> <B> <C> <D> : \"E\".";
        let expected = Ok((
            "ABCD".to_string(),
            MappedString::Basic("E".to_smolstr().into()),
        ));
        let result = decode_entry(line, &regex2, &regex3, &regex4, &keysymdef);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_compose_def() {
        let mut seqdef = SequenceDefinitionInternal::new(Weak::new());
        let keysymdef = KeySymDef::new(KEYSYMDEF, &mut seqdef).unwrap(); // Assuming a constructor for simplicity
        let map = get_compose_def(&keysymdef, COMPOSEDEF, &mut seqdef).unwrap();

        assert!(map.contains_key("wkwk"));
        assert_eq!(
            map.get("wkwk").unwrap(),
            &MappedString::Basic("🤣".to_smolstr().into())
        );
        assert!(map.contains_key("pr"));
        assert_eq!(
            map.get("pr").unwrap(),
            &MappedString::Extra("peradaban".to_string())
        );
    }
}
