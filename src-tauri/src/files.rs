use crate::state::{AppStateManager, AudioFile};
use chrono::Utc;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use tokio::fs;
use uuid::Uuid;
use walkdir::WalkDir;

const SUPPORTED_AUDIO_EXTENSIONS: &[&str] = &["mp3", "m4a", "mp4", "m4b"];
const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "bmp", "tiff"];
pub const ARTWORK_FILENAME: &str = "artwork.jpg";

fn get_extension_lower(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
}

pub fn is_audio_file(path: &Path) -> bool {
    get_extension_lower(path)
        .map(|e| SUPPORTED_AUDIO_EXTENSIONS.contains(&e.as_str()))
        .unwrap_or(false)
}

pub fn is_image_file(path: &Path) -> bool {
    get_extension_lower(path)
        .map(|e| SUPPORTED_IMAGE_EXTENSIONS.contains(&e.as_str()))
        .unwrap_or(false)
}

/// Normalize filename extension (mp4/m4b -> m4a)
fn normalize_extension(filename: &str) -> String {
    let path = Path::new(filename);
    let stem = match path.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => return filename.to_string(),
    };
    match get_extension_lower(path).as_deref() {
        Some("mp4") | Some("m4b") => format!("{}.m4a", stem),
        _ => filename.to_string(),
    }
}

/// Remove a temp file and its parent UUID directory
async fn cleanup_temp_file(temp_path: &Path) {
    if temp_path.exists() {
        if let Err(e) = fs::remove_file(temp_path).await {
            log::warn!("Failed to remove temp file {}: {}", temp_path.display(), e);
        }
    }
    if let Some(parent) = temp_path.parent() {
        // remove_dir only succeeds if empty, which is what we want
        let _ = fs::remove_dir(parent).await;
    }
}

fn validate_filename(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Filename cannot be empty".to_string());
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("Filename contains invalid characters".to_string());
    }
    Ok(())
}

/// Compare names case-insensitively, treating each run of ASCII digits as a
/// number. This keeps "Episode 2" before "Episode 10" without requiring
/// filenames to zero-pad their episode numbers.
fn natural_name_cmp(a: &str, b: &str) -> Ordering {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();
    let a_bytes = a_lower.as_bytes();
    let b_bytes = b_lower.as_bytes();
    let mut a_index = 0;
    let mut b_index = 0;

    while a_index < a_bytes.len() && b_index < b_bytes.len() {
        if a_bytes[a_index].is_ascii_digit() && b_bytes[b_index].is_ascii_digit() {
            let a_start = a_index;
            let b_start = b_index;

            while a_index < a_bytes.len() && a_bytes[a_index].is_ascii_digit() {
                a_index += 1;
            }
            while b_index < b_bytes.len() && b_bytes[b_index].is_ascii_digit() {
                b_index += 1;
            }

            let a_digits = &a_bytes[a_start..a_index];
            let b_digits = &b_bytes[b_start..b_index];
            let a_significant = &a_digits[a_digits
                .iter()
                .position(|digit| *digit != b'0')
                .unwrap_or(a_digits.len())..];
            let b_significant = &b_digits[b_digits
                .iter()
                .position(|digit| *digit != b'0')
                .unwrap_or(b_digits.len())..];

            match a_significant.len().cmp(&b_significant.len()) {
                Ordering::Equal => {}
                ordering => return ordering,
            }
            match a_significant.cmp(b_significant) {
                Ordering::Equal => {}
                ordering => return ordering,
            }
        } else {
            match a_bytes[a_index].cmp(&b_bytes[b_index]) {
                Ordering::Equal => {
                    a_index += 1;
                    b_index += 1;
                }
                ordering => return ordering,
            }
        }
    }

    a_bytes
        .len()
        .cmp(&b_bytes.len())
        .then_with(|| a_lower.cmp(&b_lower))
        .then_with(|| a.cmp(b))
}

fn path_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}

/// Normalize a file-picker or drag/drop batch because native APIs do not
/// guarantee the order in which selected paths are returned.
pub fn sort_paths_naturally(paths: &mut [String]) {
    paths.sort_by(|a, b| natural_name_cmp(&path_name(Path::new(a)), &path_name(Path::new(b))));
}

