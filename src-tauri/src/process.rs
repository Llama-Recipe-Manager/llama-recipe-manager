use crate::db::Settings;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

/// How long we wait between sending SIGTERM and giving up + sending SIGKILL.
/// Long enough for llama-server to flush slots and free GPU memory, short
/// enough that a hung server doesn't keep the user staring at a frozen UI.
const GRACEFUL_TIMEOUT: Duration = Duration::from_secs(8);

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

/// Emitted exactly once per server run, the moment the child process exits
/// (whether the user asked for it or it crashed on its own).
#[derive(Debug, Clone, Serialize)]
pub struct ServerExit {
    pub recipe_id: String,
    /// Numeric exit code (POSIX & Windows). `None` if the process was killed
    /// by a signal on Unix.
    pub code: Option<i32>,
    /// Unix signal number, when applicable. Always `None` on Windows or for
    /// normal exits.
    pub signal: Option<i32>,
    /// `true` if the user (or the app shutting down) explicitly stopped the
    /// server. `false` means the process exited on its own — i.e. a crash,
    /// OOM kill, bad flag, etc.
    pub intentional: bool,
}

struct RunningServer {
    recipe_id: String,
    child: Child,
    intentional_stop: bool,
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
            && (recipe_tokens[0] == "llama-server" || recipe_tokens[0].ends_with("/llama-server"))
        {
            recipe_tokens[1..].to_vec()
        } else {
            recipe_tokens
        };

        let program = expand_tilde(&settings.llama_server_path);

        let mut final_args: Vec<String> = args.iter().map(|a| expand_tilde(a)).collect();

        let resolved_model = resolve_model_path("Model path", model_path, &settings.model_dir)?;
        final_args.push("-m".to_string());
        final_args.push(resolved_model);

        if !mmproj_path.is_empty() {
            let resolved_mmproj =
                resolve_model_path("Mmproj path", mmproj_path, &settings.model_dir)?;
            final_args.push("--mmproj".to_string());
            final_args.push(resolved_mmproj);
        }

        // Inject host and port from settings
        final_args.push("--host".to_string());
        final_args.push(settings.host.clone());
        final_args.push("--port".to_string());
        final_args.push(settings.port.to_string());

        // Security
        if !settings.api_key.is_empty() {
            final_args.push("--api-key".to_string());
            final_args.push(settings.api_key.clone());
        }
        if !settings.ssl_cert_file.is_empty() && !settings.ssl_key_file.is_empty() {
            final_args.push("--ssl-cert-file".to_string());
            final_args.push(expand_tilde(&settings.ssl_cert_file));
            final_args.push("--ssl-key-file".to_string());
            final_args.push(expand_tilde(&settings.ssl_key_file));
        }

        // Server behavior
        if !settings.webui_enabled {
            final_args.push("--no-webui".to_string());
        }
        if settings.metrics_enabled {
            final_args.push("--metrics".to_string());
        }
        if !settings.slots_enabled {
            final_args.push("--no-slots".to_string());
        }
        if !settings.api_prefix.is_empty() {
            final_args.push("--api-prefix".to_string());
            final_args.push(settings.api_prefix.clone());
        }
        if settings.timeout_secs != 600 {
            final_args.push("--timeout".to_string());
            final_args.push(settings.timeout_secs.to_string());
        }

        // Diagnostics
        if settings.log_verbosity != 3 {
            final_args.push("--log-verbosity".to_string());
            final_args.push(settings.log_verbosity.to_string());
        }

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

        let (program, args) =
            Self::build_command(&recipe_command, &model_path, &mmproj_path, &settings)?;

