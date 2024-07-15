use windows::core::Weak;

use crate::bindings;

pub(super) struct SequenceSearcherInternal {
    pub(super) sequence_definition: Weak<bindings::SequenceDefinition>,
    pub(super) parent: Weak<bindings::SequenceSearcher>,
}

impl SequenceSearcherInternal {
    pub(super) fn new(
        sequence_definition: Weak<bindings::SequenceDefinition>,
        parent: Weak<bindings::SequenceSearcher>,
    ) -> Self {
        Self {
            sequence_definition,
            parent,
        }
    }
}
