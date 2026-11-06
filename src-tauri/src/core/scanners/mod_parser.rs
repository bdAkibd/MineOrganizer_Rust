use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use base64::Engine;
use rayon::prelude::*;
use regex::Regex;
use sha1::Sha1;
use sha2::{Digest, Sha256};
use zip::ZipArchive;

use crate::models::mod_file::{ModDependency, ModFile, ModMetadata};

/// Read and parse metadata for a single mod JAR file.
pub fn parse_mod_jar(path: &Path) -> Result<ModFile, String> {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.jar")
        .to_string();

    let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let metadata = file
        .metadata()
        .map_err(|e| format!("Failed to read file metadata: {}", e))?;
    let size = metadata.len();

    // Compute SHA1 and SHA256 hashes
    let mut sha1_hasher = Sha1::new();
    let mut sha256_hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read file for hashing: {}", e))?;
        if bytes_read == 0 {
            break;
        }
        sha1_hasher.update(&buffer[..bytes_read]);
        sha256_hasher.update(&buffer[..bytes_read]);
    }

    let sha1 = hex::encode(sha1_hasher.finalize());
    let sha256 = hex::encode(sha256_hasher.finalize());

    // Extract archive metadata
    let mod_meta = extract_archive_metadata(path).unwrap_or_else(|| {
        fallback_metadata_from_filename(path)
    });

    let disabled = file_name.ends_with(".disabled") || file_name.ends_with(".disabled.jar");

    Ok(ModFile {
        file_name,
        path: path.to_path_buf(),
        size,
        sha1,
        sha256: Some(sha256),
        sha512: None,
        metadata: mod_meta,
        tags: Vec::new(),
        disabled,
    })
}

/// Scan a directory of mods in parallel across CPU threads using Rayon.
pub fn scan_mods_dir_parallel(dir: &Path) -> Vec<ModFile> {
    if !dir.is_dir() {
        return Vec::new();
    }

    let entries: Vec<_> = match fs::read_dir(dir) {
        Ok(read_dir) => read_dir
            .filter_map(|e| e.ok().map(|d| d.path()))
            .filter(|p| {
                if !p.is_file() {
                    return false;
                }
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
                name.ends_with(".jar") || name.ends_with(".jar.disabled") || name.ends_with(".disabled")
            })
            .collect(),
        Err(_) => return Vec::new(),
    };

    let mut mods: Vec<ModFile> = entries
        .par_iter()
        .filter_map(|path| parse_mod_jar(path).ok())
        .collect();

    mods.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));
    mods
}

// ----------------------------------------------------------------------------
// Internal extraction logic
// ----------------------------------------------------------------------------

fn extract_archive_metadata(path: &Path) -> Option<ModMetadata> {
    let file = File::open(path).ok()?;
    let mut archive = ZipArchive::new(file).ok()?;

    let mut names: HashSet<String> = HashSet::new();
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i) {
            names.insert(file.name().to_string());
        }
    }

    // 1. Bukkit/Spigot/Paper plugin.yml
    if names.contains("plugin.yml") {
        if let Some(meta) = parse_plugin_yml(&mut archive) {
            return Some(meta);
        }
    }

    // 2. Fabric fabric.mod.json
    if names.contains("fabric.mod.json") {
        if let Some(meta) = parse_fabric_json(&mut archive, &names) {
            return Some(meta);
        }
    }

    // 3. Quilt quilt.mod.json
    if names.contains("quilt.mod.json") {
        if let Some(meta) = parse_quilt_json(&mut archive, &names) {
            return Some(meta);
        }
    }

    // 4. NeoForge / Forge META-INF/neoforge.mods.toml or META-INF/mods.toml
    let toml_entry = if names.contains("META-INF/neoforge.mods.toml") {
        Some("META-INF/neoforge.mods.toml")
    } else if names.contains("META-INF/mods.toml") {
        Some("META-INF/mods.toml")
    } else {
        None
    };

    if let Some(entry_name) = toml_entry {
        if let Some(meta) = parse_forge_toml(&mut archive, entry_name, &names) {
            return Some(meta);
        }
    }

    // 5. Legacy Forge mcmod.info
    if names.contains("mcmod.info") {
        if let Some(meta) = parse_mcmod_info(&mut archive, &names) {
            return Some(meta);
        }
    }

    None
}

