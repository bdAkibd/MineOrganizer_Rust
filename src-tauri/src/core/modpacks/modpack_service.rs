use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::core::network::modrinth::download_file;
use crate::core::scanners::mod_parser::parse_mod_jar;
use crate::models::mod_file::ModFile;
use crate::models::profile::Profile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackInspectResult {
    pub name: String,
    pub minecraft_version: String,
    pub loader: String,
    pub loader_version: String,
    pub mod_count: usize,
    pub resourcepacks_count: usize,
    pub shaderpacks_count: usize,
    pub saves_count: usize,
    pub has_configs: bool,
    pub has_resourcepacks: bool,
    pub has_shaderpacks: bool,
    pub has_saves: bool,
    pub format: String, // "morgpack", "modrinth", "curseforge", "generic_zip"
    pub mod_filenames: Vec<String>,
    pub save_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackExportOptions {
    pub include_configs: bool,
    pub include_resourcepacks: bool,
    pub include_shaderpacks: bool,
    pub include_saves: bool,
    pub selected_saves: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackExportReport {
    pub mods_count: usize,
    pub configs_count: usize,
    pub resourcepacks_count: usize,
    pub shaderpacks_count: usize,
    pub saves_count: usize,
    pub archive_size: u64,
    pub output_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackImportResult {
    pub name: String,
    pub format: String,
    pub minecraft_version: String,
    pub loader: String,
    pub loader_version: String,
    pub imported_mods_count: usize,
    pub imported_configs_count: usize,
    pub imported_resourcepacks_count: usize,
    pub imported_shaderpacks_count: usize,
    pub imported_saves_count: usize,
    pub imported_mod_hashes: Vec<String>,
}

/// Inspect an archive file to identify modpack metadata and structure.
pub fn inspect_modpack(archive_path: &Path) -> Result<ModpackInspectResult, String> {
    if !archive_path.is_file() {
        return Err(format!("Archive not found: {}", archive_path.display()));
    }

    let file = File::open(archive_path).map_err(|e| format!("Failed to open archive: {}", e))?;
    let mut arc = ZipArchive::new(file).map_err(|e| format!("Invalid zip archive: {}", e))?;

    let mut names = HashSet::new();
    for i in 0..arc.len() {
        if let Ok(f) = arc.by_index(i) {
            names.insert(f.name().to_string());
        }
    }

    let archive_stem = archive_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Modpack")
        .to_string();

    let mut result = ModpackInspectResult {
        name: archive_stem,
        minecraft_version: String::new(),
        loader: String::new(),
        loader_version: String::new(),
        mod_count: 0,
        resourcepacks_count: 0,
        shaderpacks_count: 0,
        saves_count: 0,
        has_configs: false,
        has_resourcepacks: false,
        has_shaderpacks: false,
        has_saves: false,
        format: "generic_zip".to_string(),
        mod_filenames: Vec::new(),
        save_names: Vec::new(),
    };

    // 1. MineOrganizer modpack.json
    if names.contains("modpack.json") {
        if let Ok(mut f) = arc.by_name("modpack.json") {
            let mut content = String::new();
            if f.read_to_string(&mut content).is_ok() {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    result.format = "morgpack".to_string();
                    if let Some(n) = val.get("name").and_then(|v| v.as_str()) {
                        result.name = n.to_string();
                    }
                    result.minecraft_version = val.get("minecraft_version").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    result.loader = val.get("loader").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    result.loader_version = val.get("loader_version").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    result.has_configs = val.get("has_configs").and_then(|v| v.as_bool()).unwrap_or(false);
                    result.has_resourcepacks = val.get("has_resourcepacks").and_then(|v| v.as_bool()).unwrap_or(false);
                    result.has_shaderpacks = val.get("has_shaderpacks").and_then(|v| v.as_bool()).unwrap_or(false);
                    result.has_saves = val.get("has_saves").and_then(|v| v.as_bool()).unwrap_or(false);

                    if let Some(arr) = val.get("saves").and_then(|s| s.as_array()) {
                        result.save_names = arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect();
                        result.saves_count = result.save_names.len();
                    }

                    if let Some(arr) = val.get("mods").and_then(|m| m.as_array()) {
                        result.mod_filenames = arr
                            .iter()
                            .filter_map(|m| m.get("filename").and_then(|f| f.as_str().map(|s| s.to_string())))
                            .collect();
                    }
                }
            }
        }
    }
    // 2. Modrinth modrinth.index.json
    else if names.contains("modrinth.index.json") {
        if let Ok(mut f) = arc.by_name("modrinth.index.json") {
            let mut content = String::new();
            if f.read_to_string(&mut content).is_ok() {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    result.format = "modrinth".to_string();
                    if let Some(n) = val.get("name").and_then(|v| v.as_str()) {
                        result.name = n.to_string();
                    }
                    if let Some(deps) = val.get("dependencies").and_then(|d| d.as_object()) {
                        result.minecraft_version = deps.get("minecraft").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        for ldr in &["fabric-loader", "forge", "neoforge", "quilt-loader"] {
                            if let Some(ver) = deps.get(*ldr).and_then(|v| v.as_str()) {
                                result.loader = ldr.replace("-loader", "");
                                result.loader_version = ver.to_string();
                                break;
                            }
                        }
                    }
                    if let Some(files) = val.get("files").and_then(|f| f.as_array()) {
                        result.mod_filenames = files
                            .iter()
                            .filter_map(|f| f.get("path").and_then(|p| p.as_str()))
                            .map(|p| Path::new(p).file_name().unwrap_or_default().to_string_lossy().to_string())
                            .collect();
                    }
                }
            }
        }
    }
    // 3. CurseForge manifest.json
    else if names.contains("manifest.json") {
        if let Ok(mut f) = arc.by_name("manifest.json") {
            let mut content = String::new();
            if f.read_to_string(&mut content).is_ok() {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    result.format = "curseforge".to_string();
                    if let Some(n) = val.get("name").and_then(|v| v.as_str()) {
                        result.name = n.to_string();
                    }
                    if let Some(mc) = val.get("minecraft") {
                        result.minecraft_version = mc.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        if let Some(loaders) = mc.get("modLoaders").and_then(|l| l.as_array()) {
                            if let Some(first) = loaders.first().and_then(|m| m.get("id")).and_then(|id| id.as_str()) {
                                let lower = first.to_lowercase();
                                if lower.contains("fabric") {
                                    result.loader = "Fabric".to_string();
                                } else if lower.contains("forge") {
                                    result.loader = "Forge".to_string();
                                } else if lower.contains("neoforge") {
                                    result.loader = "NeoForge".to_string();
                                } else if lower.contains("quilt") {
                                    result.loader = "Quilt".to_string();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Inspect entries
    let jar_entries: Vec<_> = names
        .iter()
        .filter(|n| n.to_lowercase().ends_with(".jar") && !n.ends_with('/'))
        .cloned()
        .collect();

    if !jar_entries.is_empty() {
        result.mod_count = jar_entries.len();
        if result.mod_filenames.is_empty() {
            result.mod_filenames = jar_entries
                .iter()
                .map(|p| Path::new(p).file_name().unwrap_or_default().to_string_lossy().to_string())
                .collect();
        }
    } else if !result.mod_filenames.is_empty() {
        result.mod_count = result.mod_filenames.len();
    }

    let rp_count = names.iter().filter(|n| n.starts_with("resourcepacks/") && n.ends_with(".zip")).count();
    let sp_count = names.iter().filter(|n| n.starts_with("shaderpacks/") && n.ends_with(".zip")).count();
    let save_level_dats: Vec<_> = names.iter().filter(|n| n.starts_with("saves/") && n.ends_with("level.dat")).collect();

    if rp_count > 0 {
        result.resourcepacks_count = rp_count;
        result.has_resourcepacks = true;
    }
    if sp_count > 0 {
        result.shaderpacks_count = sp_count;
        result.has_shaderpacks = true;
    }
    if !save_level_dats.is_empty() {
        result.saves_count = save_level_dats.len();
        result.has_saves = true;
        if result.save_names.is_empty() {
            result.save_names = save_level_dats
                .iter()
                .filter_map(|p| Path::new(p).parent()?.file_name().map(|n| n.to_string_lossy().to_string()))
                .collect();
        }
    }

    if names.iter().any(|n| n.starts_with("config/") || n.starts_with("overrides/config/")) {
        result.has_configs = true;
    }

    Ok(result)
}

/// Check if a relative path avoids directory traversal (`..`) attacks.
pub fn is_safe_relative_path(base_dir: &Path, rel_path: &str) -> bool {
    let clean = rel_path.replace('\\', "/");
    if clean.starts_with('/') || clean.starts_with("..") || clean.contains("/../") {
        return false;
    }
    let dest = base_dir.join(&clean);
    if let (Ok(can_base), Ok(can_dest)) = (fs::canonicalize(base_dir), fs::canonicalize(&dest)) {
        can_dest.starts_with(&can_base)
    } else {
        !clean.contains("..")
    }
}

/// Export a profile to a `.morgpack` (or `.zip`) archive.
pub fn export_profile_to_modpack(
    profile: &Profile,
    library_mods: &[ModFile],
    output_archive: &Path,
    options: &ModpackExportOptions,
) -> Result<ModpackExportReport, String> {
    if let Some(parent) = output_archive.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let file = File::create(output_archive).map_err(|e| format!("Failed to create output file: {}", e))?;
    let mut zip = ZipWriter::new(file);
    let zip_options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // 1. Gather Mods
    let desired_hashes: HashSet<_> = profile.enabled_mod_hashes.iter().collect();
    let mods_to_export: Vec<_> = library_mods
        .iter()
        .filter(|m| {
            if let Some(ref h) = m.sha256 {
                if desired_hashes.contains(h) {
                    return true;
                }
            }
            desired_hashes.contains(&m.sha1)
        })
        .collect();

    // 2. Write Manifest modpack.json
    let manifest_val = serde_json::json!({
        "format_version": 1,
        "generator": "MineOrganizer",
        "name": profile.name,
        "minecraft_version": profile.minecraft_version.clone().unwrap_or_default(),
        "loader": profile.loader.clone().unwrap_or_default(),
        "loader_version": profile.loader_version.clone().unwrap_or_default(),
        "created_at": Utc::now().to_rfc3339(),
        "mods": mods_to_export.iter().map(|m| {
            serde_json::json!({
                "filename": m.file_name,
                "sha256": m.sha256,
                "sha1": m.sha1,
                "size": m.size
            })
        }).collect::<Vec<_>>(),
        "has_configs": options.include_configs,
        "has_resourcepacks": options.include_resourcepacks,
        "has_shaderpacks": options.include_shaderpacks,
        "has_saves": options.include_saves
    });

    zip.start_file("modpack.json", zip_options).map_err(|e| e.to_string())?;
    zip.write_all(serde_json::to_string_pretty(&manifest_val).unwrap().as_bytes())
        .map_err(|e| e.to_string())?;

    // 3. Write Mods
    let mut mods_written = 0;
    for m in &mods_to_export {
        if m.path.is_file() {
            if let Ok(mut src) = File::open(&m.path) {
                let arc_name = format!("mods/{}", m.file_name);
                if zip.start_file(arc_name, zip_options).is_ok() {
                    let mut buf = Vec::new();
                    if src.read_to_end(&mut buf).is_ok() {
                        let _ = zip.write_all(&buf);
                        mods_written += 1;
                    }
                }
            }
        }
    }

    // 4. Configs (if instance_path is known and include_configs is true)
    let mut configs_written = 0;
    if options.include_configs {
        if let Some(ref inst_path_str) = profile.instance_path {
            let config_dir = Path::new(inst_path_str).join("config");
            if config_dir.is_dir() {
                for entry in walkdir::WalkDir::new(&config_dir).into_iter().filter_map(|e| e.ok()) {
                    if entry.file_type().is_file() {
                        if let Ok(rel) = entry.path().strip_prefix(&config_dir) {
                            let rel_str = rel.to_string_lossy().replace('\\', "/");
                            let arc_name = format!("config/{}", rel_str);
                            if zip.start_file(arc_name, zip_options).is_ok() {
                                if let Ok(mut src) = File::open(entry.path()) {
                                    let mut buf = Vec::new();
                                    if src.read_to_end(&mut buf).is_ok() {
                                        let _ = zip.write_all(&buf);
                                        configs_written += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    zip.finish().map_err(|e| format!("Failed to finalize zip: {}", e))?;

    let archive_size = fs::metadata(output_archive).map(|m| m.len()).unwrap_or(0);

    Ok(ModpackExportReport {
        mods_count: mods_written,
        configs_count: configs_written,
        resourcepacks_count: 0,
        shaderpacks_count: 0,
        saves_count: 0,
        archive_size,
        output_path: output_archive.to_path_buf(),
    })
}

/// Import a modpack archive into the library directory and optional target instance.
pub async fn import_modpack(
    archive_path: &Path,
    library_dir: &Path,
    target_instance_path: Option<&Path>,
) -> Result<ModpackImportResult, String> {
    let inspect = inspect_modpack(archive_path)?;

    fs::create_dir_all(library_dir).map_err(|e| e.to_string())?;

    let file = File::open(archive_path).map_err(|e| format!("Failed to open archive: {}", e))?;
    let mut arc = ZipArchive::new(file).map_err(|e| format!("Failed to read zip: {}", e))?;

    let mut imported_mod_hashes = Vec::new();
    let mut imported_mods_count = 0;
    let mut imported_configs_count = 0;

    // Handle Modrinth (.mrpack) index download if remote files are defined
    if inspect.format == "modrinth" {
        if let Ok(mut f) = arc.by_name("modrinth.index.json") {
            let mut content = String::new();
            if f.read_to_string(&mut content).is_ok() {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(files) = val.get("files").and_then(|f| f.as_array()) {
                        for item in files {
                            let downloads = item.get("downloads").and_then(|d| d.as_array());
                            let path_str = item.get("path").and_then(|p| p.as_str()).unwrap_or("");
                            let filename = Path::new(path_str).file_name().unwrap_or_default().to_string_lossy().to_string();

                            if let Some(dl_arr) = downloads {
                                if let Some(first_url) = dl_arr.first().and_then(|u| u.as_str()) {
                                    let dest = library_dir.join(&filename);
                                    if !dest.exists() {
                                        if let Ok(downloaded) = download_file(first_url, &dest).await {
                                            if let Ok(parsed) = parse_mod_jar(&downloaded) {
                                                if let Some(h) = parsed.sha256 {
                                                    imported_mod_hashes.push(h);
                                                    imported_mods_count += 1;
                                                }
                                            }
                                        }
                                    } else if let Ok(parsed) = parse_mod_jar(&dest) {
                                        if let Some(h) = parsed.sha256 {
                                            imported_mod_hashes.push(h);
                                            imported_mods_count += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Extract packaged .jar files into library_dir
    for i in 0..arc.len() {
        let mut entry = match arc.by_index(i) {
            Ok(e) => e,
            Err(_) => continue,
        };

        let entry_name = entry.name().to_string();

        if entry_name.to_lowercase().ends_with(".jar") && !entry.is_dir() {
            let file_name = Path::new(&entry_name).file_name().unwrap_or_default().to_string_lossy().to_string();
            let dest_path = library_dir.join(&file_name);

            let mut out = File::create(&dest_path).map_err(|e| e.to_string())?;
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).map_err(|e| e.to_string())?;
            out.write_all(&buf).map_err(|e| e.to_string())?;

            if let Ok(parsed) = parse_mod_jar(&dest_path) {
                if let Some(h) = parsed.sha256 {
                    imported_mod_hashes.push(h);
                    imported_mods_count += 1;
                }
            }
        }
        // Extract configs to target instance
        else if (entry_name.starts_with("config/") || entry_name.starts_with("overrides/config/")) && !entry.is_dir() {
            if let Some(inst_dir) = target_instance_path {
                let rel = if entry_name.starts_with("config/") {
                    &entry_name["config/".len()..]
                } else {
                    &entry_name["overrides/config/".len()..]
                };

                let target_config_dir = inst_dir.join("config");
                if is_safe_relative_path(&target_config_dir, rel) {
                    let dest = target_config_dir.join(rel);
                    if let Some(parent) = dest.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    if let Ok(mut out) = File::create(&dest) {
                        let mut buf = Vec::new();
                        if entry.read_to_end(&mut buf).is_ok() {
                            let _ = out.write_all(&buf);
                            imported_configs_count += 1;
                        }
                    }
                }
            }
        }
    }

    imported_mod_hashes.sort();
    imported_mod_hashes.dedup();

    Ok(ModpackImportResult {
        name: inspect.name,
        format: inspect.format,
        minecraft_version: inspect.minecraft_version,
        loader: inspect.loader,
        loader_version: inspect.loader_version,
        imported_mods_count,
        imported_configs_count,
        imported_resourcepacks_count: 0,
        imported_shaderpacks_count: 0,
        imported_saves_count: 0,
        imported_mod_hashes,
    })
}
