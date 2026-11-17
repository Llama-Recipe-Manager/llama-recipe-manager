//! Integration tests for the database layer.
//!
//! Each test uses a fresh tempdir so they can run in parallel without
//! interfering with each other or with the user's real data dir.

use llama_recipe_manager_lib::db::{CreateRecipe, Database, UpdateRecipe};

fn fresh_db() -> (Database, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = Database::new(dir.path().to_path_buf()).expect("open db");
    (db, dir)
}

fn sample_recipe() -> CreateRecipe {
    CreateRecipe {
        name: "Llama 3 8B".to_string(),
        description: "Local chat".to_string(),
        command: "--ctx-size 4096".to_string(),
        model_path: "llama-3-8b.gguf".to_string(),
        mmproj_path: String::new(),
        gpu_info: "RTX 4090".to_string(),
        tags: "chat,local".to_string(),
    }
}

#[test]
fn migrations_create_a_usable_db() {
    let (db, _dir) = fresh_db();
    assert!(db.list_recipes().expect("list").is_empty());
    let _ = db.get_settings().expect("settings have defaults");
}

#[test]
fn opening_twice_is_idempotent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let _ = Database::new(dir.path().to_path_buf()).expect("open 1");
    let _ = Database::new(dir.path().to_path_buf()).expect("open 2");
}

#[test]
fn create_get_update_delete_roundtrip() {
    let (db, _dir) = fresh_db();

    let created = db.create_recipe(sample_recipe()).expect("create");
    assert_eq!(created.name, "Llama 3 8B");
    assert_ne!(created.id, "");

    let fetched = db.get_recipe(&created.id).expect("get");
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.command, "--ctx-size 4096");

    let updated = db
        .update_recipe(UpdateRecipe {
            id: created.id.clone(),
            name: "Llama 3 8B (tuned)".to_string(),
            description: created.description.clone(),
            command: "--ctx-size 8192".to_string(),
            model_path: created.model_path.clone(),
            mmproj_path: String::new(),
            gpu_info: created.gpu_info.clone(),
            tags: created.tags.clone(),
        })
        .expect("update");
    assert_eq!(updated.name, "Llama 3 8B (tuned)");
    assert_eq!(updated.command, "--ctx-size 8192");

    db.delete_recipe(&created.id).expect("delete");
    assert!(
        db.get_recipe(&created.id).is_err(),
        "deleted recipe should be gone"
    );
}

#[test]
fn duplicate_creates_new_id() {
    let (db, _dir) = fresh_db();
    let original = db.create_recipe(sample_recipe()).expect("create");
    let dup = db.duplicate_recipe(&original.id).expect("duplicate");
    assert_ne!(dup.id, original.id);
    assert!(dup.name.contains(&original.name) || dup.name.contains("Copy"));
    assert_eq!(db.list_recipes().unwrap().len(), 2);
}

#[test]
fn create_rejects_forbidden_flag_in_command() {
    let (db, _dir) = fresh_db();
    let mut bad = sample_recipe();
    bad.command = "--host 0.0.0.0".to_string();
    let err = db.create_recipe(bad).unwrap_err();
    assert!(
        err.contains("--host"),
        "expected --host rejection, got {}",
        err
    );
}

#[test]
fn create_rejects_oversized_name() {
    let (db, _dir) = fresh_db();
    let mut bad = sample_recipe();
    bad.name = "x".repeat(10_000);
    assert!(db.create_recipe(bad).is_err());
}

#[test]
fn create_requires_model_path() {
    let (db, _dir) = fresh_db();
    let mut bad = sample_recipe();
    bad.model_path = String::new();
    let err = db.create_recipe(bad).unwrap_err();
    assert!(err.to_lowercase().contains("model path"));
}

#[test]
fn import_renames_on_name_clash() {
    let (db, _dir) = fresh_db();
    db.create_recipe(sample_recipe()).expect("create");
    let first = db.import_recipe(sample_recipe()).expect("import 1");
    let second = db.import_recipe(sample_recipe()).expect("import 2");
    assert_eq!(first.name, "Llama 3 8B (imported)");
    assert_eq!(second.name, "Llama 3 8B (imported 2)");
}

#[test]
fn import_rejects_unsafe_flags_that_local_recipes_allow() {
    let (db, _dir) = fresh_db();
    let mut risky = sample_recipe();
    risky.command = "--ctx-size 4096 --tools all".to_string();
    assert!(db.import_recipe(risky).unwrap_err().contains("--tools"));
    assert!(db.list_recipes().expect("list").is_empty());
}

#[test]
fn settings_roundtrip() {
    let (db, _dir) = fresh_db();
    let mut s = db.get_settings().expect("defaults");
    s.host = "127.0.0.1".to_string();
    s.port = 9090;
    s.api_key = "secret-key".to_string();
    s.webui_enabled = false;
    s.keep_server_on_exit = true;
    db.update_settings(&s).expect("update");
    let read = db.get_settings().expect("read");
    assert_eq!(read.host, "127.0.0.1");
    assert_eq!(read.port, 9090);
    assert_eq!(read.api_key, "secret-key");
    assert!(!read.webui_enabled);
    assert!(read.keep_server_on_exit);
}

#[test]
fn pinning_and_use_do_not_count_as_edits() {
    let (db, _dir) = fresh_db();
    let created = db.create_recipe(sample_recipe()).expect("create");
    assert!(!created.pinned);
    assert_eq!(created.last_used_at, "");

    let pinned = db.set_recipe_pinned(&created.id, true).expect("pin");
    assert!(pinned.pinned);
    db.mark_recipe_used(&created.id).expect("mark used");

    let after = db.get_recipe(&created.id).expect("get");
    assert!(after.pinned);
    assert_ne!(after.last_used_at, "");
    assert_eq!(after.updated_at, created.updated_at);

    let copy = db.duplicate_recipe(&created.id).expect("duplicate");
    assert!(!copy.pinned);
    assert!(db.set_recipe_pinned("missing", true).is_err());
}
