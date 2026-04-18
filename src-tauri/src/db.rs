//! SQLite persistence for recipes and settings.
//!
//! All schema changes happen through [`migrations`]. Validation rules that
//! span layers live in [`crate::validate`].

mod migrations;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::validate::{validate_recipe_command, validate_recipe_fields};

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
#[allow(clippy::struct_excessive_bools)]
pub struct Settings {
    pub host: String,
    pub port: u16,
    pub model_dir: String,
    pub llama_server_path: String,
    // Security
    pub api_key: String,
    pub ssl_cert_file: String,
    pub ssl_key_file: String,
    pub hf_token: String,
    // Server behavior
    pub webui_enabled: bool,
    pub metrics_enabled: bool,
    pub slots_enabled: bool,
    pub api_prefix: String,
    pub timeout_secs: u32,
    // Diagnostics
    pub log_verbosity: u8,
    // Lifecycle
    pub keep_server_on_exit: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            model_dir: "~/Models".to_string(),
            llama_server_path: "llama-server".to_string(),
            api_key: String::new(),
            ssl_cert_file: String::new(),
            ssl_key_file: String::new(),
            hf_token: String::new(),
            webui_enabled: true,
            metrics_enabled: true,
            slots_enabled: true,
            api_prefix: String::new(),
            timeout_secs: 600,
            log_verbosity: 3,
            keep_server_on_exit: false,
        }
    }
}

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open (or create) the database at `<app_dir>/recipes.db` and run any
    /// pending migrations.
    pub fn new(app_dir: PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
        let db_path = app_dir.join("recipes.db");
        let mut conn = Connection::open(db_path).map_err(|e| e.to_string())?;
        migrations::run(&mut conn)?;
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
                "api_key" => settings.api_key = value,
                "ssl_cert_file" => settings.ssl_cert_file = value,
                "ssl_key_file" => settings.ssl_key_file = value,
                "hf_token" => settings.hf_token = value,
                "webui_enabled" => settings.webui_enabled = value != "0",
                "metrics_enabled" => settings.metrics_enabled = value == "1",
                "slots_enabled" => settings.slots_enabled = value != "0",
                "api_prefix" => settings.api_prefix = value,
                "timeout_secs" => {
                    if let Ok(n) = value.parse::<u32>() {
                        settings.timeout_secs = n;
                    }
                }
                "log_verbosity" => {
                    if let Ok(n) = value.parse::<u8>() {
                        settings.log_verbosity = n;
                    }
                }
                "keep_server_on_exit" => settings.keep_server_on_exit = value == "1",
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
            ("api_key", settings.api_key.clone()),
            ("ssl_cert_file", settings.ssl_cert_file.clone()),
            ("ssl_key_file", settings.ssl_key_file.clone()),
            ("hf_token", settings.hf_token.clone()),
            (
                "webui_enabled",
                u8::from(settings.webui_enabled).to_string(),
            ),
            (
                "metrics_enabled",
                u8::from(settings.metrics_enabled).to_string(),
            ),
            (
                "slots_enabled",
                u8::from(settings.slots_enabled).to_string(),
            ),
            ("api_prefix", settings.api_prefix.clone()),
            ("timeout_secs", settings.timeout_secs.to_string()),
            ("log_verbosity", settings.log_verbosity.to_string()),
            (
                "keep_server_on_exit",
                u8::from(settings.keep_server_on_exit).to_string(),
            ),
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
            .query_map([], row_to_recipe)
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
                row_to_recipe,
            )
            .map_err(|e| e.to_string())
    }

    pub fn create_recipe(&self, input: CreateRecipe) -> Result<Recipe, String> {
        validate_recipe_fields(
            &input.name,
            &input.description,
            &input.command,
            &input.model_path,
            &input.mmproj_path,
            &input.gpu_info,
            &input.tags,
        )?;
        validate_recipe_command(&input.command)?;

        if input.name.trim().is_empty() {
            return Err("Name is required".to_string());
        }
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
        validate_recipe_fields(
            &input.name,
            &input.description,
            &input.command,
            &input.model_path,
            &input.mmproj_path,
            &input.gpu_info,
            &input.tags,
        )?;
        validate_recipe_command(&input.command)?;

        if input.name.trim().is_empty() {
            return Err("Name is required".to_string());
        }
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

fn row_to_recipe(row: &rusqlite::Row<'_>) -> rusqlite::Result<Recipe> {
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
}
