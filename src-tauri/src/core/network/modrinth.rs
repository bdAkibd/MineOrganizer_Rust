use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

pub const APP_USER_AGENT: &str = "MineOrganizer (https://github.com/MainOrganizer)";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModrinthSearchResult {
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub author: String,
    #[serde(default)]
    pub icon_url: String,
    #[serde(default)]
    pub downloads: u64,
    #[serde(default)]
    pub follows: u64,
    #[serde(default)]
    pub versions: Vec<String>,
    #[serde(default)]
    pub loaders: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthSearchResponse {
    pub hits: Vec<ModrinthSearchResult>,
    pub total_hits: usize,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModrinthVersionFile {
    pub url: String,
    pub filename: String,
    #[serde(default)]
    pub primary: bool,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub sha1: String,
    #[serde(default)]
    pub sha512: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModrinthDependency {
    pub project_id: Option<String>,
    pub version_id: Option<String>,
    #[serde(default = "default_dep_type")]
    pub dependency_type: String, // "required", "optional", "incompatible"
    #[serde(default)]
    pub file_name: String,
}

fn default_dep_type() -> String {
    "required".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModrinthVersion {
    pub version_id: String,
    pub project_id: String,
    pub name: String,
    pub version_number: String,
    #[serde(default)]
    pub game_versions: Vec<String>,
    #[serde(default)]
    pub loaders: Vec<String>,
    #[serde(default)]
    pub files: Vec<ModrinthVersionFile>,
    #[serde(default)]
    pub dependencies: Vec<ModrinthDependency>,
    #[serde(default)]
    pub changelog: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModUpdateInfo {
    pub file_hash: String,
    pub mod_name: String,
    pub current_version: String,
    pub new_version_number: String,
    pub new_version_name: String,
    pub download_url: String,
    pub filename: String,
    pub size: u64,
    pub changelog: String,
}

fn create_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))
}

/// Search Modrinth projects matching query, loader, and Minecraft version.
pub async fn search_modrinth(
    query: &str,
    loader: &str,
    minecraft_version: &str,
    limit: usize,
    offset: usize,
) -> Result<ModrinthSearchResponse, String> {
    let client = create_client()?;

    let mut facets: Vec<Vec<String>> = vec![vec!["project_type:mod".to_string()]];

    let ldr_clean = loader.trim().to_lowercase();
    if !ldr_clean.is_empty() && ldr_clean != "all" && ldr_clean != "all loaders" && ldr_clean != "—" {
        facets.push(vec![format!("categories:{}", ldr_clean)]);
    }

    let mc_clean = minecraft_version.trim().to_lowercase();
    if !mc_clean.is_empty() && mc_clean != "all" && mc_clean != "all versions" && mc_clean != "—" {
        facets.push(vec![format!("versions:{}", mc_clean)]);
    }

    let facets_json = serde_json::to_string(&facets).map_err(|e| e.to_string())?;

    let sort_index = if query.trim().is_empty() {
        "downloads"
    } else {
        "relevance"
    };

    let res = client
        .get("https://api.modrinth.com/v2/search")
        .query(&[
            ("query", query),
            ("index", sort_index),
            ("limit", &limit.to_string()),
            ("offset", &offset.to_string()),
            ("facets", &facets_json),
        ])
        .send()
        .await
        .map_err(|e| format!("Modrinth search network request failed: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("Modrinth API error: status {}", res.status()));
    }

    let body: serde_json::Value = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse Modrinth search JSON: {}", e))?;

    let total_hits = body.get("total_hits").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let resp_offset = body.get("offset").and_then(|v| v.as_u64()).unwrap_or(offset as u64) as usize;
    let resp_limit = body.get("limit").and_then(|v| v.as_u64()).unwrap_or(limit as u64) as usize;

    let hits = body.get("hits").and_then(|h| h.as_array()).ok_or("No hits in response")?;

    let mut results = Vec::new();
    for hit in hits {
        let project_id = hit.get("project_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let slug = hit.get("slug").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let title = hit.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let description = hit.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let author = hit.get("author").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let icon_url = hit.get("icon_url").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let downloads = hit.get("downloads").and_then(|v| v.as_u64()).unwrap_or(0);
        let follows = hit.get("follows").and_then(|v| v.as_u64()).unwrap_or(0);

        let versions: Vec<String> = hit
            .get("versions")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        let loaders: Vec<String> = hit
            .get("categories")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        results.push(ModrinthSearchResult {
            project_id,
            slug,
            title,
            description,
            author,
            icon_url,
            downloads,
            follows,
            versions,
            loaders,
        });
    }

    Ok(ModrinthSearchResponse {
        hits: results,
        total_hits,
        offset: resp_offset,
        limit: resp_limit,
    })
}

/// Fetch version list for a specific project filtered by loader and version.
pub async fn get_project_versions(
    project_id_or_slug: &str,
    loader: &str,
    minecraft_version: &str,
) -> Result<Vec<ModrinthVersion>, String> {
    let client = create_client()?;
    let url = format!("https://api.modrinth.com/v2/project/{}/version", project_id_or_slug);

    let mut req = client.get(&url);

    let ldr_clean = loader.trim().to_lowercase();
    if !ldr_clean.is_empty() && ldr_clean != "all" && ldr_clean != "all loaders" && ldr_clean != "—" {
        let loaders_json = serde_json::to_string(&[ldr_clean]).unwrap_or_default();
        req = req.query(&[("loaders", loaders_json)]);
    }

    let mc_clean = minecraft_version.trim().to_lowercase();
    if !mc_clean.is_empty() && mc_clean != "all" && mc_clean != "all versions" && mc_clean != "—" {
        let mc_json = serde_json::to_string(&[mc_clean]).unwrap_or_default();
        req = req.query(&[("game_versions", mc_json)]);
    }

    let res = req
        .send()
        .await
        .map_err(|e| format!("Modrinth get versions request failed: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("Modrinth API error: status {}", res.status()));
    }

    let body: Vec<serde_json::Value> = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse version list JSON: {}", e))?;

    let mut versions = Vec::new();
    for item in body {
        let version_id = item.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let project_id = item.get("project_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let version_number = item.get("version_number").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let changelog = item.get("changelog").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let game_versions = item
            .get("game_versions")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        let loaders = item
            .get("loaders")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        let mut files = Vec::new();
        if let Some(arr) = item.get("files").and_then(|f| f.as_array()) {
            for f in arr {
                let url = f.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let filename = f.get("filename").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let primary = f.get("primary").and_then(|v| v.as_bool()).unwrap_or(false);
                let size = f.get("size").and_then(|v| v.as_u64()).unwrap_or(0);

                let hashes = f.get("hashes");
                let sha1 = hashes
                    .and_then(|h| h.get("sha1"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let sha512 = hashes
                    .and_then(|h| h.get("sha512"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                files.push(ModrinthVersionFile {
                    url,
                    filename,
                    primary,
                    size,
                    sha1,
                    sha512,
                });
            }
        }

        let mut dependencies = Vec::new();
        if let Some(arr) = item.get("dependencies").and_then(|d| d.as_array()) {
            for d in arr {
                let p_id = d.get("project_id").and_then(|v| v.as_str()).map(|s| s.to_string());
                let v_id = d.get("version_id").and_then(|v| v.as_str()).map(|s| s.to_string());
                let dep_type = d.get("dependency_type").and_then(|v| v.as_str()).unwrap_or("required").to_string();
                let f_name = d.get("file_name").and_then(|v| v.as_str()).unwrap_or("").to_string();

                dependencies.push(ModrinthDependency {
                    project_id: p_id,
                    version_id: v_id,
                    dependency_type: dep_type,
                    file_name: f_name,
                });
            }
        }

        versions.push(ModrinthVersion {
            version_id,
            project_id,
            name,
            version_number,
            game_versions,
            loaders,
            files,
            dependencies,
            changelog,
        });
    }

    Ok(versions)
}

/// Check Modrinth for newer versions of the specified file hashes.
pub async fn check_mod_updates(
    file_hashes: &[String],
    loader: &str,
    minecraft_version: &str,
) -> Result<Vec<ModUpdateInfo>, String> {
    if file_hashes.is_empty() {
        return Ok(Vec::new());
    }

    let client = create_client()?;
    let mut payload = serde_json::json!({
        "hashes": file_hashes,
        "algorithm": "sha1"
    });

    let ldr_clean = loader.trim().to_lowercase();
    if !ldr_clean.is_empty() && ldr_clean != "all" && ldr_clean != "—" {
        payload["loaders"] = serde_json::json!([ldr_clean]);
    }

    let mc_clean = minecraft_version.trim().to_lowercase();
    if !mc_clean.is_empty() && mc_clean != "all" && mc_clean != "—" {
        payload["game_versions"] = serde_json::json!([mc_clean]);
    }

    let res = client
        .post("https://api.modrinth.com/v2/version_files/update")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Mod update check request failed: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("Modrinth API error: status {}", res.status()));
    }

    let data: HashMap<String, serde_json::Value> = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse update check response: {}", e))?;

    let mut updates = Vec::new();
    for (hash, val) in data {
        let name = val.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let version_number = val.get("version_number").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let changelog = val.get("changelog").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let files = val.get("files").and_then(|f| f.as_array());
        if let Some(f_arr) = files {
            if let Some(first_file) = f_arr.iter().find(|f| f.get("primary").and_then(|p| p.as_bool()).unwrap_or(false)).or_else(|| f_arr.first()) {
                let download_url = first_file.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let filename = first_file.get("filename").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let size = first_file.get("size").and_then(|v| v.as_u64()).unwrap_or(0);

                updates.push(ModUpdateInfo {
                    file_hash: hash,
                    mod_name: name.clone(),
                    current_version: "".to_string(),
                    new_version_number: version_number,
                    new_version_name: name,
                    download_url,
                    filename,
                    size,
                    changelog,
                });
            }
        }
    }

    Ok(updates)
}

/// Download a file streaming into the destination directory.
pub async fn download_file(url: &str, destination_path: &Path) -> Result<PathBuf, String> {
    if let Some(parent) = destination_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let client = create_client()?;
    let res = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Download request failed: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("Download failed with status: {}", res.status()));
    }

    let temp_path = destination_path.with_extension("download_tmp");
    let mut file = File::create(&temp_path).map_err(|e| format!("Failed to create temporary file: {}", e))?;

    let mut stream = res.bytes_stream();
    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| format!("Error downloading chunk: {}", e))?;
        file.write_all(&chunk).map_err(|e| format!("Error writing file: {}", e))?;
    }

    file.flush().map_err(|e| e.to_string())?;
    drop(file);

    std::fs::rename(&temp_path, destination_path)
        .map_err(|e| format!("Failed to move downloaded file to destination: {}", e))?;

    Ok(destination_path.to_path_buf())
}
