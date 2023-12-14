use fst::{automaton::Str, Automaton, IntoStreamer, Map, MapBuilder, Streamer};

use crate::sequence::{key::Key, key_sequence::KeySequence};

#[derive(PartialEq, Debug)]
pub enum ComposeError {
    Incomplete,
    NotFound,
}

pub struct Composer {
    key_index: Map<Vec<u8>>,
}

impl Composer {
    pub fn new() -> Self {
        let mut build = MapBuilder::memory();
        build.insert(">=", '≥'.into()).unwrap();
        build.insert("oe", 'œ'.into()).unwrap();
        build.insert("wkwk", '🤣'.into()).unwrap();

        Self {
            key_index: build.into_map(),
        }
    }

    pub fn search(&self, seq: &KeySequence) -> Result<char, ComposeError> {
        let mut key = String::new();
        for k in seq {
            match k {
                Key::Char(c) => key.push(*c),
                _ => return Err(ComposeError::NotFound),
            }
        }

        if let Some(value) = self.key_index.get(&key) {
            return Ok(char::from_u32(value as u32).unwrap());
        }

        let prefix = Str::new(&key).starts_with();
        if self
            .key_index
            .search(&prefix)
            .into_stream()
            .next()
            .is_some()
        {
            return Err(ComposeError::Incomplete);
        }

        Err(ComposeError::NotFound)
    }
}
