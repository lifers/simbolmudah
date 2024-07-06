use windows::{
    core::{Result, Weak},
    Foundation::Collections::IVectorView,
};

use crate::bindings;

pub(super) struct SequenceSearcherInternal {
    pub(super) parent: Weak<bindings::SequenceSearcher>,
}

impl SequenceSearcherInternal {
    pub(super) fn new(parent: Weak<bindings::SequenceSearcher>) -> Self {
        Self { parent }
    }

    pub(super) fn search(
        &self,
        keyword: &str,
    ) -> Result<IVectorView<bindings::SequenceDescription>> {
        unimplemented!()
    }
}
