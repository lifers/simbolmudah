use std::ffi::OsString;

use crate::{
    composer::{ComposeError, Composer},
    key::Key,
    key_sequence::KeySequence,
};

pub(super) struct SequenceState {
    state: KeySequence,
}

impl SequenceState {
    pub(super) fn new() -> Self {
        Self {
            state: KeySequence::new(),
        }
    }

    pub(super) fn submit(&mut self, composer: &Composer) -> Result<OsString, ComposeError> {
        println!("{:?}", &self.state);
        let res = composer.search(&self.state.clone().try_into().unwrap());
        if res == Err(ComposeError::NotFound) || res.is_ok() {
            self.state.clear();
        }
        dbg!(res)
    }

    pub(super) fn push(&mut self, key: Key) {
        self.state.push(key);
    }
}
