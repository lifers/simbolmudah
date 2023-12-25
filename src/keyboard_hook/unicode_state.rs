use std::ffi::OsString;

use crate::{composer::ComposeError, key::Key};

pub(super) struct UnicodeState {
    state: String,
}

impl UnicodeState {
    pub(super) fn new() -> Self {
        Self {
            state: "U".to_string(),
        }
    }

    pub(super) fn submit(&mut self) -> Result<OsString, ComposeError> {
        println!("{:?}", &self.state);
        let res = Key::from_unicode_string(&self.state);
        self.state.truncate(1);

        if let Some(Key::Char(c)) = res {
            Ok(c.to_string().into())
        } else {
            Err(ComposeError::NotFound)
        }
    }

    pub(super) fn push(&mut self, ch: char) {
        self.state.push(ch);
    }
}
