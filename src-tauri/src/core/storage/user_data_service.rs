use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use chrono::Utc;
use uuid::Uuid;

use crate::models::profile::Profile;
use crate::models::user_data::{AppSettings, UserDataStore};

/// Return the standard path to user_data.json outside game directories.
pub fn default_user_data_path() -> PathBuf {
    if let Ok(local_appdata) = env::var("LOCALAPPDATA") {
        PathBuf::from(local_appdata).join("MineOrganizer").join("user_data.json")
    } else if let Ok(home) = env::var("USERPROFILE") {
        PathBuf::from(home).join(".mineorganizer").join("user_data.json")
    } else {
        PathBuf::from("./user_data.json")
    }
}

/// Load UserDataStore from disk, or return default if missing or invalid.
pub fn load_user_data(custom_path: Option<&Path>) -> UserDataStore {
    let path = custom_path.map(|p| p.to_path_buf()).unwrap_or_else(default_user_data_path);
    if !path.is_file() {
        return UserDataStore::default();
    }

    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => UserDataStore::default(),
    }
}

/// Atomically save UserDataStore to disk via a temporary file.
pub fn save_user_data(store: &UserDataStore, custom_path: Option<&Path>) -> Result<(), String> {
    let path = custom_path.map(|p| p.to_path_buf()).unwrap_or_else(default_user_data_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    let tmp_path = path.with_extension("tmp");
    let json_text = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;

    {
        let mut file = File::create(&tmp_path).map_err(|e| format!("Failed to create temporary file: {}", e))?;
        file.write_all(json_text.as_bytes()).map_err(|e| e.to_string())?;
        file.flush().map_err(|e| e.to_string())?;
    }

    fs::rename(&tmp_path, &path).map_err(|e| format!("Failed to atomically replace user data file: {}", e))?;
    Ok(())
}

/// Insert or update a profile in the store and save it.
pub fn save_profile(store: &mut UserDataStore, mut profile: Profile, custom_path: Option<&Path>) -> Result<Profile, String> {
    let now = Utc::now().to_rfc3339();
    if profile.id.is_empty() {
        profile.id = Uuid::new_v4().to_string();
        profile.created_at = now.clone();
    }
    profile.updated_at = now;

    if let Some(existing) = store.profiles.iter_mut().find(|p| p.id == profile.id) {
        *existing = profile.clone();
    } else {
        store.profiles.push(profile.clone());
    }

    save_user_data(store, custom_path)?;
    Ok(profile)
}

/// Delete a profile from the store and save it.
pub fn delete_profile(store: &mut UserDataStore, profile_id: &str, custom_path: Option<&Path>) -> Result<bool, String> {
    let before_len = store.profiles.len();
    store.profiles.retain(|p| p.id != profile_id);
    if store.active_profile_id.as_deref() == Some(profile_id) {
        store.active_profile_id = None;
    }

    if store.profiles.len() != before_len {
        save_user_data(store, custom_path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Update tags for a given mod SHA256 and save.
pub fn set_mod_tags(store: &mut UserDataStore, sha256: &str, tags: Vec<String>, custom_path: Option<&Path>) -> Result<(), String> {
    let mut clean_tags: Vec<String> = tags.into_iter().map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect();
    clean_tags.dedup();

    if clean_tags.is_empty() {
        store.mod_tags.remove(sha256);
    } else {
        store.mod_tags.insert(sha256.to_string(), clean_tags);
    }

    save_user_data(store, custom_path)
}

/// Update app settings and save.
pub fn save_settings(store: &mut UserDataStore, settings: AppSettings, custom_path: Option<&Path>) -> Result<(), String> {
    store.settings = settings;
    save_user_data(store, custom_path)
}
