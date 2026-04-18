use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Flags that must not appear in a recipe command because they are
// controlled by app settings or are local-machine-specific.
const FORBIDDEN_FLAGS: &[&str] = &[
    "--port",
    "-p",
    "--host",
    "-m",
    "--model",
    "--model-path",
    "--mmproj",
    "--log-file",
];

/// Validate that a recipe command does not contain any forbidden flags.
pub fn validate_recipe_command(command: &str) -> Result<(), String> {
    let tokens =
        shell_words::split(command).map_err(|e| format!("Invalid command syntax: {}", e))?;

    for token in &tokens {
        let lower = token.to_lowercase();
        for flag in FORBIDDEN_FLAGS {
            if lower == *flag {
                return Err(format!(
                    "Recipe command must not contain '{}'. This is controlled by app settings.",
                    flag
                ));
            }
            if lower.starts_with(&format!("{}=", flag)) {
                return Err(format!(
                    "Recipe command must not contain '{}'. This is controlled by app settings.",
                    flag
                ));
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub description: String,
    pub command: String,
    pub model_path: String,
    pub mmproj_path: String,
    pub gpu_info: String,
    pub tags: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateRecipe {
    pub name: String,
    pub description: String,
    pub command: String,
    pub model_path: String,
    pub mmproj_path: String,
    pub gpu_info: String,
    pub tags: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRecipe {
    pub id: String,
    pub name: String,
    pub description: String,
    pub command: String,
    pub model_path: String,
    pub mmproj_path: String,
    pub gpu_info: String,
    pub tags: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub host: String,
    pub port: u16,
    pub model_dir: String,
    pub llama_server_path: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            model_dir: "~/Models".to_string(),
            llama_server_path: "llama-server".to_string(),
        }
    }
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(app_dir: PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
        let db_path = app_dir.join("recipes.db");
        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS recipes (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                command TEXT NOT NULL,
                model_path TEXT NOT NULL DEFAULT '',
                mmproj_path TEXT NOT NULL DEFAULT '',
                gpu_info TEXT NOT NULL DEFAULT '',
                tags TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );",
        )
        .map_err(|e| e.to_string())?;

        // Migration: add mmproj_path column if missing
        let has_mmproj = conn
            .prepare("SELECT mmproj_path FROM recipes LIMIT 0")
            .is_ok();
        if !has_mmproj {
            conn.execute_batch(
                "ALTER TABLE recipes ADD COLUMN mmproj_path TEXT NOT NULL DEFAULT '';",
            )
            .map_err(|e| e.to_string())?;
        }

        Ok(Self { conn })
    }

    // ── Settings ──

    pub fn get_settings(&self) -> Result<Settings, String> {
        let mut settings = Settings::default();

        let mut stmt = self
            .conn
            .prepare("SELECT key, value FROM settings")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| e.to_string())?;

        for row in rows {
            let (key, value) = row.map_err(|e| e.to_string())?;
            match key.as_str() {
                "host" => settings.host = value,
                "port" => {
                    if let Ok(p) = value.parse::<u16>() {
                        settings.port = p;
                    }
                }
                "model_dir" => settings.model_dir = value,
                "llama_server_path" => settings.llama_server_path = value,
                _ => {}
            }
        }

        Ok(settings)
    }

    pub fn update_settings(&self, settings: &Settings) -> Result<(), String> {
        let pairs: Vec<(&str, String)> = vec![
            ("host", settings.host.clone()),
            ("port", settings.port.to_string()),
            ("model_dir", settings.model_dir.clone()),
            ("llama_server_path", settings.llama_server_path.clone()),
        ];

        for (key, value) in pairs {
            self.conn
                .execute(
                    "INSERT INTO settings (key, value) VALUES (?1, ?2)
                     ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                    params![key, value],
                )
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    // ── Recipes ──

    pub fn list_recipes(&self) -> Result<Vec<Recipe>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, description, command, model_path, mmproj_path, gpu_info, tags, created_at, updated_at
                 FROM recipes ORDER BY updated_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let recipes = stmt
            .query_map([], |row| {
                Ok(Recipe {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    command: row.get(3)?,
                    model_path: row.get(4)?,
                    mmproj_path: row.get(5)?,
                    gpu_info: row.get(6)?,
                    tags: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(recipes)
    }

    pub fn get_recipe(&self, id: &str) -> Result<Recipe, String> {
        self.conn
            .query_row(
                "SELECT id, name, description, command, model_path, mmproj_path, gpu_info, tags, created_at, updated_at
                 FROM recipes WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Recipe {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                        command: row.get(3)?,
                        model_path: row.get(4)?,
                        mmproj_path: row.get(5)?,
                        gpu_info: row.get(6)?,
                        tags: row.get(7)?,
                        created_at: row.get(8)?,
                        updated_at: row.get(9)?,
                    })
                },
            )
            .map_err(|e| e.to_string())
    }

    pub fn create_recipe(&self, input: CreateRecipe) -> Result<Recipe, String> {
        validate_recipe_command(&input.command)?;

        if input.model_path.trim().is_empty() {
            return Err("Model path is required".to_string());
        }

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO recipes (id, name, description, command, model_path, mmproj_path, gpu_info, tags, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    id,
                    input.name,
                    input.description,
                    input.command,
                    input.model_path,
                    input.mmproj_path,
                    input.gpu_info,
                    input.tags,
                    now,
                    now
                ],
            )
            .map_err(|e| e.to_string())?;

        self.get_recipe(&id)
    }

    pub fn update_recipe(&self, input: UpdateRecipe) -> Result<Recipe, String> {
        validate_recipe_command(&input.command)?;

        if input.model_path.trim().is_empty() {
            return Err("Model path is required".to_string());
        }

        let now = chrono::Utc::now().to_rfc3339();

        let rows = self
            .conn
            .execute(
                "UPDATE recipes SET name = ?1, description = ?2, command = ?3, model_path = ?4, mmproj_path = ?5, gpu_info = ?6, tags = ?7, updated_at = ?8
                 WHERE id = ?9",
                params![
                    input.name,
                    input.description,
                    input.command,
                    input.model_path,
                    input.mmproj_path,
                    input.gpu_info,
                    input.tags,
                    now,
                    input.id
                ],
            )
            .map_err(|e| e.to_string())?;

        if rows == 0 {
            return Err("Recipe not found".to_string());
        }

        self.get_recipe(&input.id)
    }

    pub fn delete_recipe(&self, id: &str) -> Result<(), String> {
        let rows = self
            .conn
            .execute("DELETE FROM recipes WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;

        if rows == 0 {
            return Err("Recipe not found".to_string());
        }

        Ok(())
    }

    pub fn duplicate_recipe(&self, id: &str) -> Result<Recipe, String> {
        let recipe = self.get_recipe(id)?;
        self.create_recipe(CreateRecipe {
            name: format!("{} (copy)", recipe.name),
            description: recipe.description,
            command: recipe.command,
            model_path: recipe.model_path,
            mmproj_path: recipe.mmproj_path,
            gpu_info: recipe.gpu_info,
            tags: recipe.tags,
        })
    }
}
