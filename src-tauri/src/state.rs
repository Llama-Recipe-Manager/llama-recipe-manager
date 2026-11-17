//! Tauri-managed application state.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use crate::db::Database;
use crate::process::ProcessManager;

pub struct AppState {
    pub db: Mutex<Database>,
    pub pm: ProcessManager,
    /// In-flight model downloads keyed by destination path, with their cancel flag.
    pub downloads: Mutex<HashMap<PathBuf, Arc<AtomicBool>>>,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        Self {
            db: Mutex::new(db),
            pm: ProcessManager::new(),
            downloads: Mutex::new(HashMap::new()),
        }
    }
}
