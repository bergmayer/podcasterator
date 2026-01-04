use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFile {
    pub id: String,
    pub original_path: String,
    pub temp_path: String,
    pub display_name: String,
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
            // Linux/Windows - use XDG directories
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
        Ok(())
    }

    pub fn get_state(&self) -> AppState {
        self.state.clone()
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
    loaded_state.files.retain(|f| {
        let path = PathBuf::from(&f.temp_path);
        path.exists()
    });

    // Validate artwork exists
    if let Some(ref artwork) = loaded_state.artwork_path {
        if !PathBuf::from(artwork).exists() {
            loaded_state.artwork_path = None;
        }
    }

    *state = loaded_state;
    Ok(())
}

