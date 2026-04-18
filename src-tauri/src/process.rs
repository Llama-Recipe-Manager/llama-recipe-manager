use crate::db::Settings;
use serde::Serialize;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize)]
pub struct ServerStatus {
    pub recipe_id: String,
    pub running: bool,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogLine {
    pub recipe_id: String,
    pub line: String,
    pub is_stderr: bool,
}

struct RunningServer {
    recipe_id: String,
    child: Child,
}

pub struct ProcessManager {
    server: Arc<Mutex<Option<RunningServer>>>,
    logs: Arc<Mutex<Vec<LogLine>>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            server: Arc::new(Mutex::new(None)),
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Build the final command by taking the recipe args and prepending
    /// the llama-server binary path, then appending --host, --port, and -m from settings.
    fn build_command(
        recipe_command: &str,
        model_path: &str,
        mmproj_path: &str,
        settings: &Settings,
    ) -> Result<(String, Vec<String>), String> {
        let recipe_tokens =
            shell_words::split(recipe_command).map_err(|e| format!("Invalid command: {}", e))?;

        // The recipe command should contain only the flags (no program name).
        // But if the user typed "llama-server ..." we strip it.
        let args: Vec<String> = if !recipe_tokens.is_empty()
            && (recipe_tokens[0] == "llama-server"
                || recipe_tokens[0].ends_with("/llama-server"))
        {
            recipe_tokens[1..].to_vec()
        } else {
            recipe_tokens
        };

        let program = expand_tilde(&settings.llama_server_path);

        let mut final_args: Vec<String> = args.iter().map(|a| expand_tilde(a)).collect();

        // Inject model path: if it's absolute or starts with ~, use as-is;
        // otherwise prepend model_dir from settings.
        let resolved_model = if model_path.starts_with('/')
            || model_path.starts_with('~')
            || model_path.starts_with('\\')
        {
            expand_tilde(model_path)
        } else {
            let dir = expand_tilde(&settings.model_dir);
            let dir = dir.trim_end_matches('/');
            format!("{}/{}", dir, model_path)
        };

        final_args.push("-m".to_string());
        final_args.push(resolved_model);

        // Inject mmproj path if provided (for vision models)
        if !mmproj_path.is_empty() {
            let resolved_mmproj = if mmproj_path.starts_with('/')
                || mmproj_path.starts_with('~')
                || mmproj_path.starts_with('\\')
            {
                expand_tilde(mmproj_path)
            } else {
                let dir = expand_tilde(&settings.model_dir);
                let dir = dir.trim_end_matches('/');
                format!("{}/{}", dir, mmproj_path)
            };
            final_args.push("--mmproj".to_string());
            final_args.push(resolved_mmproj);
        }

        // Inject host and port from settings
        final_args.push("--host".to_string());
        final_args.push(settings.host.clone());
        final_args.push("--port".to_string());
        final_args.push(settings.port.to_string());

        Ok((program, final_args))
    }

    pub async fn start_server(
        &self,
        recipe_id: String,
        recipe_command: String,
        model_path: String,
        mmproj_path: String,
        settings: Settings,
        app_handle: tauri::AppHandle,
    ) -> Result<(), String> {
        // Enforce: only one server at a time
        {
            let current = self.server.lock().await;
            if let Some(ref srv) = *current {
                return Err(format!(
                    "A server is already running (recipe: {}). Stop it first.",
                    srv.recipe_id
                ));
            }
        }

        let (program, args) = Self::build_command(&recipe_command, &model_path, &mmproj_path, &settings)?;

        let mut child = Command::new(&program)
            .args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("Failed to start server: {}", e))?;

        // Clear old logs
        {
            let mut logs = self.logs.lock().await;
            logs.clear();
        }

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let logs_clone = self.logs.clone();
        let rid = recipe_id.clone();
        let ah = app_handle.clone();
        if let Some(stdout) = stdout {
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let log = LogLine {
                        recipe_id: rid.clone(),
                        line,
                        is_stderr: false,
                    };
                    {
                        let mut l = logs_clone.lock().await;
                        l.push(log.clone());
                        if l.len() > 2000 {
                            let drain = l.len() - 2000;
                            l.drain(..drain);
                        }
                    }
                    let _ = tauri::Emitter::emit(&ah, "server-log", &log);
                }
            });
        }

        let logs_clone2 = self.logs.clone();
        let rid2 = recipe_id.clone();
        let ah2 = app_handle.clone();
        if let Some(stderr) = stderr {
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let log = LogLine {
                        recipe_id: rid2.clone(),
                        line,
                        is_stderr: true,
                    };
                    {
                        let mut l = logs_clone2.lock().await;
                        l.push(log.clone());
                        if l.len() > 2000 {
                            let drain = l.len() - 2000;
                            l.drain(..drain);
                        }
                    }
                    let _ = tauri::Emitter::emit(&ah2, "server-log", &log);
                }
            });
        }

        let pid = child.id();

        // Store the running server
        {
            let mut srv = self.server.lock().await;
            *srv = Some(RunningServer {
                recipe_id: recipe_id.clone(),
                child,
            });
        }

        // Emit running status
        let _ = tauri::Emitter::emit(
            &app_handle,
            "server-status",
            &ServerStatus {
                recipe_id: recipe_id.clone(),
                running: true,
                pid,
            },
        );

        // Monitor process exit
        let server_clone = self.server.clone();
        let rid3 = recipe_id.clone();
        let ah3 = app_handle.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                let mut srv = server_clone.lock().await;
                if let Some(ref mut s) = *srv {
                    if s.recipe_id != rid3 {
                        break; // different server now
                    }
                    match s.child.try_wait() {
                        Ok(Some(_)) => {
                            *srv = None;
                            let _ = tauri::Emitter::emit(
                                &ah3,
                                "server-status",
                                &ServerStatus {
                                    recipe_id: rid3.clone(),
                                    running: false,
                                    pid: None,
                                },
                            );
                            break;
                        }
                        Ok(None) => {} // still running
                        Err(_) => {
                            *srv = None;
                            let _ = tauri::Emitter::emit(
                                &ah3,
                                "server-status",
                                &ServerStatus {
                                    recipe_id: rid3.clone(),
                                    running: false,
                                    pid: None,
                                },
                            );
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
        });

        Ok(())
    }

    pub async fn stop_server(&self) -> Result<String, String> {
        let mut srv = self.server.lock().await;
        if let Some(mut s) = srv.take() {
            let rid = s.recipe_id.clone();
            s.child
                .kill()
                .await
                .map_err(|e| format!("Failed to stop server: {}", e))?;
            Ok(rid)
        } else {
            Err("No server is currently running".to_string())
        }
    }

    pub async fn get_status(&self) -> Option<ServerStatus> {
        let srv = self.server.lock().await;
        srv.as_ref().map(|s| ServerStatus {
            recipe_id: s.recipe_id.clone(),
            running: true,
            pid: s.child.id(),
        })
    }

    pub async fn get_logs(&self) -> Vec<LogLine> {
        let logs = self.logs.lock().await;
        logs.clone()
    }
}

fn expand_tilde(path: &str) -> String {
    expand_tilde_pub(path)
}

pub fn expand_tilde_pub(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{}{}", home, &path[1..]);
        }
        if let Ok(home) = std::env::var("USERPROFILE") {
            return format!("{}{}", home, &path[1..]);
        }
    }
    path.to_string()
}
