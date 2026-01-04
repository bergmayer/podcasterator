use crate::state::{AppStateManager, AudioFile};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use walkdir::WalkDir;

const SUPPORTED_AUDIO_EXTENSIONS: &[&str] = &["mp3", "m4a", "mp4", "m4b"];
const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "bmp", "tiff"];

pub fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| SUPPORTED_AUDIO_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| SUPPORTED_IMAGE_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Normalize filename extension (mp4/m4b -> m4a)
fn normalize_extension(filename: &str) -> String {
    let path = Path::new(filename);
    let stem = match path.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => return filename.to_string(),
    };
    let ext = match path.extension().and_then(|s| s.to_str()) {
        Some(s) => s.to_lowercase(),
        None => return filename.to_string(),
    };

    match ext.as_str() {
        "mp4" | "m4b" => format!("{}.m4a", stem),
        _ => filename.to_string(),
    }
}

pub fn add_file(manager: &mut AppStateManager, path: &str) -> Result<(), String> {
    let source_path = PathBuf::from(path);

    if !source_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    if !is_audio_file(&source_path) {
        return Err(format!("Unsupported audio format: {}", path));
    }

    let id = Uuid::new_v4().to_string();
    let original_name = source_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let display_name = normalize_extension(&original_name);

    // Create UUID directory in cache
    let uuid_dir = manager.cache_dir().join(&id);
    fs::create_dir_all(&uuid_dir).map_err(|e| format!("Failed to create temp directory: {}", e))?;

    let temp_path = uuid_dir.join(&display_name);

    // Copy file to temp location
    fs::copy(&source_path, &temp_path).map_err(|e| format!("Failed to copy file: {}", e))?;

    let audio_file = AudioFile {
        id,
        original_path: path.to_string(),
        temp_path: temp_path.to_string_lossy().to_string(),
        display_name,
    };

    manager.state.files.push(audio_file);
    manager.save_state()?;

    Ok(())
}

pub fn add_folder(manager: &mut AppStateManager, path: &str) -> Result<(), String> {
    let folder_path = PathBuf::from(path);

    if !folder_path.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }

    let mut errors = Vec::new();

    for entry in WalkDir::new(&folder_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let entry_path = entry.path();
        if entry_path.is_file() && is_audio_file(entry_path) {
            if let Some(path_str) = entry_path.to_str() {
                if let Err(e) = add_file(manager, path_str) {
                    errors.push(e);
                }
            }
        }
    }

    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }

    Ok(())
}

pub fn delete_file(manager: &mut AppStateManager, index: usize) -> Result<(), String> {
    if index >= manager.state.files.len() {
        return Err("Index out of bounds".to_string());
    }

    let file = &manager.state.files[index];
    let temp_path = PathBuf::from(&file.temp_path);

    // Remove the temp file and its parent UUID directory
    if temp_path.exists() {
        fs::remove_file(&temp_path)
            .map_err(|e| format!("Failed to delete temp file: {}", e))?;
    }
    if let Some(parent) = temp_path.parent() {
        fs::remove_dir(parent).map_err(|e| format!("Failed to delete temp directory: {}", e))?;
    }

    manager.state.files.remove(index);
    manager.save_state()?;

    Ok(())
}

pub fn rename_file(
    manager: &mut AppStateManager,
    index: usize,
    new_name: String,
) -> Result<(), String> {
    if index >= manager.state.files.len() {
        return Err("Index out of bounds".to_string());
    }

    let file = &manager.state.files[index];

    let extension = Path::new(&file.display_name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    let new_display_name = if Path::new(&new_name).extension().is_none() {
        format!("{}.{}", new_name, extension)
    } else {
        new_name
    };

    let old_temp_path = PathBuf::from(&file.temp_path);
    let new_temp_path = old_temp_path.with_file_name(&new_display_name);

    if old_temp_path.exists() {
        fs::rename(&old_temp_path, &new_temp_path)
            .map_err(|e| format!("Failed to rename file: {}", e))?;
    }

    manager.state.files[index].display_name = new_display_name;
    manager.state.files[index].temp_path = new_temp_path.to_string_lossy().to_string();
    manager.save_state()?;

    Ok(())
}

pub fn move_up(manager: &mut AppStateManager, index: usize) -> Result<(), String> {
    if index == 0 || index >= manager.state.files.len() {
        return Ok(()); // Nothing to do
    }

    manager.state.files.swap(index, index - 1);
    manager.save_state()?;

    Ok(())
}

pub fn move_down(manager: &mut AppStateManager, index: usize) -> Result<(), String> {
    if index >= manager.state.files.len() - 1 {
        return Ok(()); // Nothing to do
    }

    manager.state.files.swap(index, index + 1);
    manager.save_state()?;

    Ok(())
}

pub fn alphabetize(manager: &mut AppStateManager) -> Result<(), String> {
    manager
        .state
        .files
        .sort_by(|a, b| a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase()));
    manager.save_state()?;

    Ok(())
}

pub fn reverse(manager: &mut AppStateManager) -> Result<(), String> {
    manager.state.files.reverse();
    manager.save_state()?;

    Ok(())
}

pub fn clear_all(manager: &mut AppStateManager) -> Result<(), String> {
    // Remove all temp files and directories
    for file in &manager.state.files {
        let temp_path = PathBuf::from(&file.temp_path);
        if temp_path.exists() {
            fs::remove_file(&temp_path)
                .map_err(|e| format!("Failed to delete temp file: {}", e))?;
        }
        if let Some(parent) = temp_path.parent() {
            fs::remove_dir(parent).map_err(|e| format!("Failed to delete temp directory: {}", e))?;
        }
    }

    manager.state.files.clear();
    manager.save_state()?;

    Ok(())
}

/// Get MIME type for audio file
pub fn get_mime_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .as_deref()
    {
        Some("mp3") => "audio/mpeg",
        Some("m4a") | Some("mp4") | Some("m4b") => "audio/mp4",
        _ => "application/octet-stream",
    }
}
