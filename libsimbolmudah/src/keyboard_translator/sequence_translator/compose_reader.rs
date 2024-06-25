use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
};

use regex::Regex;

use super::{keysym_reader::KeySymDef, mapped_string::MappedString, SequenceTranslatorError};

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
    pub(super) fn build(keysym: &KeySymDef) -> Result<Self, SequenceTranslatorError> {
        let content = get_compose_def(keysym)?;
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
) -> Result<BTreeMap<String, MappedString>, SequenceTranslatorError> {
    let file = File::open(COMPOSEDEF).map_err(|_| SequenceTranslatorError::FileRead)?;
    let reader = BufReader::new(file);
    let output = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("../resource/composelist.txt")
        .map_err(|_| SequenceTranslatorError::FileWrite)?;
    let mut writer = BufWriter::new(output);

    let mut result = BTreeMap::new();
    let regex2 =
        Regex::new(COMPOSE_REGEX_2_STR).map_err(|_| SequenceTranslatorError::RegexBuild)?;
    let regex3 =
        Regex::new(COMPOSE_REGEX_3_STR).map_err(|_| SequenceTranslatorError::RegexBuild)?;
    let regex4 =
        Regex::new(COMPOSE_REGEX_4_STR).map_err(|_| SequenceTranslatorError::RegexBuild)?;

    for line in reader.lines() {
        let line = line.map_err(|_| SequenceTranslatorError::ReadLine)?;
        let (key, value) = decode_entry(&line, &regex2, &regex3, &regex4, keysym)?;
        writeln!(writer, "{:?} {}", key, value).map_err(|_| SequenceTranslatorError::WriteLine)?;
        result.insert(key, value);
    }

    // result.insert(">=".into(), MappedString::Basic('≥'));
    // result.insert("oe".into(), MappedString::Basic('œ'));
    result.insert("wkwk".into(), '🤣'.into());
    result.insert("pr".into(), "peradaban".to_string().into());

    Ok(result)
}

fn decode_entry(
    line: &str,
    regex2: &Regex,
    regex3: &Regex,
    regex4: &Regex,
    keysymdef: &KeySymDef,
) -> Result<(String, MappedString), SequenceTranslatorError> {
    if let Some(caps) = regex2.captures(line) {
        let key1 = keysymdef.get_key(
            caps.get(1)
                .ok_or_else(|| SequenceTranslatorError::RegexParse)?
                .as_str(),
        )?;
        let key2 = keysymdef.get_key(
            caps.get(2)
                .ok_or_else(|| SequenceTranslatorError::RegexParse)?
                .as_str(),
        )?;
        let value = caps
            .get(3)
            .ok_or_else(|| SequenceTranslatorError::RegexParse)?
            .as_str();

        Ok((
            [key1, key2].into_iter().collect(),
            value.chars().next().unwrap().into(),
        ))
    } else if let Some(caps) = regex3.captures(line) {
        let key1 = keysymdef.get_key(
            caps.get(1)
                .ok_or_else(|| SequenceTranslatorError::RegexParse)?
                .as_str(),
        )?;
        let key2 = keysymdef.get_key(
            caps.get(2)
                .ok_or_else(|| SequenceTranslatorError::RegexParse)?
                .as_str(),
        )?;
        let key3 = keysymdef.get_key(
            caps.get(3)
                .ok_or_else(|| SequenceTranslatorError::RegexParse)?
                .as_str(),
        )?;
        let value = caps
            .get(4)
            .ok_or_else(|| SequenceTranslatorError::RegexParse)?
            .as_str();

        Ok((
            [key1, key2, key3].into_iter().collect(),
            value.chars().next().unwrap().into(),
        ))
    } else if let Some(caps) = regex4.captures(line) {
        let key1 = keysymdef.get_key(
            caps.get(1)
                .ok_or_else(|| SequenceTranslatorError::RegexParse)?
                .as_str(),
        )?;
        let key2 = keysymdef.get_key(
            caps.get(2)
                .ok_or_else(|| SequenceTranslatorError::RegexParse)?
                .as_str(),
        )?;
        let key3 = keysymdef.get_key(
            caps.get(3)
                .ok_or_else(|| SequenceTranslatorError::RegexParse)?
                .as_str(),
        )?;
        let key4 = keysymdef.get_key(
            caps.get(4)
                .ok_or_else(|| SequenceTranslatorError::RegexParse)?
                .as_str(),
        )?;
        let value = caps
            .get(5)
            .ok_or_else(|| SequenceTranslatorError::RegexParse)?
            .as_str();

        Ok((
            [key1, key2, key3, key4].into_iter().collect(),
            value.chars().next().unwrap().into(),
        ))
    } else {
        Err(SequenceTranslatorError::RegexParse)
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