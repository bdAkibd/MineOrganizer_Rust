use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrashAnalysis {
    pub file_path: PathBuf,
    pub file_type: String, // "crash_report" or "log"
    pub timestamp: String,
    pub title: String,
    pub category: String, // "out_of_memory", "java_version", "missing_dependency", "mixin_conflict", "duplicate_mod", "driver_opengl", "mod_exception", "unknown"
    pub summary: String,
    pub recommendation: String,
    pub suspected_mods: Vec<String>,
    pub exception_class: String,
    pub exception_message: String,
    pub minecraft_version: String,
    pub java_version: String,
    pub raw_text: String,
}

pub fn get_known_package_mods() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert("me.jellysquid.mods.sodium", "Sodium");
    map.insert("net.coderbot.iris", "Iris Shaders");
    map.insert("me.jellysquid.mods.lithium", "Lithium");
    map.insert("me.jellysquid.mods.phosphor", "Phosphor");
    map.insert("me.jellysquid.mods.indium", "Indium");
    map.insert("link.infra.indium", "Indium");
    map.insert("net.fabricmc.fabric", "Fabric API");
    map.insert("com.mojang.blaze3d", "Minecraft Engine (OpenGL)");
    map.insert("vazkii.botania", "Botania");
    map.insert("vazkii.quark", "Quark");
    map.insert("mezz.jei", "JEI (Just Enough Items)");
    map.insert("appeng", "Applied Energistics 2");
    map.insert("blusunrize.immersiveengineering", "Immersive Engineering");
    map.insert("mekanism", "Mekanism");
    map.insert("com.simibubi.create", "Create");
    map.insert("pie.ilikepiefoo.quark", "Quark");
    map.insert("dev.latvian.mods", "Latvian Mod");
    map.insert("embeddedt.embeddium", "Embeddium");
    map.insert("org.embeddedt.embeddium", "Embeddium");
    map.insert("xaero", "Xaero's Minimap / World Map");
    map.insert("journeymap", "JourneyMap");
    map
}

/// Find the game root folder containing logs, crash-reports, or mods.
pub fn find_game_root(instance_path: &Path) -> PathBuf {
    for candidate in &[
        instance_path.join(".minecraft"),
        instance_path.join("minecraft"),
        instance_path.to_path_buf(),
    ] {
        if candidate.join("logs").is_dir()
            || candidate.join("crash-reports").is_dir()
            || candidate.join("mods").is_dir()
        {
            return candidate.clone();
        }
    }
    instance_path.to_path_buf()
}

/// List available crash reports and log files, sorted by newest first.
pub fn list_available_log_files(instance_path: &Path) -> Vec<(String, PathBuf)> {
    let game_root = find_game_root(instance_path);
    let mut entries = Vec::new();

    // 1. Crash reports
    let crash_dir = game_root.join("crash-reports");
    if crash_dir.is_dir() {
        if let Ok(read_dir) = fs::read_dir(&crash_dir) {
            for entry in read_dir.filter_map(|e| e.ok()) {
                let p = entry.path();
                let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                if p.is_file() && name.starts_with("crash-") && name.ends_with(".txt") {
                    entries.push((format!("Crash Report: {}", name), p));
                }
            }
        }
    }

    // 2. Logs
    let logs_dir = game_root.join("logs");
    if logs_dir.is_dir() {
        for name in &["latest.log", "debug.log"] {
            let log_file = logs_dir.join(name);
            if log_file.is_file() {
                entries.push((format!("Log: {}", name), log_file));
            }
        }
    }

    // Sort newest first
    entries.sort_by(|a, b| {
        let mtime_a = a.1.metadata().and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH);
        let mtime_b = b.1.metadata().and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH);
        mtime_b.cmp(&mtime_a)
    });

    entries
}

pub const MAX_LOG_READ_BYTES: u64 = 2 * 1024 * 1024; // 2 MB

/// Safely read log or crash text, limiting large files to the tail where error traces appear.
pub fn read_log_text_safely(path: &Path, max_bytes: u64) -> String {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return String::new(),
    };

    let len = match file.metadata().map(|m| m.len()) {
        Ok(l) => l,
        Err(_) => return String::new(),
    };

    if len <= max_bytes {
        let mut content = String::new();
        let _ = file.read_to_string(&mut content);
        return content;
    }

    // Seek to tail
    if file.seek(SeekFrom::Start(len - max_bytes)).is_err() {
        return String::new();
    }

    let mut buf = Vec::new();
    if file.read_to_end(&mut buf).is_err() {
        return String::new();
    }

    // Find next newline to avoid cutting a line in half
    let start_idx = buf.iter().position(|&b| b == b'\n').map(|i| i + 1).unwrap_or(0);
    let slice = &buf[start_idx..];
    let tail_text = String::from_utf8_lossy(slice);

    format!(
        "[... Large log file ({:.1} MB). Displaying latest entries ...]\n\n{}",
        len as f64 / (1024.0 * 1024.0),
        tail_text
    )
}

