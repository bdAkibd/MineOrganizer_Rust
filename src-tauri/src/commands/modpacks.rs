use std::path::Path;
use tauri::command;

use crate::core::modpacks::modpack_service::{
    export_profile_to_modpack, import_modpack, inspect_modpack, ModpackExportOptions,
    ModpackExportReport, ModpackImportResult, ModpackInspectResult,
};
use crate::core::scanners::mod_parser::scan_mods_dir_parallel;
use crate::core::storage::user_data_service::load_user_data;

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
        selected_saves: None,
    };

    export_profile_to_modpack(profile, &library_mods, Path::new(&output_path), &options)
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
