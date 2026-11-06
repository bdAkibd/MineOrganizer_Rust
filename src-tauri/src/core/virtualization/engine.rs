use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::core::scanners::mod_parser::scan_mods_dir_parallel;
use crate::core::virtualization::rollback_guard::RollbackGuard;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VirtualizationAction {
    pub filename: String,
    pub action: String, // "Create link", "Remove link", "Keep link", "Keep external file", "Conflict", "Not in library", "No changes"
    pub details: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VirtualizationPlan {
    pub profile_name: String,
    pub instance_path: PathBuf,
    pub mods_dir: Option<PathBuf>,
    pub actions: Vec<VirtualizationAction>,
    pub errors: Vec<String>,
    pub can_apply: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualizationResult {
    pub success: bool,
    pub message: String,
    pub created_count: usize,
    pub removed_count: usize,
    pub manifest_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestRecord {
    pub path: String,
    pub target: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackManifest {
    pub id: String,
    pub profile_name: String,
    pub instance_path: String,
    pub mods_dir: String,
    pub created_links: Vec<ManifestRecord>,
    pub removed_links: Vec<ManifestRecord>,
    pub completed: bool,
    pub rolled_back: bool,
    #[serde(default)]
    pub rollback_errors: Vec<String>,
}

pub fn default_backup_dir() -> PathBuf {
    if let Ok(local_appdata) = env::var("LOCALAPPDATA") {
        PathBuf::from(local_appdata).join("MineOrganizer").join("rollback")
    } else if let Ok(home) = env::var("USERPROFILE") {
        PathBuf::from(home).join(".mineorganizer").join("rollback")
    } else {
        PathBuf::from("./rollback")
    }
}

/// Find the appropriate mods folder for an instance or environment.
pub fn find_mods_dir(
    instance_path: &Path,
    mc_version: Option<&str>,
    loader: Option<&str>,
) -> Option<PathBuf> {
    let name_lower = instance_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    // 1. If it's the root .minecraft, look for isolated versions first
    if name_lower == ".minecraft" {
        let versions_dir = instance_path.join("versions");
        let mut isolated = Vec::new();
        if versions_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&versions_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let p = entry.path();
                    if !p.is_dir() {
                        continue;
                    }
                    let folder_lower = p.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                    let has_modded = folder_lower.contains("fabric")
                        || folder_lower.contains("forge")
                        || folder_lower.contains("neoforge")
                        || folder_lower.contains("quilt")
                        || folder_lower.contains("optifine");
                    let mods_sub = p.join("mods");
                    if mods_sub.is_dir() || has_modded {
                        isolated.push(mods_sub);
                    }
                }
            }
        }

        if !isolated.is_empty() {
            let ver = mc_version.unwrap_or("").to_lowercase();
            let lod = loader.unwrap_or("").to_lowercase();
            let mut matching = Vec::new();

            for m in &isolated {
                let parent_name = m.parent().unwrap().file_name().unwrap().to_string_lossy().to_lowercase();
                let ver_match = ver.is_empty() || parent_name.contains(&ver);
                let lod_match = lod.is_empty() || parent_name.contains(&lod);
                if ver_match && lod_match {
                    matching.push(m.clone());
                }
            }

            if matching.len() == 1 {
                return Some(matching[0].clone());
            }
            if isolated.len() == 1 {
                return Some(isolated[0].clone());
            }
            // Ambiguous
            return None;
        }
    }

    // 2. If instance_path is inside versions/<version>
    if let Some(parent) = instance_path.parent() {
        if parent.file_name().map(|n| n.to_string_lossy().eq_ignore_ascii_case("versions")).unwrap_or(false) {
            let name = instance_path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
            if name.contains("fabric") || name.contains("forge") || name.contains("neoforge") || name.contains("quilt") || name.contains("optifine") {
                return Some(instance_path.join("mods"));
            }
        }
    }

    // 3. Candidates: .minecraft/mods, minecraft/mods, or mods
    for cand in &[
        instance_path.join(".minecraft").join("mods"),
        instance_path.join("minecraft").join("mods"),
        instance_path.join("mods"),
    ] {
        if cand.is_dir() {
            return Some(cand.clone());
        }
    }

    // Fallback: standard instance_path/mods
    Some(instance_path.join("mods"))
}

/// Preview profile actions without modifying any files.
pub fn preview_profile(
    instance_path: &Path,
    profile_name: &str,
    desired_mod_hashes: &[String],
    library_dir: &Path,
    mc_version: Option<&str>,
    loader: Option<&str>,
) -> VirtualizationPlan {
    let mods_dir = find_mods_dir(instance_path, mc_version, loader);
    let mut actions = Vec::new();
    let mut errors = Vec::new();

    if !instance_path.is_dir() {
        errors.push(format!("Instance directory not found: {}", instance_path.display()));
        return VirtualizationPlan {
            profile_name: profile_name.to_string(),
            instance_path: instance_path.to_path_buf(),
            mods_dir,
            actions,
            errors,
            can_apply: false,
        };
    }

    let target_mods_dir = match &mods_dir {
        Some(d) => d.clone(),
        None => {
            errors.push("Could not determine this environment's mods folder unambiguously.".to_string());
            return VirtualizationPlan {
                profile_name: profile_name.to_string(),
                instance_path: instance_path.to_path_buf(),
                mods_dir: None,
                actions,
                errors,
                can_apply: false,
            };
        }
    };

    let library_mods = scan_mods_dir_parallel(library_dir);
    let mut by_sha256 = HashMap::new();
    let mut by_sha1 = HashMap::new();
    for m in &library_mods {
        if let Some(ref h) = m.sha256 {
            by_sha256.insert(h.clone(), m.clone());
        }
        by_sha1.insert(m.sha1.clone(), m.clone());
    }

    let mut desired = Vec::new();
    let mut missing_hashes = Vec::new();

    for h in desired_mod_hashes {
        if let Some(m) = by_sha256.get(h).or_else(|| by_sha1.get(h)) {
            desired.push(m.clone());
        } else {
            missing_hashes.push(h.clone());
        }
    }

    for missing in missing_hashes {
        let short_hash = if missing.len() > 12 { &missing[..12] } else { &missing };
        actions.push(VirtualizationAction {
            filename: short_hash.to_string(),
            action: "Not in library".to_string(),
            details: "Mod selected in profile but missing from library".to_string(),
            path: target_mods_dir.join(short_hash),
        });
        errors.push(format!("Selected mod missing from library: {}", short_hash));
    }

    let desired_by_name: HashMap<String, _> = desired
        .iter()
        .map(|m| (m.file_name.to_lowercase(), m.clone()))
        .collect();

    let mut current_by_name = HashSet::new();

    if target_mods_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&target_mods_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();
                let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                let name_lower = name.to_lowercase();
                if !name_lower.ends_with(".jar") {
                    continue;
                }
                current_by_name.insert(name_lower.clone());

                if is_library_link(&p, library_dir) {
                    let resolved = fs::canonicalize(&p).unwrap_or_else(|_| p.clone());
                    if let Some(wanted) = desired_by_name.get(&name_lower) {
                        let wanted_resolved = fs::canonicalize(&wanted.path).unwrap_or_else(|_| wanted.path.clone());
                        if resolved == wanted_resolved {
                            actions.push(VirtualizationAction {
                                filename: name,
                                action: "Keep link".to_string(),
                                details: "Already matches profile".to_string(),
                                path: p,
                            });
                        } else {
                            actions.push(VirtualizationAction {
                                filename: name,
                                action: "Remove link".to_string(),
                                details: "Managed link to a different library mod version".to_string(),
                                path: p,
                            });
                        }
                    } else {
                        actions.push(VirtualizationAction {
                            filename: name,
                            action: "Remove link".to_string(),
                            details: "Managed link to mod not selected in profile".to_string(),
                            path: p,
                        });
                    }
                } else if let Some(wanted) = desired_by_name.get(&name_lower) {
                    actions.push(VirtualizationAction {
                        filename: name.clone(),
                        action: "Conflict".to_string(),
                        details: format!("A regular file already exists; will not overwrite. Expected {}", wanted.file_name),
                        path: p,
                    });
                    errors.push(format!("Name conflict with existing unmanaged file: {}", name));
                } else {
                    actions.push(VirtualizationAction {
                        filename: name,
                        action: "Keep external file".to_string(),
                        details: "Not managed by MineOrganizer and will not be changed".to_string(),
                        path: p,
                    });
                }
            }
        }
    }

    for mod_file in desired {
        let name_lower = mod_file.file_name.to_lowercase();
        if !current_by_name.contains(&name_lower) {
            actions.push(VirtualizationAction {
                filename: mod_file.file_name.clone(),
                action: "Create link".to_string(),
                details: format!("Link to {}", mod_file.path.display()),
                path: target_mods_dir.join(&mod_file.file_name),
            });
        }
    }

    if actions.is_empty() {
        actions.push(VirtualizationAction {
            filename: "—".to_string(),
            action: "No changes".to_string(),
            details: "Folder already matches the profile".to_string(),
            path: target_mods_dir.clone(),
        });
    }

    let can_apply = errors.is_empty()
        && actions.iter().any(|a| a.action == "Create link" || a.action == "Remove link");

    VirtualizationPlan {
        profile_name: profile_name.to_string(),
        instance_path: instance_path.to_path_buf(),
        mods_dir: Some(target_mods_dir),
        actions,
        errors,
        can_apply,
    }
}

