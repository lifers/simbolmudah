use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

pub(super) struct Counter(AtomicU32);

impl Counter {
    pub(super) const fn new() -> Self {
        Self(AtomicU32::new(1))
    }

    pub(super) fn next(&self) -> u32 {
        self.0.fetch_add(1, Relaxed)
    }
}

pub(super) static GLOBAL_COUNTER: Counter = Counter::new();
