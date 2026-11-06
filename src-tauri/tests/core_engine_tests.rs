use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;
use zip::write::{SimpleFileOptions, ZipWriter};

use tauri_app_lib::core::diagnostics::log_analyzer::{
    analyze_log_content, read_log_text_safely,
};
use tauri_app_lib::core::scanners::launchers::{
    guess_loader, guess_minecraft_version, loader_from_coordinate, parse_curseforge_metadata,
    parse_version_metadata,
};
use tauri_app_lib::core::scanners::mod_parser::{
    check_mod_compatibility, parse_mod_jar, scan_mods_dir_parallel, version_matches,
};
use tauri_app_lib::core::virtualization::engine::{
    apply_profile, preview_profile, rollback_last,
};
use tauri_app_lib::models::mod_file::{ModDependency, ModMetadata};

// ============================================================================
// Scanners Tests
// ============================================================================

#[test]
fn test_guess_minecraft_version() {
    assert_eq!(guess_minecraft_version("fabric-loader-0.15.7-1.20.1"), "1.20.1");
    assert_eq!(guess_minecraft_version("1.19.2-forge-43.2.0"), "1.19.2");
    assert_eq!(guess_minecraft_version("release 1.21"), "1.21");
    assert_eq!(guess_minecraft_version("no_version_here"), "");
}

#[test]
fn test_guess_loader() {
    let (loader, ver) = guess_loader("fabric-loader-0.15.7-1.20.1");
    assert_eq!(loader, "Fabric");
    assert_eq!(ver, "0.15.7");

    let (loader, ver) = guess_loader("forge-47.2.0-1.20.1");
    assert_eq!(loader, "Forge");
    assert_eq!(ver, "47.2.0");

    let (loader, _) = guess_loader("optifine_1.20.1_hd_u_i6");
    assert_eq!(loader, "OptiFine");
}

#[test]
fn test_loader_from_coordinate() {
    let (loader, ver) = loader_from_coordinate("net.fabricmc:fabric-loader:0.15.11");
    assert_eq!(loader, "Fabric");
    assert_eq!(ver, "0.15.11");

    let (loader, ver) = loader_from_coordinate("net.neoforged:neoforge:20.4.190");
    assert_eq!(loader, "NeoForge");
    assert_eq!(ver, "20.4.190");
}

#[test]
fn test_parse_version_metadata() {
    let dir = tempdir().unwrap();
    let json_file = dir.path().join("1.20.1-Fabric.json");

    let content = r#"{
        "id": "1.20.1-Fabric",
        "inheritsFrom": "1.20.1",
        "libraries": [
            { "name": "net.fabricmc:fabric-loader:0.15.7" }
        ]
    }"#;
    fs::write(&json_file, content).unwrap();

    let (mc_ver, loader, loader_ver) = parse_version_metadata(&json_file);
    assert_eq!(mc_ver, "1.20.1");
    assert_eq!(loader, "Fabric");
    assert_eq!(loader_ver, "0.15.7");
}

#[test]
fn test_parse_curseforge_metadata() {
    let dir = tempdir().unwrap();
    let manifest_file = dir.path().join("manifest.json");

    let content = r#"{
        "minecraft": {
            "version": "1.20.1",
            "modLoaders": [
                { "id": "forge-47.2.0", "primary": true }
            ]
        }
    }"#;
    fs::write(&manifest_file, content).unwrap();

    let meta = parse_curseforge_metadata(dir.path()).unwrap();
    assert_eq!(meta.0, "1.20.1");
    assert_eq!(meta.1, "Forge");
    assert_eq!(meta.2, "47.2.0");
}

// ============================================================================
// Mod Parser & Compatibility Tests
// ============================================================================

#[test]
fn test_version_matches() {
    assert!(version_matches("1.20.1", "1.20.1"));
    assert!(version_matches("1.20.1", "*"));
    assert!(version_matches("1.20.1", "1.20, 1.20.1"));
    assert!(version_matches("1.20.1", "1.20"));
    assert!(!version_matches("1.20.1", "1.19.2"));
}

