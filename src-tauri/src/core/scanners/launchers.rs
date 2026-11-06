use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use regex::Regex;
use sha2::{Digest, Sha256};
use crate::models::instance::Instance;

/// Return standard candidate directories for TLauncher, Legacy Launcher, Minecraft, and CurseForge.
pub fn default_game_directory_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(appdata) = env::var("APPDATA") {
        let roaming = PathBuf::from(&appdata);
        candidates.push(roaming.join(".minecraft"));
        candidates.push(roaming.join("TLauncher").join("minecraft"));
        candidates.push(
            roaming
                .join(".tlauncher")
                .join("legacy")
                .join("Minecraft")
                .join("game"),
        );
    }

    if let Ok(userprofile) = env::var("USERPROFILE") {
        let home = PathBuf::from(userprofile);
        candidates.push(home.join(".minecraft"));
        candidates.push(
            home.join("curseforge")
                .join("minecraft")
                .join("Instances"),
        );
    }

    deduplicate_paths(candidates)
}

/// Return known candidate directories for Prism Launcher.
pub fn default_prism_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(appdata) = env::var("APPDATA") {
        candidates.push(PathBuf::from(&appdata).join("PrismLauncher"));
    }

    if let Ok(localappdata) = env::var("LOCALAPPDATA") {
        candidates.push(
            PathBuf::from(&localappdata)
                .join("Programs")
                .join("PrismLauncher"),
        );
    }

    if let Ok(userprofile) = env::var("USERPROFILE") {
        let home = PathBuf::from(userprofile);
        candidates.push(home.join("AppData").join("Roaming").join("PrismLauncher"));
        candidates.push(home.join("PrismLauncher"));
    }

    deduplicate_paths(candidates)
}

/// Find the standard Prism directory containing an instances folder.
pub fn find_default_prism_data_dir() -> Option<PathBuf> {
    for candidate in default_prism_candidates() {
        if candidate.join("instances").is_dir() {
            return Some(candidate);
        }
    }
    None
}

/// Check whether a folder resembles a Prism data directory.
pub fn is_prism_data_dir(path: &Path) -> bool {
    path.is_dir() && path.join("instances").is_dir()
}

/// Scan Prism instances and return structured records.
pub fn scan_prism_instances(prism_data_dir: &Path) -> Vec<Instance> {
    let instances_dir = prism_data_dir.join("instances");
    if !instances_dir.is_dir() {
        return Vec::new();
    }

    let mut results = Vec::new();
    if let Ok(entries) = fs::read_dir(&instances_dir) {
        let mut dirs: Vec<PathBuf> = entries
            .filter_map(|e| e.ok().map(|d| d.path()))
            .filter(|p| p.is_dir())
            .collect();
        dirs.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        for dir in dirs {
            if !dir.join("instance.cfg").is_file() && !dir.join("mmc-pack.json").is_file() {
                continue;
            }
            results.push(parse_prism_instance(&dir));
        }
    }

    results
}

/// Parse metadata for one Prism instance folder.
pub fn parse_prism_instance(instance_dir: &Path) -> Instance {
    let dir_name = instance_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let cfg_path = instance_dir.join("instance.cfg");
    let pack_path = instance_dir.join("mmc-pack.json");

    let cfg = read_instance_cfg(&cfg_path);
    let name = cfg.get("name").cloned().unwrap_or_else(|| dir_name.clone());
    let (minecraft_version, loader, loader_version) = read_mmc_pack(&pack_path);

    let status = if !cfg_path.is_file() {
        "instance.cfg is missing".to_string()
    } else if !pack_path.is_file() {
        "mmc-pack.json is missing".to_string()
    } else if minecraft_version == "Not detected" {
        "Could not detect version".to_string()
    } else {
        "Ready to check".to_string()
    };

    let icon = cfg.get("iconkey").cloned();

    Instance {
        id: dir_name,
        name,
        path: instance_dir.to_path_buf(),
        launcher: "Prism Launcher".to_string(),
        minecraft_version,
        loader,
        loader_version,
        status,
        icon,
    }
}

