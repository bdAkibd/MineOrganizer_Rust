use std::path::Path;
use tauri::command;

use crate::core::modpacks::modpack_service::{
    export_profile_to_modpack, import_modpack, inspect_modpack, ModpackExportOptions,
    ModpackExportReport, ModpackImportResult, ModpackInspectResult,
};
use crate::core::scanners::mod_parser::scan_mods_dir_parallel;
use crate::core::storage::user_data_service::load_user_data;
use crate::models::instance::{AssetItem, InstanceAssets, SaveItem};

#[command]
pub fn inspect_modpack_cmd(archive_path: String) -> Result<ModpackInspectResult, String> {
    inspect_modpack(Path::new(&archive_path))
}

#[command]
pub fn export_modpack_cmd(
    profile_id: String,
    library_dir: String,
    output_path: String,
    include_configs: bool,
    include_resourcepacks: bool,
    include_shaderpacks: bool,
    include_saves: bool,
    selected_saves: Option<Vec<String>>,
) -> Result<ModpackExportReport, String> {
    let store = load_user_data(None);
    let profile = store
        .profiles
        .iter()
        .find(|p| p.id == profile_id)
        .ok_or_else(|| format!("Profile not found: {}", profile_id))?;

    let library_mods = scan_mods_dir_parallel(Path::new(&library_dir));

    let options = ModpackExportOptions {
        include_configs,
        include_resourcepacks,
        include_shaderpacks,
        include_saves,
        selected_saves,
    };

    export_profile_to_modpack(profile, &library_mods, Path::new(&output_path), &options)
}

#[command]
pub fn list_instance_assets_cmd(instance_path: String) -> InstanceAssets {
    let base = Path::new(&instance_path);
    let mc_dir = if base.join(".minecraft").is_dir() {
        base.join(".minecraft")
    } else {
        base.to_path_buf()
    };

    let mut resourcepacks = Vec::new();
    let rp_dir = mc_dir.join("resourcepacks");
    if rp_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&rp_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();
                let fname = entry.file_name().to_string_lossy().to_string();
                if fname.starts_with('.') {
                    continue;
                }
                let size = if p.is_file() {
                    p.metadata().map(|m| m.len()).unwrap_or(0)
                } else if p.is_dir() {
                    walkdir::WalkDir::new(&p)
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .filter_map(|e| e.metadata().ok())
                        .filter(|m| m.is_file())
                        .map(|m| m.len())
                        .sum()
                } else {
                    0
                };
                resourcepacks.push(AssetItem {
                    name: fname.trim_end_matches(".zip").to_string(),
                    filename: fname,
                    path: p.to_string_lossy().to_string(),
                    size,
                });
            }
        }
    }

    let mut shaderpacks = Vec::new();
    let sp_dir = mc_dir.join("shaderpacks");
    if sp_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&sp_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();
                let fname = entry.file_name().to_string_lossy().to_string();
                if fname.starts_with('.') {
                    continue;
                }
                let size = if p.is_file() {
                    p.metadata().map(|m| m.len()).unwrap_or(0)
                } else if p.is_dir() {
                    walkdir::WalkDir::new(&p)
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .filter_map(|e| e.metadata().ok())
                        .filter(|m| m.is_file())
                        .map(|m| m.len())
                        .sum()
                } else {
                    0
                };
                shaderpacks.push(AssetItem {
                    name: fname.trim_end_matches(".zip").to_string(),
                    filename: fname,
                    path: p.to_string_lossy().to_string(),
                    size,
                });
            }
        }
    }

    let mut saves = Vec::new();
    let saves_dir = mc_dir.join("saves");
    if saves_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&saves_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.is_dir() {
                    let folder_name = entry.file_name().to_string_lossy().to_string();
                    if folder_name.starts_with('.') {
                        continue;
                    }
                    let size = walkdir::WalkDir::new(&p)
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .filter_map(|e| e.metadata().ok())
                        .filter(|m| m.is_file())
                        .map(|m| m.len())
                        .sum();
                    saves.push(SaveItem {
                        level_name: folder_name.clone(),
                        folder_name,
                        path: p.to_string_lossy().to_string(),
                        size,
                    });
                }
            }
        }
    }

    InstanceAssets {
        resourcepacks,
        shaderpacks,
        saves,
    }
}

#[command]
pub async fn import_modpack_cmd(
    archive_path: String,
    library_dir: String,
    target_instance_path: Option<String>,
) -> Result<ModpackImportResult, String> {
    import_modpack(
        Path::new(&archive_path),
        Path::new(&library_dir),
        target_instance_path.as_deref().map(Path::new),
    )
    .await
}
