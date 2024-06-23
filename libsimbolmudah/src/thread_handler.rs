use windows::{
    core::Result,
    System::{DispatcherQueueController, DispatcherQueueHandler},
};

pub(crate) struct ThreadHandler {
    thread: DispatcherQueueController,
}

impl ThreadHandler {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self {
            thread: DispatcherQueueController::CreateOnDedicatedThread()?,
        })
    }

    pub(crate) fn try_enqueue<F>(&self, callback: F) -> Result<bool>
    where
        F: FnMut() -> Result<()> + Send + 'static,
    {
        self.thread
            .DispatcherQueue()?
            .TryEnqueue(&DispatcherQueueHandler::new(callback))
    }
}

impl Drop for ThreadHandler {
    fn drop(&mut self) {
        let _ = self.thread.ShutdownQueueAsync();
    }
}
