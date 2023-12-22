use super::key::Key;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(super) struct KeySequence(Vec<Key>);

impl KeySequence {
    pub(super) fn new() -> Self {
        Self(Vec::new())
    }

    pub(super) fn push(&mut self, key: Key) {
        self.0.push(key);
    }

    pub(super) fn clear(&mut self) {
        self.0.clear();
    }

    pub(super) fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<Vec<Key>> for KeySequence {
    fn from(vec: Vec<Key>) -> Self {
        Self(vec)
    }
}

impl TryInto<String> for &KeySequence {
    type Error = &'static str;

    fn try_into(self) -> Result<String, Self::Error> {
        let mut s = String::new();
        for k in &self.0 {
            match k {
                Key::Char(c) => s.push(*c),
                _ => return Err("Cannot convert non-char key to string"),
            }
        }
        Ok(s)
    }
}

impl TryInto<String> for KeySequence {
    type Error = &'static str;

    fn try_into(self) -> Result<String, Self::Error> {
        (&self).try_into()
    }
}
