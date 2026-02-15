mod commands;
mod dialog;
mod files;
mod image;
mod server;
mod state;

use state::AppStateManager;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

pub fn run() {
    env_logger::init();
    let state_manager = Arc::new(Mutex::new(AppStateManager::new()));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            // Register the cache directory with the asset protocol
            if let Some(cache_dir) = dirs::cache_dir() {
                let podcasterator_cache = cache_dir.join("podcasterator");
                let scope = app.asset_protocol_scope();
                let _ = scope.allow_directory(&podcasterator_cache, true);
            }
            Ok(())
        })
        .manage(state_manager)
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::add_files,
            commands::delete_file,
            commands::rename_file,
            commands::move_file,
            commands::alphabetize,
            commands::reverse,
            commands::clear_all,
            commands::set_artwork,
            commands::delete_artwork,
            commands::set_podcast_name,
            commands::start_server,
            commands::stop_server,
            commands::get_server_url,
            commands::pick_files_or_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