/// Apply a virtualization plan transactionally with automatic rollback on error.
pub fn apply_profile(
    instance_path: &Path,
    profile_name: &str,
    desired_mod_hashes: &[String],
    library_dir: &Path,
    backup_dir: Option<&Path>,
    mc_version: Option<&str>,
    loader: Option<&str>,
) -> Result<VirtualizationResult, String> {
    let plan = preview_profile(
        instance_path,
        profile_name,
        desired_mod_hashes,
        library_dir,
        mc_version,
        loader,
    );

    if !plan.errors.is_empty() {
        return Err(plan.errors.join("\n"));
    }

    let mods_dir = plan.mods_dir.ok_or_else(|| "Target mods directory could not be resolved.".to_string())?;

    if !plan.can_apply {
        return Ok(VirtualizationResult {
            success: true,
            message: "No changes required; environment is up to date.".to_string(),
            created_count: 0,
            removed_count: 0,
            manifest_path: None,
        });
    }

    let bdir = backup_dir.map(|p| p.to_path_buf()).unwrap_or_else(default_backup_dir);
    fs::create_dir_all(&bdir).map_err(|e| format!("Failed to create backup dir: {}", e))?;
    fs::create_dir_all(&mods_dir).map_err(|e| format!("Failed to create mods dir: {}", e))?;

    let manifest_id = Uuid::new_v4().to_string();
    let manifest_path = bdir.join(format!("{}.json", manifest_id));

    let mut manifest = RollbackManifest {
        id: manifest_id,
        profile_name: profile_name.to_string(),
        instance_path: instance_path.to_string_lossy().to_string(),
        mods_dir: mods_dir.to_string_lossy().to_string(),
        created_links: Vec::new(),
        removed_links: Vec::new(),
        completed: false,
        rolled_back: false,
        rollback_errors: Vec::new(),
    };

    write_manifest(&manifest_path, &manifest)?;

    let mut guard = RollbackGuard::new();

    // 1. Process removals
    let remove_actions: Vec<_> = plan.actions.iter().filter(|a| a.action == "Remove link").collect();
    for action in remove_actions {
        if !is_library_link(&action.path, library_dir) {
            return Err(format!("File link changed unexpectedly before execution: {}", action.path.display()));
        }

        let target = if is_symlink(&action.path) {
            fs::canonicalize(&action.path).unwrap_or_else(|_| action.path.clone())
        } else {
            library_dir.join(&action.filename)
        };

        let kind = if is_symlink(&action.path) { "symlink" } else { "hardlink" }.to_string();

        manifest.removed_links.push(ManifestRecord {
            path: action.path.to_string_lossy().to_string(),
            target: target.to_string_lossy().to_string(),
            kind: kind.clone(),
        });
        write_manifest(&manifest_path, &manifest)?;

        guard.record_remove(action.path.clone(), target, kind);

        fs::remove_file(&action.path).map_err(|e| format!("Failed to remove link {}: {}", action.path.display(), e))?;
    }

    // 2. Process creations
    let library_mods = scan_mods_dir_parallel(library_dir);
    let mut lib_by_name = HashMap::new();
    for m in library_mods {
        lib_by_name.insert(m.file_name.to_lowercase(), m);
    }

    let create_actions: Vec<_> = plan.actions.iter().filter(|a| a.action == "Create link").collect();
    for action in create_actions {
        let destination = &action.path;
        if destination.exists() || is_symlink(destination) {
            return Err(format!("Destination path already exists: {}", destination.display()));
        }

        let mod_file = lib_by_name
            .get(&action.filename.to_lowercase())
            .ok_or_else(|| format!("Mod file not found in library: {}", action.filename))?;

        let kind = create_reference_link(&mod_file.path, destination)
            .map_err(|e| format!("Failed to create link for {}: {}", action.filename, e))?;

        manifest.created_links.push(ManifestRecord {
            path: destination.to_string_lossy().to_string(),
            target: mod_file.path.to_string_lossy().to_string(),
            kind: kind.clone(),
        });
        write_manifest(&manifest_path, &manifest)?;

        guard.record_create(destination.clone(), mod_file.path.clone(), kind);
    }

    // Transaction committed successfully
    let created_count = manifest.created_links.len();
    let removed_count = manifest.removed_links.len();

    guard.commit();

    manifest.completed = true;
    write_manifest(&manifest_path, &manifest)?;

    Ok(VirtualizationResult {
        success: true,
        message: format!(
            "Profile applied successfully. Created {} link(s), removed {} link(s).",
            created_count, removed_count
        ),
        created_count,
        removed_count,
        manifest_path: Some(manifest_path),
    })
}

