use crate::dialog;
use crate::files;
use crate::server::launch_server;
use crate::state::{AppState, AppStateManager};
use std::path::Path;
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
    let mut errors = Vec::new();

    for path in paths {
        let p = Path::new(&path);

        if p.is_dir() {
            if let Err(e) = files::add_folder(&mut manager, &path).await {
                errors.push(e);
            }
        } else if files::is_audio_file(p) {
            if let Err(e) = files::add_file(&mut manager, &path).await {
                errors.push(e);
            }
        } else if files::is_image_file(p) {
            if let Err(e) = files::add_artwork(&mut manager, &path) {
                errors.push(e);
            }
        }
    }

    // Save once after all files processed
    manager.save_state()?;

    if !errors.is_empty() {
        log::warn!("Some files could not be added: {}", errors.join(", "));
    }

    Ok(manager.get_state())
}

#[tauri::command]
pub async fn delete_file(
    state: tauri::State<'_, StateManager>,
    index: usize,
) -> Result<AppState, String> {
    let mut manager = state.lock().await;
    files::delete_file(&mut manager, index).await?;
    Ok(manager.get_state())
}

#[tauri::command]
pub async fn rename_file(
    state: tauri::State<'_, StateManager>,
    index: usize,
    new_name: String,
) -> Result<AppState, String> {
    let mut manager = state.lock().await;
    files::rename_file(&mut manager, index, new_name).await?;
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
    files::clear_all(&mut manager).await?;
    Ok(manager.get_state())
}

#[tauri::command]
pub async fn set_artwork(
    state: tauri::State<'_, StateManager>,
    path: String,
) -> Result<AppState, String> {
    let mut manager = state.lock().await;
    files::add_artwork(&mut manager, &path)?;
    manager.save_state()?;
    Ok(manager.get_state())
}

#[tauri::command]
pub async fn delete_artwork(state: tauri::State<'_, StateManager>) -> Result<AppState, String> {
    let mut manager = state.lock().await;
    files::delete_artwork(&mut manager).await?;
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

    let (url, shutdown_tx, state_tx) = launch_server(app_state, cache_dir).await?;

    manager.server_url = Some(url.clone());
    manager.server_shutdown = Some(shutdown_tx);
    manager.server_state_tx = Some(state_tx);

    Ok(url)
}

#[tauri::command]
pub async fn stop_server(state: tauri::State<'_, StateManager>) -> Result<(), String> {
    let mut manager = state.lock().await;

    if let Some(shutdown_tx) = manager.server_shutdown.take() {
        if shutdown_tx.send(()).is_err() {
            log::warn!("Server shutdown receiver already dropped");
        }
    }

    manager.server_url = None;
    manager.server_state_tx = None;
    Ok(())
}

#[tauri::command]
pub async fn get_server_url(state: tauri::State<'_, StateManager>) -> Result<Option<String>, ()> {
    let manager = state.lock().await;
    Ok(manager.server_url.clone())
}

#[tauri::command]
pub fn pick_files_or_folder() -> Vec<String> {
    dialog::pick_files_or_folder()
}
