use std::collections::HashMap;

use fst::{automaton::Str, Automaton, IntoStreamer, Map, MapBuilder, Streamer};
use windows::{
    core::{Result, Weak, PSTR},
    Win32::Globalization::{u_charName, UErrorCode, U_EXTENDED_CHAR_NAME},
};

use crate::{bindings, utils::functions::fail};

use super::{
    compose_reader::ComposeDef, keysym_reader::KeySymDef, mapped_string::MappedString,
    SequenceDefinitionError,
};

pub(super) struct SequenceDefinitionInternal {
    prefix_map: Map<Vec<u8>>,
    index_map: HashMap<String, MappedString>,
    value_to_string: HashMap<u64, MappedString>,
    string_to_value: HashMap<MappedString, u64>,
    char_to_name: HashMap<char, String>,
    pub(super) parent: Weak<bindings::SequenceDefinition>,
}

impl SequenceDefinitionInternal {
    pub(super) fn new(parent: Weak<bindings::SequenceDefinition>) -> Self {
        Self {
            prefix_map: Map::default(),
            index_map: HashMap::new(),
            value_to_string: HashMap::new(),
            string_to_value: HashMap::new(),
            char_to_name: HashMap::new(),
            parent,
        }
    }

    pub(super) fn build(&mut self, keysymdef: &str, composedef: &str) -> Result<()> {
        let keysymdef = KeySymDef::new(keysymdef, self)?;
        let composedef = ComposeDef::build(&keysymdef, composedef, self)?;
        let mut build = MapBuilder::memory();
        self.index_map.clear();
        self.value_to_string.clear();
        self.string_to_value.clear();
        let mut basic_index = 0;
        let mut extra_index = u32::MAX.into();

        for (key, value) in composedef {
            match value {
                MappedString::Basic(_) => {
                    basic_index += 1;
                    self.value_to_string.insert(basic_index, value.clone());
                    self.string_to_value.insert(value.clone(), basic_index);
                    build.insert(key.clone(), basic_index).map_err(fail)?;
                }
                MappedString::Extra(_) => {
                    extra_index += 1;
                    self.value_to_string.insert(extra_index, value.clone());
                    self.string_to_value.insert(value.clone(), extra_index);
                    build.insert(key.clone(), extra_index).map_err(fail)?;
                }
            }

            self.index_map.insert(key, value);
        }

        self.prefix_map = build.into_map();
        Ok(())
    }

    pub(super) fn translate_sequence(
        &self,
        sequence: &str,
    ) -> std::result::Result<String, SequenceDefinitionError> {
        self.prefix_map.get(sequence.as_bytes()).map_or_else(
            || {
                Err(if self.potential_prefix(sequence, 1).is_empty() {
                    SequenceDefinitionError::ValueNotFound
                } else {
                    SequenceDefinitionError::Incomplete
                })
            },
            |value| {
                Ok(self
                    .value_to_string
                    .get(&value)
                    .expect("value previously mapped")
                    .to_string())
            },
        )
    }

    pub(super) fn potential_prefix(
        &self,
        sequence: &str,
        limit: usize,
    ) -> Vec<bindings::SequenceDescription> {
        let mut stream = self
            .prefix_map
            .search(Str::new(sequence).starts_with())
            .into_stream();
        let mut result = Vec::with_capacity(limit);

        for _ in 0..limit {
            if let Some(element) = stream.next() {
                let (seq, value) = element;
                result.push(
                    self.to_sequence_description(
                        &unsafe { String::from_utf8_unchecked(seq.to_vec()) },
                        self.value_to_string
                            .get(&value)
                            .expect("value previously mapped"),
                    ),
                );
            } else {
                return result;
            }
        }

        result
    }

    pub(super) fn index_char(&mut self, value: char) -> Result<()> {
        let mut buffer = [0; 88];
        let pstr = PSTR::from_raw(buffer.as_mut_ptr());
        let mut errcode = UErrorCode::default();
        let len = unsafe { u_charName(value as i32, U_EXTENDED_CHAR_NAME, pstr, 88, &mut errcode) };

        assert!(len.is_positive());
        let name = unsafe { pstr.to_string() }.map_err(fail)?;

        assert_eq!(len as usize, name.len());
        self.char_to_name.insert(value, name.to_string());

        Ok(())
    }

    pub(super) fn filter_sequence(
        &self,
        tokens: Vec<String>,
        limit: usize,
    ) -> Vec<bindings::SequenceDescription> {
        let mut result = Vec::with_capacity(limit);

        // prioritize exact character match
        for (seq, value) in self.index_map.iter() {
            if let MappedString::Basic(c) = value {
                if tokens.iter().any(|t| c.contains(t)) {
                    result.push(self.to_sequence_description(seq, value));
                }
            }

            if result.len() == limit {
                return result;
            }
        }

        // search in descriptions
        for (seq, value) in self.index_map.iter() {
            match value {
                MappedString::Basic(c) => {
                    let ch = c.chars().next().unwrap();
                    let name = self.char_to_name.get(&ch).unwrap();
                    if tokens.iter().all(|t| name.contains(&t.to_uppercase())) {
                        result.push(self.to_sequence_description(seq, value));
                    }
                }
                MappedString::Extra(s) => {
                    if tokens.iter().any(|t| s.contains(t)) {
                        result.push(self.to_sequence_description(seq, value));
                    }
                }
            }

            if result.len() == limit {
                return result;
            }
        }

        result
    }

    fn to_sequence_description(
        &self,
        seq: &str,
        value: &MappedString,
    ) -> bindings::SequenceDescription {
        bindings::SequenceDescription {
            sequence: seq.into(),
            result: value.to_string().into(),
            description: match value {
                MappedString::Basic(c) => self
                    .char_to_name
                    .get(&c.chars().next().unwrap())
                    .unwrap()
                    .to_string()
                    .into(),
                MappedString::Extra(s) => s.to_string().into(),
            },
        }
    }
}