/// Undo the most recent successful virtualization operation.
pub fn rollback_last(
    backup_dir: Option<&Path>,
    _library_dir: Option<&Path>,
) -> Result<String, String> {
    let bdir = backup_dir.map(|p| p.to_path_buf()).unwrap_or_else(default_backup_dir);
    if !bdir.is_dir() {
        return Err("No saved operations are available for rollback.".to_string());
    }

    let mut manifests: Vec<(std::time::SystemTime, PathBuf, RollbackManifest)> = Vec::new();
    if let Ok(entries) = fs::read_dir(&bdir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.extension().map(|ext| ext.eq_ignore_ascii_case("json")).unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&p) {
                    if let Ok(man) = serde_json::from_str::<RollbackManifest>(&content) {
                        if man.completed && !man.rolled_back {
                            let mtime = entry.metadata().and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH);
                            manifests.push((mtime, p, man));
                        }
                    }
                }
            }
        }
    }

    if manifests.is_empty() {
        return Err("No applied operations are available for rollback.".to_string());
    }

    manifests.sort_by_key(|(mtime, _, _)| *mtime);
    let (_, manifest_path, mut manifest) = manifests.pop().unwrap();

    let mut guard = RollbackGuard::new();
    for rec in &manifest.created_links {
        guard.record_create(PathBuf::from(&rec.path), PathBuf::from(&rec.target), rec.kind.clone());
    }
    for rec in &manifest.removed_links {
        guard.record_remove(PathBuf::from(&rec.path), PathBuf::from(&rec.target), rec.kind.clone());
    }

    let errors = guard.execute_rollback();
    manifest.rolled_back = errors.is_empty();
    manifest.rollback_errors = errors.clone();
    let _ = write_manifest(&manifest_path, &manifest);

    if !errors.is_empty() {
        return Err(format!("Rollback was not fully completed:\n{}", errors.join("\n")));
    }

    Ok(format!("The last operation was undone for profile \"{}\".", manifest.profile_name))
}