/// Determine a launcher label from a path.
pub fn launcher_for_game_directory(path: &Path) -> String {
    let path_str = path.to_string_lossy().to_lowercase();
    if path_str.contains(".tlauncher") || (path_str.contains("legacy") && path_str.contains("minecraft") && path_str.contains("game")) {
        "Legacy Launcher".to_string()
    } else if path_str.contains("tlauncher") {
        "TLauncher".to_string()
    } else if path_str.contains("curseforge") {
        "CurseForge".to_string()
    } else if path.file_name().map(|n| n.to_string_lossy().eq_ignore_ascii_case(".minecraft")).unwrap_or(false) {
        "Minecraft Launcher / TLauncher".to_string()
    } else {
        "Minecraft (user folder)".to_string()
    }
}

/// Check whether a folder looks like a Minecraft game directory.
pub fn is_game_directory(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    for sub in &["versions", "mods", "assets", "libraries", "home", "instances"] {
        if path.join(sub).is_dir() {
            return true;
        }
    }
    let name = path.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
    if name == "instances" && path.to_string_lossy().to_lowercase().contains("curseforge") {
        if let Ok(entries) = fs::read_dir(path) {
            return entries.filter_map(|e| e.ok()).any(|entry| {
                let p = entry.path();
                p.is_dir() && (p.join("manifest.json").is_file() || p.join("minecraft").is_dir())
            });
        }
    }
    if let Some(parent) = path.parent() {
        if parent.file_name().map(|n| n.to_string_lossy().eq_ignore_ascii_case("versions")).unwrap_or(false) {
            if let Ok(entries) = fs::read_dir(path) {
                return entries.filter_map(|e| e.ok()).any(|entry| {
                    entry.path().extension().map(|ext| ext.eq_ignore_ascii_case("json")).unwrap_or(false)
                });
            }
        }
    }
    false
}

/// Discover instances/environments within a directory.
pub fn discover_game_environments(path: &Path, launcher_override: Option<&str>) -> Vec<Instance> {
    if !path.is_dir() {
        return Vec::new();
    }
    let launcher = launcher_override
        .map(|s| s.to_string())
        .unwrap_or_else(|| launcher_for_game_directory(path));

    let mut env_paths: Vec<PathBuf> = Vec::new();

    // 1. instances subdirectory
    let instances_dir = if path.file_name().map(|n| n.to_string_lossy().eq_ignore_ascii_case("instances")).unwrap_or(false) {
        path.to_path_buf()
    } else {
        path.join("instances")
    };
    if instances_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&instances_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.is_dir()
                    && (p.join("mods").is_dir()
                        || p.join("config").is_dir()
                        || p.join("resourcepacks").is_dir()
                        || p.join("shaderpacks").is_dir()
                        || p.join("manifest.json").is_file()
                        || p.join("minecraft").is_dir())
                {
                    env_paths.push(p);
                }
            }
        }
    }

    // 2. home subdirectory (Legacy Launcher)
    let home_dir = if path.file_name().map(|n| n.to_string_lossy().eq_ignore_ascii_case("home")).unwrap_or(false) {
        path.to_path_buf()
    } else {
        path.join("home")
    };
    if home_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&home_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();
                if looks_like_home_environment(&p) {
                    env_paths.push(p);
                }
            }
        }
    }

    // 3. versions subdirectory
    let versions_dir = if path.file_name().map(|n| n.to_string_lossy().eq_ignore_ascii_case("versions")).unwrap_or(false) {
        path.to_path_buf()
    } else {
        path.join("versions")
    };
    if versions_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&versions_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();
                if !p.is_dir() {
                    continue;
                }
                let version_files = list_json_files(&p);
                let is_modded = version_files.iter().any(|f| {
                    let (_, loader, _) = parse_version_metadata(f);
                    !loader.is_empty() && loader != "Vanilla" && loader != "Not detected"
                });
                if p.join("mods").is_dir() || is_modded {
                    env_paths.push(p);
                }
            }
        }
    }

    // 4. User directly selected a version folder inside versions/
    if let Some(parent) = path.parent() {
        if parent.file_name().map(|n| n.to_string_lossy().eq_ignore_ascii_case("versions")).unwrap_or(false) && path.is_dir() {
            let version_files = list_json_files(path);
            let is_modded = version_files.iter().any(|f| {
                let (_, loader, _) = parse_version_metadata(f);
                !loader.is_empty() && loader != "Vanilla" && loader != "Not detected"
            });
            if is_modded || path.join("mods").is_dir() {
                env_paths.push(path.to_path_buf());
            }
        }
    }

    let mut unique_paths = deduplicate_paths(env_paths);

    // If .minecraft has a mods directory and no isolated instances, include .minecraft itself
    if path.file_name().map(|n| n.to_string_lossy().eq_ignore_ascii_case(".minecraft")).unwrap_or(false)
        && path.join("mods").is_dir()
        && unique_paths.is_empty()
    {
        unique_paths.push(path.to_path_buf());
    }

    if !unique_paths.is_empty() {
        unique_paths
            .into_iter()
            .map(|p| scan_game_directory(&p, &launcher))
            .collect()
    } else {
        vec![scan_game_directory(path, &launcher)]
    }
}

