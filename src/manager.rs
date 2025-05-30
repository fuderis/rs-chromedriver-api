use crate::prelude::*;

/// The tasks manager
#[derive(Debug, Clone)]
pub struct TaskManager {
    state: Arc<Mutex<bool>>, // true — занят, false — свободен
    notify: Arc<Notify>,
}

impl TaskManager {
    /// Creates a new task manager
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(false)),
            notify: Arc::new(Notify::new()),
        }
    }

    /// Locking tasks execxuting
    pub async fn lock(&self) {
        loop {
            {
                let mut locked = self.state.lock().await;
                if !*locked {
                    *locked = true;
                    return;
                }
            }

            self.notify.notified().await;
        }
    }

    /// Unlocking tasks execxuting
    pub async fn unlock(&self) {
        {
            let mut locked = self.state.lock().await;
            *locked = false;
        }

        self.notify.notify_waiters();
    }
}
