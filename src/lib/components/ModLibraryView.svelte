<script lang="ts">
  import { appState } from '$lib/store.svelte';
  import { t } from '$lib/i18n.svelte';
  import type { ModFile } from '$lib/types';
  import {
    Boxes,
    Filter,
    Tag,
    CheckCircle2,
    AlertCircle,
    XCircle,
    Server,
    ExternalLink,
    ChevronRight,
    HelpCircle,
    FileBox,
    Plus,
    Upload,
    FolderOpen,
    Trash2,
    Copy,
    Layers,
    Check,
    X,
  } from '@lucide/svelte';
  import { contextMenu } from '$lib/contextMenu.svelte';
  import {
    deleteModFile,
    importFilesToLibrary,
    openPathInExplorer,
    saveBinaryToLibrary,
  } from '$lib/api';

  const loaders = ['all', 'Fabric', 'Forge', 'NeoForge', 'Quilt'];
  let fileInputRef = $state<HTMLInputElement | null>(null);
  let isDraggingOver = $state(false);

  $effect(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      try {
        const { getCurrentWebview } = await import('@tauri-apps/api/webview');
        unlisten = await getCurrentWebview().onDragDropEvent((event) => {
          if (event.payload.type === 'enter' || event.payload.type === 'over') {
            isDraggingOver = true;
          } else if (event.payload.type === 'leave') {
            isDraggingOver = false;
          } else if (event.payload.type === 'drop') {
            isDraggingOver = false;
            if (event.payload.paths && event.payload.paths.length > 0) {
              handleImportPaths(event.payload.paths);
            }
          }
        });
      } catch (e) {
        // Fallback for non-tauri or browser dev
      }
    })();

    return () => {
      if (unlisten) unlisten();
    };
  });

  async function handleImportPaths(paths: string[]) {
    const libDir = appState.userData.settings.library_dir || './mods_library';
    appState.isLoading = true;
    try {
      const imported = await importFilesToLibrary(paths, libDir);
      await appState.refreshLibraryMods();
      appState.showToast(`Imported ${imported.length} file(s) to library`, 'success');
    } catch (e: any) {
      appState.showToast(`Import failed: ${e?.message || e}`, 'error');
    } finally {
      appState.isLoading = false;
    }
  }

  async function handleFileInputChange(e: Event) {
    const input = e.target as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;

    const libDir = appState.userData.settings.library_dir || './mods_library';
    appState.isLoading = true;
    let count = 0;

    try {
      for (const file of Array.from(input.files)) {
        const buffer = await file.arrayBuffer();
        const res = await saveBinaryToLibrary(file.name, new Uint8Array(buffer), libDir);
        count += res.length;
      }
      await appState.refreshLibraryMods();
      appState.showToast(`Imported ${count} file(s) to library`, 'success');
    } catch (e: any) {
      appState.showToast(`Import failed: ${e?.message || e}`, 'error');
    } finally {
      appState.isLoading = false;
      input.value = '';
    }
  }

  async function handleHtmlDrop(e: DragEvent) {
    e.preventDefault();
    isDraggingOver = false;
    if (!e.dataTransfer?.files || e.dataTransfer.files.length === 0) return;

    const libDir = appState.userData.settings.library_dir || './mods_library';
    appState.isLoading = true;
    let count = 0;

    try {
      for (const file of Array.from(e.dataTransfer.files)) {
        const buffer = await file.arrayBuffer();
        const res = await saveBinaryToLibrary(file.name, new Uint8Array(buffer), libDir);
        count += res.length;
      }
      await appState.refreshLibraryMods();
      appState.showToast(`Imported ${count} file(s) to library`, 'success');
    } catch (e: any) {
      appState.showToast(`Import failed: ${e?.message || e}`, 'error');
    } finally {
      appState.isLoading = false;
    }
  }

  function handleModContextMenu(e: MouseEvent, mod: ModFile) {
    const isActive = appState.isModActiveInProfile(mod);
    const hash = mod.sha256 || mod.sha1;

    contextMenu.open(e, [
      {
        label: isActive ? 'Disable in Active Profile' : 'Enable in Active Profile',
        icon: isActive ? X : Check,
        action: () => appState.toggleModInActiveProfile(hash),
      },
      {
        label: 'View Mod Details',
        icon: Layers,
        action: () => (appState.selectedMod = mod),
      },
      {
        label: 'Open File in Explorer',
        icon: FolderOpen,
        action: () => {
          openPathInExplorer(mod.path);
        },
      },
      {
        label: 'Copy Mod Name',
        icon: Copy,
        action: () => {
          navigator.clipboard.writeText(mod.metadata.name || mod.file_name);
          appState.showToast('Copied mod name to clipboard', 'info');
        },
      },
      {
        label: 'Copy SHA256 Hash',
        icon: Copy,
        action: () => {
          navigator.clipboard.writeText(hash);
          appState.showToast('Copied hash to clipboard', 'info');
        },
      },
      { separator: true, label: '', action: () => {} },
      {
        label: 'Delete from Library',
        icon: Trash2,
        danger: true,
        action: async () => {
          try {
            await deleteModFile(mod.path);
            await appState.refreshLibraryMods();
            appState.showToast(`Deleted "${mod.file_name}"`, 'success');
          } catch (err: any) {
            appState.showToast(`Failed to delete: ${err?.message || err}`, 'error');
          }
        },
      },
    ]);
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  function getStatusBadge(mod: ModFile): { label: string; color: string; icon: any } {
    if (mod.metadata.kind === 'Plugin') {
      return { label: 'Server plugin', color: 'bg-blue-500/10 text-blue-400 border-blue-500/30', icon: Server };
    }

    const currentProf = appState.activeProfile;
    const currentLoader = currentProf?.loader || 'Fabric';
    const currentMc = currentProf?.minecraft_version || '1.20.1';

    const modLoaders = mod.metadata.loaders.map((l) => l.toLowerCase());
    if (modLoaders.length > 0 && !modLoaders.includes(currentLoader.toLowerCase())) {
      return { label: 'Incompatible', color: 'bg-rose-500/10 text-rose-400 border-rose-500/30', icon: XCircle };
    }

    if (mod.metadata.mc_versions.length > 0) {
      const match = mod.metadata.mc_versions.some((v) => currentMc.includes(v) || v.includes(currentMc) || v === '*');
      if (!match) {
        return { label: 'Incompatible', color: 'bg-rose-500/10 text-rose-400 border-rose-500/30', icon: XCircle };
      }
    }

    return { label: 'Likely compatible', color: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30', icon: CheckCircle2 };
  }
</script>

<div
  role="region"
  aria-label="Mod Library"
  class="flex-1 flex flex-col h-full bg-[#09090b] overflow-hidden relative"
  ondragover={(e) => {
    e.preventDefault();
    isDraggingOver = true;
  }}
  ondragleave={() => (isDraggingOver = false)}
  ondrop={handleHtmlDrop}
>
  <!-- Drag and drop overlay -->
  {#if isDraggingOver}
    <div class="absolute inset-0 z-40 bg-violet-950/80 backdrop-blur-xs border-2 border-dashed border-violet-400 rounded-xl flex flex-col items-center justify-center pointer-events-none p-6 text-center animate-in fade-in duration-150">
      <div class="w-16 h-16 rounded-2xl bg-violet-600/30 border border-violet-400 flex items-center justify-center text-violet-300 mb-4 shadow-xl">
        <Upload class="w-8 h-8 animate-bounce" />
      </div>
      <h3 class="text-base font-bold text-white mb-1">{t('Drop files to import into Library')}</h3>
      <p class="text-xs text-violet-200">{t('Supports .jar mods, .zip archives, and .mrpack / .morgpack packages')}</p>
    </div>
  {/if}

  <!-- Filter Toolbar -->
  <div class="px-6 py-3 border-b border-[#27272a]/60 bg-[#0c0d10]/40 flex flex-wrap items-center justify-between gap-3 shrink-0">
    <!-- Filter pills -->
    <div class="flex flex-wrap items-center gap-2">
      <!-- Import / Add Files Button -->
      <button
        onclick={() => fileInputRef?.click()}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-violet-600 hover:bg-violet-500 text-white text-xs font-semibold transition-all shadow-md shadow-violet-950/40"
      >
        <Plus class="w-3.5 h-3.5" />
        <span>{t('Import Mods')}</span>
      </button>

      <input
        type="file"
        multiple
        accept=".jar,.zip,.mrpack,.morgpack"
        class="hidden"
        bind:this={fileInputRef}
        onchange={handleFileInputChange}
      />

      <!-- Loader filter -->
      <div class="flex items-center bg-zinc-900 border border-zinc-700/60 rounded-lg p-0.5 text-xs">
        {#each loaders as ldr}
          <button
            onclick={() => (appState.selectedLoader = ldr)}
            class="px-2.5 py-1 rounded-md capitalize transition-colors {appState.selectedLoader === ldr ? 'bg-violet-600 text-white font-medium shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
          >
            {ldr === 'all' ? t('All Loaders') : ldr}
          </button>
        {/each}
      </div>

      <!-- MC Version dropdown -->
      {#if appState.allMcVersions.length > 0}
        <select
          bind:value={appState.selectedMcVersion}
          class="bg-zinc-900 border border-zinc-700/60 text-xs text-zinc-300 rounded-lg px-2.5 py-1.5 focus:outline-none focus:border-violet-500"
        >
          <option value="all">{t('All Versions')}</option>
          {#each appState.allMcVersions as ver}
            <option value={ver}>{ver}</option>
          {/each}
        </select>
      {/if}

      <!-- Tag filter -->
      {#if appState.allTags.length > 0}
        <select
          bind:value={appState.selectedTag}
          class="bg-zinc-900 border border-zinc-700/60 text-xs text-zinc-300 rounded-lg px-2.5 py-1.5 focus:outline-none focus:border-violet-500"
        >
          <option value="all">{t('All Tags')}</option>
          {#each appState.allTags as tag}
            <option value={tag}>{tag}</option>
          {/each}
        </select>
      {/if}
    </div>

    <!-- Stats -->
    <div class="flex items-center gap-3 text-xs text-zinc-400">
      <div>
        <span>{t('Total')}:</span>
        <span class="font-mono font-semibold text-zinc-200">{appState.libraryMods.length}</span>
      </div>
      <div class="w-1 h-1 rounded-full bg-zinc-600"></div>
      <div>
        <span>{t('Filtered')}:</span>
        <span class="font-mono font-semibold text-violet-400">{appState.filteredMods.length}</span>
      </div>
    </div>
  </div>

  <!-- Table Container -->
  <div class="flex-1 overflow-auto">
    {#if appState.filteredMods.length === 0}
      <div class="h-full flex flex-col items-center justify-center text-center p-8 text-zinc-400">
        <FileBox class="w-12 h-12 text-zinc-600 mb-3" />
        <h3 class="text-sm font-semibold text-zinc-300 mb-1">{t('No mods found')}</h3>
        <p class="text-xs text-zinc-500 max-w-sm mb-4">
          {appState.searchQuery || appState.selectedLoader !== 'all'
            ? t('Try adjusting your search or filters.')
            : t('Drop .jar files here, click "Import Mods", or install from Modrinth.')}
        </p>
        <div class="flex items-center gap-3">
          <button
            onclick={() => fileInputRef?.click()}
            class="px-4 py-2 rounded-lg bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-xs font-semibold text-white transition-all"
          >
            {t('Import Mods')}
          </button>
          <button
            onclick={() => (appState.activeTab = 'modrinth')}
            class="px-4 py-2 rounded-lg bg-violet-600 hover:bg-violet-500 text-xs font-semibold text-white transition-all shadow-md shadow-violet-900/40"
          >
            {t('Browse Modrinth')}
          </button>
        </div>
      </div>
    {:else}
      <table class="w-full text-left border-collapse text-xs">
        <thead class="bg-[#0c0d10] text-zinc-400 border-b border-[#27272a] sticky top-0 z-10 select-none">
          <tr>
            <th class="py-2.5 px-4 w-12 text-center">{t('Status')}</th>
            <th class="py-2.5 px-4">{t('Mod Name')}</th>
            <th class="py-2.5 px-3 w-28">{t('Version')}</th>
            <th class="py-2.5 px-3 w-32">{t('Loaders')}</th>
            <th class="py-2.5 px-3 w-32">{t('Compatibility')}</th>
            <th class="py-2.5 px-3 w-24 text-right">{t('Size')}</th>
            <th class="py-2.5 px-4 w-10"></th>
          </tr>
        </thead>
        <tbody class="divide-y divide-[#27272a]/40 text-zinc-300">
          {#each appState.filteredMods as mod}
            {@const isActive = appState.isModActiveInProfile(mod)}
            {@const badge = getStatusBadge(mod)}
            {@const BadgeIcon = badge.icon}
            {@const isSelected = appState.selectedMod?.file_name === mod.file_name}

            <tr
              onclick={() => (appState.selectedMod = mod)}
              oncontextmenu={(e) => handleModContextMenu(e, mod)}
              class="group hover:bg-zinc-800/40 cursor-pointer transition-colors {isSelected ? 'bg-violet-950/20 hover:bg-violet-950/30' : ''}"
            >
              <!-- Enable / Disable Toggle -->
              <td class="py-2.5 px-4 text-center" onclick={(e) => e.stopPropagation()}>
                <input
                  type="checkbox"
                  checked={isActive}
                  onchange={() => appState.toggleModInActiveProfile(mod.sha256 || mod.sha1)}
                  class="w-4 h-4 rounded border-zinc-700 bg-zinc-900 text-violet-600 focus:ring-0 focus:ring-offset-0 cursor-pointer accent-violet-600"
                  title={isActive ? t('Disable in Profile') : t('Enable in Profile')}
                />
              </td>

              <!-- Mod Info & Icon -->
              <td class="py-2.5 px-4">
                <div class="flex items-center gap-3">
                  {#if mod.metadata.icon_url}
                    <img
                      src={mod.metadata.icon_url}
                      alt={mod.metadata.name}
                      class="w-8 h-8 rounded-lg object-contain bg-zinc-900 border border-zinc-800 shrink-0"
                    />
                  {:else}
                    <div class="w-8 h-8 rounded-lg bg-zinc-800 border border-zinc-700 flex items-center justify-center shrink-0 text-zinc-400">
                      <Boxes class="w-4 h-4" />
                    </div>
                  {/if}

                  <div class="truncate max-w-xs md:max-w-md">
                    <div class="font-medium text-zinc-100 truncate group-hover:text-violet-300 transition-colors">
                      {mod.metadata.name || mod.file_name}
                    </div>
                    <div class="text-[11px] text-zinc-500 font-mono truncate">
                      {mod.file_name}
                    </div>
                  </div>
                </div>
              </td>

              <!-- Version -->
              <td class="py-2.5 px-3 font-mono text-zinc-300">
                <span class="px-1.5 py-0.5 rounded bg-zinc-900 border border-zinc-800 text-[11px]">
                  {mod.metadata.version || '1.0.0'}
                </span>
              </td>

              <!-- Loaders -->
              <td class="py-2.5 px-3">
                <div class="flex flex-wrap gap-1">
                  {#each mod.metadata.loaders as ldr}
                    <span class="px-1.5 py-0.5 rounded text-[10px] font-medium bg-zinc-800/80 text-zinc-300 border border-zinc-700/50">
                      {ldr}
                    </span>
                  {:else}
                    <span class="text-zinc-600 text-[11px]">—</span>
                  {/each}
                </div>
              </td>

              <!-- Status Badge -->
              <td class="py-2.5 px-3">
                <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-medium border {badge.color}">
                  <BadgeIcon class="w-3 h-3" />
                  <span>{t(badge.label)}</span>
                </span>
              </td>

              <!-- Size -->
              <td class="py-2.5 px-3 text-right font-mono text-zinc-400 text-[11px]">
                {formatBytes(mod.size)}
              </td>

              <!-- Detail arrow -->
              <td class="py-2.5 px-4 text-zinc-600 group-hover:text-zinc-300 transition-colors">
                <ChevronRight class="w-4 h-4" />
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
</div>
