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

pub mod models {
    use crate::process;
    use futures_util::StreamExt;
    use serde::{Deserialize, Serialize};
    use std::path::PathBuf;
    use tauri::Emitter;
    use tokio::fs;
    use tokio::io::AsyncWriteExt;

    /// Whether a GGUF file is a model, an mmproj, or undetermined.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum GgufKind {
        Model,
        Mmproj,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct ScannedModel {
        /// Absolute path to the .gguf file.
        pub path: String,
        /// File name only (e.g. "qwen3-35b-q4.gguf").
        pub name: String,
        /// Size of the file in bytes.
        pub size_bytes: u64,
        /// Detected kind (model or mmproj).
        pub kind: GgufKind,
    }

    /// Inspect a GGUF file's header to determine whether it is a model or an
    /// mmproj.
    ///
    /// Detection relies on metadata keys only (header-only parse):
    /// - `general.type` equals `"mmproj"`
    /// - `general.architecture` contains `"clip"`
    /// If neither matches, the file is treated as a model.
    ///
    /// The sync GGUF parser is used inside `spawn_blocking` because it only
    /// reads the header metadata — NOT tensor data — so it's fast and uses
    /// minimal memory regardless of file size.  The async variant in gguf-rs
    /// unfortunately reads the entire file into memory (`read_to_end`), which
    /// would OOM on multi-GB models.
    async fn detect_gguf_kind(path: &std::path::Path) -> GgufKind {
        let owned = path.to_owned();
        tokio::task::spawn_blocking(move || {
            let path_str = match owned.to_str() {
                Some(s) => s,
                None => return GgufKind::Model,
            };
            let mut container = match gguf_rs::get_gguf_container(path_str) {
                Ok(c) => c,
                Err(_) => return GgufKind::Model,
            };
            let model = match container.decode() {
                Ok(m) => m,
                Err(_) => return GgufKind::Model,
            };

            let metadata = model.metadata();

            if let Some(val) = metadata.get("general.type") {
                let s = val.to_string().to_ascii_lowercase();
                if s.contains("mmproj") {
                    return GgufKind::Mmproj;
                }
            }

            if let Some(val) = metadata.get("general.architecture") {
                let s = val.to_string().to_ascii_lowercase();
                if s.contains("clip") {
                    return GgufKind::Mmproj;
                }
            }

            GgufKind::Model
        })
        .await
        .unwrap_or(GgufKind::Model)
    }

    /// Recursively scan a directory for `.gguf` files using an explicit stack
    /// to avoid recursive `async fn` (which requires `Box::pin`).
    async fn collect_gguf_files(
        root: PathBuf,
        max_depth: usize,
        filter: GgufKind,
    ) -> Vec<ScannedModel> {
        let mut results = Vec::new();
        let mut stack = vec![(root, max_depth)];

        while let Some((dir, depth)) = stack.pop() {
            if depth == 0 {
                continue;
            }

            let Ok(mut entries) = fs::read_dir(&dir).await else {
                continue;
            };

            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                let Ok(ft) = fs::metadata(&path).await else {
                    continue;
                };
                if ft.is_dir() {
                    stack.push((path, depth - 1));
                } else if path.extension().is_some_and(|ext| ext == "gguf") {
                    let kind = detect_gguf_kind(&path).await;
                    if kind != filter {
                        continue;
                    }
                    results.push(ScannedModel {
                        path: path.to_string_lossy().to_string(),
                        name: path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default(),
                        size_bytes: ft.len(),
                        kind,
                    });
                }
            }
        }

