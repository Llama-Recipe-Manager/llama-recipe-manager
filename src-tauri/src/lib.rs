//! Llama Recipe Manager — Tauri application entry point.
//!
//! This file is intentionally thin: it wires modules together, registers
//! plugins, sets up state, and routes invoke handlers. All business logic
//! lives in the dedicated modules below.

mod commands;
pub mod db;
mod process;
mod state;
pub mod validate;

use db::Database;
use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // GUI launches on macOS / Linux inherit a stripped-down PATH that
    // doesn't include Homebrew, cargo, or ~/.local/bin — so a user who
    // can run `llama-server` in their terminal would otherwise get
    // "command not found" inside the app. Widen PATH up-front so every
    // child process (version probe + the actual server spawn) sees the
    // standard install locations.
    process::augment_gui_path();

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init());

    // Updater + process (relaunch) plugins are desktop-only; mobile platforms
    // expect updates via their respective app stores.
    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_updater::Builder::new().build())
            .plugin(tauri_plugin_process::init());
    }

    let app = builder
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            let db = Database::new(app_dir).expect("failed to initialize database");
            app.manage(AppState::new(db));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::recipes::list_recipes,
            commands::recipes::get_recipe,
            commands::recipes::create_recipe,
            commands::recipes::update_recipe,
            commands::recipes::delete_recipe,
            commands::recipes::duplicate_recipe,
            commands::server::get_llama_server_info,
            commands::server::start_server,
            commands::server::stop_server,
            commands::server::get_server_status,
            commands::server::get_server_logs,
            commands::server::clear_server_logs,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    // Tear down any running llama-server when the user quits the app, unless
    // they explicitly opted into leaving it running in the background.
    app.run(|app_handle, event| {
        if let tauri::RunEvent::ExitRequested { .. } = event {
            let state = app_handle.state::<AppState>();
            let keep = state
                .db
                .lock()
                .ok()
                .and_then(|db| db.get_settings().ok())
                .is_some_and(|s| s.keep_server_on_exit);
            if !keep {
                // Block here so llama-server gets the full graceful window
                // before tokio's `kill_on_drop` would otherwise SIGKILL it.
                tauri::async_runtime::block_on(async {
                    let _ = state.pm.stop_server_blocking().await;
                });
            }
        }
    });
}
