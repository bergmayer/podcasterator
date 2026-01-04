use crate::files::{self, is_audio_file, is_image_file};
use crate::image::process_artwork;
use crate::server::launch_server;
use crate::state::{AppState, AppStateManager};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

type StateManager = Arc<Mutex<AppStateManager>>;

#[tauri::command]
pub async fn get_state(state: tauri::State<'_, StateManager>) -> Result<AppState, String> {
    let manager = state.lock().await;
    Ok(manager.get_state())
}

#[tauri::command]
pub async fn add_files(
    state: tauri::State<'_, StateManager>,
    paths: Vec<String>,
) -> Result<AppState, String> {
    let mut manager = state.lock().await;

    for path in paths {
        let p = Path::new(&path);

        if p.is_dir() {
            files::add_folder(&mut manager, &path)?;
        } else if is_audio_file(p) {
            files::add_file(&mut manager, &path)?;
        } else if is_image_file(p) {
            // Handle as artwork
            let artwork_path = manager.cache_dir().join("artwork.jpg");
            process_artwork(p, &artwork_path)?;
            manager.state.artwork_path = Some(artwork_path.to_string_lossy().to_string());
            manager.save_state()?;
        }
    }

    Ok(manager.get_state())
}

#[tauri::command]
pub async fn delete_file(
    state: tauri::State<'_, StateManager>,
    index: usize,
) -> Result<AppState, String> {
    let mut manager = state.lock().await;
    files::delete_file(&mut manager, index)?;
    Ok(manager.get_state())
}

#[tauri::command]
pub async fn rename_file(
    state: tauri::State<'_, StateManager>,
    index: usize,
    new_name: String,
) -> Result<AppState, String> {
    let mut manager = state.lock().await;
    files::rename_file(&mut manager, index, new_name)?;
    Ok(manager.get_state())
}

#[tauri::command]
pub async fn move_file(
    state: tauri::State<'_, StateManager>,
    index: usize,
    direction: String,
) -> Result<AppState, String> {
    let mut manager = state.lock().await;

    match direction.as_str() {
        "up" => files::move_up(&mut manager, index)?,
        "down" => files::move_down(&mut manager, index)?,
        _ => return Err("Invalid direction".to_string()),
    }

    Ok(manager.get_state())
}

#[tauri::command]
pub async fn alphabetize(state: tauri::State<'_, StateManager>) -> Result<AppState, String> {
    let mut manager = state.lock().await;
    files::alphabetize(&mut manager)?;
    Ok(manager.get_state())
}

#[tauri::command]
pub async fn reverse(state: tauri::State<'_, StateManager>) -> Result<AppState, String> {
    let mut manager = state.lock().await;
    files::reverse(&mut manager)?;
    Ok(manager.get_state())
}

#[tauri::command]
pub async fn clear_all(state: tauri::State<'_, StateManager>) -> Result<AppState, String> {
    let mut manager = state.lock().await;
    files::clear_all(&mut manager)?;
    Ok(manager.get_state())
}

#[tauri::command]
pub async fn set_artwork(
    state: tauri::State<'_, StateManager>,
    path: String,
) -> Result<AppState, String> {
    let mut manager = state.lock().await;

    let source_path = Path::new(&path);
    if !is_image_file(source_path) {
        return Err("Unsupported image format".to_string());
    }

    let artwork_path = manager.cache_dir().join("artwork.jpg");
    process_artwork(source_path, &artwork_path)?;

    manager.state.artwork_path = Some(artwork_path.to_string_lossy().to_string());
    manager.save_state()?;

    Ok(manager.get_state())
}

#[tauri::command]
pub async fn delete_artwork(state: tauri::State<'_, StateManager>) -> Result<AppState, String> {
    let mut manager = state.lock().await;

    if let Some(ref artwork_path) = manager.state.artwork_path {
        let path = PathBuf::from(artwork_path);
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| format!("Failed to delete artwork: {}", e))?;
        }
    }

    manager.state.artwork_path = None;
    manager.save_state()?;

    Ok(manager.get_state())
}

#[tauri::command]
pub async fn set_podcast_name(
    state: tauri::State<'_, StateManager>,
    name: String,
) -> Result<AppState, String> {
    let mut manager = state.lock().await;
    manager.state.podcast_name = name;
    manager.save_state()?;
    Ok(manager.get_state())
}

#[tauri::command]
pub async fn start_server(
    state: tauri::State<'_, StateManager>,
) -> Result<String, String> {
    let mut manager = state.lock().await;

    if manager.server_url.is_some() {
        return Err("Server is already running".to_string());
    }

    let cache_dir = manager.cache_dir().clone();
    let app_state = manager.state.clone();

    let (url, shutdown_tx) = launch_server(app_state, cache_dir).await?;

    manager.server_url = Some(url.clone());
    manager.server_shutdown = Some(shutdown_tx);

    Ok(url)
}

#[tauri::command]
pub async fn stop_server(state: tauri::State<'_, StateManager>) -> Result<(), String> {
    let mut manager = state.lock().await;

    if let Some(shutdown_tx) = manager.server_shutdown.take() {
        shutdown_tx.send(()).ok();
    }

    manager.server_url = None;
    Ok(())
}

#[tauri::command]
pub async fn get_server_url(state: tauri::State<'_, StateManager>) -> Result<Option<String>, ()> {
    let manager = state.lock().await;
    Ok(manager.server_url.clone())
}
