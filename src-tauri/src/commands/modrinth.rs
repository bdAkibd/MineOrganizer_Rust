use std::path::Path;
use tauri::command;

use crate::core::network::modrinth::{
    check_mod_updates, download_file, get_project_versions, search_modrinth,
    ModrinthSearchResult, ModrinthVersion, ModUpdateInfo,
};

#[command]
pub async fn search_modrinth_cmd(
    query: String,
    loader: String,
    minecraft_version: String,
    limit: usize,
    offset: usize,
) -> Result<Vec<ModrinthSearchResult>, String> {
    search_modrinth(&query, &loader, &minecraft_version, limit, offset).await
}

#[command]
pub async fn get_project_versions_cmd(
    project_id_or_slug: String,
    loader: String,
    minecraft_version: String,
) -> Result<Vec<ModrinthVersion>, String> {
    get_project_versions(&project_id_or_slug, &loader, &minecraft_version).await
}

#[command]
pub async fn download_modrinth_file_cmd(
    url: String,
    filename: String,
    destination_dir: String,
) -> Result<String, String> {
    let dest_path = Path::new(&destination_dir).join(&filename);
    let downloaded = download_file(&url, &dest_path).await?;
    Ok(downloaded.to_string_lossy().to_string())
}

#[command]
pub async fn check_mod_updates_cmd(
    file_hashes: Vec<String>,
    loader: String,
    minecraft_version: String,
) -> Result<Vec<ModUpdateInfo>, String> {
    check_mod_updates(&file_hashes, &loader, &minecraft_version).await
}
