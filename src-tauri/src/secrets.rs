//! Keeps the API key and HF token in the OS credential store, falling back to
//! the settings table when no store is available (e.g. Linux without a
//! Secret Service daemon).

use crate::db::{Database, Settings};

const SERVICE: &str = "com.coder3101.llama-recipe-manager";

pub trait SecretStore {
    /// `None` when the entry is missing or the store is unavailable.
    fn get(&self, key: &str) -> Option<String>;
    /// Store `value`, deleting the entry when it is empty. Returns `false` if
    /// the store is unavailable.
    fn set(&self, key: &str, value: &str) -> bool;
}

pub struct Keychain;

impl SecretStore for Keychain {
    fn get(&self, key: &str) -> Option<String> {
        keyring::Entry::new(SERVICE, key)
            .and_then(|e| e.get_password())
            .ok()
    }

    fn set(&self, key: &str, value: &str) -> bool {
        let Ok(entry) = keyring::Entry::new(SERVICE, key) else {
            return false;
        };
        if value.is_empty() {
            matches!(
                entry.delete_credential(),
                Ok(()) | Err(keyring::Error::NoEntry)
            )
        } else {
            entry.set_password(value).is_ok()
        }
    }
}

fn fields(settings: &mut Settings) -> [(&'static str, &mut String); 2] {
    [
        ("api_key", &mut settings.api_key),
        ("hf_token", &mut settings.hf_token),
    ]
}

/// Load settings with secrets resolved. Secrets still in the database (from
/// older versions or an earlier fallback) are moved into the store when
/// possible.
pub fn load(db: &Database, store: &impl SecretStore) -> Result<Settings, String> {
    let mut settings = db.get_settings()?;
    let mut migrated = false;
    for (key, value) in fields(&mut settings) {
        if value.is_empty() {
            *value = store.get(key).unwrap_or_default();
        } else if store.set(key, value) {
            migrated = true;
        }
    }
    if migrated {
        save(db, store, &settings)?;
    }
    Ok(settings)
}

pub fn save(db: &Database, store: &impl SecretStore, settings: &Settings) -> Result<(), String> {
    let mut stored = settings.clone();
    for (key, value) in fields(&mut stored) {
        if store.set(key, value) {
            value.clear();
        }
    }
    db.update_settings(&stored)
}

#[cfg(test)]
mod tests {
    use super::{load, save, SecretStore};
    use crate::db::{Database, Settings};
    use std::cell::RefCell;
    use std::collections::HashMap;

    struct MemoryStore(RefCell<HashMap<String, String>>);

    impl SecretStore for MemoryStore {
        fn get(&self, key: &str) -> Option<String> {
            self.0.borrow().get(key).cloned()
        }
        fn set(&self, key: &str, value: &str) -> bool {
            let mut map = self.0.borrow_mut();
            if value.is_empty() {
                map.remove(key);
            } else {
                map.insert(key.to_string(), value.to_string());
            }
            true
        }
    }

    struct Unavailable;

    impl SecretStore for Unavailable {
        fn get(&self, _: &str) -> Option<String> {
            None
        }
        fn set(&self, _: &str, _: &str) -> bool {
            false
        }
    }

    fn memory() -> MemoryStore {
        MemoryStore(RefCell::new(HashMap::new()))
    }

    fn db() -> (tempfile::TempDir, Database) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = Database::new(dir.path().to_path_buf()).expect("db");
        (dir, db)
    }

    fn with_secrets() -> Settings {
        Settings {
            api_key: "key".into(),
            hf_token: "hf_tok".into(),
            ..Settings::default()
        }
    }

    #[test]
    fn secrets_live_only_in_the_store() {
        let (_dir, db) = db();
        let store = memory();
        save(&db, &store, &with_secrets()).unwrap();

        let raw = db.get_settings().unwrap();
        assert_eq!((raw.api_key.as_str(), raw.hf_token.as_str()), ("", ""));
        let loaded = load(&db, &store).unwrap();
        assert_eq!(loaded.api_key, "key");
        assert_eq!(loaded.hf_token, "hf_tok");
    }

    #[test]
    fn falls_back_to_the_database_without_a_store() {
        let (_dir, db) = db();
        save(&db, &Unavailable, &with_secrets()).unwrap();

        assert_eq!(db.get_settings().unwrap().api_key, "key");
        assert_eq!(load(&db, &Unavailable).unwrap().hf_token, "hf_tok");
    }

    #[test]
    fn database_secrets_migrate_into_the_store() {
        let (_dir, db) = db();
        db.update_settings(&with_secrets()).unwrap();
        let store = memory();

        assert_eq!(load(&db, &store).unwrap().api_key, "key");
        assert_eq!(db.get_settings().unwrap().api_key, "");
        assert_eq!(store.get("hf_token").as_deref(), Some("hf_tok"));
    }

    #[test]
    fn clearing_a_secret_removes_it_from_the_store() {
        let (_dir, db) = db();
        let store = memory();
        save(&db, &store, &with_secrets()).unwrap();
        save(&db, &store, &Settings::default()).unwrap();

        assert!(store.get("api_key").is_none());
        assert_eq!(load(&db, &store).unwrap().api_key, "");
    }
}
