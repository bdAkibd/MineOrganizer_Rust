<script lang="ts">
  import { appState } from '$lib/store.svelte';
  import { t } from '$lib/i18n.svelte';
  import { previewProfile, rollbackLast } from '$lib/api';
  import {
    Play,
    RotateCcw,
    RefreshCw,
    Search,
    Download,
    Layers,
    Sparkles,
  } from '@lucide/svelte';

  const sectionTitles: Record<string, string> = {
    mods: 'Mod Library',
    instances: 'Game Environments',
    profiles: 'Profile Management',
    modrinth: 'Modrinth Marketplace',
    diagnostics: 'Crash & Log Diagnostics',
    settings: 'Settings & Preferences',
  };

  async function handleDeployProfile() {
    const prof = appState.activeProfile;
    if (!prof) {
      appState.showToast('Select an active profile first.', 'info');
      return;
    }

    const instancePath = prof.instance_path || appState.selectedInstance?.path;
    if (!instancePath) {
      appState.showToast('Please link an environment/instance to this profile first.', 'error');
      return;
    }

    const libraryDir = appState.userData.settings.library_dir || './mods_library';

    appState.virtualizationLoading = true;
    appState.virtualizationModalOpen = true;

    try {
      const plan = await previewProfile(
        instancePath,
        prof.name,
        prof.enabled_mod_hashes,
        libraryDir,
        prof.minecraft_version,
        prof.loader
      );
      appState.virtualizationPlan = plan;
    } catch (e: any) {
      appState.showToast(`Virtualization preview failed: ${e?.message || e}`, 'error');
      appState.virtualizationModalOpen = false;
    } finally {
      appState.virtualizationLoading = false;
    }
  }

  async function handleRollback() {
    try {
      const msg = await rollbackLast();
      appState.showToast(msg, 'success');
      await appState.refreshInstances();
    } catch (e: any) {
      appState.showToast(e?.message || String(e), 'error');
    }
  }

  async function handleRefreshAll() {
    appState.isLoading = true;
    await appState.refreshInstances();
    await appState.refreshLibraryMods();
    appState.isLoading = false;
    appState.showToast('Refreshed library and environments', 'success');
  }
</script>

<header class="h-16 bg-[#0c0d10]/90 backdrop-blur-md border-b border-[#27272a] px-6 flex items-center justify-between z-10 shrink-0">
  <!-- Title / Breadcrumb -->
  <div class="flex items-center gap-3">
    <h2 class="text-base font-semibold text-zinc-100 flex items-center gap-2">
      {t(sectionTitles[appState.activeTab] || 'MineOrganizer')}
    </h2>
    {#if appState.activeProfile}
      <div class="hidden sm:flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-zinc-900 border border-zinc-700/60 text-[11px] text-zinc-300">
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
        <span class="text-zinc-400">{t('Target')}:</span>
        <span class="font-medium text-zinc-200">{appState.activeProfile?.name}</span>
      </div>
    {/if}
  </div>

  <!-- Search in Library or Marketplace -->
  {#if appState.activeTab === 'mods'}
    <div class="flex-1 max-w-xs mx-4">
      <div class="relative">
        <Search class="w-3.5 h-3.5 text-zinc-400 absolute left-3 top-1/2 -translate-y-1/2" />
        <input
          type="text"
          placeholder={t('Search mods...')}
          bind:value={appState.searchQuery}
          class="w-full pl-9 pr-3 py-1.5 bg-zinc-900/90 border border-zinc-700/70 focus:border-violet-500 rounded-lg text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none transition-colors"
        />
      </div>
    </div>
  {/if}

  <!-- Action Controls -->
  <div class="flex items-center gap-2">
    <!-- Refresh -->
    <button
      onclick={handleRefreshAll}
      title={t('Refresh')}
      class="p-2 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 border border-zinc-800 transition-colors"
    >
      <RefreshCw class="w-4 h-4 {appState.isLoading ? 'animate-spin text-violet-400' : ''}" />
    </button>

    <!-- Rollback -->
    <button
      onclick={handleRollback}
      title={t('Rollback Last Virtualization')}
      class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium text-zinc-300 hover:text-zinc-100 bg-zinc-900/80 hover:bg-zinc-800 border border-zinc-700/60 transition-all shadow-sm"
    >
      <RotateCcw class="w-3.5 h-3.5 text-amber-400" />
      <span class="hidden md:inline">{t('Rollback')}</span>
    </button>

    <!-- Deploy Profile via Virtualization Engine -->
    <button
      onclick={handleDeployProfile}
      class="flex items-center gap-2 px-4 py-1.5 rounded-lg text-xs font-semibold text-white bg-gradient-to-r from-violet-600 to-indigo-600 hover:from-violet-500 hover:to-indigo-500 active:scale-[0.98] transition-all shadow-lg shadow-violet-600/25 border border-violet-400/30"
    >
      <Play class="w-3.5 h-3.5 fill-current" />
      <span>{t('Deploy to Game')}</span>
    </button>
  </div>
</header>
