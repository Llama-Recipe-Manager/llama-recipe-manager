//! Forward-only schema migration runner using SQLite's `PRAGMA user_version`.
//!
//! # Adding a new migration
//!
//! 1. Create `src-tauri/src/migrations/<NNNN>_<short_description>.sql` using
//!    the next sequential 4-digit number.
//! 2. Write forward-only SQL inside it (`ALTER TABLE`, `CREATE INDEX`, etc.).
//! 3. Append a `Migration` entry below with a matching `version` number.
//! 4. Never edit or reorder existing migrations once they have shipped.
//! 5. Bump the package version in `src-tauri/Cargo.toml` when the migration
//!    is released.

use rusqlite::Connection;

pub struct Migration {
    pub version: i32,
    pub description: &'static str,
    pub sql: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        description: "initial schema",
        sql: include_str!("../migrations/0001_initial.sql"),
    },
    Migration {
        version: 2,
        description: "enable metrics by default",
        sql: include_str!("../migrations/0002_enable_metrics_by_default.sql"),
    },
];

/// Apply all pending migrations in order, each inside its own transaction.
pub fn run(conn: &mut Connection) -> Result<(), String> {
    let current: i32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;

    for m in MIGRATIONS.iter().filter(|m| m.version > current) {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        tx.execute_batch(m.sql)
            .map_err(|e| format!("migration {} ({}): {}", m.version, m.description, e))?;
        tx.pragma_update(None, "user_version", m.version)
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
    }

    Ok(())
}
