import { invoke } from '@tauri-apps/api/core';
import type {
  AppSettings,
  CrashAnalysis,
  Instance,
  InstanceAssets,
  ModFile,
  ModMetadata,
  ModpackExportReport,
  ModpackImportResult,
  ModpackInspectResult,
  ModrinthSearchResponse,
  ModrinthSearchResult,
  ModrinthVersion,
  ModUpdateInfo,
  Profile,
  SaveItem,
  AssetItem,
  UserDataStore,
  VirtualizationPlan,
  VirtualizationResult,
} from './types';

// ============================================================================
// Instances
// ============================================================================

export async function getDefaultCandidatePaths(): Promise<string[]> {
  return invoke<string[]>('get_default_candidate_paths');
}

export async function getDefaultPrismPath(): Promise<string | null> {
  return invoke<string | null>('get_default_prism_path');
}

export async function scanAllInstances(
  prismPath?: string | null,
  customPaths: string[] = []
): Promise<Instance[]> {
  return invoke<Instance[]>('scan_all_instances', {
    prismPath: prismPath || null,
    customPaths,
  });
}

export async function discoverInstanceEnvironments(path: string): Promise<Instance[]> {
  return invoke<Instance[]>('discover_instance_environments', { path });
}

export async function scanSingleDirectory(
  path: string,
  launcherOverride?: string | null
): Promise<Instance> {
  return invoke<Instance>('scan_single_directory', {
    path,
    launcherOverride: launcherOverride || null,
  });
}

// ============================================================================
// Library
// ============================================================================

export async function scanLibraryMods(dirPath: string): Promise<ModFile[]> {
  return invoke<ModFile[]>('scan_library_mods', { dirPath });
}

export async function parseSingleMod(filePath: string): Promise<ModFile> {
  return invoke<ModFile>('parse_single_mod', { filePath });
}

export async function checkModCompatibility(
  metadata: ModMetadata,
  minecraftVersion: string,
  loader: string,
  availableIds: string[] = []
): Promise<[string, string]> {
  return invoke<[string, string]>('check_mod_compatibility_cmd', {
    metadata,
    minecraftVersion,
    loader,
    availableIds,
  });
}

export async function importFilesToLibrary(
  sourcePaths: string[],
  libraryDir: string
): Promise<string[]> {
  return invoke<string[]>('import_files_to_library_cmd', {
    sourcePaths,
    libraryDir,
  });
}

export async function saveBinaryToLibrary(
  fileName: string,
  bytes: number[] | Uint8Array,
  libraryDir: string
): Promise<string[]> {
  return invoke<string[]>('save_binary_to_library_cmd', {
    fileName,
    bytes: Array.from(bytes),
    libraryDir,
  });
}

export async function deleteModFile(filePath: string): Promise<boolean> {
  return invoke<boolean>('delete_mod_file_cmd', { filePath });
}

// ============================================================================
// Virtualization
// ============================================================================

export async function previewProfile(
  instancePath: string,
  profileName: string,
  desiredHashes: string[],
  libraryDir: string,
  mcVersion?: string | null,
  loader?: string | null
): Promise<VirtualizationPlan> {
  return invoke<VirtualizationPlan>('preview_profile_cmd', {
    instancePath,
    profileName,
    desiredHashes,
    libraryDir,
    mcVersion: mcVersion || null,
    loader: loader || null,
  });
}

export async function applyProfile(
  instancePath: string,
  profileName: string,
  desiredHashes: string[],
  libraryDir: string,
  backupDir?: string | null,
  mcVersion?: string | null,
  loader?: string | null
): Promise<VirtualizationResult> {
  return invoke<VirtualizationResult>('apply_profile_cmd', {
    instancePath,
    profileName,
    desiredHashes,
    libraryDir,
    backupDir: backupDir || null,
    mcVersion: mcVersion || null,
    loader: loader || null,
  });
}

export async function rollbackLast(
  backupDir?: string | null,
  libraryDir?: string | null
): Promise<string> {
  return invoke<string>('rollback_last_cmd', {
    backupDir: backupDir || null,
    libraryDir: libraryDir || null,
  });
}

// ============================================================================
// Diagnostics & Logs
// ============================================================================

export async function listInstanceLogFiles(instancePath: string): Promise<[string, string][]> {
  return invoke<[string, string][]>('list_instance_log_files', { instancePath });
}

export async function readLogFileContent(filePath: string, maxBytes?: number | null): Promise<string> {
  return invoke<string>('read_log_file_content', {
    filePath,
    maxBytes: maxBytes || null,
  });
}

