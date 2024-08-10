use std::{
    cell::RefCell,
    sync::{mpsc::channel, RwLock},
    thread::LocalKey,
};

use windows::{
    core::{Error, Result},
    System::{DispatcherQueueController, DispatcherQueueHandler},
    Win32::Foundation::E_FAIL,
};

pub(crate) struct SingleThreaded<T: 'static> {
    thread: RwLock<Option<DispatcherQueueController>>,
    data: LocalKey<RefCell<Option<T>>>,
}

macro_rules! single_threaded {
    ($t:ty) => {
        SingleThreaded::new(
            {
                thread_local!(static __INIT: std::cell::RefCell<Option<$t>> = const { std::cell::RefCell::new(None) });
                __INIT
            }
        )
    };
}
pub(crate) use single_threaded;

use super::functions::{fail, fail_message};

impl<T: 'static> SingleThreaded<T> {
    fn enqueue<F>(&'static self, callback: F) -> Result<()>
    where
        F: FnMut() -> Result<()> + Send + 'static,
    {
        if self
            .thread
            .read()
            .map_err(fail)?
            .as_ref()
            .expect("Should be initialized")
            .DispatcherQueue()?
            .TryEnqueue(&DispatcherQueueHandler::new(callback))?
        {
            Ok(())
        } else {
            Err(Error::new(E_FAIL, "Failed to enqueue"))
        }
    }

    pub(crate) const fn new(data: LocalKey<RefCell<Option<T>>>) -> Self {
        Self {
            thread: RwLock::new(None),
            data,
        }
    }

    pub(crate) fn initialize<F>(&'static self, init: F) -> Result<()>
    where
        F: FnOnce() -> Result<T> + Send + 'static,
    {
        let new_thread = DispatcherQueueController::CreateOnDedicatedThread()?;
        new_thread
            .DispatcherQueue()?
            .TryEnqueue(&DispatcherQueueHandler::new(move || {
                if let Some(t) = self
                    .thread
                    .write()
                    .map_err(fail)?
                    .replace(new_thread.clone())
                {
                    Err(fail_message(&format!("Already initialized: {:?}", t)))
                } else {
                    Ok(())
                }
            }))?;

        let (tx, rx) = channel();
        tx.send(init).map_err(fail)?;

        self.enqueue(move || {
            self.data.with_borrow_mut(|data: &mut Option<T>| {
                let init = rx.try_recv().map_err(fail)?;
                *data = Some(init()?);
                Ok(())
            })
        })
    }

    pub(crate) fn with_borrow<F>(&'static self, f: F) -> Result<()>
    where
        F: FnOnce(&T) -> Result<()> + Send + 'static,
    {
        let (tx, rx) = channel();
        tx.send(f).map_err(fail)?;

        self.enqueue(move || {
            self.data.with_borrow(|data: &Option<T>| {
                let data = data.as_ref().ok_or_else(|| {
                    fail_message("Uninitialized. Did you call initialize before?")
                })?;

                rx.try_recv().map_err(fail)?(data)
            })
        })
    }

    pub(crate) fn with_borrow_mut<F>(&'static self, f: F) -> Result<()>
    where
        F: FnOnce(&mut T) -> Result<()> + Send + 'static,
    {
        let (tx, rx) = channel();
        tx.send(f).map_err(fail)?;

        self.enqueue(move || {
            self.data.with_borrow_mut(|data: &mut Option<T>| {
                let data = data.as_mut().ok_or_else(|| {
                    fail_message("Uninitialized. Did you call initialize before?")
                })?;

                rx.try_recv().map_err(fail)?(data)
            })
        })
    }

    /// Borrow the internal data mutably.
    /// # Safety
    /// Caller must ensure that they call this function only from the thread that initialized the data.
    pub(crate) unsafe fn in_thread_borrow_mut<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        self.data
            .with_borrow_mut(|data: &mut Option<T>| f(data.as_mut().expect("must be initialized")))
    }

    pub(crate) fn destroy(&'static self) -> Result<()> {
        if let Some(controller) = self.thread.write().map_err(fail)?.take() {
            controller
                .DispatcherQueue()?
                .TryEnqueue(&DispatcherQueueHandler::new(|| {
                    let _ = self.data.take();
                    Ok(())
                }))?;
            let _ = controller.ShutdownQueueAsync()?;
        }
        Ok(())
    }
}
