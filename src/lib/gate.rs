use async_std::sync::{Mutex, Condvar};
struct Gate {
    mutex: Mutex<bool>,
    condvar: Condvar,
}

unsafe impl Send for Gate {}
unsafe impl Sync for Gate {}

impl Default for Gate {
    fn default() -> Self {
        Gate::new()
    }
}

impl Gate {
    pub fn new() -> Self {
        Self {
            mutex: Mutex::new(false),
            condvar: Condvar::new(),
        }
    }

    pub async fn open(&self) {
        let mut guard = self.mutex.lock().await;
        *guard = true;
        self.condvar.notify_all();
    }

    pub async fn close(&self) {
        let mut guard = self.mutex.lock().await;
        *guard = false;
    }

    pub async fn wait(&self) {
        let guard = self.mutex.lock().await;
        if !*guard {
            self.condvar.wait(guard).await;
        }
    }
}