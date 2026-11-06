use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModpackExportOptions {
    pub include_mods: bool,
    pub include_configs: bool,
    pub include_resource_packs: bool,
    pub include_shader_packs: bool,
    pub include_saves: bool,
}

impl Default for ModpackExportOptions {
    fn default() -> Self {
        Self {
            include_mods: true,
            include_configs: true,
            include_resource_packs: false,
            include_shader_packs: false,
            include_saves: false,
        }
    }
}
