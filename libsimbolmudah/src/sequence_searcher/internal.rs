use windows::core::Weak;

use crate::bindings;

pub(super) struct SequenceSearcherInternal {
    pub(super) parent: Weak<bindings::SequenceSearcher>,
}

impl SequenceSearcherInternal {
    pub(super) fn new(parent: Weak<bindings::SequenceSearcher>) -> Self {
        Self { parent }
    }
}
