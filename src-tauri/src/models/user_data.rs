use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use super::profile::Profile;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppSettings {
    #[serde(default = "default_locale")]
    pub locale: String,
    #[serde(default)]
    pub prism_data_dir: Option<PathBuf>,
    #[serde(default)]
    pub library_dir: Option<PathBuf>,
    #[serde(default)]
    pub custom_game_dirs: Vec<PathBuf>,
    #[serde(default)]
    pub close_launcher_on_apply: bool,
}

fn default_locale() -> String {
    "en".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            locale: default_locale(),
            prism_data_dir: None,
            library_dir: None,
            custom_game_dirs: Vec::new(),
            close_launcher_on_apply: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserDataStore {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub active_profile_id: Option<String>,
    #[serde(default)]
    pub mod_tags: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub minecraft_version_overrides: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub profiles: Vec<Profile>,
    #[serde(default)]
    pub settings: AppSettings,
}

fn default_version() -> u32 {
    1
}

impl Default for UserDataStore {
    fn default() -> Self {
        Self {
            version: 1,
            active_profile_id: None,
            mod_tags: HashMap::new(),
            minecraft_version_overrides: HashMap::new(),
            profiles: Vec::new(),
            settings: AppSettings::default(),
        }
    }
}
