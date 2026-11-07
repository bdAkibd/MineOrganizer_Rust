export interface Instance {
  id: string;
  name: string;
  path: string;
  launcher: string;
  minecraft_version: string;
  loader: string;
  loader_version: string;
  status: string;
  icon?: string | null;
}

export interface ModDependency {
  id: string;
  version_requirement?: string | null;
  required: boolean;
}

export interface ModMetadata {
  id: string;
  name: string;
  version: string;
  description?: string | null;
  loaders: string[];
  mc_versions: string[];
  dependencies: ModDependency[];
  icon_url?: string | null;
  homepage?: string | null;
  source_url?: string | null;
  kind: string;
}

export interface ModFile {
  file_name: string;
  path: string;
  size: u64;
  sha1: string;
  sha256?: string | null;
  sha512?: string | null;
  metadata: ModMetadata;
  tags: string[];
  disabled: boolean;
}

export type u64 = number;

export interface Profile {
  id: string;
  name: string;
  instance_id?: string | null;
  instance_path?: string | null;
  minecraft_version?: string | null;
  loader?: string | null;
  loader_version?: string | null;
  enabled_mod_hashes: string[];
  enabled_resourcepack_hashes: string[];
  enabled_shaderpack_hashes: string[];
  active_mods: string[];
  active_assets: string[];
  created_at: string;
  updated_at: string;
}

export interface AppSettings {
  locale: string;
  prism_data_dir?: string | null;
  library_dir?: string | null;
  custom_game_dirs: string[];
  close_launcher_on_apply: boolean;
}

export interface UserDataStore {
  version: number;
  active_profile_id?: string | null;
  mod_tags: Record<string, string[]>;
  minecraft_version_overrides: Record<string, string[]>;
  profiles: Profile[];
  settings: AppSettings;
}

export interface VirtualizationAction {
  filename: string;
  action: string; // "Create link", "Remove link", "Keep link", "Keep external file", "Conflict", "Not in library", "No changes"
  details: string;
  path: string;
}

export interface VirtualizationPlan {
  profile_name: string;
  instance_path: string;
  mods_dir?: string | null;
  actions: VirtualizationAction[];
  errors: string[];
  can_apply: bool;
}

export type bool = boolean;

export interface VirtualizationResult {
  success: boolean;
  message: string;
  created_count: number;
  removed_count: number;
  manifest_path?: string | null;
}

export interface CrashAnalysis {
  file_path: string;
  file_type: string;
  timestamp: string;
  title: string;
  category: string; // "out_of_memory", "java_version", "missing_dependency", "mixin_conflict", "duplicate_mod", "driver_opengl", "mod_exception", "unknown"
  summary: string;
  recommendation: string;
  suspected_mods: string[];
  exception_class: string;
  exception_message: string;
  minecraft_version: string;
  java_version: string;
  raw_text: string;
}

export interface ModrinthSearchResult {
  project_id: string;
  slug: string;
  title: string;
  description: string;
  author: string;
  icon_url: string;
  downloads: number;
  follows: number;
  versions: string[];
  loaders: string[];
}

export interface ModrinthSearchResponse {
  hits: ModrinthSearchResult[];
  total_hits: number;
  offset: number;
  limit: number;
}

export interface ModrinthVersionFile {
  url: string;
  filename: string;
  primary: boolean;
  size: number;
  sha1: string;
  sha512: string;
}

export interface ModrinthDependencyItem {
  project_id?: string | null;
  version_id?: string | null;
  dependency_type: string;
  file_name: string;
}

export interface ModrinthVersion {
  version_id: string;
  project_id: string;
  name: string;
  version_number: string;
  game_versions: string[];
  loaders: string[];
  files: ModrinthVersionFile[];
  dependencies: ModrinthDependencyItem[];
  changelog: string;
}

export interface ModUpdateInfo {
  file_hash: string;
  mod_name: string;
  current_version: string;
  new_version_number: string;
  new_version_name: string;
  download_url: string;
  filename: string;
  size: number;
  changelog: string;
}

export interface ModpackInspectResult {
  name: string;
  minecraft_version: string;
  loader: string;
  loader_version: string;
  mod_count: number;
  resourcepacks_count: number;
  shaderpacks_count: number;
  saves_count: number;
  has_configs: boolean;
  has_resourcepacks: boolean;
  has_shaderpacks: boolean;
  has_saves: boolean;
  format: string;
  mod_filenames: string[];
  save_names: string[];
}

export interface ModpackExportReport {
  mods_count: number;
  configs_count: number;
  resourcepacks_count: number;
  shaderpacks_count: number;
  saves_count: number;
  archive_size: number;
  output_path: string;
}

export interface ModpackImportResult {
  name: string;
  format: string;
  minecraft_version: string;
  loader: string;
  loader_version: string;
  imported_mods_count: number;
  imported_configs_count: number;
  imported_resourcepacks_count: number;
  imported_shaderpacks_count: number;
  imported_saves_count: number;
  imported_mod_hashes: string[];
}
