<script lang="ts">
  import { appState } from '$lib/store.svelte';
  import { t } from '$lib/i18n';
  import {
    downloadModrinthFile,
    getProjectVersions,
    searchModrinth,
  } from '$lib/api';
  import type { ModrinthSearchResult, ModrinthVersion } from '$lib/types';
  import {
    Search,
    Store,
    Download,
    Check,
    Layers,
    X,
    ExternalLink,
    Filter,
  } from '@lucide/svelte';

  let query = $state('optimization');
  let loaderFilter = $state('fabric');
  let versionFilter = $state('1.20.1');
  let isSearching = $state(false);
  let searchResults = $state<ModrinthSearchResult[]>([]);

  // Version selector drawer
  let selectedProject = $state<ModrinthSearchResult | null>(null);
  let projectVersions = $state<ModrinthVersion[]>([]);
  let isLoadingVersions = $state(false);
  let downloadingVersionId = $state<string | null>(null);

  async function handleSearch() {
    isSearching = true;
    try {
      const results = await searchModrinth(query, loaderFilter, versionFilter, 24, 0);
      searchResults = results;
    } catch (e: any) {
      appState.showToast(`Search failed: ${e?.message || e}`, 'error');
    } finally {
      isSearching = false;
    }
  }

  async function handleOpenVersions(project: ModrinthSearchResult) {
    selectedProject = project;
    isLoadingVersions = true;
    projectVersions = [];
    try {
      const versions = await getProjectVersions(project.project_id, loaderFilter, versionFilter);
      projectVersions = versions;
    } catch (e: any) {
      appState.showToast(`Failed to load versions: ${e?.message || e}`, 'error');
    } finally {
      isLoadingVersions = false;
    }
  }

  async function handleDownload(ver: ModrinthVersion) {
    const file = ver.files.find((f) => f.primary) || ver.files[0];
    if (!file) {
      appState.showToast('No downloadable files in this version', 'error');
      return;
    }

    const libDir = appState.userData.settings.library_dir || './mods_library';
    downloadingVersionId = ver.version_id;

    try {
      await downloadModrinthFile(file.url, file.filename, libDir);
      appState.showToast(`Downloaded "${file.filename}" to library!`, 'success');
      await appState.refreshLibraryMods();
    } catch (e: any) {
      appState.showToast(`Download failed: ${e?.message || e}`, 'error');
    } finally {
      downloadingVersionId = null;
    }
  }

  // Initial search
  $effect(() => {
    if (searchResults.length === 0 && !isSearching) {
      handleSearch();
    }
  });
</script>

