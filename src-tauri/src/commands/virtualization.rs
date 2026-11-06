use std::path::Path;
use tauri::command;

use crate::core::virtualization::engine::{
    apply_profile, preview_profile, rollback_last, VirtualizationPlan, VirtualizationResult,
};

#[command]
pub fn preview_profile_cmd(
    instance_path: String,
    profile_name: String,
    desired_hashes: Vec<String>,
    library_dir: String,
    mc_version: Option<String>,
    loader: Option<String>,
) -> VirtualizationPlan {
    preview_profile(
        Path::new(&instance_path),
        &profile_name,
        &desired_hashes,
        Path::new(&library_dir),
        mc_version.as_deref(),
        loader.as_deref(),
    )
}

#[command]
pub fn apply_profile_cmd(
    instance_path: String,
    profile_name: String,
    desired_hashes: Vec<String>,
    library_dir: String,
    backup_dir: Option<String>,
    mc_version: Option<String>,
    loader: Option<String>,
) -> Result<VirtualizationResult, String> {
    apply_profile(
        Path::new(&instance_path),
        &profile_name,
        &desired_hashes,
        Path::new(&library_dir),
        backup_dir.as_deref().map(Path::new),
        mc_version.as_deref(),
        loader.as_deref(),
    )
}

#[command]
pub fn rollback_last_cmd(
    backup_dir: Option<String>,
    library_dir: Option<String>,
) -> Result<String, String> {
    rollback_last(
        backup_dir.as_deref().map(Path::new),
        library_dir.as_deref().map(Path::new),
    )
}