export async function analyzeCrashLogs(
  instancePath: string,
  knownModNames?: string[] | null
): Promise<CrashAnalysis | null> {
  return invoke<CrashAnalysis | null>('analyze_crash_logs', {
    instancePath,
    knownModNames: knownModNames || null,
  });
}

export async function analyzeRawLog(
  content: string,
  filePath: string,
  fileType: string,
  knownModNames?: string[] | null
): Promise<CrashAnalysis> {
  return invoke<CrashAnalysis>('analyze_raw_log', {
    content,
    filePath,
    fileType,
    knownModNames: knownModNames || null,
  });
}

// ============================================================================
// Modrinth Network Client
// ============================================================================

export async function searchModrinth(
  query: string,
  loader: string = '',
  minecraftVersion: string = '',
  limit: number = 24,
  offset: number = 0
): Promise<ModrinthSearchResponse> {
  return invoke<ModrinthSearchResponse>('search_modrinth_cmd', {
    query,
    loader,
    minecraftVersion,
    limit,
    offset,
  });
}

export async function getProjectVersions(
  projectIdOrSlug: string,
  loader: string = '',
  minecraftVersion: string = ''
): Promise<ModrinthVersion[]> {
  return invoke<ModrinthVersion[]>('get_project_versions_cmd', {
    projectIdOrSlug,
    loader,
    minecraftVersion,
  });
}

export async function downloadModrinthFile(
  url: string,
  filename: string,
  destinationDir: string
): Promise<string> {
  return invoke<string>('download_modrinth_file_cmd', {
    url,
    filename,
    destinationDir,
  });
}

export async function checkModUpdates(
  fileHashes: string[],
  loader: string = '',
  minecraftVersion: string = ''
): Promise<ModUpdateInfo[]> {
  return invoke<ModUpdateInfo[]>('check_mod_updates_cmd', {
    fileHashes,
    loader,
    minecraftVersion,
  });
}

// ============================================================================
// Modpacks
// ============================================================================

export async function inspectModpack(archivePath: string): Promise<ModpackInspectResult> {
  return invoke<ModpackInspectResult>('inspect_modpack_cmd', { archivePath });
}

export async function exportModpack(
  profileId: string,
  libraryDir: string,
  outputPath: string,
  includeConfigs: boolean = true,
  includeResourcepacks: boolean = true,
  includeShaderpacks: boolean = true,
  includeSaves: boolean = false,
  selectedSaves?: string[] | null
): Promise<ModpackExportReport> {
  return invoke<ModpackExportReport>('export_modpack_cmd', {
    profileId,
    libraryDir,
    outputPath,
    includeConfigs,
    includeResourcepacks,
    includeShaderpacks,
    includeSaves,
    selectedSaves: selectedSaves || null,
  });
}

export async function listInstanceAssets(instancePath: string): Promise<InstanceAssets> {
  return invoke<InstanceAssets>('list_instance_assets_cmd', { instancePath });
}

export async function importModpack(
  archivePath: string,
  libraryDir: string,
  targetInstancePath?: string | null
): Promise<ModpackImportResult> {
  return invoke<ModpackImportResult>('import_modpack_cmd', {
    archivePath,
    libraryDir,
    targetInstancePath: targetInstancePath || null,
  });
}

// ============================================================================
// User Data & Profiles
// ============================================================================

export async function loadUserData(): Promise<UserDataStore> {
  return invoke<UserDataStore>('load_user_data_cmd');
}

export async function saveProfileItem(profile: Profile): Promise<Profile> {
  return invoke<Profile>('save_profile_cmd', { profile });
}

export async function deleteProfileItem(profileId: string): Promise<boolean> {
  return invoke<boolean>('delete_profile_cmd', { profileId });
}

export async function saveAppSettings(settings: AppSettings): Promise<boolean> {
  return invoke<boolean>('save_settings_cmd', { settings });
}

export async function setModTags(sha256: string, tags: string[]): Promise<boolean> {
  return invoke<boolean>('set_mod_tags_cmd', { sha256, tags });
}

export async function setActiveProfileItem(profileId: string | null): Promise<boolean> {
  return invoke<boolean>('set_active_profile_cmd', { profileId });
}

export async function getAppDataDir(): Promise<string> {
  return invoke<string>('get_app_data_dir_cmd');
}

export async function openPathInExplorer(path: string): Promise<boolean> {
  return invoke<boolean>('open_path_in_explorer_cmd', { path });
}

// ============================================================================
// User Manual
// ============================================================================

export async function getUserManual(locale?: string | null): Promise<string> {
  return invoke<string>('get_user_manual_cmd', { locale: locale || null });
}
