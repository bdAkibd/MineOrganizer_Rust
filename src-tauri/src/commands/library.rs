use std::collections::HashSet;
use std::path::Path;
use tauri::command;

use crate::core::scanners::mod_parser::{check_mod_compatibility, parse_mod_jar, scan_mods_dir_parallel};
use crate::models::mod_file::{ModFile, ModMetadata};

#[command]
pub fn scan_library_mods(dir_path: String) -> Vec<ModFile> {
    let p = Path::new(&dir_path);
    scan_mods_dir_parallel(p)
}

#[command]
pub fn parse_single_mod(file_path: String) -> Result<ModFile, String> {
    let p = Path::new(&file_path);
    parse_mod_jar(p)
}

#[command]
pub fn check_mod_compatibility_cmd(
    metadata: ModMetadata,
    minecraft_version: String,
    loader: String,
    available_ids: Vec<String>,
) -> (String, String) {
    let avail_set: HashSet<String> = available_ids.into_iter().map(|id| id.to_lowercase()).collect();
    check_mod_compatibility(&metadata, &minecraft_version, &loader, Some(&avail_set))
}
