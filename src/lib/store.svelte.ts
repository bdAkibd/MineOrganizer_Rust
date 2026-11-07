import * as api from './api';
import { i18n } from './i18n';
import type {
  AppSettings,
  Instance,
  ModFile,
  Profile,
  UserDataStore,
  VirtualizationPlan,
} from './types';

export interface Toast {
  id: string;
  message: string;
  type: 'success' | 'error' | 'info';
}

class AppState {
  activeTab = $state<'mods' | 'instances' | 'profiles' | 'modrinth' | 'diagnostics' | 'settings'>('mods');
  isLoading = $state<boolean>(false);
  instances = $state<Instance[]>([]);
  libraryMods = $state<ModFile[]>([]);
  userData = $state<UserDataStore>({
    version: 1,
    active_profile_id: null,
    mod_tags: {},
    minecraft_version_overrides: {},
    profiles: [],
    settings: {
      locale: 'en',
      prism_data_dir: null,
      library_dir: null,
      custom_game_dirs: [],
      close_launcher_on_apply: false,
    },
  });

  // Selection
  selectedMod = $state<ModFile | null>(null);
  selectedInstance = $state<Instance | null>(null);

  // Filters
  searchQuery = $state<string>('');
  selectedLoader = $state<string>('all');
  selectedMcVersion = $state<string>('all');
  selectedTag = $state<string>('all');

  // Virtualization Dialog
  virtualizationModalOpen = $state<boolean>(false);
  virtualizationPlan = $state<VirtualizationPlan | null>(null);
  virtualizationLoading = $state<boolean>(false);

  // Notifications
  toasts = $state<Toast[]>([]);

  // Computed
  activeProfile = $derived.by<Profile | null>(() => {
    if (!this.userData.active_profile_id) {
      return this.userData.profiles[0] || null;
    }
    return this.userData.profiles.find((p) => p.id === this.userData.active_profile_id) || this.userData.profiles[0] || null;
  });

  filteredMods = $derived.by<ModFile[]>(() => {
    return this.libraryMods.filter((mod) => {
      // Search text
      if (this.searchQuery.trim()) {
        const q = this.searchQuery.toLowerCase();
        const matchesName = mod.file_name.toLowerCase().includes(q) || mod.metadata.name.toLowerCase().includes(q);
        const matchesId = mod.metadata.id.toLowerCase().includes(q);
        const matchesDesc = mod.metadata.description?.toLowerCase().includes(q) || false;
        if (!matchesName && !matchesId && !matchesDesc) {
          return false;
        }
      }

      // Loader
      if (this.selectedLoader !== 'all') {
        const modLoaders = mod.metadata.loaders.map((l) => l.toLowerCase());
        if (!modLoaders.includes(this.selectedLoader.toLowerCase())) {
          return false;
        }
      }

      // MC Version
      if (this.selectedMcVersion !== 'all') {
        const matchesVer = mod.metadata.mc_versions.some((v) =>
          v.includes(this.selectedMcVersion) || this.selectedMcVersion.includes(v)
        );
        if (!matchesVer && mod.metadata.mc_versions.length > 0) {
          return false;
        }
      }

      // Tag
      if (this.selectedTag !== 'all') {
        const hash = mod.sha256 || mod.sha1;
        const tags = this.userData.mod_tags[hash] || [];
        if (!tags.includes(this.selectedTag)) {
          return false;
        }
      }

      return true;
    });
  });

  allTags = $derived.by<string[]>(() => {
    const set = new Set<string>();
    for (const tags of Object.values(this.userData.mod_tags)) {
      for (const t of tags) {
        set.add(t);
      }
    }
    return Array.from(set).sort();
  });

  allMcVersions = $derived.by<string[]>(() => {
    const set = new Set<string>();
    for (const mod of this.libraryMods) {
      for (const v of mod.metadata.mc_versions) {
        const clean = v.trim();
        if (clean && !clean.includes('>') && !clean.includes('<')) {
          set.add(clean);
        }
      }
    }
    return Array.from(set).sort().reverse();
  });

  showToast(message: string, type: 'success' | 'error' | 'info' = 'info') {
    const id = Math.random().toString(36).substring(2, 9);
    this.toasts = [...this.toasts, { id, message, type }];
    setTimeout(() => {
      this.toasts = this.toasts.filter((t) => t.id !== id);
    }, 4500);
  }

  async init() {
    this.isLoading = true;
    try {
      // 1. Load User Data
      const data = await api.loadUserData();
      this.userData = data;

      // Sync language with store setting if set
      if (data.settings.locale) {
        i18n.setLocale(data.settings.locale);
      }

      // 2. Scan instances
      await this.refreshInstances();

      // 3. Scan library mods
      await this.refreshLibraryMods();
    } catch (e: any) {
      this.showToast(`Initialization failed: ${e?.message || e}`, 'error');
    } finally {
      this.isLoading = false;
    }
  }

