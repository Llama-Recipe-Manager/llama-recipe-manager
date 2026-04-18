//! Tauri command handlers.
//!
//! Grouped into nested modules by domain. The flat re-exports at the bottom
//! keep `tauri::generate_handler!` invocations short.

pub mod settings {
    use crate::db::Settings;
    use crate::state::AppState;

    #[tauri::command]
    pub fn get_settings(state: tauri::State<AppState>) -> Result<Settings, String> {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.get_settings()
    }

    #[tauri::command]
    pub fn update_settings(
        state: tauri::State<AppState>,
        settings: Settings,
    ) -> Result<Settings, String> {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.update_settings(&settings)?;
        db.get_settings()
    }
}

pub mod recipes {
    use crate::db::{CreateRecipe, Recipe, UpdateRecipe};
    use crate::state::AppState;

    #[tauri::command]
    pub fn list_recipes(state: tauri::State<AppState>) -> Result<Vec<Recipe>, String> {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.list_recipes()
    }

    #[tauri::command]
    pub fn get_recipe(state: tauri::State<AppState>, id: String) -> Result<Recipe, String> {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.get_recipe(&id)
    }

    #[tauri::command]
    pub fn create_recipe(
        state: tauri::State<AppState>,
        input: CreateRecipe,
    ) -> Result<Recipe, String> {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.create_recipe(input)
    }

    #[tauri::command]
    pub fn update_recipe(
        state: tauri::State<AppState>,
        input: UpdateRecipe,
    ) -> Result<Recipe, String> {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.update_recipe(input)
    }

    #[tauri::command]
    pub fn delete_recipe(state: tauri::State<AppState>, id: String) -> Result<(), String> {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.delete_recipe(&id)
    }

    #[tauri::command]
    pub fn duplicate_recipe(state: tauri::State<AppState>, id: String) -> Result<Recipe, String> {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.duplicate_recipe(&id)
    }
}

pub mod server {
    use serde::Serialize;

    use crate::process::{self, LogLine, ServerStatus};
    use crate::state::AppState;

    #[derive(Debug, Clone, Serialize)]
    pub struct GpuDevice {
        pub name: String,
        pub vram_mib: u64,
        pub compute_capability: String,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct LlamaServerInfo {
        pub version: String,
        pub compiler: String,
        pub gpu_devices: Vec<GpuDevice>,
        pub raw_output: String,
    }

    #[tauri::command]
    pub async fn get_llama_server_info(
        state: tauri::State<'_, AppState>,
    ) -> Result<LlamaServerInfo, String> {
        let settings = {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            db.get_settings()?
        };

        let path = process::expand_tilde_pub(&settings.llama_server_path);

        let output = tokio::process::Command::new(&path)
            .arg("--version")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to run '{}': {}", path, e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let combined = format!("{}{}", stderr, stdout);

        Ok(parse_llama_server_info(combined))
    }