<div class="flex-1 flex flex-col h-full bg-[#09090b] overflow-hidden">
  <!-- Search & Filter Bar -->
  <div class="px-6 py-4 border-b border-[#27272a]/60 bg-[#0c0d10]/40 flex flex-wrap items-center justify-between gap-4 shrink-0">
    <div class="flex items-center gap-3 flex-1 max-w-lg">
      <div class="relative flex-1">
        <Search class="w-4 h-4 text-zinc-400 absolute left-3 top-1/2 -translate-y-1/2" />
        <input
          type="text"
          placeholder={t('Search Modrinth mods...')}
          bind:value={query}
          onkeydown={(e) => e.key === 'Enter' && handleSearch()}
          class="w-full pl-9 pr-3 py-2 bg-zinc-900 border border-zinc-700/80 rounded-lg text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none focus:border-violet-500"
        />
      </div>

      <button
        onclick={handleSearch}
        disabled={isSearching}
        class="px-4 py-2 bg-violet-600 hover:bg-violet-500 text-white rounded-lg text-xs font-semibold transition-colors shrink-0 shadow-md shadow-violet-950/40"
      >
        {isSearching ? t('Searching...') : t('Search')}
      </button>
    </div>

    <!-- Filter dropdowns -->
    <div class="flex items-center gap-2">
      <select
        bind:value={loaderFilter}
        onchange={handleSearch}
        class="bg-zinc-900 border border-zinc-700 text-xs text-zinc-300 rounded-lg px-3 py-2 focus:outline-none focus:border-violet-500"
      >
        <option value="all">{t('All Loaders')}</option>
        <option value="fabric">Fabric</option>
        <option value="forge">Forge</option>
        <option value="neoforge">NeoForge</option>
        <option value="quilt">Quilt</option>
      </select>

      <input
        type="text"
        placeholder={t('Minecraft Version')}
        bind:value={versionFilter}
        onkeydown={(e) => e.key === 'Enter' && handleSearch()}
        class="w-28 px-3 py-2 bg-zinc-900 border border-zinc-700 text-xs text-zinc-300 rounded-lg focus:outline-none focus:border-violet-500 font-mono"
      />
    </div>
  </div>

  <!-- Results Grid -->
  <div class="flex-1 overflow-y-auto p-6">
    {#if searchResults.length === 0 && !isSearching}
      <div class="h-64 flex flex-col items-center justify-center text-center p-8 text-zinc-400">
        <Store class="w-12 h-12 text-zinc-600 mb-3" />
        <h4 class="text-sm font-semibold text-zinc-300 mb-1">{t('No results found')}</h4>
        <p class="text-xs text-zinc-500 max-w-sm">{t('Try modifying your search term or version filters.')}</p>
      </div>
    {:else}
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {#each searchResults as item}
          <div class="p-4 rounded-xl bg-zinc-900/60 border border-[#27272a] hover:border-zinc-700 hover:bg-zinc-900/90 transition-all flex flex-col justify-between">
            <div class="space-y-3">
              <!-- Top Header: Icon & Title -->
              <div class="flex items-start gap-3">
                {#if item.icon_url}
                  <img
                    src={item.icon_url}
                    alt={item.title}
                    class="w-10 h-10 rounded-lg object-contain bg-zinc-800 border border-zinc-700 shrink-0"
                  />
                {:else}
                  <div class="w-10 h-10 rounded-lg bg-zinc-800 border border-zinc-700 flex items-center justify-center shrink-0 text-zinc-400">
                    <Store class="w-5 h-5" />
                  </div>
                {/if}

                <div class="truncate flex-1">
                  <h4 class="font-bold text-sm text-zinc-100 truncate hover:text-violet-300 transition-colors">
                    {item.title}
                  </h4>
                  <p class="text-[11px] text-zinc-500 truncate">by {item.author}</p>
                </div>
              </div>

              <!-- Description -->
              <p class="text-xs text-zinc-400 line-clamp-2 leading-relaxed">
                {item.description}
              </p>

              <!-- Tags / Categories -->
              <div class="flex flex-wrap gap-1">
                {#each item.loaders as ldr}
                  <span class="px-1.5 py-0.5 rounded text-[10px] font-medium bg-zinc-800 text-zinc-300 border border-zinc-700/60 uppercase">
                    {ldr}
                  </span>
                {/each}
              </div>
            </div>

            <!-- Footer: Downloads and Button -->
            <div class="pt-3 border-t border-zinc-800/60 flex items-center justify-between gap-2 mt-4 text-xs">
              <span class="text-zinc-500 font-mono text-[11px]">
                {(item.downloads / 1000).toFixed(0)}k downloads
              </span>

              <button
                onclick={() => handleOpenVersions(item)}
                class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold bg-violet-600/15 hover:bg-violet-600 text-violet-300 hover:text-white border border-violet-500/30 transition-all shadow-sm"
              >
                <Download class="w-3.5 h-3.5" />
                <span>{t('Install / Versions')}</span>
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<!-- Drawer: Versions for Selected Project -->
{#if selectedProject}
  <button
    type="button"
    aria-label="Close version selector"
    class="fixed inset-0 bg-black/50 backdrop-blur-xs z-30 transition-opacity w-full h-full border-0 cursor-default"
    onclick={() => (selectedProject = null)}
  ></button>

  <div class="fixed inset-y-0 right-0 w-full max-w-lg bg-[#0c0d10] border-l border-[#27272a] shadow-2xl z-40 flex flex-col text-xs text-zinc-300">
    <div class="h-16 px-6 border-b border-[#27272a] flex items-center justify-between shrink-0 bg-[#09090b]/40">
      <div class="flex items-center gap-3 truncate">
        {#if selectedProject.icon_url}
          <img src={selectedProject.icon_url} alt="" class="w-8 h-8 rounded-lg object-contain bg-zinc-800 shrink-0" />
        {/if}
        <div class="truncate">
          <h3 class="font-bold text-sm text-zinc-100 truncate">{selectedProject.title}</h3>
          <p class="text-[11px] text-zinc-500 font-mono">{t('Releases & Versions')}</p>
        </div>
      </div>

      <button
        onclick={() => (selectedProject = null)}
        class="p-1.5 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 transition-colors"
      >
        <X class="w-4 h-4" />
      </button>
    </div>

    <div class="flex-1 overflow-y-auto p-6 space-y-3">
      {#if isLoadingVersions}
        <div class="p-8 text-center text-zinc-400">{t('Loading versions...')}</div>
      {:else if projectVersions.length === 0}
        <div class="p-8 text-center text-zinc-400">{t('No versions found for selected filters.')}</div>
      {:else}
        {#each projectVersions as ver}
          {@const isDownloading = downloadingVersionId === ver.version_id}
          <div class="p-4 rounded-xl bg-zinc-900/60 border border-zinc-800 space-y-2">
            <div class="flex items-center justify-between">
              <span class="font-bold text-zinc-100 text-xs font-mono">{ver.name || ver.version_number}</span>
              <span class="text-[10px] font-mono text-zinc-500">{ver.version_number}</span>
            </div>

            <div class="flex flex-wrap gap-1 text-[10px]">
              {#each ver.game_versions as gv}
                <span class="px-1.5 py-0.5 rounded bg-zinc-800 text-zinc-300 font-mono">{gv}</span>
              {/each}
              {#each ver.loaders as ldr}
                <span class="px-1.5 py-0.5 rounded bg-violet-500/10 text-violet-300">{ldr}</span>
              {/each}
            </div>

            <div class="pt-2 flex justify-end">
              <button
                onclick={() => handleDownload(ver)}
                disabled={isDownloading}
                class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold bg-violet-600 hover:bg-violet-500 text-white transition-all shadow-sm"
              >
                <Download class="w-3.5 h-3.5 {isDownloading ? 'animate-bounce' : ''}" />
                <span>{isDownloading ? t('Downloading...') : t('Download to Library')}</span>
              </button>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>
{/if}