        let mut cmd = Command::new(&program);
        cmd.args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        // Windows: spawn into its own process group so we can later send
        // CTRL_BREAK_EVENT to it for a graceful shutdown. Without this flag
        // the child shares our console group (we have none) and would only
        // be killable via TerminateProcess.
        //
        // CREATE_NO_WINDOW tells Windows not to create a console window for
        // this child process — without it, spawning a console-subsystem
        // binary (like llama-server) from a GUI app causes a brief CMD
        // window to flash on screen.
        #[cfg(windows)]
        {
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            cmd.creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);
        }

        if !settings.hf_token.is_empty() {
            cmd.env("HF_TOKEN", &settings.hf_token);
        }

        let mut child = cmd
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
                intentional_stop: false,
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

        // Monitor process exit. Polling `try_wait` (rather than calling
        // `wait()` directly) lets us keep `Child` accessible to `stop_server`
        // for `kill()` without juggling ownership.
        let server_clone = self.server.clone();
        let rid3 = recipe_id.clone();
        let ah3 = app_handle.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                let mut srv = server_clone.lock().await;
                let Some(s) = srv.as_mut() else { break };
                if s.recipe_id != rid3 {
                    break; // a newer server has taken over the slot
                }

                match s.child.try_wait() {
                    Ok(Some(status)) => {
                        let intentional = s.intentional_stop;
                        let exit = exit_event(&rid3, status, intentional);
                        *srv = None;
                        let _ = tauri::Emitter::emit(&ah3, "server-exit", &exit);
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
                        let intentional = s.intentional_stop;
                        *srv = None;
                        let _ = tauri::Emitter::emit(
                            &ah3,
                            "server-exit",
                            &ServerExit {
                                recipe_id: rid3.clone(),
                                code: None,
                                signal: None,
                                intentional,
                            },
                        );
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
            }
        });

        Ok(())
    }

    /// Ask the running server to shut down gracefully.
    ///
    /// We:
    ///   1. Mark the run as intentional so the watcher emits the right exit event.
    ///   2. Send SIGTERM (Unix) / Ctrl-C-style request, giving llama-server a
    ///      chance to flush slots, drop GPU memory, and tear down the HTTP
    ///      listener cleanly.
    ///   3. Spawn a background task that escalates to SIGKILL after
    ///      `GRACEFUL_TIMEOUT` if the process is still alive.
    ///
    /// We deliberately leave the `RunningServer` in the slot — the existing
    /// watcher task is what observes the exit, sees `intentional_stop`, and
    /// emits the `server-exit` / `server-status` events.
    pub async fn stop_server(&self) -> Result<String, String> {
        let (rid, pid_opt) = {
            let mut srv = self.server.lock().await;
            let Some(s) = srv.as_mut() else {
                return Err("No server is currently running".to_string());
            };
            s.intentional_stop = true;
            (s.recipe_id.clone(), s.child.id())
        };

        // Polite "please shut down" signal. On Unix this is SIGTERM, which
        // llama-server traps and uses to drain in-flight requests. On Windows
        // there's no equivalent for a windowless console child without
        // CREATE_NEW_PROCESS_GROUP plumbing, so the escalation path below is
        // what actually stops it there — still wrapped in the same timeout.
        send_graceful_signal(pid_opt);

        // Escalate to SIGKILL / TerminateProcess after the grace window.
        let server_clone = self.server.clone();
        let rid_clone = rid.clone();
        tokio::spawn(async move {
            tokio::time::sleep(GRACEFUL_TIMEOUT).await;
            let mut srv = server_clone.lock().await;
            if let Some(s) = srv.as_mut() {
                if s.recipe_id == rid_clone {
                    // Watcher hasn't reaped this process yet — escalate.
                    let _ = s.child.kill().await;
                }
            }
        });

        Ok(rid)
    }

    /// Like {@link stop_server} but blocks until the child has actually
    /// exited (or `GRACEFUL_TIMEOUT` elapses, after which we SIGKILL it).
    /// Used on app shutdown where we can't return early — otherwise
    /// `kill_on_drop` would SIGKILL the moment the runtime tears down,
    /// negating the graceful signal we just sent.
    pub async fn stop_server_blocking(&self) -> Result<(), String> {
        let (rid, pid_opt) = {
            let mut srv = self.server.lock().await;
            let Some(s) = srv.as_mut() else { return Ok(()) };
            s.intentional_stop = true;
            (s.recipe_id.clone(), s.child.id())
        };

        send_graceful_signal(pid_opt);

        // Poll until either the watcher reaps the child or we hit the
        // timeout, in which case we forcefully kill it.
        let deadline = tokio::time::Instant::now() + GRACEFUL_TIMEOUT;
        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let mut srv = self.server.lock().await;
            let Some(s) = srv.as_mut() else { return Ok(()) };
            if s.recipe_id != rid {
                return Ok(());
            }
            if let Ok(Some(_)) = s.child.try_wait() {
                *srv = None;
                return Ok(());
            }
            if tokio::time::Instant::now() >= deadline {
                let _ = s.child.kill().await;
                let _ = s.child.wait().await;
                *srv = None;
                return Ok(());
            }
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

    pub async fn clear_logs(&self) {
        let mut logs = self.logs.lock().await;
        logs.clear();
    }
}

