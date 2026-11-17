//! `.ripe` files: a single recipe serialised as TOML for sharing.
//!
//! Unknown keys are ignored so newer files with extra metadata still load.
//! Empty optional fields are omitted on export.

use serde::{Deserialize, Serialize};

use crate::db::{CreateRecipe, Recipe};

pub const EXTENSION: &str = "ripe";
pub const VERSION: u32 = 1;
pub const MAX_FILE_BYTES: u64 = 64 * 1024;

#[derive(Debug, Serialize, Deserialize)]
struct RipeFile {
    #[serde(default = "default_version")]
    version: u32,
    name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    description: String,
    #[serde(default)]
    command: String,
    model_path: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    mmproj_path: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    gpu_info: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    tags: String,
}

fn default_version() -> u32 {
    VERSION
}

pub fn to_toml(recipe: Recipe) -> Result<String, String> {
    toml::to_string(&RipeFile {
        version: VERSION,
        name: recipe.name,
        description: recipe.description,
        command: recipe.command,
        model_path: recipe.model_path,
        mmproj_path: recipe.mmproj_path,
        gpu_info: recipe.gpu_info,
        tags: recipe.tags,
    })
    .map_err(|e| e.to_string())
}

pub fn from_toml(text: &str) -> Result<CreateRecipe, String> {
    let file: RipeFile =
        toml::from_str(text).map_err(|e| format!("Not a valid .ripe file: {}", e.message()))?;
    if file.version > VERSION {
        return Err(format!(
            "This .ripe file uses format version {}, which needs a newer version of the app",
            file.version
        ));
    }
    Ok(CreateRecipe {
        name: file.name,
        description: file.description,
        command: file.command,
        model_path: file.model_path,
        mmproj_path: file.mmproj_path,
        gpu_info: file.gpu_info,
        tags: file.tags,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recipe() -> Recipe {
        Recipe {
            id: "id".to_string(),
            name: "Qwen3 35B Q4".to_string(),
            description: "Fast coding".to_string(),
            command: "-ngl 99 -c 8192".to_string(),
            model_path: "qwen3-35b-q4.gguf".to_string(),
            mmproj_path: String::new(),
            gpu_info: "NVIDIA RTX 4090 24 GB".to_string(),
            tags: "qwen, 35b".to_string(),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn roundtrips_and_omits_empty_fields() {
        let text = to_toml(recipe()).unwrap();
        assert!(text.contains("version = 1"));
        assert!(!text.contains("mmproj_path"));
        assert!(!text.contains("id ="));
        let back = from_toml(&text).unwrap();
        assert_eq!(back.name, "Qwen3 35B Q4");
        assert_eq!(back.command, "-ngl 99 -c 8192");
        assert_eq!(back.mmproj_path, "");
    }

    #[test]
    fn accepts_the_minimal_issue_example_without_version() {
        let back = from_toml(
            "# shared by a friend\nname = \"x\"\nmodel_path = \"x.gguf\"\nfuture_key = 1\n",
        )
        .unwrap();
        assert_eq!(back.name, "x");
        assert_eq!(back.command, "");
    }

    #[test]
    fn rejects_missing_required_fields() {
        assert!(from_toml("name = \"x\"\n").is_err());
        assert!(from_toml("not toml at all").is_err());
    }

    #[test]
    fn rejects_newer_versions() {
        let err = from_toml("version = 2\nname = \"x\"\nmodel_path = \"x.gguf\"\n").unwrap_err();
        assert!(err.contains("version 2"));
    }
}
