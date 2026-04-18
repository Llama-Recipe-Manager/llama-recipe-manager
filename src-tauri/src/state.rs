//! Tauri-managed application state.

use std::sync::Mutex;

use crate::db::Database;
use crate::process::ProcessManager;

pub struct AppState {
    pub db: Mutex<Database>,
    pub pm: ProcessManager,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        Self {
            db: Mutex::new(db),
            pm: ProcessManager::new(),
        }
    }
}
