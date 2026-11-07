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

#[command]
pub fn import_files_to_library_cmd(
    source_paths: Vec<String>,
    library_dir: String,
) -> Result<Vec<String>, String> {
    let lib_path = Path::new(&library_dir);
    if !lib_path.exists() {
        std::fs::create_dir_all(lib_path).map_err(|e| e.to_string())?;
    }

    let mut imported = Vec::new();

    for sp in source_paths {
        let p = Path::new(&sp);
        if !p.exists() {
            continue;
        }

        if p.is_dir() {
            for entry in walkdir::WalkDir::new(p).into_iter().filter_map(|e| e.ok()) {
                let ep = entry.path();
                if ep.is_file() {
                    let ext = ep.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                    if ext == "jar" {
                        if let Some(file_name) = ep.file_name() {
                            let dest = lib_path.join(file_name);
                            if std::fs::copy(ep, &dest).is_ok() {
                                imported.push(file_name.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        } else if p.is_file() {
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            if ext == "jar" {
                if let Some(file_name) = p.file_name() {
                    let dest = lib_path.join(file_name);
                    std::fs::copy(p, &dest).map_err(|e| e.to_string())?;
                    imported.push(file_name.to_string_lossy().to_string());
                }
            } else if ext == "zip" || ext == "mrpack" || ext == "morgpack" {
                if let Ok(file) = std::fs::File::open(p) {
                    if let Ok(mut archive) = zip::ZipArchive::new(file) {
                        for i in 0..archive.len() {
                            if let Ok(mut zf) = archive.by_index(i) {
                                let name = zf.name().to_string();
                                if name.ends_with(".jar") && !name.contains("__MACOSX") {
                                    let filename = Path::new(&name)
                                        .file_name()
                                        .map(|n| n.to_string_lossy().to_string())
                                        .unwrap_or_else(|| format!("mod_{}.jar", i));
                                    let dest = lib_path.join(&filename);
                                    if let Ok(mut out) = std::fs::File::create(&dest) {
                                        let _ = std::io::copy(&mut zf, &mut out);
                                        imported.push(filename);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(imported)
}

#[command]
pub fn save_binary_to_library_cmd(
    file_name: String,
    bytes: Vec<u8>,
    library_dir: String,
) -> Result<Vec<String>, String> {
    let lib_path = Path::new(&library_dir);
    if !lib_path.exists() {
        std::fs::create_dir_all(lib_path).map_err(|e| e.to_string())?;
    }

    let ext = Path::new(&file_name).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let mut imported = Vec::new();

    if ext == "jar" {
        let dest = lib_path.join(&file_name);
        std::fs::write(&dest, &bytes).map_err(|e| e.to_string())?;
        imported.push(file_name);
    } else if ext == "zip" || ext == "mrpack" || ext == "morgpack" {
        let cursor = std::io::Cursor::new(bytes);
        if let Ok(mut archive) = zip::ZipArchive::new(cursor) {
            for i in 0..archive.len() {
                if let Ok(mut zf) = archive.by_index(i) {
                    let name = zf.name().to_string();
                    if name.ends_with(".jar") && !name.contains("__MACOSX") {
                        let inner_name = Path::new(&name)
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| format!("mod_{}.jar", i));
                        let dest = lib_path.join(&inner_name);
                        if let Ok(mut out) = std::fs::File::create(&dest) {
                            let _ = std::io::copy(&mut zf, &mut out);
                            imported.push(inner_name);
                        }
                    }
                }
            }
        }
    }

    Ok(imported)
}

#[command]
pub fn delete_mod_file_cmd(file_path: String) -> Result<bool, String> {
    let p = Path::new(&file_path);
    if p.exists() && p.is_file() {
        std::fs::remove_file(p).map_err(|e| e.to_string())?;
        Ok(true)
    } else {
        Err("File not found".to_string())
    }
}