/// Add a single audio file. Does NOT call save_state — caller is responsible.
pub async fn add_file(manager: &mut AppStateManager, path: &str) -> Result<(), String> {
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

    let uuid_dir = manager.cache_dir().join(&id);
    fs::create_dir_all(&uuid_dir)
        .await
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;

    let temp_path = uuid_dir.join(&display_name);

    fs::copy(&source_path, &temp_path)
        .await
        .map_err(|e| format!("Failed to copy file: {}", e))?;

    let audio_file = AudioFile {
        id,
        original_path: path.to_string(),
        temp_path: temp_path.to_string_lossy().to_string(),
        display_name,
        added_at: Utc::now().to_rfc2822(),
    };

    manager.state.files.push(audio_file);

    Ok(())
}

/// Add all audio files from a folder. Does NOT call save_state — caller is responsible.
pub async fn add_folder(manager: &mut AppStateManager, path: &str) -> Result<(), String> {
    let folder_path = PathBuf::from(path);

    if !folder_path.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }

    let mut errors = Vec::new();

    // Collect paths first (WalkDir is sync, which is fine for directory listing)
    let mut paths: Vec<PathBuf> = WalkDir::new(&folder_path)
        .max_depth(1)
        .into_iter()
        .filter_map(|entry| match entry {
            Ok(e) => {
                let p = e.into_path();
                if p.is_file() && is_audio_file(&p) {
                    Some(p)
                } else {
                    None
                }
            }
            Err(e) => {
                errors.push(format!("Error traversing directory: {}", e));
                None
            }
        })
        .collect();

    paths.sort_by(|a, b| natural_name_cmp(&path_name(a), &path_name(b)));

    for p in paths {
        if let Some(path_str) = p.to_str() {
            if let Err(e) = add_file(manager, path_str).await {
                errors.push(e);
            }
        } else {
            errors.push(format!("Invalid UTF-8 path: {}", p.display()));
        }
    }

    if !errors.is_empty() {
        log::warn!("Some files could not be added:\n{}", errors.join("\n"));
    }

    Ok(())
}

/// Set artwork from an image file. Does NOT call save_state — caller is responsible.
pub async fn add_artwork(manager: &mut AppStateManager, path: &str) -> Result<(), String> {
    let source_path = PathBuf::from(path);
    if !is_image_file(&source_path) {
        return Err("Unsupported image format".to_string());
    }

    // Use a timestamped filename to avoid webview caching
    let filename = format!("artwork_{}.jpg", chrono::Utc::now().timestamp_millis());
    let artwork_path = manager.cache_dir().join(filename);

    // Image decode/encode is CPU-bound; keep it off the async runtime
    let dst = artwork_path.clone();
    tokio::task::spawn_blocking(move || crate::image::process_artwork(&source_path, &dst))
        .await
        .map_err(|e| format!("Artwork task failed: {}", e))??;

    // Delete old artwork file only after the new one was written successfully
    if let Some(ref old_path) = manager.state.artwork_path {
        let old = PathBuf::from(old_path);
        if old.exists() {
            let _ = std::fs::remove_file(&old);
        }
    }

    manager.state.artwork_path = Some(artwork_path.to_string_lossy().to_string());
    Ok(())
}

/// Delete artwork file and clear from state. Does NOT call save_state.
pub async fn delete_artwork(manager: &mut AppStateManager) -> Result<(), String> {
    if let Some(ref artwork_path) = manager.state.artwork_path {
        let path = PathBuf::from(artwork_path);
        if path.exists() {
            fs::remove_file(&path)
                .await
                .map_err(|e| format!("Failed to delete artwork: {}", e))?;
        }
    }
    manager.state.artwork_path = None;
    Ok(())
}

pub fn file_index(manager: &AppStateManager, id: &str) -> Result<usize, String> {
    manager
        .state
        .files
        .iter()
        .position(|file| file.id == id)
        .ok_or_else(|| "File not found".to_string())
}

pub async fn delete_file(manager: &mut AppStateManager, id: &str) -> Result<(), String> {
    let index = file_index(manager, id)?;

    let temp_path = PathBuf::from(&manager.state.files[index].temp_path);
    cleanup_temp_file(&temp_path).await;

    manager.state.files.remove(index);
    manager.save_state()?;

    Ok(())
}

pub async fn rename_file(
    manager: &mut AppStateManager,
    id: &str,
    new_name: String,
) -> Result<(), String> {
    let index = file_index(manager, id)?;

    validate_filename(&new_name)?;

    let file = &manager.state.files[index];

    let extension = Path::new(&file.display_name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    // Keep the original audio extension unless the new name already ends
    // with it — names like "Part 2.5" must not turn ".5" into the extension.
    let already_has_extension = Path::new(&new_name)
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case(extension));

    let new_display_name = if extension.is_empty() || already_has_extension {
        new_name
    } else {
        format!("{}.{}", new_name, extension)
    };

    let old_temp_path = PathBuf::from(&file.temp_path);
    let new_temp_path = old_temp_path.with_file_name(&new_display_name);

    // Perform filesystem rename first, only update state on success
    if old_temp_path.exists() {
        fs::rename(&old_temp_path, &new_temp_path)
            .await
            .map_err(|e| format!("Failed to rename file: {}", e))?;
    }

    manager.state.files[index].display_name = new_display_name;
    manager.state.files[index].temp_path = new_temp_path.to_string_lossy().to_string();
    manager.save_state()?;

    Ok(())
}