fn parse_fabric_json(archive: &mut ZipArchive<File>, names: &HashSet<String>) -> Option<ModMetadata> {
    let content = {
        let mut file = archive.by_name("fabric.mod.json").ok()?;
        let mut c = String::new();
        file.read_to_string(&mut c).ok()?;
        c
    };

    let val: serde_json::Value = serde_json::from_str(&content).ok()?;
    let id = val.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let name = val.get("name").and_then(|v| v.as_str()).unwrap_or(&id).to_string();
    let version = val.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let description = val.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());

    let mut dependencies = Vec::new();
    let mut mc_versions = Vec::new();

    if let Some(depends) = val.get("depends").and_then(|d| d.as_object()) {
        for (dep_id, dep_ver) in depends {
            let ver_str = dep_ver.as_str().unwrap_or("*").to_string();
            if dep_id == "minecraft" {
                mc_versions.push(ver_str.clone());
            } else {
                dependencies.push(ModDependency {
                    id: dep_id.clone(),
                    version_requirement: Some(ver_str),
                    required: true,
                });
            }
        }
    }

    let icon_data = extract_icon_data(archive, val.get("icon"), names);

    Some(ModMetadata {
        id,
        name,
        version: clean_version(&version),
        description,
        loaders: vec!["Fabric".to_string()],
        mc_versions,
        dependencies,
        icon_url: icon_data,
        homepage: val.get("contact").and_then(|c| c.get("homepage")).and_then(|h| h.as_str()).map(|s| s.to_string()),
        source_url: val.get("contact").and_then(|c| c.get("sources")).and_then(|s| s.as_str()).map(|s| s.to_string()),
        kind: "Mod".to_string(),
    })
}

fn parse_quilt_json(archive: &mut ZipArchive<File>, names: &HashSet<String>) -> Option<ModMetadata> {
    let content = {
        let mut file = archive.by_name("quilt.mod.json").ok()?;
        let mut c = String::new();
        file.read_to_string(&mut c).ok()?;
        c
    };

    let val: serde_json::Value = serde_json::from_str(&content).ok()?;
    let quilt = val.get("quilt_loader")?;
    let id = quilt.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let version = quilt.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let meta = quilt.get("metadata");
    let name = meta.and_then(|m| m.get("name")).and_then(|n| n.as_str()).unwrap_or(&id).to_string();
    let description = meta.and_then(|m| m.get("description")).and_then(|d| d.as_str()).map(|s| s.to_string());

    let mut dependencies = Vec::new();
    let mut mc_versions = Vec::new();

    if let Some(depends) = quilt.get("depends").and_then(|d| d.as_array()) {
        for item in depends {
            if let Some(dep_id) = item.get("id").and_then(|i| i.as_str()) {
                let ver_str = item.get("versions").and_then(|v| v.as_str()).map(|s| s.to_string());
                if dep_id == "minecraft" {
                    if let Some(ver) = ver_str {
                        mc_versions.push(ver);
                    }
                } else {
                    dependencies.push(ModDependency {
                        id: dep_id.to_string(),
                        version_requirement: ver_str,
                        required: true,
                    });
                }
            }
        }
    }

    let icon_data = extract_icon_data(archive, meta.and_then(|m| m.get("icon")), names);

    Some(ModMetadata {
        id,
        name,
        version: clean_version(&version),
        description,
        loaders: vec!["Quilt".to_string()],
        mc_versions,
        dependencies,
        icon_url: icon_data,
        homepage: None,
        source_url: None,
        kind: "Mod".to_string(),
    })
}

