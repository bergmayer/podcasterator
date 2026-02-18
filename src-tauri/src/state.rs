use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFile {
    pub id: String,
    pub original_path: String,
    pub temp_path: String,
    pub display_name: String,
    #[serde(default)]
    pub added_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppState {
    pub files: Vec<AudioFile>,
    pub podcast_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artwork_path: Option<String>,
}

pub struct AppStateManager {
    pub state: AppState,
    pub server_url: Option<String>,
    pub server_shutdown: Option<tokio::sync::oneshot::Sender<()>>,
    pub server_state_tx: Option<tokio::sync::watch::Sender<AppState>>,
    cache_dir: PathBuf,
    config_dir: PathBuf,
}

impl AppStateManager {
    pub fn new() -> Self {
        let (cache_dir, config_dir) =
            Self::get_directories().expect("Failed to get application directories");

        // Ensure directories exist
        fs::create_dir_all(&cache_dir).ok();
        fs::create_dir_all(&config_dir).ok();

        let mut state = AppState::default();
        let state_path = config_dir.join("state.json");

        if let Err(e) = load_state(&state_path, &mut state) {
            log::error!("Failed to load state: {}", e);
        }

        Self {
            state,
            server_url: None,
            server_shutdown: None,
            server_state_tx: None,
            cache_dir,
            config_dir,
        }
    }

    fn get_directories() -> Result<(PathBuf, PathBuf), String> {
        #[cfg(target_os = "macos")]
        {
            let home = dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())?;
            let cache = home.join("Library/Caches/podcasterator");
            let config = home.join("Library/Application Support/Podcasterator");
            Ok((cache, config))
        }

        #[cfg(not(target_os = "macos"))]
        {
            let cache = dirs::cache_dir()
                .ok_or_else(|| "Failed to get cache directory".to_string())?
                .join("podcasterator");
            let config = dirs::config_dir()
                .ok_or_else(|| "Failed to get config directory".to_string())?
                .join("Podcasterator");
            Ok((cache, config))
        }
    }

    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    fn state_file_path(&self) -> PathBuf {
        self.config_dir.join("state.json")
    }

    pub fn save_state(&self) -> Result<(), String> {
        let path = self.state_file_path();
        let json = serde_json::to_string_pretty(&self.state)
            .map_err(|e| format!("Failed to serialize state: {}", e))?;
        fs::write(&path, json).map_err(|e| format!("Failed to write state file: {}", e))?;

        // Sync to running server
        if let Some(ref tx) = self.server_state_tx {
            let _ = tx.send(self.state.clone());
        }

        Ok(())
    }

    pub fn get_state(&self) -> AppState {
        self.state.clone()
    }

    /// Remove all temp audio files from disk and clear the file list from state.
    /// Artwork and podcast name are preserved.
    pub fn cleanup_temp_files(&mut self) {
        for file in &self.state.files {
            let temp_path = PathBuf::from(&file.temp_path);
            if temp_path.exists() {
                if let Err(e) = fs::remove_file(&temp_path) {
                    log::warn!("Failed to remove temp file {}: {}", temp_path.display(), e);
                }
            }
            if let Some(parent) = temp_path.parent() {
                let _ = fs::remove_dir(parent);
            }
        }
        self.state.files.clear();
        if let Err(e) = self.save_state() {
            log::error!("Failed to save state after cleanup: {}", e);
        }
    }
}

fn load_state(path: &PathBuf, state: &mut AppState) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read state file: {}", e))?;

    let mut loaded_state: AppState =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse state file: {}", e))?;

    // Validate that temp files still exist
    let before_count = loaded_state.files.len();
    loaded_state.files.retain(|f| {
        let exists = PathBuf::from(&f.temp_path).exists();
        if !exists {
            log::warn!("Temp file missing, removing from state: {}", f.display_name);
        }
        exists
    });
    if loaded_state.files.len() < before_count {
        log::warn!(
            "Removed {} missing files from state",
            before_count - loaded_state.files.len()
        );
    }

    // Validate artwork exists
    if let Some(ref artwork) = loaded_state.artwork_path {
        if !PathBuf::from(artwork).exists() {
            log::warn!("Artwork file missing, clearing from state");
            loaded_state.artwork_path = None;
        }
    }

    // Backwards compat: assign added_at for files that don't have it
    let file_count = loaded_state.files.len();
    for (i, file) in loaded_state.files.iter_mut().enumerate() {
        if file.added_at.is_empty() {
            let ts = Utc::now() - chrono::Duration::hours(file_count as i64 - i as i64);
            file.added_at = ts.to_rfc2822();
        }
    }

    *state = loaded_state;
    Ok(())
}