    fn parse_llama_server_info(combined: String) -> LlamaServerInfo {
        let mut version = String::new();
        let mut compiler = String::new();
        let mut gpu_devices = Vec::new();

        for line in combined.lines() {
            let trimmed = line.trim();

            if let Some(rest) = trimmed.strip_prefix("version:") {
                version = rest.trim().to_string();
            }

            if trimmed.starts_with("built with") {
                compiler = trimmed.to_string();
            }

            if trimmed.starts_with("Device ") && trimmed.contains("VRAM:") {
                gpu_devices.push(parse_cuda_device_line(trimmed));
            }

            if trimmed.contains("found device:") && !trimmed.contains("CUDA") {
                if let Some(dev_start) = trimmed.find("found device:") {
                    let dev_name = trimmed[dev_start + "found device:".len()..]
                        .trim()
                        .to_string();
                    let total_vram = combined
                        .lines()
                        .find_map(|l| l.find("Total VRAM: ").map(|i| (l, i)))
                        .map(|(l, i)| {
                            l[i + "Total VRAM: ".len()..]
                                .chars()
                                .take_while(|c| c.is_ascii_digit())
                                .collect::<String>()
                                .parse::<u64>()
                                .unwrap_or(0)
                        })
                        .unwrap_or(0);
                    gpu_devices.push(GpuDevice {
                        name: dev_name,
                        vram_mib: total_vram,
                        compute_capability: String::new(),
                    });
                }
            }
        }

        if gpu_devices.is_empty() {
            for line in combined.lines() {
                if line.contains("Total VRAM:") {
                    let total_vram = line
                        .find("Total VRAM: ")
                        .map(|i| {
                            line[i + "Total VRAM: ".len()..]
                                .chars()
                                .take_while(|c| c.is_ascii_digit())
                                .collect::<String>()
                                .parse::<u64>()
                                .unwrap_or(0)
                        })
                        .unwrap_or(0);
                    if total_vram > 0 {
                        let device_hint = if line.contains("CUDA") {
                            "CUDA GPU"
                        } else {
                            "GPU"
                        };
                        gpu_devices.push(GpuDevice {
                            name: device_hint.to_string(),
                            vram_mib: total_vram,
                            compute_capability: String::new(),
                        });
                    }
                }
            }
        }

        LlamaServerInfo {
            version,
            compiler,
            gpu_devices,
            raw_output: combined,
        }
    }

    fn parse_cuda_device_line(trimmed: &str) -> GpuDevice {
        let mut name = String::new();
        let mut vram_mib: u64 = 0;
        let mut cc = String::new();

        if let Some((_, rest)) = trimmed.split_once(": ") {
            if let Some(comma_pos) = rest.find(", compute capability") {
                name = rest[..comma_pos].to_string();
            } else if let Some(comma_pos) = rest.find(',') {
                name = rest[..comma_pos].to_string();
            }
        }

        if let Some(cc_start) = trimmed.find("compute capability ") {
            let after = &trimmed[cc_start + "compute capability ".len()..];
            cc = match after.find(',') {
                Some(end) => after[..end].to_string(),
                None => after.to_string(),
            };
        }

        if let Some(vram_start) = trimmed.find("VRAM: ") {
            let after = &trimmed[vram_start + "VRAM: ".len()..];
            let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            vram_mib = num_str.parse().unwrap_or(0);
        }

        GpuDevice {
            name,
            vram_mib,
            compute_capability: cc,
        }
    }

    #[tauri::command]
    pub async fn start_server(
        state: tauri::State<'_, AppState>,
        app_handle: tauri::AppHandle,
        recipe_id: String,
        command: String,
        model_path: String,
        mmproj_path: String,
    ) -> Result<(), String> {
        let settings = {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            db.get_settings()?
        };
        state
            .pm
            .start_server(
                recipe_id,
                command,
                model_path,
                mmproj_path,
                settings,
                app_handle,
            )
            .await
    }

    #[tauri::command]
    pub async fn stop_server(
        state: tauri::State<'_, AppState>,
        app_handle: tauri::AppHandle,
    ) -> Result<(), String> {
        let recipe_id = state.pm.stop_server().await?;
        let _ = tauri::Emitter::emit(
            &app_handle,
            "server-status",
            &ServerStatus {
                recipe_id,
                running: false,
                pid: None,
            },
        );
        Ok(())
    }

    #[tauri::command]
    pub async fn get_server_status(
        state: tauri::State<'_, AppState>,
    ) -> Result<Option<ServerStatus>, String> {
        Ok(state.pm.get_status().await)
    }

    #[tauri::command]
    pub async fn get_server_logs(
        state: tauri::State<'_, AppState>,
    ) -> Result<Vec<LogLine>, String> {
        Ok(state.pm.get_logs().await)
    }

    #[tauri::command]
    pub async fn clear_server_logs(state: tauri::State<'_, AppState>) -> Result<(), String> {
        state.pm.clear_logs().await;
        Ok(())
    }
}