fn parse_forge_toml(
    archive: &mut ZipArchive<File>,
    entry_name: &str,
    names: &HashSet<String>,
) -> Option<ModMetadata> {
    let content = {
        let mut file = archive.by_name(entry_name).ok()?;
        let mut c = String::new();
        file.read_to_string(&mut c).ok()?;
        c
    };

    let toml_val: toml::Value = toml::from_str(&content).ok()?;
    let loader_name = if entry_name.contains("neoforge") { "NeoForge" } else { "Forge" };

    let mut mod_id = String::new();
    let mut name = String::new();
    let mut version = String::new();
    let mut description = None;
    let mut logo_file = None;

    if let Some(mods) = toml_val.get("mods").and_then(|m| m.as_array()) {
        if let Some(first) = mods.first() {
            mod_id = first.get("modId").and_then(|v| v.as_str()).unwrap_or("").to_string();
            name = first.get("displayName").and_then(|v| v.as_str()).unwrap_or(&mod_id).to_string();
            version = first.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string();
            description = first.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
            logo_file = first.get("logoFile").and_then(|v| v.as_str()).map(|s| s.to_string());
        }
    }

    let mut dependencies = Vec::new();
    let mut mc_versions = Vec::new();

    if let Some(dep_table) = toml_val.get("dependencies").and_then(|d| d.as_table()) {
        for (_, deps) in dep_table {
            if let Some(dep_list) = deps.as_array() {
                for dep in dep_list {
                    if let Some(dep_id) = dep.get("modId").and_then(|v| v.as_str()) {
                        let ver_range = dep.get("versionRange").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let mandatory = dep.get("mandatory").and_then(|v| v.as_bool()).unwrap_or(false);

                        if dep_id.eq_ignore_ascii_case("minecraft") {
                            if let Some(ver) = ver_range {
                                mc_versions.push(ver);
                            }
                        } else {
                            dependencies.push(ModDependency {
                                id: dep_id.to_string(),
                                version_requirement: ver_range,
                                required: mandatory,
                            });
                        }
                    }
                }
            }
        }
    }

    let icon_data = if let Some(ref logo) = logo_file {
        extract_raw_icon(archive, logo, names)
    } else {
        extract_fallback_icon(archive, names)
    };

    Some(ModMetadata {
        id: mod_id,
        name,
        version: clean_version(&version),
        description,
        loaders: vec![loader_name.to_string()],
        mc_versions,
        dependencies,
        icon_url: icon_data,
        homepage: None,
        source_url: None,
        kind: "Mod".to_string(),
    })
}

fn parse_mcmod_info(archive: &mut ZipArchive<File>, names: &HashSet<String>) -> Option<ModMetadata> {
    let content = {
        let mut file = archive.by_name("mcmod.info").ok()?;
        let mut c = String::new();
        file.read_to_string(&mut c).ok()?;
        c
    };

    let val: serde_json::Value = serde_json::from_str(&content).ok()?;
    let first = if let Some(arr) = val.as_array() {
        arr.first()?
    } else if let Some(modlist) = val.get("modList").and_then(|m| m.as_array()) {
        modlist.first()?
    } else {
        return None;
    };

    let id = first.get("modid").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let name = first.get("name").and_then(|v| v.as_str()).unwrap_or(&id).to_string();
    let version = first.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let description = first.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
    let mc_ver = first.get("mcversion").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let logo_file = first.get("logoFile").and_then(|v| v.as_str()).map(|s| s.to_string());
    let icon_data = if let Some(ref logo) = logo_file {
        extract_raw_icon(archive, logo, names)
    } else {
        extract_fallback_icon(archive, names)
    };

    let mut mc_versions = Vec::new();
    if !mc_ver.is_empty() {
        mc_versions.push(mc_ver);
    }

    Some(ModMetadata {
        id,
        name,
        version: clean_version(&version),
        description,
        loaders: vec!["Forge".to_string()],
        mc_versions,
        dependencies: Vec::new(),
        icon_url: icon_data,
        homepage: None,
        source_url: None,
        kind: "Mod".to_string(),
    })
}

