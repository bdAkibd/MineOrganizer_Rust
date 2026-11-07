use std::path::Path;
use tauri::command;

use crate::core::storage::user_data_service::{
    default_user_data_path, delete_profile, load_user_data, save_profile, save_settings,
    save_user_data, set_mod_tags,
};
use crate::models::profile::Profile;
use crate::models::user_data::{AppSettings, UserDataStore};

#[command]
pub fn load_user_data_cmd() -> UserDataStore {
    load_user_data(None)
}

#[command]
pub fn save_profile_cmd(profile: Profile) -> Result<Profile, String> {
    let mut store = load_user_data(None);
    save_profile(&mut store, profile, None)
}

#[command]
pub fn delete_profile_cmd(profile_id: String) -> Result<bool, String> {
    let mut store = load_user_data(None);
    delete_profile(&mut store, &profile_id, None)
}

#[command]
pub fn set_active_profile_cmd(profile_id: Option<String>) -> Result<bool, String> {
    let mut store = load_user_data(None);
    store.active_profile_id = profile_id;
    save_user_data(&store, None)?;
    Ok(true)
}

#[command]
pub fn save_settings_cmd(settings: AppSettings) -> Result<bool, String> {
    let mut store = load_user_data(None);
    save_settings(&mut store, settings, None)?;
    Ok(true)
}

#[command]
pub fn set_mod_tags_cmd(sha256: String, tags: Vec<String>) -> Result<bool, String> {
    let mut store = load_user_data(None);
    set_mod_tags(&mut store, &sha256, tags, None)?;
    Ok(true)
}

#[command]
pub fn get_app_data_dir_cmd() -> String {
    let p = default_user_data_path();
    let dir = p.parent().unwrap_or(&p);
    let _ = std::fs::create_dir_all(dir);
    dir.to_string_lossy().to_string()
}

#[command]
pub fn open_path_in_explorer_cmd(path: String) -> Result<bool, String> {
    let p = Path::new(&path);
    if !p.exists() {
        let _ = std::fs::create_dir_all(p);
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open Windows Explorer: {}", e))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
    }
    Ok(true)
}
