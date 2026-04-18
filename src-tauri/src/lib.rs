mod db;
mod process;

use db::{CreateRecipe, Database, Recipe, Settings, UpdateRecipe};
use process::{LogLine, ProcessManager, ServerStatus};
use std::sync::Mutex as StdMutex;
use tauri::Manager;

struct AppState {
    db: StdMutex<Database>,
    pm: ProcessManager,
}

// ── Settings ──

#[tauri::command]
fn get_settings(state: tauri::State<AppState>) -> Result<Settings, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_settings()
}

#[tauri::command]
fn update_settings(state: tauri::State<AppState>, settings: Settings) -> Result<Settings, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_settings(&settings)?;
    db.get_settings()
}

// ── Recipes ──

#[tauri::command]
fn list_recipes(state: tauri::State<AppState>) -> Result<Vec<Recipe>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.list_recipes()
}

#[tauri::command]
fn get_recipe(state: tauri::State<AppState>, id: String) -> Result<Recipe, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_recipe(&id)
}

#[tauri::command]
fn create_recipe(state: tauri::State<AppState>, input: CreateRecipe) -> Result<Recipe, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.create_recipe(input)
}

#[tauri::command]
fn update_recipe(state: tauri::State<AppState>, input: UpdateRecipe) -> Result<Recipe, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_recipe(input)
}

#[tauri::command]
fn delete_recipe(state: tauri::State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_recipe(&id)
}

#[tauri::command]
fn duplicate_recipe(state: tauri::State<AppState>, id: String) -> Result<Recipe, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.duplicate_recipe(&id)
}

// ── Server Info ──

#[derive(Debug, Clone, serde::Serialize)]
struct LlamaServerInfo {
    version: String,
    compiler: String,
    gpu_devices: Vec<GpuDevice>,
    raw_output: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct GpuDevice {
    name: String,
    vram_mib: u64,
    compute_capability: String,
}

#[tauri::command]
async fn get_llama_server_info(
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

    let mut version = String::new();
    let mut compiler = String::new();
    let mut gpu_devices = Vec::new();

    for line in combined.lines() {
        let trimmed = line.trim();

        // "version: 0 ()" or "version: 1234 (b1234)"
        if trimmed.starts_with("version:") {
            version = trimmed.strip_prefix("version:").unwrap_or("").trim().to_string();
        }

        // "built with GNU 15.2.1 for Linux x86_64"
        if trimmed.starts_with("built with") {
            compiler = trimmed.to_string();
        }

        // "  Device 0: NVIDIA GeForce RTX 5060 Ti, compute capability 12.0, VMM: yes, VRAM: 15841 MiB"
        if trimmed.starts_with("Device ") && trimmed.contains("VRAM:") {
            let mut name = String::new();
            let mut vram_mib: u64 = 0;
            let mut cc = String::new();

            // Extract name: between "Device N: " and ","
            if let Some(after_colon) = trimmed.split_once(": ") {
                let rest = after_colon.1;
                if let Some(comma_pos) = rest.find(", compute capability") {
                    name = rest[..comma_pos].to_string();
                } else if let Some(comma_pos) = rest.find(',') {
                    name = rest[..comma_pos].to_string();
                }
            }

            // Extract compute capability
            if let Some(cc_start) = trimmed.find("compute capability ") {
                let after = &trimmed[cc_start + "compute capability ".len()..];
                if let Some(end) = after.find(',') {
                    cc = after[..end].to_string();
                } else {
                    cc = after.to_string();
                }
            }

            // Extract VRAM
            if let Some(vram_start) = trimmed.find("VRAM: ") {
                let after = &trimmed[vram_start + "VRAM: ".len()..];
                let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
                vram_mib = num_str.parse().unwrap_or(0);
            }

            gpu_devices.push(GpuDevice {
                name,
                vram_mib,
                compute_capability: cc,
            });
        }

        // Metal / Apple Silicon: "(Total VRAM: XXXXX MiB)" in the init line
        // "ggml_metal_init: found device: Apple M4 Max"
        if trimmed.contains("found device:") && !trimmed.contains("CUDA") {
            if let Some(dev_start) = trimmed.find("found device:") {
                let dev_name = trimmed[dev_start + "found device:".len()..].trim().to_string();
                // Try to find total VRAM from other lines -- we'll check the combined output
                let mut total_vram: u64 = 0;
                for vline in combined.lines() {
                    if vline.contains("Total VRAM:") {
                        if let Some(vs) = vline.find("Total VRAM: ") {
                            let after = &vline[vs + "Total VRAM: ".len()..];
                            let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
                            total_vram = num_str.parse().unwrap_or(0);
                        }
                    }
                }
                gpu_devices.push(GpuDevice {
                    name: dev_name,
                    vram_mib: total_vram,
                    compute_capability: String::new(),
                });
            }
        }
    }

    // If no GPU devices found from device lines, check for total VRAM summary
    if gpu_devices.is_empty() {
        for line in combined.lines() {
            if line.contains("Total VRAM:") {
                let mut total_vram: u64 = 0;
                if let Some(vs) = line.find("Total VRAM: ") {
                    let after = &line[vs + "Total VRAM: ".len()..];
                    let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
                    total_vram = num_str.parse().unwrap_or(0);
                }
                if total_vram > 0 {
                    // Try to extract device count
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

    Ok(LlamaServerInfo {
        version,
        compiler,
        gpu_devices,
        raw_output: combined,
    })
}

// ── Server Process ──

#[tauri::command]
async fn start_server(
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
        .start_server(recipe_id, command, model_path, mmproj_path, settings, app_handle)
        .await
}

#[tauri::command]
async fn stop_server(
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
async fn get_server_status(
    state: tauri::State<'_, AppState>,
) -> Result<Option<ServerStatus>, String> {
    Ok(state.pm.get_status().await)
}

#[tauri::command]
async fn get_server_logs(state: tauri::State<'_, AppState>) -> Result<Vec<LogLine>, String> {
    Ok(state.pm.get_logs().await)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            let db = Database::new(app_dir).expect("failed to initialize database");
            app.manage(AppState {
                db: StdMutex::new(db),
                pm: ProcessManager::new(),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            update_settings,
            list_recipes,
            get_recipe,
            create_recipe,
            update_recipe,
            delete_recipe,
            duplicate_recipe,
            get_llama_server_info,
            start_server,
            stop_server,
            get_server_status,
            get_server_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