pub fn move_up(manager: &mut AppStateManager, index: usize) -> Result<(), String> {
    if index == 0 || index >= manager.state.files.len() {
        return Ok(());
    }

    manager.state.files.swap(index, index - 1);
    manager.save_state()?;

    Ok(())
}

pub fn move_down(manager: &mut AppStateManager, index: usize) -> Result<(), String> {
    if manager.state.files.is_empty() || index >= manager.state.files.len() - 1 {
        return Ok(());
    }

    manager.state.files.swap(index, index + 1);
    manager.save_state()?;

    Ok(())
}

pub fn alphabetize(manager: &mut AppStateManager) -> Result<(), String> {
    manager
        .state
        .files
        .sort_by(|a, b| natural_name_cmp(&a.display_name, &b.display_name));
    manager.save_state()?;

    Ok(())
}

pub fn reverse(manager: &mut AppStateManager) -> Result<(), String> {
    manager.state.files.reverse();
    manager.save_state()?;

    Ok(())
}

pub async fn clear_all(manager: &mut AppStateManager) -> Result<(), String> {
    // Clean up all temp files, collecting errors but not stopping
    let mut cleanup_errors = Vec::new();
    for file in &manager.state.files {
        let temp_path = PathBuf::from(&file.temp_path);
        if temp_path.exists() {
            if let Err(e) = fs::remove_file(&temp_path).await {
                cleanup_errors.push(format!("{}: {}", file.display_name, e));
            }
        }
        if let Some(parent) = temp_path.parent() {
            let _ = fs::remove_dir(parent).await;
        }
    }

    // Always clear file list, even if some temp file deletions failed
    manager.state.files.clear();
    manager.save_state()?;

    if !cleanup_errors.is_empty() {
        log::warn!(
            "Some temp files could not be deleted:\n{}",
            cleanup_errors.join("\n")
        );
    }

    Ok(())
}

/// Get MIME type for served files
pub fn get_mime_type(path: &Path) -> &'static str {
    match get_extension_lower(path).as_deref() {
        Some("mp3") => "audio/mpeg",
        Some("m4a") | Some("mp4") | Some("m4b") => "audio/mp4",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::{natural_name_cmp, sort_paths_naturally};

    #[test]
    fn natural_name_sort_orders_numbered_episodes() {
        let mut names = vec![
            "The Beatles Anthology Revisited Episode 10.mp3",
            "The Beatles Anthology Revisited Episode 2.mp3",
            "The Beatles Anthology Revisited Episode 01.mp3",
            "The Beatles Anthology Revisited Episode 18.mp3",
        ];

        names.sort_by(|a, b| natural_name_cmp(a, b));

        assert_eq!(
            names,
            vec![
                "The Beatles Anthology Revisited Episode 01.mp3",
                "The Beatles Anthology Revisited Episode 2.mp3",
                "The Beatles Anthology Revisited Episode 10.mp3",
                "The Beatles Anthology Revisited Episode 18.mp3",
            ]
        );
    }

    #[test]
    fn natural_name_sort_is_case_insensitive_and_deterministic() {
        let mut names = vec!["episode 2.mp3", "Episode 1.mp3", "EPISODE 2.mp3"];

        names.sort_by(|a, b| natural_name_cmp(a, b));

        assert_eq!(
            names,
            vec!["Episode 1.mp3", "EPISODE 2.mp3", "episode 2.mp3"]
        );
    }

    #[test]
    fn selected_paths_are_naturally_sorted_by_filename() {
        let mut paths = vec![
            "/podcast/The Beatles Episode 10.mp3".to_string(),
            "/podcast/The Beatles Episode 2.mp3".to_string(),
            "/podcast/The Beatles Episode 01.mp3".to_string(),
        ];

        sort_paths_naturally(&mut paths);

        assert_eq!(
            paths,
            vec![
                "/podcast/The Beatles Episode 01.mp3",
                "/podcast/The Beatles Episode 2.mp3",
                "/podcast/The Beatles Episode 10.mp3",
            ]
        );
    }
}