fn parse_plugin_yml(archive: &mut ZipArchive<File>) -> Option<ModMetadata> {
    let mut file = archive.by_name("plugin.yml").ok()?;
    let mut content = String::new();
    file.read_to_string(&mut content).ok()?;

    let mut name = String::new();
    let mut version = String::new();
    let mut api_version = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once(':') {
            let key = k.trim().to_lowercase();
            let val = v.trim().trim_matches(['\'', '"']).to_string();
            if key == "name" {
                name = val;
            } else if key == "version" {
                version = val;
            } else if key == "api-version" {
                api_version = val;
            }
        }
    }

    let id = name.to_lowercase().replace(' ', "");
    let mut mc_versions = Vec::new();
    if !api_version.is_empty() {
        mc_versions.push(api_version);
    }

    Some(ModMetadata {
        id,
        name,
        version: clean_version(&version),
        description: Some("Bukkit/Spigot/Paper server plugin".to_string()),
        loaders: vec!["Bukkit/Spigot/Paper".to_string()],
        mc_versions,
        dependencies: Vec::new(),
        icon_url: None,
        homepage: None,
        source_url: None,
        kind: "Plugin".to_string(),
    })
}

fn fallback_metadata_from_filename(path: &Path) -> ModMetadata {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");
    let mc_ver = extract_minecraft_version_from_name(stem);
    let mut mc_versions = Vec::new();
    if let Some(ver) = mc_ver {
        mc_versions.push(ver);
    }

    ModMetadata {
        id: stem.to_lowercase().replace(' ', "-"),
        name: stem.to_string(),
        version: "1.0.0".to_string(),
        description: None,
        loaders: Vec::new(),
        mc_versions,
        dependencies: Vec::new(),
        icon_url: None,
        homepage: None,
        source_url: None,
        kind: "Mod".to_string(),
    }
}

fn extract_icon_data(
    archive: &mut ZipArchive<File>,
    icon_val: Option<&serde_json::Value>,
    names: &HashSet<String>,
) -> Option<String> {
    if let Some(val) = icon_val {
        if let Some(s) = val.as_str() {
            if let Some(data) = extract_raw_icon(archive, s, names) {
                return Some(data);
            }
        } else if let Some(obj) = val.as_object() {
            // Find highest resolution
            let mut entries: Vec<(&String, &serde_json::Value)> = obj.iter().collect();
            entries.sort_by_key(|(k, _)| k.parse::<u32>().unwrap_or(0));
            if let Some((_, v)) = entries.last() {
                if let Some(s) = v.as_str() {
                    if let Some(data) = extract_raw_icon(archive, s, names) {
                        return Some(data);
                    }
                }
            }
        }
    }
    extract_fallback_icon(archive, names)
}

fn extract_raw_icon(archive: &mut ZipArchive<File>, icon_path: &str, names: &HashSet<String>) -> Option<String> {
    let cleaned = icon_path.trim_start_matches('/').replace('\\', "/");
    let target_name = if names.contains(&cleaned) {
        cleaned
    } else {
        names
            .iter()
            .find(|n| n.ends_with(&format!("/{}", cleaned)))?
            .clone()
    };

    let mut entry = archive.by_name(&target_name).ok()?;
    if entry.size() > 2 * 1024 * 1024 {
        return None;
    }

    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes).ok()?;

    let mime = if target_name.ends_with(".png") {
        "image/png"
    } else if target_name.ends_with(".jpg") || target_name.ends_with(".jpeg") {
        "image/jpeg"
    } else {
        "image/png"
    };

    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Some(format!("data:{};base64,{}", mime, encoded))
}

