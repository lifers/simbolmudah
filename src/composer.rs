use crate::sequence::{key::Key, key_sequence::KeySequence};

#[derive(PartialEq, Debug)]
pub enum ComposeError {
    Incomplete,
    NotFound,
}

pub fn search(seq: &KeySequence) -> Result<char, ComposeError> {
    let ans = vec![Key::Char('o'), Key::Char('e')];
    let pref = vec![Key::Char('o')];
    let ans2 = vec![Key::Char('>'), Key::Char('=')];
    let pref2 = vec![Key::Char('>')];

    if seq == &ans {
        Ok('œ')
    } else if seq == &ans2 {
        Ok('≥')
    } else if seq == &pref {
        Err(ComposeError::Incomplete)
    } else if seq == &pref2 {
        Err(ComposeError::Incomplete)
    } else {
        Err(ComposeError::NotFound)
    }
}
