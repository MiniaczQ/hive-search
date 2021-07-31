//! Custom synchronization primitives.

use async_std::sync::{Mutex, Condvar};

/// Token for pausing and stopping asynchronous code.
pub struct PauseToken {
    mutex: Mutex<bool>,
    condvar: Condvar,
}

unsafe impl Send for PauseToken {}
unsafe impl Sync for PauseToken {}

impl PauseToken {
    pub fn new(paused: bool) -> Self {
        Self {
            mutex: Mutex::new(paused),
            condvar: Condvar::new(),
        }
    }

    /// Pausing will block wait calls.
    pub async fn pause(&self) {
        let mut guard = self.mutex.lock().await;
        *guard = true;
    }

    /// Resuming will unblock wait calls.
    pub async fn resume(&self) {
        let mut guard = self.mutex.lock().await;
        *guard = false;
        self.condvar.notify_all();
    }

    /// Blocks until unpaused.
    pub async fn wait(&self) {
        let guard = self.mutex.lock().await;
        if *guard {
            self.condvar.wait(guard).await;
        }
    }

    /// Returns if unpaused.
    pub async fn is_paused(&self) -> bool {
        let guard = self.mutex.lock().await;
        *guard
    }
}