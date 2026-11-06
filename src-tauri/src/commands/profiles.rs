use tauri::command;

use crate::core::storage::user_data_service::{
    delete_profile, load_user_data, save_profile, save_settings, set_mod_tags,
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