fn extract_fallback_icon(archive: &mut ZipArchive<File>, names: &HashSet<String>) -> Option<String> {
    for fallback in &["icon.png", "logo.png", "pack.png"] {
        if names.contains(*fallback) {
            return extract_raw_icon(archive, fallback, names);
        }
    }
    None
}

fn clean_version(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("${version}") || trimmed.eq_ignore_ascii_case("@version@") {
        return "1.0.0".to_string();
    }
    trimmed.to_string()
}

fn extract_minecraft_version_from_name(name: &str) -> Option<String> {
    let re = Regex::new(r"(?i)(?:mc)?1\.\d+(?:\.\d+)?").ok()?;
    re.find(name).map(|m| {
        let s = m.as_str();
        if s.to_lowercase().starts_with("mc") {
            s[2..].to_string()
        } else {
            s.to_string()
        }
    })
}

/// Check mod compatibility with an instance's Minecraft version, loader, and available dependencies.
pub fn check_mod_compatibility(
    metadata: &ModMetadata,
    minecraft_version: &str,
    loader: &str,
    available_mod_ids: Option<&HashSet<String>>,
) -> (String, String) {
    if metadata.kind == "Plugin" {
        return (
            "Server plugin".to_string(),
            "plugin.yml was found. This is a Bukkit/Spigot/Paper server plugin, not a client mod.".to_string(),
        );
    }

    let norm_loader = loader.to_lowercase();
    let mod_loaders: Vec<String> = metadata.loaders.iter().map(|l| l.to_lowercase()).collect();

    // Loader check
    if (norm_loader == "fabric" || norm_loader == "quilt" || norm_loader == "forge" || norm_loader == "neoforge")
        && !mod_loaders.is_empty()
        && !mod_loaders.contains(&norm_loader)
    {
        return (
            "Incompatible".to_string(),
            format!("Metadata specifies loader: {}.", metadata.loaders.join(", ")),
        );
    }

    // Minecraft version check
    if !metadata.mc_versions.is_empty() {
        let matches = metadata.mc_versions.iter().any(|range| version_matches(minecraft_version, range));
        if !matches {
            return (
                "Incompatible".to_string(),
                format!(
                    "Minecraft version {} does not match range: {}.",
                    minecraft_version,
                    metadata.mc_versions.join(", ")
                ),
            );
        }
    }

    // Missing dependencies check
    if let Some(available) = available_mod_ids {
        let mut missing = Vec::new();
        for dep in &metadata.dependencies {
            if dep.required {
                let id_lower = dep.id.to_lowercase();
                if id_lower != "minecraft"
                    && id_lower != "fabricloader"
                    && id_lower != "forge"
                    && id_lower != "neoforge"
                    && !available.contains(&id_lower)
                {
                    missing.push(dep.id.clone());
                }
            }
        }
        if !missing.is_empty() {
            return (
                "Missing dependencies".to_string(),
                format!("Required dependencies were not selected: {}.", missing.join(", ")),
            );
        }
    }

    (
        "Likely compatible".to_string(),
        "Metadata found no conflict with Minecraft, the loader, or known required dependencies.".to_string(),
    )
}

/// Helper function to match Minecraft version string against basic range expressions.
pub fn version_matches(version: &str, range_expr: &str) -> bool {
    let v = version.trim();
    let r = range_expr.trim();

    if r.is_empty() || r == "*" || r == "any" {
        return true;
    }

    // Exact match
    if v == r {
        return true;
    }

    // Comma separated list (e.g. "1.20, 1.20.1")
    if r.contains(',') && !r.starts_with('[') && !r.starts_with('(') {
        return r.split(',').any(|part| version_matches(v, part.trim()));
    }

    // Starts with prefix e.g. "1.20" matching "1.20.1"
    if v.starts_with(r) || r.starts_with(v) {
        return true;
    }

    false
}