/// Analyze raw log or crash text into a structured diagnostic outcome.
pub fn analyze_log_content(
    content: &str,
    file_path: &Path,
    file_type: &str,
    known_mod_names: Option<&[String]>,
) -> CrashAnalysis {
    let mut analysis = CrashAnalysis {
        file_path: file_path.to_path_buf(),
        file_type: file_type.to_string(),
        timestamp: String::new(),
        title: String::new(),
        category: "unknown".to_string(),
        summary: String::new(),
        recommendation: String::new(),
        suspected_mods: Vec::new(),
        exception_class: String::new(),
        exception_message: String::new(),
        minecraft_version: String::new(),
        java_version: String::new(),
        raw_text: content.to_string(),
    };

    // 1. System details extraction
    let mc_ver_re = Regex::new(r"Minecraft Version:\s*([^\r\n]+)").unwrap();
    if let Some(caps) = mc_ver_re.captures(content) {
        analysis.minecraft_version = caps[1].trim().to_string();
    }

    let java_ver_re = Regex::new(r"Java Version:\s*([^\r\n]+)").unwrap();
    if let Some(caps) = java_ver_re.captures(content) {
        analysis.java_version = caps[1].trim().to_string();
    }

    let time_re = Regex::new(r"Time:\s*([^\r\n]+)").unwrap();
    if let Some(caps) = time_re.captures(content) {
        analysis.timestamp = caps[1].trim().to_string();
    }

    let desc_re = Regex::new(r"Description:\s*([^\r\n]+)").unwrap();
    if let Some(caps) = desc_re.captures(content) {
        analysis.title = caps[1].trim().to_string();
    }

    // 2. Suspected mods section
    let suspected_section_re = Regex::new(r"(?is)(?:-- Suspected Mods --|Suspected Mods:)(.*?)(?:\n\s*\n|--|\z)").unwrap();
    if let Some(caps) = suspected_section_re.captures(content) {
        let raw_section = &caps[1];
        let name_clean_re = Regex::new(r"^([^()]+)").unwrap();
        for line in raw_section.lines() {
            let cleaned = line.trim().trim_start_matches(['-', '\t', '*', ' ']);
            if cleaned.is_empty() {
                continue;
            }
            let name_part = cleaned.split(',').next().unwrap_or("").trim();
            if let Some(m) = name_clean_re.captures(name_part) {
                let mod_label = m[1].trim().to_string();
                if !mod_label.is_empty() && !analysis.suspected_mods.contains(&mod_label) {
                    analysis.suspected_mods.push(mod_label);
                }
            }
        }
    }

    // 3. Out of memory check
    if content.contains("java.lang.OutOfMemoryError") || content.contains("GC overhead limit exceeded") {
        analysis.category = "out_of_memory".to_string();
        if analysis.title.is_empty() {
            analysis.title = "Out of Memory (Java heap space)".to_string();
        }
        analysis.exception_class = "java.lang.OutOfMemoryError".to_string();
        analysis.summary = "The game ran out of allocated memory (RAM).".to_string();
        analysis.recommendation = "Increase allocated RAM in your launcher settings (for example, allocate 4 GB to 6 GB). Ensure you are running 64-bit Java.".to_string();
        return analysis;
    }

    // 4. Java version mismatch
    let java_class_re = Regex::new(r"(?s)UnsupportedClassVersionError:.*?compiled by a more recent version of the Java Runtime \(class file version (\d+)\.0\).*?only recognizes.*?up to (\d+)\.0").unwrap();
    if let Some(caps) = java_class_re.captures(content) {
        let req_ver: u32 = caps[1].parse().unwrap_or(0);
        let cur_ver: u32 = caps[2].parse().unwrap_or(0);

        let class_to_java = |class_ver: u32| -> String {
            match class_ver {
                52 => "8".to_string(),
                55 => "11".to_string(),
                60 => "16".to_string(),
                61 => "17".to_string(),
                65 => "21".to_string(),
                69 => "25".to_string(),
                _ => format!("class {}", class_ver),
            }
        };

        let req_java = class_to_java(req_ver);
        let cur_java = class_to_java(cur_ver);

        analysis.category = "java_version".to_string();
        analysis.title = "Java Version Incompatibility".to_string();
        analysis.exception_class = "java.lang.UnsupportedClassVersionError".to_string();
        analysis.summary = format!(
            "A mod requires Java {}, but Minecraft was launched using Java {}.",
            req_java, cur_java
        );
        analysis.recommendation = format!(
            "Install and select Java {} (or newer) in your launcher settings.",
            req_java
        );
        return analysis;
    }

    // 5. Missing dependencies
    let missing_deps_re = Regex::new(r#"(?is)(?:Some of your mods are incomplete and require additional mods|Missing or unsupported mandatory dependencies|Mod\s+['"][^'"]+['"][^\r\n]*requires\s+[^\r\n]*|Mod\s+[^\r\n]*requires\s+[^\r\n]*)(.*?)(?:\n\s*\n|--|\z)"#).unwrap();
    if let Some(caps) = missing_deps_re.captures(content) {
        let full_match = caps.get(0).unwrap().as_str().to_lowercase();
        if full_match.contains("requires") || full_match.contains("missing") {
            analysis.category = "missing_dependency".to_string();
            analysis.title = "Missing Mod Dependencies".to_string();
            analysis.summary = "The game failed to launch because required mod dependencies are missing.".to_string();
            let matched_text = caps.get(0).unwrap().as_str();
            let snippet = &matched_text[..std::cmp::min(300, matched_text.len())];
            analysis.recommendation = format!("Install the missing dependencies listed below:\n{}", snippet.trim());

            let req_re = Regex::new(r#"requires\s+['"]?([a-zA-Z0-9_\-]+)"#).unwrap();
            for m in req_re.captures_iter(matched_text) {
                let dep_name = m[1].to_string();
                let lower = dep_name.to_lowercase();
                if lower != "minecraft" && lower != "fabric" && lower != "forge" && lower != "neoforge" && !analysis.suspected_mods.contains(&dep_name) {
                    analysis.suspected_mods.push(dep_name);
                }
            }
            return analysis;
        }
    }

    // 6. Duplicate mod detection
    let dup_re = Regex::new(r#"(?i)(?:Duplicate mods? found with id|Duplicate mods? found for|Duplicate mod ID|Duplicate mods? found)[\s:"']+([a-zA-Z0-9_\-]+)"#).unwrap();
    if let Some(caps) = dup_re.captures(content) {
        let dup_mod = caps[1].to_string();
        analysis.category = "duplicate_mod".to_string();
        analysis.title = format!("Duplicate Mod Detected: {}", dup_mod);
        analysis.summary = format!("Multiple files with the same mod ID \"{}\" were found in your mods folder.", dup_mod);
        analysis.recommendation = format!("Remove older or duplicate jar files for \"{}\" from the mods directory.", dup_mod);
        if !analysis.suspected_mods.contains(&dup_mod) {
            analysis.suspected_mods.push(dup_mod);
        }
        return analysis;
    }

    // 7. Graphics / OpenGL driver error
    if content.contains("org.lwjgl.opengl.OpenGLException")
        || content.contains("GLFW error 65542")
        || content.contains("WGL: The driver does not appear to support OpenGL")
    {
        analysis.category = "driver_opengl".to_string();
        analysis.title = "Graphics / OpenGL Driver Error".to_string();
        analysis.summary = "The graphics driver failed to initialize OpenGL.".to_string();
        analysis.recommendation = "Update your graphics card drivers (NVIDIA, AMD, or Intel). If you have dual GPUs, ensure the game is set to run on the dedicated GPU.".to_string();
        return analysis;
    }

    // 8. Mixin transformer / injection error
    if content.contains("org.spongepowered.asm.mixin") {
        analysis.category = "mixin_conflict".to_string();
        if analysis.title.is_empty() {
            analysis.title = "Mixin Injection Conflict".to_string();
        }
        analysis.summary = "A mod failed to inject bytecode into the game (Mixin conflict).".to_string();

        let mixin_re = Regex::new(r"in class \[(.*?)\]").unwrap();
        let target_info = mixin_re
            .captures(content)
            .map(|c| format!(" in {}", &c[1]))
            .unwrap_or_default();

        let known_pkgs = get_known_package_mods();
        for (pkg, mod_name) in known_pkgs {
            if content.contains(pkg) && !analysis.suspected_mods.iter().any(|m| m == mod_name) {
                analysis.suspected_mods.push(mod_name.to_string());
            }
        }

        if !analysis.suspected_mods.is_empty() {
            let names_str = analysis.suspected_mods[..std::cmp::min(3, analysis.suspected_mods.len())].join(", ");
            analysis.recommendation = format!(
                "Conflict detected involving {}{}. Try updating these mods to the latest version or test removing them one by one.",
                names_str, target_info
            );
        } else {
            analysis.recommendation = format!(
                "Two or more mods conflict while modifying the same game code{}. Check for mod updates or remove recently added mods.",
                target_info
            );
        }
        return analysis;
    }

    // 9. Caused by: exception extraction
    let caused_by_re = Regex::new(r"Caused by:\s*([a-zA-Z0-9_\.]+(?:Exception|Error|Throwable))(?::\s*([^\r\n]+))?").unwrap();
    let mut last_cause = None;
    for caps in caused_by_re.captures_iter(content) {
        let ex_class = caps[1].to_string();
        let ex_msg = caps.get(2).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
        last_cause = Some((ex_class, ex_msg));
    }

    if let Some((ex_class, ex_msg)) = last_cause {
        analysis.category = "mod_exception".to_string();
        analysis.exception_class = ex_class.clone();
        analysis.exception_message = ex_msg.clone();
        let short_class = ex_class.split('.').last().unwrap_or(&ex_class);
        analysis.title = format!("Crash caused by {}", short_class);
        analysis.summary = if !ex_msg.is_empty() {
            format!("{}: {}", ex_class, ex_msg)
        } else {
            ex_class
        };

        let known_pkgs = get_known_package_mods();
        for (pkg, mod_name) in known_pkgs {
            if content.contains(pkg) && !analysis.suspected_mods.iter().any(|m| m == mod_name) {
                analysis.suspected_mods.push(mod_name.to_string());
            }
        }

        if let Some(known_mods) = known_mod_names {
            for mod_n in known_mods {
                if mod_n.len() > 3 && content.to_lowercase().contains(&mod_n.to_lowercase()) && !analysis.suspected_mods.contains(mod_n) {
                    analysis.suspected_mods.push(mod_n.clone());
                }
            }
        }

        if !analysis.suspected_mods.is_empty() {
            let names_str = analysis.suspected_mods[..std::cmp::min(4, analysis.suspected_mods.len())].join(", ");
            analysis.recommendation = format!(
                "The crash trace indicates an issue related to: {}. Check that these mods match your Minecraft and loader version.",
                names_str
            );
        } else {
            analysis.recommendation = "Review the full stack trace below to identify the failing mod or library.".to_string();
        }
        return analysis;
    }

    // 10. Fallback for logs without standard crash header
    if file_type == "log" {
        let fatal_lines: Vec<&str> = content
            .lines()
            .map(|l| l.trim())
            .filter(|l| l.contains("[FATAL]") || l.contains("[ERROR]"))
            .collect();
        if let Some(last_error) = fatal_lines.last() {
            analysis.category = "mod_exception".to_string();
            analysis.title = "Errors Found in Log".to_string();
            let summary_slice = &last_error[..std::cmp::min(200, last_error.len())];
            analysis.summary = summary_slice.to_string();
            analysis.recommendation = "Review the highlighted error lines in the log viewer.".to_string();
            return analysis;
        }
    }

    analysis.category = "unknown".to_string();
    if analysis.title.is_empty() {
        analysis.title = "Unknown Game Crash".to_string();
    }
    analysis.summary = "No standard crash signatures were matched.".to_string();
    analysis.recommendation = "Examine the raw log or crash report for details.".to_string();
    analysis
}

/// Automatically find the latest crash report or log for an instance and analyze it.
pub fn analyze_instance_logs(
    instance_path: &Path,
    known_mod_names: Option<&[String]>,
) -> Option<CrashAnalysis> {
    let log_files = list_available_log_files(instance_path);
    if log_files.is_empty() {
        return None;
    }

    let target_pair = log_files
        .iter()
        .find(|(_, p)| p.file_name().unwrap_or_default().to_string_lossy().contains("crash-"))
        .unwrap_or(&log_files[0]);

    let (_, file_path) = target_pair;
    let content = read_log_text_safely(file_path, MAX_LOG_READ_BYTES);
    if content.is_empty() {
        return None;
    }

    let file_type = if file_path.file_name().unwrap_or_default().to_string_lossy().contains("crash-") {
        "crash_report"
    } else {
        "log"
    };

    Some(analyze_log_content(&content, file_path, file_type, known_mod_names))
}