// ----------------------------------------------------------------------------
// Helpers
// ----------------------------------------------------------------------------

fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

fn is_library_link(path: &Path, library_dir: &Path) -> bool {
    if !path.exists() && !is_symlink(path) {
        return false;
    }

    if is_symlink(path) {
        if let Ok(canonical) = fs::canonicalize(path) {
            if let Ok(lib_canonical) = fs::canonicalize(library_dir) {
                return canonical.starts_with(&lib_canonical) && canonical.is_file();
            }
        }
        return false;
    }

    let candidate = library_dir.join(path.file_name().unwrap_or_default());
    if candidate.is_file() {
        if let (Ok(m1), Ok(m2)) = (path.metadata(), candidate.metadata()) {
            if m1.len() == m2.len() {
                if let (Ok(h1), Ok(h2)) = (compute_file_sha256(path), compute_file_sha256(&candidate)) {
                    return h1 == h2;
                }
            }
        }
    }

    false
}

fn compute_file_sha256(path: &Path) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 32 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn create_reference_link(source: &Path, destination: &Path) -> std::io::Result<String> {
    #[cfg(windows)]
    {
        if std::os::windows::fs::symlink_file(source, destination).is_ok() {
            return Ok("symlink".to_string());
        }
    }
    #[cfg(unix)]
    {
        if std::os::unix::fs::symlink(source, destination).is_ok() {
            return Ok("symlink".to_string());
        }
    }

    if fs::hard_link(source, destination).is_ok() {
        return Ok("hardlink".to_string());
    }

    fs::copy(source, destination).map(|_| "copy".to_string())
}

fn write_manifest(path: &Path, manifest: &RollbackManifest) -> Result<(), String> {
    let tmp = path.with_extension("tmp");
    let json = serde_json::to_string_pretty(manifest).map_err(|e| e.to_string())?;
    fs::write(&tmp, json).map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())?;
    Ok(())
}
