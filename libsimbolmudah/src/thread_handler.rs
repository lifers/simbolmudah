use windows::{
    core::{Error, InterfaceType, Param, Result},
    Foundation::{EventRegistrationToken, IAsyncAction, TypedEventHandler},
    System::{DispatcherQueue, DispatcherQueueController, DispatcherQueueHandler},
    Win32::Foundation::E_FAIL,
};
use windows_core::IInspectable;

#[derive(Debug)]
pub(crate) struct ThreadHandler {
    thread: DispatcherQueueController,
}

impl ThreadHandler {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self {
            thread: DispatcherQueueController::CreateOnDedicatedThread()?,
        })
    }

    pub(crate) fn try_enqueue<F>(&self, callback: F) -> Result<()>
    where
        F: FnMut() -> Result<()> + Send + 'static,
    {
        if self
            .thread
            .DispatcherQueue()?
            .TryEnqueue(&DispatcherQueueHandler::new(callback))?
        {
            Ok(())
        } else {
            Err(Error::new(E_FAIL, "Failed to enqueue"))
        }
    }

    pub(crate) fn register_shutdown_complete_callback<F>(
        &self,
        callback: F,
    ) -> Result<EventRegistrationToken>
    where
        F: Param<TypedEventHandler<DispatcherQueue, IInspectable>, InterfaceType>,
    {
        self.thread.DispatcherQueue()?.ShutdownCompleted(callback)
    }

    pub(crate) fn unregister_shutdown_complete_callback(
        &self,
        token: EventRegistrationToken,
    ) -> Result<()> {
        self.thread
            .DispatcherQueue()?
            .RemoveShutdownCompleted(token)
    }

    pub(crate) fn disable(&self) -> Result<IAsyncAction> {
        self.thread.ShutdownQueueAsync()
    }
}

impl Drop for ThreadHandler {
    fn drop(&mut self) {
        let _ = self.thread.ShutdownQueueAsync();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{
            AtomicU64,
            Ordering::{AcqRel, Acquire},
        },
        Arc,
    };

    use windows::Win32::{
        System::Diagnostics::Debug::MessageBeep, UI::WindowsAndMessaging::MB_ICONASTERISK,
    };

    use super::*;

    #[test]
    fn test_new_thread_handler() {
        let thread_handler = ThreadHandler::new().unwrap();
        // Assert that the thread handler is created successfully
        assert!(thread_handler.thread.DispatcherQueue().is_ok());
    }

    #[test]
    fn test_try_enqueue_success() {
        let thread_handler = ThreadHandler::new().unwrap();

        let num = Arc::new(AtomicU64::new(5));
        let clone = num.clone();

        // Check flag on ShutdownCompleted event
        let complete_handler =
            TypedEventHandler::new(move |sender: &Option<DispatcherQueue>, _| {
                assert!(sender.is_some());
                assert_eq!(num.load(Acquire), 41242);
                Ok(())
            });

        // Register the event handler
        let complete_token = thread_handler
            .register_shutdown_complete_callback(Some(&complete_handler))
            .expect("Event handler should be registered");

        let result = thread_handler.try_enqueue(move || {
            // Perform some operation here
            assert_eq!(clone.swap(41242, AcqRel), 5);
            unsafe {
                MessageBeep(MB_ICONASTERISK).unwrap();
            };
            Ok(())
        });

        // Assert that the enqueue operation is successful
        assert!(result.is_ok());
        thread_handler.disable().unwrap().get().unwrap();

        // Unregister the event handler
        thread_handler
            .unregister_shutdown_complete_callback(complete_token)
            .expect("Event handler should be unregistered");
    }

    #[test]
    fn test_drop_thread_handler() {
        {
            let _thread_handler = ThreadHandler::new().unwrap();
            // Assert that the thread handler is dropped successfully
        }
        // Add additional assertions if necessary
    }
}
