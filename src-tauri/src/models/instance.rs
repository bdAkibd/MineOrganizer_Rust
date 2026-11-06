use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub launcher: String,
    pub minecraft_version: String,
    pub loader: String,
    pub loader_version: String,
    pub status: String,
    #[serde(default)]
    pub icon: Option<String>,
}