#[test]
fn test_check_mod_compatibility() {
    let meta = ModMetadata {
        id: "sodium".to_string(),
        name: "Sodium".to_string(),
        version: "0.5.8".to_string(),
        description: None,
        loaders: vec!["Fabric".to_string()],
        mc_versions: vec!["1.20.1".to_string()],
        dependencies: vec![ModDependency {
            id: "fabric-api".to_string(),
            version_requirement: None,
            required: true,
        }],
        icon_url: None,
        homepage: None,
        source_url: None,
        kind: "Mod".to_string(),
    };

    // Compatible case
    let mut available = HashSet::new();
    available.insert("fabric-api".to_string());
    let (status, _) = check_mod_compatibility(&meta, "1.20.1", "Fabric", Some(&available));
    assert_eq!(status, "Likely compatible");

    // Incompatible loader
    let (status, _) = check_mod_compatibility(&meta, "1.20.1", "Forge", Some(&available));
    assert_eq!(status, "Incompatible");

    // Incompatible MC version
    let (status, _) = check_mod_compatibility(&meta, "1.19.2", "Fabric", Some(&available));
    assert_eq!(status, "Incompatible");

    // Missing dependency
    let empty_available = HashSet::new();
    let (status, _) = check_mod_compatibility(&meta, "1.20.1", "Fabric", Some(&empty_available));
    assert_eq!(status, "Missing dependencies");
}

#[test]
fn test_parse_mod_jar_fabric() {
    let dir = tempdir().unwrap();
    let jar_path = dir.path().join("test_mod.jar");

    // Create dummy ZIP with fabric.mod.json
    {
        let file = File::create(&jar_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        zip.start_file("fabric.mod.json", options).unwrap();
        let fabric_json = r#"{
            "schemaVersion": 1,
            "id": "my_mod",
            "name": "My Cool Mod",
            "version": "1.2.3",
            "description": "Awesome fabric mod",
            "depends": {
                "minecraft": ">=1.20.1"
            }
        }"#;
        zip.write_all(fabric_json.as_bytes()).unwrap();
        zip.finish().unwrap();
    }

    let parsed = parse_mod_jar(&jar_path).unwrap();
    assert_eq!(parsed.file_name, "test_mod.jar");
    assert_eq!(parsed.metadata.id, "my_mod");
    assert_eq!(parsed.metadata.name, "My Cool Mod");
    assert_eq!(parsed.metadata.version, "1.2.3");
    assert_eq!(parsed.metadata.loaders, vec!["Fabric"]);
    assert_eq!(parsed.metadata.mc_versions, vec![">=1.20.1"]);
    assert!(!parsed.sha1.is_empty());
    assert!(parsed.sha256.is_some());
}