/// Scan a single game or environment directory into an Instance record.
pub fn scan_game_directory(path: &Path, launcher: &str) -> Instance {
    let (version, loader, loader_version, mut status) = metadata_for_environment(path);
    if version.starts_with("Multiple") {
        status = "Multiple versions found in the folder; choose a separate environment for precise management".to_string();
    }

    let resolved_lower = path.to_string_lossy().to_lowercase();
    let mut hasher = Sha256::new();
    hasher.update(resolved_lower.as_bytes());
    let hash_hex = hex::encode(hasher.finalize());
    let short_hash = &hash_hex[..12];

    let sanitized_launcher = launcher
        .to_lowercase()
        .replace([' ', '/'], "-");
    let instance_id = format!("{}-{}", sanitized_launcher, short_hash);

    let folder_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Minecraft");
    let name = format!("{}: {}", launcher, folder_name);

    Instance {
        id: instance_id,
        name,
        path: path.to_path_buf(),
        launcher: launcher.to_string(),
        minecraft_version: version,
        loader,
        loader_version,
        status,
        icon: None,
    }
}

// ----------------------------------------------------------------------------
// Internal Helpers
// ----------------------------------------------------------------------------

fn deduplicate_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut unique = Vec::new();
    let mut seen = HashSet::new();

    for path in paths {
        let key = path.to_string_lossy().to_lowercase();
        if seen.insert(key) {
            unique.push(path);
        }
    }

    unique
}

fn read_instance_cfg(path: &Path) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return map,
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') || trimmed.starts_with('[') {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once('=') {
            map.insert(k.trim().to_lowercase(), v.trim().to_string());
        }
    }
    map
}

fn read_mmc_pack(path: &Path) -> (String, String, String) {
    let mut mc_version = "Not detected".to_string();
    let mut loader = "Not detected".to_string();
    let mut loader_version = String::new();

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (mc_version, loader, loader_version),
    };

    let val: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return (mc_version, loader, loader_version),
    };

    let components = match val.get("components").and_then(|c| c.as_array()) {
        Some(arr) => arr,
        None => return (mc_version, loader, loader_version),
    };

    for comp in components {
        let uid = comp.get("uid").and_then(|u| u.as_str()).unwrap_or("").to_lowercase();
        let version = comp.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string();

        if uid == "net.minecraft" {
            if !version.is_empty() {
                mc_version = version;
            }
        } else if uid.starts_with("net.fabricmc.fabric-loader") {
            loader = "Fabric".to_string();
            loader_version = version;
        } else if uid.starts_with("net.minecraftforge") {
            loader = "Forge".to_string();
            loader_version = version;
        } else if uid.starts_with("net.neoforged") {
            loader = "NeoForge".to_string();
            loader_version = version;
        } else if uid.starts_with("org.quiltmc.quilt-loader") {
            loader = "Quilt".to_string();
            loader_version = version;
        } else if uid.starts_with("com.mumfrey.liteloader") {
            loader = "LiteLoader".to_string();
            loader_version = version;
        }
    }

    if loader == "Not detected" && mc_version != "Not detected" {
        loader = "Vanilla".to_string();
    }

    (mc_version, loader, loader_version)
}

fn looks_like_home_environment(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    for sub in &["mods", "config", "versions", ".minecraft", "minecraft"] {
        if path.join(sub).is_dir() {
            return true;
        }
    }
    if path.join("manifest.json").is_file() {
        return true;
    }
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if !guess_minecraft_version(name).is_empty() || !guess_loader(name).0.is_empty() {
        return true;
    }
    false
}