  async refreshInstances() {
    try {
      const prismPath = this.userData.settings.prism_data_dir || null;
      const customPaths = this.userData.settings.custom_game_dirs || [];
      const insts = await api.scanAllInstances(prismPath, customPaths);
      this.instances = insts;

      if (!this.selectedInstance && insts.length > 0) {
        this.selectedInstance = insts[0];
      }
    } catch (e: any) {
      this.showToast(`Failed to scan instances: ${e?.message || e}`, 'error');
    }
  }

  async refreshLibraryMods() {
    try {
      const libDir = this.userData.settings.library_dir || './mods_library';
      const mods = await api.scanLibraryMods(libDir);
      this.libraryMods = mods;
    } catch (e: any) {
      console.warn('Library scan warning:', e);
    }
  }

  async toggleModInActiveProfile(modSha256OrSha1: string) {
    const prof = this.activeProfile;
    if (!prof) {
      this.showToast('Please create or select a profile first.', 'info');
      return;
    }

    const currentHashes = new Set(prof.enabled_mod_hashes);
    if (currentHashes.has(modSha256OrSha1)) {
      currentHashes.delete(modSha256OrSha1);
    } else {
      currentHashes.add(modSha256OrSha1);
    }

    const updatedProfile: Profile = {
      ...prof,
      enabled_mod_hashes: Array.from(currentHashes),
      active_mods: Array.from(currentHashes),
    };

    try {
      const saved = await api.saveProfileItem(updatedProfile);
      this.userData.profiles = this.userData.profiles.map((p) => (p.id === saved.id ? saved : p));
    } catch (e: any) {
      this.showToast(`Failed to update profile: ${e?.message || e}`, 'error');
    }
  }

  isModActiveInProfile(mod: ModFile): boolean {
    const prof = this.activeProfile;
    if (!prof) return false;
    const hash = mod.sha256 || mod.sha1;
    return prof.enabled_mod_hashes.includes(hash) || prof.active_mods.includes(hash);
  }

  async saveProfileItem(profile: Profile): Promise<Profile> {
    try {
      const saved = await api.saveProfileItem(profile);
      const idx = this.userData.profiles.findIndex((p) => p.id === saved.id);
      if (idx !== -1) {
        this.userData.profiles[idx] = saved;
      } else {
        this.userData.profiles.push(saved);
      }
      return saved;
    } catch (e: any) {
      this.showToast(`Failed to save profile: ${e?.message || e}`, 'error');
      throw e;
    }
  }

  async setActiveProfile(profileId: string) {
    this.userData.active_profile_id = profileId;
    const prof = this.userData.profiles.find((p) => p.id === profileId);
    if (prof) {
      try {
        await api.saveProfileItem(prof);
      } catch (e) {
        // Ignored
      }
    }
  }

  async createNewProfile(name: string, instance?: Instance | null) {
    const newProf: Profile = {
      id: '',
      name: name.trim() || 'New Profile',
      instance_id: instance?.id || null,
      instance_path: instance?.path || null,
      minecraft_version: instance?.minecraft_version || '1.20.1',
      loader: instance?.loader || 'Fabric',
      loader_version: instance?.loader_version || null,
      enabled_mod_hashes: [],
      enabled_resourcepack_hashes: [],
      enabled_shaderpack_hashes: [],
      active_mods: [],
      active_assets: [],
      created_at: '',
      updated_at: '',
    };

    try {
      const saved = await api.saveProfileItem(newProf);
      this.userData.profiles.push(saved);
      this.userData.active_profile_id = saved.id;
      this.showToast(`Profile "${saved.name}" created!`, 'success');
    } catch (e: any) {
      this.showToast(`Failed to create profile: ${e?.message || e}`, 'error');
    }
  }

  async deleteProfile(profileId: string) {
    try {
      await api.deleteProfileItem(profileId);
      this.userData.profiles = this.userData.profiles.filter((p) => p.id !== profileId);
      if (this.userData.active_profile_id === profileId) {
        this.userData.active_profile_id = this.userData.profiles[0]?.id || null;
      }
      this.showToast('Profile deleted', 'info');
    } catch (e: any) {
      this.showToast(`Failed to delete profile: ${e?.message || e}`, 'error');
    }
  }

  async setModTagsFor(sha256: string, tags: string[]) {
    try {
      await api.setModTags(sha256, tags);
      this.userData.mod_tags[sha256] = tags;
      this.showToast('Tags saved', 'success');
    } catch (e: any) {
      this.showToast(`Failed to save tags: ${e?.message || e}`, 'error');
    }
  }

  async saveSettings(newSettings: AppSettings) {
    try {
      await api.saveAppSettings(newSettings);
      this.userData.settings = newSettings;
      i18n.setLocale(newSettings.locale);
      this.showToast('Settings saved', 'success');
    } catch (e: any) {
      this.showToast(`Failed to save settings: ${e?.message || e}`, 'error');
    }
  }
}

export const appState = new AppState();
