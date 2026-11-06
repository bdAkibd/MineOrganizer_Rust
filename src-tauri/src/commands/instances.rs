use std::path::{Path, PathBuf};
use tauri::command;

use crate::core::scanners::launchers::{
    default_game_directory_candidates, discover_game_environments,
    find_default_prism_data_dir, scan_game_directory, scan_prism_instances,
};
use crate::models::instance::Instance;

#[command]
pub fn get_default_candidate_paths() -> Vec<PathBuf> {
    default_game_directory_candidates()
}

#[command]
pub fn get_default_prism_path() -> Option<PathBuf> {
    find_default_prism_data_dir()
}

#[command]
pub fn scan_all_instances(
    prism_path: Option<String>,
    custom_paths: Vec<String>,
) -> Vec<Instance> {
    let mut instances = Vec::new();

    // 1. Scan Prism
    let prism_dir = prism_path
        .map(PathBuf::from)
        .or_else(find_default_prism_data_dir);

    if let Some(ref p) = prism_dir {
        instances.extend(scan_prism_instances(p));
    }

    // 2. Scan standard default candidate directories
    let default_cands = default_game_directory_candidates();
    for cand in default_cands {
        if cand.is_dir() {
            instances.extend(discover_game_environments(&cand, None));
        }
    }

    // 3. Scan user-specified custom game directories
    for custom in custom_paths {
        let p = PathBuf::from(custom);
        if p.is_dir() {
            instances.extend(discover_game_environments(&p, None));
        }
    }

    // Deduplicate by instance ID
    let mut seen = std::collections::HashSet::new();
    instances.retain(|inst| seen.insert(inst.id.clone()));

    instances
}

#[command]
pub fn discover_instance_environments(path: String) -> Vec<Instance> {
    let p = Path::new(&path);
    discover_game_environments(p, None)
}

#[command]
pub fn scan_single_directory(path: String, launcher_override: Option<String>) -> Instance {
    let p = Path::new(&path);
    let launcher = launcher_override.as_deref().unwrap_or("Minecraft (user folder)");
    scan_game_directory(p, launcher)
}
