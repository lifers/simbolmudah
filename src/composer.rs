mod compose_reader;
mod keysym_reader;
mod mapped_string;

use std::ffi::OsString;

use fst::{automaton::Str, Automaton, IntoStreamer, Map, MapBuilder, Streamer};

use self::{compose_reader::ComposeDef, keysym_reader::KeySymDef, mapped_string::MappedString};

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
        let keysymdef = KeySymDef::new();
        let composedef = ComposeDef::build(&keysymdef);
        let mut build = MapBuilder::memory();

        for (key, value) in composedef {
            build.insert(key, value.into()).unwrap();
        }

        Self {
            key_index: build.into_map(),
        }
    }

    pub fn search(&self, seq: &String) -> Result<OsString, ComposeError> {
        if let Some(value) = self.key_index.get(seq) {
            let value: MappedString = value.into();
            return Ok(value.into());
        }

        let prefix = Str::new(seq).starts_with();
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