fn list_json_files(path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_file() && p.extension().map(|e| e.eq_ignore_ascii_case("json")).unwrap_or(false) {
                files.push(p);
            }
        }
    }
    files.sort();
    files
}

pub fn guess_minecraft_version(value: &str) -> String {
    // Check for 1.X.X first (standard Minecraft version format)
    let mc_re = Regex::new(r"(?:^|[^\d])(1\.\d+(?:\.\d+)?)(?:[^\d]|$)").unwrap();
    if let Some(caps) = mc_re.captures(value) {
        return caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
    }
    // Fallback for general \d+\.\d+
    let general_re = Regex::new(r"(?:^|[^\d])(\d+\.\d+(?:\.\d+)?)(?:[^\d]|$)").unwrap();
    if let Some(caps) = general_re.captures(value) {
        return caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
    }
    String::new()
}

pub fn guess_loader(version_id: &str) -> (String, String) {
    let lower = version_id.to_lowercase();
    let patterns = [
        (Regex::new(r"fabric-loader[-_]?([\w.+-]+?)-\d+\.\d+").unwrap(), "Fabric"),
        (Regex::new(r"forge[-_]?([\w.+-]+?)-\d+\.\d+").unwrap(), "Forge"),
        (Regex::new(r"neoforge[-_]?([\w.+-]+?)-\d+\.\d+").unwrap(), "NeoForge"),
        (Regex::new(r"quilt-loader[-_]?([\w.+-]+?)-\d+\.\d+").unwrap(), "Quilt"),
    ];

    for (re, name) in &patterns {
        if let Some(caps) = re.captures(&lower) {
            let ver = caps.get(1).map(|m| m.as_str().trim_matches(['-', '_']).to_string()).unwrap_or_default();
            return (name.to_string(), ver);
        }
    }

    for (marker, name) in &[
        ("fabric", "Fabric"),
        ("neoforge", "NeoForge"),
        ("forge", "Forge"),
        ("quilt", "Quilt"),
        ("optifine", "OptiFine"),
    ] {
        if lower.contains(marker) {
            return (name.to_string(), String::new());
        }
    }

    (String::new(), String::new())
}

pub fn loader_from_coordinate(coordinate: &str) -> (String, String) {
    let lower = coordinate.to_lowercase();
    let markers = [
        ("net.fabricmc:fabric-loader:", "Fabric"),
        ("net.minecraftforge:forge:", "Forge"),
        ("net.neoforged:neoforge:", "NeoForge"),
        ("org.quiltmc:quilt-loader:", "Quilt"),
    ];
    for (marker, name) in &markers {
        if lower.contains(marker) {
            let ver = coordinate.split(':').last().unwrap_or("").to_string();
            return (name.to_string(), ver);
        }
    }
    if lower.contains("optifine:optifine:") {
        let ver = coordinate.split(':').last().unwrap_or("").to_string();
        return ("OptiFine".to_string(), ver);
    }
    (String::new(), String::new())
}

pub fn parse_version_metadata(version_json: &Path) -> (String, String, String) {
    let content = match fs::read_to_string(version_json) {
        Ok(c) => c,
        Err(_) => return (String::new(), String::new(), String::new()),
    };

    let data: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return (String::new(), String::new(), String::new()),
    };

    let default_id = version_json
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let version_id = data.get("id").and_then(|v| v.as_str()).unwrap_or(&default_id).to_string();
    let mut mc_version = data.get("inheritsFrom").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if mc_version.is_empty() {
        mc_version = guess_minecraft_version(&version_id);
    }

    let (mut loader, mut loader_version) = guess_loader(&version_id);

    if let Some(libraries) = data.get("libraries").and_then(|l| l.as_array()) {
        for lib in libraries {
            if let Some(coord) = lib.get("name").and_then(|n| n.as_str()) {
                let (det_loader, det_version) = loader_from_coordinate(coord);
                if !det_loader.is_empty() {
                    loader = det_loader;
                    loader_version = det_version;
                    break;
                }
            }
        }
    }

    if loader.is_empty() {
        loader = if !mc_version.is_empty() { "Vanilla".to_string() } else { "Not detected".to_string() };
    }

    let final_mc_ver = if mc_version.is_empty() { "Not detected".to_string() } else { mc_version };
    (final_mc_ver, loader, loader_version)
}