/// Best-effort polite shutdown signal. Errors are intentionally swallowed —
/// the watcher + escalation timer guarantee we eventually reap the child
/// even if the signal can't be delivered (e.g. process already exited).
#[cfg(unix)]
fn send_graceful_signal(pid: Option<u32>) {
    if let Some(pid) = pid {
        // Safety: `kill(2)` with SIGTERM is signal-safe and side-effect-free
        // on a non-existent pid (returns ESRCH which we ignore).
        unsafe {
            libc::kill(pid.cast_signed(), libc::SIGTERM);
        }
    }
}

/// Windows: send Ctrl+Break to the child's process group. We spawned the
/// child with CREATE_NEW_PROCESS_GROUP precisely so this works — its PID is
/// also its process-group id. CTRL_BREAK_EVENT is the only console control
/// event that can be addressed at a specific group from a callerless of
/// whether the caller has a console attached, which fits a Tauri GUI app.
///
/// llama-server installs a handler for it that triggers an orderly shutdown
/// (drain slots, close sockets, free GPU memory) just like SIGTERM on Unix.
#[cfg(windows)]
fn send_graceful_signal(pid: Option<u32>) {
    use windows_sys::Win32::System::Console::{GenerateConsoleCtrlEvent, CTRL_BREAK_EVENT};
    if let Some(pid) = pid {
        // Safety: GenerateConsoleCtrlEvent is safe to call from any thread,
        // returns 0 on failure (which we ignore — escalation timer handles it).
        unsafe {
            GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, pid);
        }
    }
}

fn exit_event(recipe_id: &str, status: std::process::ExitStatus, intentional: bool) -> ServerExit {
    ServerExit {
        recipe_id: recipe_id.to_string(),
        code: status.code(),
        signal: unix_signal(status),
        intentional,
    }
}

#[cfg(unix)]
fn unix_signal(status: std::process::ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    status.signal()
}

#[cfg(not(unix))]
fn unix_signal(_status: std::process::ExitStatus) -> Option<i32> {
    None
}

fn expand_tilde(path: &str) -> String {
    expand_tilde_pub(path)
}

