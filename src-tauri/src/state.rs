use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFile {
    pub id: String,
    pub original_path: String,
    pub temp_path: String,
    pub display_name: String,
    #[serde(default)]
    pub added_at: String,
}

fn default_podcast_name() -> String {
    "My Podcast".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub files: Vec<AudioFile>,
    #[serde(default = "default_podcast_name")]
    pub podcast_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artwork_path: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            podcast_name: default_podcast_name(),
            artwork_path: None,
        }
    }
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

        // Audio files are ephemeral — drop them on every launch and sweep the
        // cache directory so leftovers from a crash don't accumulate. Only the
        // current artwork survives.
        state.files.clear();
        sweep_cache(&cache_dir, state.artwork_path.as_deref());

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
}

/// Delete everything in the cache directory except the current artwork.
fn sweep_cache(cache_dir: &Path, artwork_path: Option<&str>) {
    let artwork = artwork_path.map(PathBuf::from);

    let entries = match fs::read_dir(cache_dir) {
        Ok(entries) => entries,
        Err(e) => {
            log::warn!("Failed to read cache directory: {}", e);
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if artwork.as_deref() == Some(path.as_path()) {
            continue;
        }
        let result = if path.is_dir() {
            fs::remove_dir_all(&path)
        } else {
            fs::remove_file(&path)
        };
        if let Err(e) = result {
            log::warn!("Failed to remove cache entry {}: {}", path.display(), e);
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

    // Validate artwork exists
    if let Some(ref artwork) = loaded_state.artwork_path {
        if !PathBuf::from(artwork).exists() {
            log::warn!("Artwork file missing, clearing from state");
            loaded_state.artwork_path = None;
        }
    }

    *state = loaded_state;
    Ok(())
}