pub fn parse_curseforge_metadata(path: &Path) -> Option<(String, String, String, String)> {
    let manifest = path.join("manifest.json");
    if !manifest.is_file() {
        return None;
    }

    let content = fs::read_to_string(&manifest).ok()?;
    let data: serde_json::Value = serde_json::from_str(&content).ok()?;
    let minecraft = data.get("minecraft")?;
    let version = minecraft.get("version")?.as_str()?.to_string();

    let mut loader = if !version.is_empty() { "Vanilla".to_string() } else { "Not detected".to_string() };
    let mut loader_version = String::new();

    if let Some(mod_loaders) = minecraft.get("modLoaders").and_then(|m| m.as_array()) {
        for item in mod_loaders {
            if let Some(id) = item.get("id").and_then(|i| i.as_str()) {
                let id_lower = id.to_lowercase();
                let markers = [
                    ("fabric-loader", "Fabric"),
                    ("quilt-loader", "Quilt"),
                    ("neoforge", "NeoForge"),
                    ("forge", "Forge"),
                ];

                let mut matched = false;
                for (marker, name) in &markers {
                    if id_lower == *marker || id_lower.starts_with(&format!("{}-", marker)) {
                        loader = name.to_string();
                        loader_version = id[marker.len()..].trim_start_matches('-').to_string();
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    if let Some((l, v)) = id.split_once('-') {
                        loader = l.to_string();
                        loader_version = v.to_string();
                    } else {
                        loader = id.to_string();
                    }
                }
                break;
            }
        }
    }

    if version.is_empty() {
        return None;
    }

    Some((
        version,
        loader,
        loader_version,
        "Metadata read from CurseForge manifest.json".to_string(),
    ))
}

pub fn metadata_for_environment(path: &Path) -> (String, String, String, String) {
    if let Some(cf) = parse_curseforge_metadata(path) {
        return cf;
    }

    let direct_jsons = list_json_files(path);
    let mut files = direct_jsons;
    if files.is_empty() {
        let nested_versions = path.join("versions");
        if nested_versions.is_dir() {
            if let Ok(entries) = fs::read_dir(&nested_versions) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let sub = entry.path();
                    if sub.is_dir() {
                        files.extend(list_json_files(&sub));
                    }
                }
            }
        }
    }

    let detected: Vec<(String, String, String)> = files
        .iter()
        .map(|f| parse_version_metadata(f))
        .filter(|(ver, _, _)| !ver.is_empty() && ver != "Not detected")
        .collect();

    if !detected.is_empty() {
        let mut versions: Vec<String> = detected.iter().map(|(v, _, _)| v.clone()).collect();
        versions.dedup();
        let mut loaders: Vec<String> = detected.iter().map(|(_, l, _)| l.clone()).collect();
        loaders.dedup();

        let version = if versions.len() == 1 {
            versions[0].clone()
        } else {
            format!("Multiple versions ({})", versions.len())
        };

        let loader = if loaders.len() == 1 {
            loaders[0].clone()
        } else {
            "Mixed".to_string()
        };

        let mut loader_versions: Vec<String> = detected
            .iter()
            .filter(|(_, l, v)| l == &loader && !v.is_empty())
            .map(|(_, _, v)| v.clone())
            .collect();
        loader_versions.dedup();
        let loader_version_str = loader_versions.into_iter().take(3).collect::<Vec<_>>().join(", ");

        return (
            version,
            loader,
            loader_version_str,
            "Metadata read from environment JSON".to_string(),
        );
    }

    let folder_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    let guessed_ver = guess_minecraft_version(folder_name);
    let (guessed_loader, guessed_loader_ver) = guess_loader(folder_name);

    if !guessed_ver.is_empty() {
        let final_loader = if guessed_loader.is_empty() { "Vanilla".to_string() } else { guessed_loader };
        return (
            guessed_ver,
            final_loader,
            guessed_loader_ver,
            "Version detected from folder name".to_string(),
        );
    }

    (
        "Not detected".to_string(),
        "Not detected".to_string(),
        String::new(),
        "Version not detected; check the folder metadata".to_string(),
    )
}