/// Augment the process `PATH` with the install locations that GUI-launched
/// apps don't see by default.
///
/// On macOS, apps started from Finder / Dock / Spotlight inherit
/// `launchd`'s minimal `PATH` (`/usr/bin:/bin:/usr/sbin:/sbin`) — the user's
/// shell `rc` files never run, so `/opt/homebrew/bin`, `/usr/local/bin`,
/// `~/.cargo/bin`, etc. are invisible. This breaks any plain
/// `Command::new("llama-server")` lookup even when `which llama-server`
/// works fine in the terminal. Linux .desktop launches have a similar
/// (though less aggressive) version of the same problem.
///
/// We prepend a curated set of standard install dirs that exist on disk,
/// keeping the original `PATH` afterwards so anything the user *did* set
/// still wins for ties. Called once at app startup; subsequent
/// `Command::new` calls and child processes inherit the wider lookup.
pub fn augment_gui_path() {
    // Order matters: earlier entries are searched first. We list the most
    // likely-to-contain-`llama-server` locations first so resolution is
    // both correct and fast.
    let candidates: &[&str] = &[
        // Apple Silicon Homebrew.
        "/opt/homebrew/bin",
        "/opt/homebrew/sbin",
        // Intel Homebrew + most "manual install" prefixes.
        "/usr/local/bin",
        "/usr/local/sbin",
        // MacPorts.
        "/opt/local/bin",
        "/opt/local/sbin",
        // Per-user Linuxbrew.
        "/home/linuxbrew/.linuxbrew/bin",
        // Conda / pixi default user prefix.
        "/opt/miniconda3/bin",
    ];

    let home = std::env::var("HOME").ok();
    let home_dirs: Vec<String> = home
        .iter()
        .flat_map(|h| {
            [
                format!("{h}/.cargo/bin"),
                format!("{h}/.local/bin"),
                format!("{h}/bin"),
                format!("{h}/.linuxbrew/bin"),
            ]
        })
        .collect();

    let existing = std::env::var_os("PATH").unwrap_or_default();
    let separator = if cfg!(windows) { ';' } else { ':' };

    let mut already: std::collections::HashSet<std::path::PathBuf> =
        std::env::split_paths(&existing).collect();

    let mut new_entries: Vec<std::path::PathBuf> = Vec::new();
    let push_if_new = |p: std::path::PathBuf,
                       already: &mut std::collections::HashSet<std::path::PathBuf>,
                       out: &mut Vec<std::path::PathBuf>| {
        if p.is_dir() && already.insert(p.clone()) {
            out.push(p);
        }
    };

    for c in candidates {
        push_if_new((*c).into(), &mut already, &mut new_entries);
    }
    for c in &home_dirs {
        push_if_new(c.into(), &mut already, &mut new_entries);
    }

    if new_entries.is_empty() {
        return;
    }

    // Prepend new entries so freshly-discovered locations take precedence
    // only over launchd's empty default — anything the user set in their
    // current `PATH` is preserved verbatim after them.
    let mut combined = String::new();
    for (i, p) in new_entries.iter().enumerate() {
        if i > 0 {
            combined.push(separator);
        }
        combined.push_str(&p.to_string_lossy());
    }
    if !existing.is_empty() {
        combined.push(separator);
        combined.push_str(&existing.to_string_lossy());
    }
    // Safety: we're in the synchronous setup path before any worker
    // threads spawn, so racing readers are a non-issue.
    unsafe {
        std::env::set_var("PATH", combined);
    }
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

/// Resolve a recipe-supplied model/mmproj path against `model_dir`.
///
/// Behaviour:
///   * Absolute paths and `~/...` paths are honoured verbatim (these only
///     reach this code when the user explicitly chose the file via the file
///     picker, so we trust them).
///   * Relative paths are joined under `model_dir`. Both sides are canonicalised
///     (when they exist on disk) and the result must remain inside `model_dir`,
///     otherwise we refuse to launch. This blocks `../../etc/...` style
///     escapes that an imported community recipe could attempt.
fn resolve_model_path(field: &str, raw: &str, model_dir: &str) -> Result<String, String> {
    if raw.contains('\0') || raw.contains('\n') || raw.contains('\r') {
        return Err(format!("{} contains invalid characters", field));
    }

    let is_absolute = raw.starts_with('/') || raw.starts_with('~') || raw.starts_with('\\');
    if is_absolute {
        return Ok(expand_tilde(raw));
    }

    let dir = PathBuf::from(expand_tilde(model_dir));
    let joined = dir.join(raw);

    let canon_dir = canonicalize_or(&dir);
    let canon_joined = canonicalize_or(&joined);

    if !canon_joined.starts_with(&canon_dir) {
        return Err(format!(
            "{} '{}' resolves outside the configured model directory",
            field, raw
        ));
    }

    Ok(joined.to_string_lossy().into_owned())
}

/// Best-effort canonicalisation: if the file/dir does not yet exist, fall back
/// to a lexically normalised path so containment checks still work for a model
/// the user has not downloaded yet.
fn canonicalize_or(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| lexical_normalize(path))
}

fn lexical_normalize(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}
