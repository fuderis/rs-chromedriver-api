// use crate::prelude::*;
use atomic_state::AtomFlag;

/// The session task manager
#[derive(Clone)]
pub struct SessionManager {
    flag: AtomFlag,
}

impl SessionManager {
    /// Creates a new session task manager
    pub fn new() -> Self {
        Self {
            flag: AtomFlag::new(false),
        }
    }

    /// Locking tasks execution
    pub async fn lock(&self) {
        self.flag.swap(true).await;
    }

    /// Unlocking tasks execution
    pub async fn unlock(&self) {
        if self.flag.get() == false { return; }
        self.flag.swap(false).await;
    }
}
