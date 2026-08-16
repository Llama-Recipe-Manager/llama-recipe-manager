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
    ///
    ///  - `general.type` equals `"mmproj"`
    ///  - `general.architecture` contains `"clip"`
    ///
    /// If neither matches, the file is treated as a model.
    ///
    /// Only the GGUF header is read — tensor data is never loaded — so this
    /// is fast and uses minimal memory regardless of file size.
    fn detect_gguf_kind(path: &std::path::Path) -> GgufKind {
        let Some(path_str) = path.to_str() else {
            return GgufKind::Model;
        };
        let Ok(mut container) = gguf_rs::get_gguf_container(path_str) else {
            return GgufKind::Model;
        };
        let Ok(model) = container.decode() else {
            return GgufKind::Model;
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
    }

    /// Walk a directory tree (up to `max_depth` levels deep) collecting `.gguf`
    /// files, returning the total match count plus only the slice for the
    /// requested page.
    ///
    /// The whole tree is still walked — each `.gguf` header must be inspected
    /// to determine its kind — but only the requested page is materialized into
    /// a `Vec`, so memory stays bounded regardless of how many models exist.
    ///
    /// `filter` is `None` to match all files, or a specific kind to match only
    /// those. `page` is 1-based; `page_size` is the max items per page. An
    /// explicit stack is used to avoid recursive `async fn` (which would
    /// require `Box::pin`).
    async fn scan_gguf_files(
        root: PathBuf,
        max_depth: usize,
        filter: Option<GgufKind>,
        page: usize,
        page_size: usize,
    ) -> (usize, Vec<ScannedModel>) {
        let mut total = 0usize;
        let mut page_items = Vec::with_capacity(page_size);
        let start = (page.saturating_sub(1)) * page_size;
        let end = start + page_size;
        // Reuse a single buffer for entry names to avoid re-allocating.
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
                    continue;
                }
                if path.extension().is_none_or(|ext| ext != "gguf") {
                    continue;
                }
                let kind = detect_gguf_kind(&path);
                if let Some(expected) = filter {
                    if kind != expected {
                        continue;
                    }
                }
                // 1-based index over the matching files.
                total += 1;
                if total > start && total <= end {
                    page_items.push(ScannedModel {
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

        (total, page_items)
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

    /// Result of a paginated [`scan_models`] request.
    #[derive(Debug, Clone, Serialize)]
    pub struct ModelScanPage {
        /// The requested page of matching models.
        pub items: Vec<ScannedModel>,
        /// Total number of matching models across all pages.
        pub total: usize,
        /// 1-based page that `items` belongs to.
        pub page: usize,
        /// Number of items returned per page.
        pub page_size: usize,
    }

    /// Scan a directory for `.gguf` files, optionally filtered by kind and
    /// paginated.
    ///
    /// `filter` must be one of: `"model"`, `"mmproj"`, or `"all"`.
    /// `page` defaults to 1 and `page_size` to 100 (clamped to 1..=1000).
    #[tauri::command]
    pub async fn scan_models(
        directory: String,
        filter: String,
        page: Option<usize>,
        page_size: Option<usize>,
    ) -> Result<ModelScanPage, String> {
        let expanded = process::expand_tilde_pub(&directory);
        let dir = PathBuf::from(&expanded);

        if !dir.exists() {
            return Err(format!("Directory does not exist: {}", directory));
        }
        if !dir.is_dir() {
            return Err(format!("Not a directory: {}", directory));
        }

        let page = page.unwrap_or(1).max(1);
        let page_size = page_size.unwrap_or(100).clamp(1, 1000);

        let filter_kind = match filter.as_str() {
            "all" => None,
            "mmproj" => Some(GgufKind::Mmproj),
            _ => Some(GgufKind::Model),
        };

        let (total, items) = scan_gguf_files(dir, 8, filter_kind, page, page_size).await;
        Ok(ModelScanPage {
            items,
            total,
            page,
            page_size,
        })
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
        let part_path = dest.join(format!("{}.part", safe_name));

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

        use super::{scan_gguf_files, GgufKind};

        /// Create a temp dir with `count` empty `.gguf` files. Empty files
        /// aren't parseable as GGUF, so `detect_gguf_kind` classifies them all
        /// as `Model` — perfect for exercising pagination deterministically.
        fn temp_dir_with_ggufs(count: usize) -> tempfile::TempDir {
            let dir = tempfile::tempdir().expect("tempdir");
            for i in 0..count {
                let name = format!("model-{}.gguf", i);
                std::fs::write(dir.path().join(&name), b"x").expect("write gguf");
            }
            dir
        }

        #[test]
        fn scan_returns_total_across_all_pages() {
            let dir = temp_dir_with_ggufs(10);
            let rt = tokio::runtime::Runtime::new().expect("runtime");

            for page in 1..=3 {
                let (total, items) = rt.block_on(scan_gguf_files(
                    dir.path().to_path_buf(),
                    8,
                    Some(GgufKind::Model),
                    page,
                    4,
                ));
                assert_eq!(total, 10, "total must be stable across pages");
                assert!(items.len() <= 4);
            }
        }

        #[test]
        fn scan_pages_cover_every_file_exactly_once() {
            let dir = temp_dir_with_ggufs(10);
            let rt = tokio::runtime::Runtime::new().expect("runtime");

            let mut names = Vec::new();
            let mut seen_total = 0;
            for page in 1..=3 {
                let (total, items) = rt.block_on(scan_gguf_files(
                    dir.path().to_path_buf(),
                    8,
                    Some(GgufKind::Model),
                    page,
                    4,
                ));
                seen_total = total;
                names.extend(items.iter().map(|m| m.name.clone()));
            }
            assert_eq!(seen_total, 10);
            assert_eq!(names.len(), 10, "each file must appear in exactly one page");

            let expected: Vec<String> = (0..10).map(|i| format!("model-{}.gguf", i)).collect();
            names.sort();
            assert_eq!(names, expected);
        }

        #[test]
        fn scan_returns_empty_page_when_page_is_out_of_range() {
            let dir = temp_dir_with_ggufs(10);
            let rt = tokio::runtime::Runtime::new().expect("runtime");

            let (total, items) = rt.block_on(scan_gguf_files(
                dir.path().to_path_buf(),
                8,
                Some(GgufKind::Model),
                99,
                4,
            ));
            assert_eq!(total, 10);
            assert!(items.is_empty());
        }

        #[test]
        fn scan_filters_out_non_matching_kinds() {
            let dir = temp_dir_with_ggufs(5);
            let rt = tokio::runtime::Runtime::new().expect("runtime");

            // Files are plain (unparseable) so they're all `Model`; ask for
            // `Mmproj` and confirm none match.
            let (total, items) = rt.block_on(scan_gguf_files(
                dir.path().to_path_buf(),
                8,
                Some(GgufKind::Mmproj),
                1,
                100,
            ));
            assert_eq!(total, 0);
            assert!(items.is_empty());
        }

        #[test]
        fn scan_with_no_filter_matches_every_file() {
            let dir = temp_dir_with_ggufs(4);
            let rt = tokio::runtime::Runtime::new().expect("runtime");

            let (total, items) =
                rt.block_on(scan_gguf_files(dir.path().to_path_buf(), 8, None, 1, 100));
            assert_eq!(total, 4);
            assert_eq!(items.len(), 4);
        }

        #[test]
        fn scan_respects_max_depth() {
            let dir = tempfile::tempdir().expect("tempdir");
            std::fs::write(dir.path().join("top.gguf"), b"x").expect("write");
            std::fs::create_dir(dir.path().join("nested")).expect("mkdir");
            std::fs::write(dir.path().join("nested/deep.gguf"), b"x").expect("write");

            let rt = tokio::runtime::Runtime::new().expect("runtime");
            // Depth 1: only the top-level file is reached.
            let (total, _) =
                rt.block_on(scan_gguf_files(dir.path().to_path_buf(), 1, None, 1, 100));
            assert_eq!(total, 1);
            // Depth 2: both files.
            let (total2, _) =
                rt.block_on(scan_gguf_files(dir.path().to_path_buf(), 2, None, 1, 100));
            assert_eq!(total2, 2);
        }

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

    use crate::gpu::GpuDevice;
    use crate::process::{self, LogLine, ServerStatus};
    use crate::state::AppState;

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

        let mut info = parse_llama_server_info(combined);

        // Prefer vendor/OS GPU tooling (nvidia-smi, rocm-smi/amd-smi,
        // system_profiler) over the `--version` output, which does not
        // reliably list devices. Fall back to the llama parse if none found.
        let tool_devices = crate::gpu::detect_gpu_devices().await;
        if !tool_devices.is_empty() {
            info.gpu_devices = tool_devices;
        }

        Ok(info)
    }

    fn parse_llama_server_info(combined: String) -> LlamaServerInfo {
        let mut version = String::new();
        let mut compiler = String::new();
        let mut gpu_devices = Vec::new();
        let lines: Vec<&str> = combined.lines().collect();

        for line in &lines {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("version:") {
                version = rest.trim().to_string();
            } else if trimmed.starts_with("built with") {
                compiler = trimmed.to_string();
            }
        }

        // Parse every device entry. CUDA prints "Device N: NAME, compute
        // capability Y.Z, VMM: ..." (VRAM is usually on a separate line or
        // absent entirely), while Metal/Vulkan print "found device: NAME"
        // plus a "Total VRAM: N MiB" line elsewhere in the output.
        for line in &lines {
            let trimmed = line.trim();
            let Some((name, cc)) = parse_device_line(trimmed) else {
                continue;
            };
            let vram_mib = vram_mib_from_line(trimmed)
                .or_else(|| total_vram_mib(&lines))
                .unwrap_or(0);
            gpu_devices.push(GpuDevice {
                name,
                vram_mib,
                compute_capability: cc,
            });
        }

        // Fallback: some builds report a generic total without a device line.
        if gpu_devices.is_empty() {
            if let Some(total_vram) = total_vram_mib(&lines) {
                if total_vram > 0 {
                    let device_hint = if combined.contains("CUDA") {
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

        LlamaServerInfo {
            version,
            compiler,
            gpu_devices,
            raw_output: combined,
        }
    }

    /// Try to extract a device (name, compute capability) from a single
    /// `--version` line. Handles CUDA's `Device N:` form and Metal/Vulkan's
    /// `found device:` form.
    fn parse_device_line(trimmed: &str) -> Option<(String, String)> {
        if let Some(rest) = trimmed.strip_prefix("Device ") {
            // e.g. "0: NVIDIA GeForce RTX 4090, compute capability 8.9, VMM: yes"
            let after_colon = rest.split_once(':').map(|(_, r)| r).unwrap_or(rest);
            let name = after_colon.split(',').next().unwrap_or("").trim();
            if name.is_empty() {
                return None;
            }
            return Some((name.to_string(), compute_capability(trimmed)));
        }
        if let Some(idx) = trimmed.find("found device:") {
            let name = trimmed[idx + "found device:".len()..].trim();
            if name.is_empty() {
                return None;
            }
            return Some((name.to_string(), compute_capability(trimmed)));
        }
        None
    }

    /// Extract `compute capability X.Y` from a line, if present.
    fn compute_capability(trimmed: &str) -> String {
        const PREFIX: &str = "compute capability ";
        if let Some(start) = trimmed.find(PREFIX) {
            let after = &trimmed[start + PREFIX.len()..];
            return after
                .chars()
                .take_while(|c| !c.is_whitespace() && *c != ',' && *c != ')')
                .collect::<String>();
        }
        String::new()
    }

    /// VRAM in MiB if it appears on the same line as a device ("VRAM: N MiB").
    fn vram_mib_from_line(trimmed: &str) -> Option<u64> {
        trimmed.find("VRAM:").map(|i| {
            let after = &trimmed[i + "VRAM:".len()..];
            parse_leading_u64(after)
        })
    }

    /// First positive `Total VRAM: N` across all lines (Metal/Vulkan).
    fn total_vram_mib(lines: &[&str]) -> Option<u64> {
        lines.iter().find_map(|l| {
            let i = l.find("Total VRAM: ")?;
            let v = parse_leading_u64(&l[i + "Total VRAM: ".len()..]);
            (v > 0).then_some(v)
        })
    }

    fn parse_leading_u64(s: &str) -> u64 {
        s.chars()
            .skip_while(|c| c.is_whitespace())
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse()
            .unwrap_or(0)
    }

    #[cfg(test)]
    mod tests {
        use super::{parse_llama_server_info, GpuDevice};

        fn assert_device(gpu: &GpuDevice, name: &str, vram_mib: u64, cc: &str) {
            assert_eq!(gpu.name, name, "device name");
            assert_eq!(gpu.vram_mib, vram_mib, "vram_mib for {name}");
            assert_eq!(gpu.compute_capability, cc, "compute capability for {name}");
        }

        #[test]
        fn parses_cuda_device_without_vram_in_line() {
            // llama.cpp's real CUDA line has no VRAM token — this is the case
            // that previously produced a "no GPU detected" result.
            let info = parse_llama_server_info(
                "version: 4471 (abc123)\n\
                 built with cc (GCC) 13.2.0 for x86_64-linux-gnu\n\
                   Device 0: NVIDIA GeForce RTX 4090, compute capability 8.9, VMM: yes"
                    .to_string(),
            );
            assert_eq!(info.gpu_devices.len(), 1);
            assert_device(&info.gpu_devices[0], "NVIDIA GeForce RTX 4090", 0, "8.9");
        }

        #[test]
        fn parses_cuda_device_with_vram_in_line() {
            let info = parse_llama_server_info(
                "version: 4471\n\
                 Device 0: NVIDIA GeForce RTX 3090, VRAM: 24564 MiB, compute capability 8.6"
                    .to_string(),
            );
            assert_eq!(info.gpu_devices.len(), 1);
            assert_device(
                &info.gpu_devices[0],
                "NVIDIA GeForce RTX 3090",
                24_564,
                "8.6",
            );
        }

        #[test]
        fn parses_multiple_cuda_devices() {
            let info = parse_llama_server_info(
                "version: 4471\n\
                 Device 0: NVIDIA GeForce RTX 4090, compute capability 8.9, VMM: yes\n\
                 Device 1: NVIDIA GeForce RTX 4090, compute capability 8.9, VMM: yes"
                    .to_string(),
            );
            assert_eq!(info.gpu_devices.len(), 2);
            assert_eq!(info.gpu_devices[0].name, "NVIDIA GeForce RTX 4090");
            assert_eq!(info.gpu_devices[1].name, "NVIDIA GeForce RTX 4090");
        }

        #[test]
        fn parses_metal_found_device_with_total_vram() {
            let info = parse_llama_server_info(
                "version: 4471\n\
                 ggml_metal_init: found device: Apple M4 Max\n\
                 Total VRAM: 131072 MiB"
                    .to_string(),
            );
            assert_eq!(info.gpu_devices.len(), 1);
            assert_device(&info.gpu_devices[0], "Apple M4 Max", 131_072, "");
        }

        #[test]
        fn parses_vulkan_style_found_device() {
            let info = parse_llama_server_info(
                "vulkan_find_device: found device: AMD Radeon RX 7900 XTX\n\
                 Total VRAM: 24576 MiB"
                    .to_string(),
            );
            assert_eq!(info.gpu_devices.len(), 1);
            assert_device(&info.gpu_devices[0], "AMD Radeon RX 7900 XTX", 24_576, "");
        }

        #[test]
        fn cpu_only_yields_no_devices() {
            let info =
                parse_llama_server_info("version: 4471\nbuilt with cc (GCC) 13.2.0\n".to_string());
            assert!(info.gpu_devices.is_empty());
        }

        #[test]
        fn falls_back_to_generic_gpu_when_only_total_vram_present() {
            let info =
                parse_llama_server_info("something\nTotal VRAM: 12288 MiB\ntrailing".to_string());
            assert_eq!(info.gpu_devices.len(), 1);
            assert_device(&info.gpu_devices[0], "GPU", 12_288, "");
        }

        #[test]
        fn does_not_treat_cuda_init_count_as_device() {
            let info = parse_llama_server_info(
                "ggml_cuda_init: found 1 CUDA devices\nDevice 0: NVIDIA A100, compute capability 8.0"
                    .to_string(),
            );
            assert_eq!(info.gpu_devices.len(), 1);
            assert_eq!(info.gpu_devices[0].name, "NVIDIA A100");
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