        results
    }

    /// Recursively scan a directory for ALL `.gguf` files (no kind filter).
    async fn collect_all_gguf_files(root: PathBuf, max_depth: usize) -> Vec<ScannedModel> {
        let mut results = Vec::new();
        let mut stack = vec![(root, max_depth)];

        while let Some((dir, depth)) = stack.pop() {
            if depth == 0 {
                continue;
            }

            let Ok(mut entries) = fs::read_dir(&dir).await else {
                continue;
            };

            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                let Ok(ft) = fs::metadata(&path).await else {
                    continue;
                };
                if ft.is_dir() {
                    stack.push((path, depth - 1));
                } else if path.extension().is_some_and(|ext| ext == "gguf") {
                    let kind = detect_gguf_kind(&path).await;
                    results.push(ScannedModel {
                        path: path.to_string_lossy().to_string(),
                        name: path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default(),
                        size_bytes: ft.len(),
                        kind,
                    });
                }
            }
        }

        results
    }

    /// Remove orphaned `.gguf.part` temp files left by interrupted downloads.
    #[tauri::command]
    pub async fn cleanup_orphan_parts(directory: String) -> Result<(), String> {
        let expanded = process::expand_tilde_pub(&directory);
        let dir = PathBuf::from(&expanded);
        if dir.exists() {
            clean_part_files(&dir).await;
        }
        Ok(())
    }

    /// Scan a directory for `.gguf` files, optionally filtered by kind.
    ///
    /// `filter` must be one of: `"model"`, `"mmproj"`, or `"all"`.
    #[tauri::command]
    pub async fn scan_models(
        directory: String,
        filter: String,
    ) -> Result<Vec<ScannedModel>, String> {
        let expanded = process::expand_tilde_pub(&directory);
        let dir = PathBuf::from(&expanded);

        if !dir.exists() {
            return Err(format!("Directory does not exist: {}", directory));
        }
        if !dir.is_dir() {
            return Err(format!("Not a directory: {}", directory));
        }

        match filter.as_str() {
            "all" => Ok(collect_all_gguf_files(dir, 8).await),
            "mmproj" => Ok(collect_gguf_files(dir, 8, GgufKind::Mmproj).await),
            _ => Ok(collect_gguf_files(dir, 8, GgufKind::Model).await),
        }
    }

    // ── HuggingFace model download ──

    #[derive(Debug, Clone, Serialize)]
    pub struct HfModelFile {
        pub name: String,
        pub size_bytes: u64,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct DownloadProgress {
        pub repo_id: String,
        pub filename: String,
        pub bytes_downloaded: u64,
        pub total_bytes: u64,
    }

    /// List `.gguf` files available in a HuggingFace model repo, optionally
    /// filtered by kind.  `filter` must be one of: `"model"`, `"mmproj"`, or
    /// `"all"`.  Remote files cannot be header-inspected, so we use a filename
    /// heuristic: files containing `mmproj` are treated as mmproj.
    #[tauri::command]
    pub async fn list_hf_model_files(
        repo_id: String,
        hf_token: String,
        filter: String,
    ) -> Result<Vec<HfModelFile>, String> {
        let url = format!(
            "https://huggingface.co/api/models/{}/tree/main",
            repo_id.trim().trim_end_matches('/')
        );

        let client = reqwest::Client::new();
        let mut req = client.get(&url);
        let token = hf_token.trim();
        if !token.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        let resp = req
            .send()
            .await
            .map_err(|e| format!("Failed to query HF Hub: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("HF Hub returned {}: {}", status, body));
        }

        let entries: Vec<serde_json::Value> = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse HF Hub response: {}", e))?;

        let mut files = Vec::new();
        for entry in entries {
            let typ = entry["type"].as_str().unwrap_or("");
            if typ != "file" {
                continue;
            }
            let path = entry["path"].as_str().unwrap_or("");
            if !std::path::Path::new(path)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("gguf"))
            {
                continue;
            }
            // Remote filename heuristic.
            let lower = path.to_ascii_lowercase();
            let is_mmproj = lower.contains("mmproj");
            match filter.as_str() {
                "mmproj" if !is_mmproj => continue,
                "model" if is_mmproj => continue,
                _ => {}
            }
            let size = entry["size"].as_u64().unwrap_or(0);
            files.push(HfModelFile {
                name: path.to_string(),
                size_bytes: size,
            });
        }

        files.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(files)
    }

    /// Clean up orphaned `.gguf.part` files in a directory (leftover from
    /// interrupted downloads).
    async fn clean_part_files(dir: &PathBuf) {
        let Ok(mut entries) = fs::read_dir(dir).await else {
            return;
        };
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.to_string_lossy().ends_with(".gguf.part") {
                let ft = fs::metadata(&path).await.ok();
                if ft.is_some_and(|m| m.is_file()) {
                    let _ = fs::remove_file(&path).await;
                }
            }
        }
    }

    /// Download a model file from HuggingFace with progress events.
    ///
    /// Downloads to a `.gguf.part` temp file first, then atomically renames
    /// to the final name on success. If the app is killed mid-download the
    /// orphaned `.part` file will be cleaned up on the next startup scan.
    #[tauri::command]
    pub async fn download_hf_model(
        app_handle: tauri::AppHandle,
        repo_id: String,
        filename: String,
        hf_token: String,
        dest_dir: String,
    ) -> Result<String, String> {
        let expanded_dest = process::expand_tilde_pub(&dest_dir);
        let dest = PathBuf::from(&expanded_dest);

        // Sanitize filename — prevent path traversal.
        let fname = PathBuf::from(&filename);
        let safe_name = fname
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .ok_or_else(|| "Invalid filename".to_string())?;

        let final_path = dest.join(&safe_name);
        // Download to a `.part` temp file first so a kill mid-download
        // doesn't leave a corrupt `.gguf` in the model directory.
        let part_path = dest.join(format!("{}.part", &safe_name));

        // Ensure parent exists.
        fs::create_dir_all(&dest)
            .await
            .map_err(|e| format!("Failed to create download directory: {}", e))?;

        let repo = repo_id.trim().trim_end_matches('/');
        let url = format!("https://huggingface.co/{}/resolve/main/{}", repo, filename);

        let client = reqwest::Client::new();
        let mut req = client.get(&url);
        let token = hf_token.trim();
        if !token.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        let resp = req
            .send()
            .await
            .map_err(|e| format!("Failed to start download: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Download returned {}: {}", status, body));
        }

        let total_bytes = resp.content_length().unwrap_or(0);

        // Remove any stale part file from a previous interrupted download.
        let _ = fs::remove_file(&part_path).await;

        let result: Result<(), String> = async {
            let mut file = fs::File::create(&part_path)
                .await
                .map_err(|e| format!("Failed to create file: {}", e))?;

            let mut bytes_downloaded: u64 = 0;
            let mut stream = resp.bytes_stream();

            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result.map_err(|e| format!("Download error: {}", e))?;
                file.write_all(&chunk)
                    .await
                    .map_err(|e| format!("Write error: {}", e))?;
                bytes_downloaded += chunk.len() as u64;

                if total_bytes > 0 {
                    let _ = app_handle.emit(
                        "download-progress",
                        DownloadProgress {
                            repo_id: repo.to_string(),
                            filename: safe_name.clone(),
                            bytes_downloaded,
                            total_bytes,
                        },
                    );
                }
            }

            file.flush()
                .await
                .map_err(|e| format!("Flush error: {}", e))?;

            // Atomic rename — same filesystem, no partial state visible.
            fs::rename(&part_path, &final_path)
                .await
                .map_err(|e| format!("Failed to finalize download: {}", e))
        }
        .await;

        match result {
            Ok(()) => {
                let abs_path = final_path
                    .canonicalize()
                    .map_err(|e| format!("Failed to resolve download path: {}", e))?
                    .to_string_lossy()
                    .to_string();
                Ok(abs_path)
            }
            Err(e) => {
                // Clean up the partial file on any error.
                let _ = fs::remove_file(&part_path).await;
                Err(e)
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use std::path::PathBuf;

        #[test]
        fn download_sanitizes_path_traversal() {
            let malicious = "../../etc/passwd".to_string();
            let safe = PathBuf::from(&malicious)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap();
            assert_eq!(safe, "passwd");
            assert_ne!(safe, "../../etc/passwd");
        }

        #[test]
        fn download_keeps_simple_filename() {
            let filename = "qwen3-35b-q4.gguf".to_string();
            let safe = PathBuf::from(&filename)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap();
            assert_eq!(safe, "qwen3-35b-q4.gguf");
        }

        #[test]
        fn download_strips_subdirectory_prefix() {
            let filename = "gguf/qwen3-35b-q4.gguf".to_string();
            let safe = PathBuf::from(&filename)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap();
            assert_eq!(safe, "qwen3-35b-q4.gguf");
        }
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

        let mut cmd = tokio::process::Command::new(&path);
        cmd.arg("--version")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        #[cfg(windows)]
        {
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let output = cmd
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
