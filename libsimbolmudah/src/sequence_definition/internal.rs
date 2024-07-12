use std::collections::HashMap;

use fst::{automaton::Str, Automaton, IntoStreamer, Map, MapBuilder, Streamer};
use windows::core::{Result, Weak};

use crate::{bindings, fail};

use super::{compose_reader::ComposeDef, mapped_string::MappedString, SequenceDefinitionError};

pub(super) struct SequenceDefinitionInternal {
    key_to_value: Map<Vec<u8>>,
    value_to_string: HashMap<u64, MappedString>,
    string_to_value: HashMap<MappedString, u64>,
    pub(super) parent: Weak<bindings::SequenceDefinition>,
}

impl SequenceDefinitionInternal {
    pub(super) fn new(parent: Weak<bindings::SequenceDefinition>) -> Self {
        Self {
            key_to_value: Map::default(),
            value_to_string: HashMap::new(),
            string_to_value: HashMap::new(),
            parent,
        }
    }

    pub(super) fn build(&mut self, composedef: ComposeDef) -> Result<()> {
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
                    build.insert(key, basic_index).map_err(fail)?;
                }
                MappedString::Extra(_) => {
                    extra_index += 1;
                    self.value_to_string.insert(extra_index, value.clone());
                    self.string_to_value.insert(value, extra_index);
                    build.insert(key, extra_index).map_err(fail)?;
                }
            }
        }

        self.key_to_value = build.into_map();
        Ok(())
    }

    pub(super) fn translate_sequence(
        &self,
        sequence: &str,
    ) -> std::result::Result<String, SequenceDefinitionError> {
        self.key_to_value.get(sequence.as_bytes()).map_or_else(
            || {
                Err(self
                    .key_to_value
                    .search(Str::new(sequence).starts_with())
                    .into_stream()
                    .next()
                    .map_or(SequenceDefinitionError::ValueNotFound, |_| {
                        SequenceDefinitionError::Incomplete
                    }))
            },
            |value| {
                let value: MappedString = value.into();
                Ok(value.into())
            },
        )
    }
}