#[test]
fn test_scan_mods_dir_parallel() {
    let dir = tempdir().unwrap();

    // Create 3 dummy jars
    for i in 1..=3 {
        let jar_path = dir.path().join(format!("mod_{}.jar", i));
        let file = File::create(&jar_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        zip.start_file("fabric.mod.json", options).unwrap();
        let fabric_json = format!(r#"{{ "id": "mod_{}", "name": "Mod {}", "version": "1.0.0" }}"#, i, i);
        zip.write_all(fabric_json.as_bytes()).unwrap();
        zip.finish().unwrap();
    }

    let mods = scan_mods_dir_parallel(dir.path());
    assert_eq!(mods.len(), 3);
}

// ============================================================================
// Virtualization Tests
// ============================================================================

#[test]
fn test_virtualization_preview_and_apply() {
    let temp = tempdir().unwrap();
    let library_dir = temp.path().join("library");
    let instance_dir = temp.path().join("instance");
    let mods_dir = instance_dir.join("mods");
    let backup_dir = temp.path().join("backup");

    fs::create_dir_all(&library_dir).unwrap();
    fs::create_dir_all(&mods_dir).unwrap();
    fs::create_dir_all(&backup_dir).unwrap();

    // Create mod in library
    let lib_mod = library_dir.join("jei.jar");
    {
        let file = File::create(&lib_mod).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        zip.start_file("fabric.mod.json", options).unwrap();
        zip.write_all(b"{\"id\":\"jei\",\"name\":\"JEI\",\"version\":\"15.0.0\"}").unwrap();
        zip.finish().unwrap();
    }

    let parsed_lib = parse_mod_jar(&lib_mod).unwrap();
    let mod_hash = parsed_lib.sha256.unwrap();

    // 1. Preview
    let plan = preview_profile(
        &instance_dir,
        "Test Profile",
        &[mod_hash.clone()],
        &library_dir,
        Some("1.20.1"),
        Some("Fabric"),
    );
    assert!(plan.errors.is_empty());
    assert!(plan.can_apply);
    assert_eq!(plan.actions.len(), 1);
    assert_eq!(plan.actions[0].action, "Create link");
    assert_eq!(plan.actions[0].filename, "jei.jar");

    // 2. Apply
    let result = apply_profile(
        &instance_dir,
        "Test Profile",
        &[mod_hash.clone()],
        &library_dir,
        Some(&backup_dir),
        Some("1.20.1"),
        Some("Fabric"),
    ).unwrap();

    assert!(result.success);
    assert_eq!(result.created_count, 1);
    assert!(mods_dir.join("jei.jar").exists());

    // 3. Rollback
    let rollback_msg = rollback_last(Some(&backup_dir), Some(&library_dir)).unwrap();
    assert!(rollback_msg.contains("The last operation was undone"));
    assert!(!mods_dir.join("jei.jar").exists());
}

// ============================================================================
// Diagnostics Tests
// ============================================================================

#[test]
fn test_diagnostics_out_of_memory() {
    let dummy_path = std::path::PathBuf::from("crash-2026-11-06.txt");
    let content = r#"
---- Minecraft Crash Report ----
Description: Rendering screen
java.lang.OutOfMemoryError: Java heap space
    at java.base/java.util.Arrays.copyOf(Arrays.java:3537)
"#;
    let res = analyze_log_content(content, &dummy_path, "crash_report", None);
    assert_eq!(res.category, "out_of_memory");
    assert_eq!(res.exception_class, "java.lang.OutOfMemoryError");
    assert!(res.recommendation.contains("Increase allocated RAM"));
}

#[test]
fn test_diagnostics_java_version() {
    let dummy_path = std::path::PathBuf::from("latest.log");
    let content = r#"
Exception in thread "main" java.lang.UnsupportedClassVersionError: com/example/mymod/Main has been compiled by a more recent version of the Java Runtime (class file version 65.0), this compiler only recognizes class file versions up to 61.0
"#;
    let res = analyze_log_content(content, &dummy_path, "log", None);
    assert_eq!(res.category, "java_version");
    assert!(res.summary.contains("requires Java 21"));
    assert!(res.recommendation.contains("Java 21"));
}

#[test]
fn test_diagnostics_missing_dependency() {
    let dummy_path = std::path::PathBuf::from("latest.log");
    let content = r#"
Incomplete reindexing:
Mod 'sodium' (0.5.8) requires fabricloader >=0.15.0, but only 0.14.21 is present!
"#;
    let res = analyze_log_content(content, &dummy_path, "log", None);
    assert_eq!(res.category, "missing_dependency");
    assert!(res.title.contains("Missing Mod Dependencies"));
}

#[test]
fn test_diagnostics_duplicate_mod() {
    let dummy_path = std::path::PathBuf::from("latest.log");
    let content = r#"
[main/ERROR]: Duplicate mods found with id 'cloth-config': [cloth-config-11.1.118.jar, cloth-config-11.0.106.jar]
"#;
    let res = analyze_log_content(content, &dummy_path, "log", None);
    assert_eq!(res.category, "duplicate_mod");
    assert_eq!(res.suspected_mods, vec!["cloth-config"]);
}

#[test]
fn test_diagnostics_mixin_conflict() {
    let dummy_path = std::path::PathBuf::from("crash-2026-11-06.txt");
    let content = r#"
---- Minecraft Crash Report ----
Description: Initializing game
org.spongepowered.asm.mixin.transformer.throwables.MixinTransformerError: An unexpected critical error was encountered in class [net.minecraft.client.renderer.LevelRenderer]
    at me.jellysquid.mods.sodium.client.render.SodiumWorldRenderer.init(SodiumWorldRenderer.java:120)
"#;
    let res = analyze_log_content(content, &dummy_path, "crash_report", None);
    assert_eq!(res.category, "mixin_conflict");
    assert!(res.suspected_mods.contains(&"Sodium".to_string()));
}

#[test]
fn test_diagnostics_caused_by() {
    let dummy_path = std::path::PathBuf::from("crash-2026-11-06.txt");
    let content = r#"
Minecraft Crash Report:
Time: 2026-11-06 20:00:00
Minecraft Version: 1.20.1
Java Version: 17.0.8

Caused by: java.lang.NullPointerException: Cannot read field "world"
    at com.simibubi.create.content.kinetics.base.KineticBlockEntity.tick(KineticBlockEntity.java:45)
"#;
    let res = analyze_log_content(content, &dummy_path, "crash_report", None);
    assert_eq!(res.category, "mod_exception");
    assert_eq!(res.exception_class, "java.lang.NullPointerException");
    assert_eq!(res.minecraft_version, "1.20.1");
    assert!(res.suspected_mods.contains(&"Create".to_string()));
}

#[test]
fn test_read_log_text_safely() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.log");

    let text = "Line 1\nLine 2\nLine 3\nLine 4\n";
    fs::write(&file_path, text).unwrap();

    let read_text = read_log_text_safely(&file_path, 1024);
    assert_eq!(read_text, text);
}

// ============================================================================
// Phase 3 Tests: Modpacks & User Data Storage
// ============================================================================

use tauri_app_lib::core::modpacks::modpack_service::{
    export_profile_to_modpack, inspect_modpack, is_safe_relative_path, ModpackExportOptions,
};
use tauri_app_lib::core::storage::user_data_service::{
    delete_profile, load_user_data, save_profile, save_settings, set_mod_tags,
};
use tauri_app_lib::models::profile::Profile;
use tauri_app_lib::models::user_data::AppSettings;

#[test]
fn test_is_safe_relative_path() {
    let dir = tempdir().unwrap();
    let base = dir.path();

    assert!(is_safe_relative_path(base, "config/mod.toml"));
    assert!(is_safe_relative_path(base, "sub/dir/nested.json"));
    assert!(!is_safe_relative_path(base, "../escaping.txt"));
    assert!(!is_safe_relative_path(base, "sub/../../escaping.txt"));
    assert!(!is_safe_relative_path(base, "/absolute/path"));
}

#[test]
fn test_modpack_inspect_morgpack() {
    let dir = tempdir().unwrap();
    let archive_path = dir.path().join("sample.morgpack");

    {
        let file = File::create(&archive_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        zip.start_file("modpack.json", options).unwrap();
        let json_data = r#"{
            "format_version": 1,
            "name": "Super Pack",
            "minecraft_version": "1.20.1",
            "loader": "Fabric",
            "loader_version": "0.15.7",
            "has_configs": true,
            "mods": [
                { "filename": "sodium.jar", "sha256": "abc" }
            ]
        }"#;
        zip.write_all(json_data.as_bytes()).unwrap();
        zip.finish().unwrap();
    }

    let inspect = inspect_modpack(&archive_path).unwrap();
    assert_eq!(inspect.format, "morgpack");
    assert_eq!(inspect.name, "Super Pack");
    assert_eq!(inspect.minecraft_version, "1.20.1");
    assert_eq!(inspect.loader, "Fabric");
    assert_eq!(inspect.loader_version, "0.15.7");
    assert!(inspect.has_configs);
    assert_eq!(inspect.mod_filenames, vec!["sodium.jar"]);
}

#[test]
fn test_modpack_inspect_mrpack() {
    let dir = tempdir().unwrap();
    let archive_path = dir.path().join("modrinth_pack.mrpack");

    {
        let file = File::create(&archive_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        zip.start_file("modrinth.index.json", options).unwrap();
        let json_data = r#"{
            "formatVersion": 1,
            "game": "minecraft",
            "name": "Fabulously Optimized",
            "versionId": "1.0.0",
            "dependencies": {
                "minecraft": "1.20.1",
                "fabric-loader": "0.15.11"
            },
            "files": [
                {
                    "path": "mods/sodium-fabric-0.5.8.jar",
                    "hashes": { "sha1": "12345" }
                }
            ]
        }"#;
        zip.write_all(json_data.as_bytes()).unwrap();
        zip.finish().unwrap();
    }

    let inspect = inspect_modpack(&archive_path).unwrap();
    assert_eq!(inspect.format, "modrinth");
    assert_eq!(inspect.name, "Fabulously Optimized");
    assert_eq!(inspect.minecraft_version, "1.20.1");
    assert_eq!(inspect.loader, "fabric");
    assert_eq!(inspect.loader_version, "0.15.11");
    assert_eq!(inspect.mod_filenames, vec!["sodium-fabric-0.5.8.jar"]);
}

#[test]
fn test_modpack_export_profile() {
    let dir = tempdir().unwrap();
    let library_dir = dir.path().join("library");
    fs::create_dir_all(&library_dir).unwrap();

    let jar_path = library_dir.join("test_mod.jar");
    {
        let file = File::create(&jar_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        zip.start_file("fabric.mod.json", options).unwrap();
        zip.write_all(b"{\"id\":\"test_mod\",\"name\":\"Test Mod\",\"version\":\"1.0.0\"}").unwrap();
        zip.finish().unwrap();
    }
    let parsed_mod = parse_mod_jar(&jar_path).unwrap();

    let profile = Profile {
        id: "prof_123".to_string(),
        name: "My Cool Profile".to_string(),
        instance_id: Some("inst_1".to_string()),
        instance_path: None,
        minecraft_version: Some("1.20.1".to_string()),
        loader: Some("Fabric".to_string()),
        loader_version: Some("0.15.7".to_string()),
        enabled_mod_hashes: vec![parsed_mod.sha256.clone().unwrap()],
        enabled_resourcepack_hashes: Vec::new(),
        enabled_shaderpack_hashes: Vec::new(),
        active_mods: vec![parsed_mod.sha256.clone().unwrap()],
        active_assets: Vec::new(),
        created_at: String::new(),
        updated_at: String::new(),
    };

    let output_archive = dir.path().join("exported.morgpack");
    let options = ModpackExportOptions {
        include_configs: false,
        include_resourcepacks: false,
        include_shaderpacks: false,
        include_saves: false,
        selected_saves: None,
    };

    let report = export_profile_to_modpack(&profile, &[parsed_mod], &output_archive, &options).unwrap();
    assert_eq!(report.mods_count, 1);
    assert!(output_archive.exists());

    // Verify inspecting exported pack
    let inspect = inspect_modpack(&output_archive).unwrap();
    assert_eq!(inspect.format, "morgpack");
    assert_eq!(inspect.name, "My Cool Profile");
    assert_eq!(inspect.mod_count, 1);
}

#[test]
fn test_user_data_store_persistence() {
    let dir = tempdir().unwrap();
    let user_data_path = dir.path().join("user_data.json");

    let mut store = load_user_data(Some(&user_data_path));
    assert_eq!(store.profiles.len(), 0);

    // Add profile
    let prof = Profile {
        id: "p1".to_string(),
        name: "Profile Alpha".to_string(),
        instance_id: None,
        instance_path: None,
        minecraft_version: Some("1.20.1".to_string()),
        loader: Some("Fabric".to_string()),
        loader_version: None,
        enabled_mod_hashes: vec!["hash123".to_string()],
        enabled_resourcepack_hashes: Vec::new(),
        enabled_shaderpack_hashes: Vec::new(),
        active_mods: vec!["hash123".to_string()],
        active_assets: Vec::new(),
        created_at: String::new(),
        updated_at: String::new(),
    };

    let saved = save_profile(&mut store, prof, Some(&user_data_path)).unwrap();
    assert_eq!(saved.name, "Profile Alpha");

    // Add tags
    set_mod_tags(&mut store, "hash123", vec!["optimization".to_string(), "client".to_string()], Some(&user_data_path)).unwrap();

    // Save settings
    let mut settings = AppSettings::default();
    settings.locale = "ru".to_string();
    settings.close_launcher_on_apply = true;
    save_settings(&mut store, settings, Some(&user_data_path)).unwrap();

    // Reload from file to ensure atomic disk persistence
    let reloaded = load_user_data(Some(&user_data_path));
    assert_eq!(reloaded.profiles.len(), 1);
    assert_eq!(reloaded.profiles[0].name, "Profile Alpha");
    assert_eq!(reloaded.settings.locale, "ru");
    assert!(reloaded.settings.close_launcher_on_apply);
    assert_eq!(reloaded.mod_tags.get("hash123").unwrap(), &vec!["optimization".to_string(), "client".to_string()]);

    // Delete profile
    let deleted = delete_profile(&mut store, "p1", Some(&user_data_path)).unwrap();
    assert!(deleted);

    let reloaded_after_delete = load_user_data(Some(&user_data_path));
    assert_eq!(reloaded_after_delete.profiles.len(), 0);
}
