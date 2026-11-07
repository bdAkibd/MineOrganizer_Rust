pub mod commands;
pub mod core;
pub mod models;

use commands::diagnostics::{
    analyze_crash_logs, analyze_raw_log, list_instance_log_files, read_log_file_content,
};
use commands::instances::{
    discover_instance_environments, get_default_candidate_paths, get_default_prism_path,
    scan_all_instances, scan_single_directory,
};
use commands::library::{check_mod_compatibility_cmd, parse_single_mod, scan_library_mods};
use commands::modpacks::{export_modpack_cmd, import_modpack_cmd, inspect_modpack_cmd};
use commands::modrinth::{
    check_mod_updates_cmd, download_modrinth_file_cmd, get_project_versions_cmd,
    search_modrinth_cmd,
};
use commands::profiles::{
    delete_profile_cmd, get_app_data_dir_cmd, load_user_data_cmd, open_path_in_explorer_cmd,
    save_profile_cmd, save_settings_cmd, set_active_profile_cmd, set_mod_tags_cmd,
};
use commands::virtualization::{apply_profile_cmd, preview_profile_cmd, rollback_last_cmd};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            // Instances
            get_default_candidate_paths,
            get_default_prism_path,
            scan_all_instances,
            discover_instance_environments,
            scan_single_directory,
            // Library
            scan_library_mods,
            parse_single_mod,
            check_mod_compatibility_cmd,
            // Virtualization
            preview_profile_cmd,
            apply_profile_cmd,
            rollback_last_cmd,
            // Diagnostics
            list_instance_log_files,
            read_log_file_content,
            analyze_crash_logs,
            analyze_raw_log,
            // Modrinth
            search_modrinth_cmd,
            get_project_versions_cmd,
            download_modrinth_file_cmd,
            check_mod_updates_cmd,
            // Modpacks
            inspect_modpack_cmd,
            export_modpack_cmd,
            import_modpack_cmd,
            // Profiles & Storage
            load_user_data_cmd,
            save_profile_cmd,
            delete_profile_cmd,
            set_active_profile_cmd,
            save_settings_cmd,
            set_mod_tags_cmd,
            get_app_data_dir_cmd,
            open_path_in_explorer_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
