use std::path::{Path, PathBuf};
use tauri::command;

use crate::core::diagnostics::log_analyzer::{
    analyze_instance_logs, analyze_log_content, list_available_log_files, read_log_text_safely,
    CrashAnalysis, MAX_LOG_READ_BYTES,
};

#[command]
pub fn list_instance_log_files(instance_path: String) -> Vec<(String, PathBuf)> {
    list_available_log_files(Path::new(&instance_path))
}

#[command]
pub fn read_log_file_content(file_path: String, max_bytes: Option<u64>) -> String {
    let limit = max_bytes.unwrap_or(MAX_LOG_READ_BYTES);
    read_log_text_safely(Path::new(&file_path), limit)
}

#[command]
pub fn analyze_crash_logs(
    instance_path: String,
    known_mod_names: Option<Vec<String>>,
) -> Option<CrashAnalysis> {
    analyze_instance_logs(Path::new(&instance_path), known_mod_names.as_deref())
}

#[command]
pub fn analyze_raw_log(
    content: String,
    file_path: String,
    file_type: String,
    known_mod_names: Option<Vec<String>>,
) -> CrashAnalysis {
    analyze_log_content(&content, Path::new(&file_path), &file_type, known_mod_names.as_deref())
}
