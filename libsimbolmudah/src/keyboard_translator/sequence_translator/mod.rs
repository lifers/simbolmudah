mod compose_reader;
mod keysym_reader;
mod mapped_string;

use std::collections::HashMap;

use compose_reader::ComposeDef;
use fst::{automaton::Str, Automaton, IntoStreamer, Map, MapBuilder, Streamer};
use keysym_reader::KeySymDef;
use mapped_string::MappedString;
use windows::{core::Error, Win32::Foundation::E_FAIL};

use super::TranslateError;

#[derive(Debug, PartialEq)]
pub(super) enum SequenceTranslatorError {
    FileRead,
    FileWrite,
    ReadLine,
    WriteLine,
    RegexBuild,
    RegexParse,
    ParseInt,
    InvalidChar,
    InvalidKeyname,
    FstBuild,
}

impl Into<Error> for SequenceTranslatorError {
    fn into(self) -> Error {
        Error::new(E_FAIL, format!("{:?} at {}", self, std::env::current_dir().unwrap().display()))
    }
}

#[derive(Default, Debug)]
pub(super) struct SequenceTranslator {
    key_to_value: Map<Vec<u8>>,
    value_to_string: HashMap<u64, MappedString>,
    string_to_value: HashMap<MappedString, u64>,
    state: String,
}

impl SequenceTranslator {
    pub(super) fn translate(&mut self, seq: &str) -> Result<String, TranslateError> {
        self.state.push_str(seq);

        if let Some(value) = self.key_to_value.get(self.state.as_bytes()) {
            self.state.clear();
            let value: MappedString = value.into();
            Ok(value.into())
        } else {
            let prefix = Str::new(seq).starts_with();
            if self
                .key_to_value
                .search(&prefix)
                .into_stream()
                .next()
                .is_some()
            {
                Err(TranslateError::Incomplete)
            } else {
                self.state.clear();
                Err(TranslateError::ValueNotFound)
            }
        }
    }

    pub(super) fn build(&mut self) -> Result<(), SequenceTranslatorError> {
        let keysymdef = KeySymDef::new()?;
        let composedef = ComposeDef::build(&keysymdef)?;
        let mut build = MapBuilder::memory();
        self.value_to_string = HashMap::new();
        self.string_to_value = HashMap::new();
        let mut extra_index = u32::MAX.into();

        for (key, value) in composedef {
            match value {
                MappedString::Basic(c) => {
                    let basic_index = c.into();
                    self.value_to_string.insert(basic_index, value.clone());
                    self.string_to_value.insert(value, basic_index);
                    build
                        .insert(key, basic_index)
                        .map_err(|_| SequenceTranslatorError::FstBuild)?;
                }
                MappedString::Extra(_) => {
                    extra_index += 1;
                    self.value_to_string.insert(extra_index, value.clone());
                    self.string_to_value.insert(value, extra_index);
                    build
                        .insert(key, extra_index)
                        .map_err(|_| SequenceTranslatorError::FstBuild)?;
                }
            }
        }

        self.key_to_value = build.into_map();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_incomplete_sequence() {
        let mut translator = SequenceTranslator::default();
        translator.build().unwrap();
        let result = translator.translate("f");
        assert!(matches!(result, Err(TranslateError::Incomplete)));
    }

    #[test]
    fn test_translate_value_not_found() {
        let mut translator = SequenceTranslator::default();
        translator.build().unwrap();
        let result = translator.translate("nonexistent");
        assert!(matches!(result, Err(TranslateError::ValueNotFound)));
    }

    #[test]
    fn test_translate_valid_sequence() {
        let mut translator = SequenceTranslator::default();
        translator.build().unwrap();
        // Assuming "fl" is a valid sequence mapped to a basic MappedString for this test
        let result = translator.translate("fl");
        assert!(result.is_ok());
        let expected = "ﬂ"; // Expected result for the sequence "fl"
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_build_success() {
        let mut translator = SequenceTranslator::default();
        let result = translator.build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_state_clear_after_translation() {
        let mut translator = SequenceTranslator::default();
        translator.build().unwrap();
        let _ = translator.translate("omg");
        assert!(translator.state.is_empty());
    }

    #[test]
    fn test_state_accumulation() {
        let mut translator = SequenceTranslator::default();
        translator.build().unwrap();

        let result = translator.translate("/");
        assert_eq!(result, Err(TranslateError::Incomplete));
        let result = translator.translate("=");
        assert_eq!(result, Ok("≠".to_string()));
        assert!(translator.state.is_empty());
    }
}
