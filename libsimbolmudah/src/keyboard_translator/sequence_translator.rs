use std::collections::HashMap;
use super::TranslateError;

pub(super) struct BuilderError;

pub(super) struct SequenceTranslator {
    keysymdef: HashMap<String, String>,
}

impl SequenceTranslator {
    pub(super) fn new() -> Self {
        Self {
            keysymdef: HashMap::new(),
        }
    }

    pub(super) fn translate(&self, value: &String) -> Result<String, TranslateError> {
        if let Some(result) = self.keysymdef.get(value) {
            Ok(result.into())
        } else {
            Err(TranslateError::ValueNotFound)
        }
    }

    pub(super) fn build_dictionary(&mut self) -> Result<(), BuilderError> {
        self.keysymdef.clear();
        self.keysymdef.insert(">=".into(), "≥".into());
        self.keysymdef.insert("fm".into(), "👨🏿‍👩🏻‍👧🏿‍👦🏼".into());
        Ok(())
    }
}
