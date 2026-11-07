<script lang="ts">
  import { appState } from '$lib/store.svelte';
  import { t } from '$lib/i18n.svelte';
  import { exportModpack, listInstanceAssets } from '$lib/api';
  import type { InstanceAssets, Profile } from '$lib/types';
  import {
    X,
    Download,
    Package,
    FolderHeart,
    Gamepad2,
    Check,
    Boxes,
    Palette,
    Sun,
    Map,
    FileText,
    HardDrive,
  } from '@lucide/svelte';

  let { profile, onclose }: { profile: Profile; onclose: () => void } = $props();

  let includeConfigs = $state(true);
  let includeResourcepacks = $state(true);
  let includeShaderpacks = $state(true);
  let includeSaves = $state(false);

  let selectedSaves = $state<string[]>([]);
  let outputPath = $state('');
  let isExporting = $state(false);

  let assets = $state<InstanceAssets>({
    resourcepacks: [],
    shaderpacks: [],
    saves: [],
  });

  $effect(() => {
    if (!outputPath) {
      outputPath = `./exports/${profile.name.replace(/[^a-zA-Z0-9_\-]/g, '_')}.morgpack`;
    }
    if (profile.instance_path) {
      listInstanceAssets(profile.instance_path)
        .then((res) => {
          assets = res;
          selectedSaves = res.saves.map((s) => s.folder_name);
        })
        .catch((e) => console.warn('Could not list instance assets', e));
    }
  });

  function toggleSaveSelection(name: string) {
    if (selectedSaves.includes(name)) {
      selectedSaves = selectedSaves.filter((s) => s !== name);
    } else {
      selectedSaves = [...selectedSaves, name];
    }
  }

  async function handleExport() {
    isExporting = true;
    const libraryDir = appState.userData.settings.library_dir || './mods_library';

    try {
      const res = await exportModpack(
        profile.id,
        libraryDir,
        outputPath,
        includeConfigs,
        includeResourcepacks,
        includeShaderpacks,
        includeSaves,
        includeSaves ? selectedSaves : []
      );

      const sizeMb = (res.archive_size / (1024 * 1024)).toFixed(1);
      appState.showToast(
        `Exported modpack "${profile.name}" (${sizeMb} MB) with ${res.mods_count} mods, ${res.resourcepacks_count} resourcepacks, ${res.shaderpacks_count} shaders, ${res.saves_count} saves!`,
        'success'
      );
      onclose();
    } catch (e: any) {
      appState.showToast(`Export failed: ${e?.message || e}`, 'error');
    } finally {
      isExporting = false;
    }
  }
</script>

<!-- Backdrop -->
<button
  type="button"
  aria-label="Close export modal"
  class="fixed inset-0 bg-black/60 backdrop-blur-xs z-50 flex items-center justify-center p-4 w-full h-full border-0 cursor-default"
  onclick={onclose}
></button>

<!-- Modal Container -->
<div
  role="dialog"
  aria-modal="true"
  tabindex="-1"
  onkeydown={(e) => e.stopPropagation()}
  class="fixed inset-4 md:inset-x-auto md:top-16 md:bottom-16 md:w-[640px] mx-auto z-50 bg-[#121316] border border-zinc-700 rounded-2xl shadow-2xl flex flex-col overflow-hidden text-xs text-zinc-300"
  onclick={(e) => e.stopPropagation()}
>
  <!-- Header -->
  <div class="px-6 py-4 border-b border-zinc-800 bg-[#0c0d10]/60 flex items-center justify-between shrink-0">
    <div class="flex items-center gap-3">
      <div class="w-8 h-8 rounded-lg bg-violet-600/20 border border-violet-500/40 flex items-center justify-center text-violet-400">
        <Package class="w-4 h-4" />
      </div>
      <div>
        <h3 class="font-bold text-sm text-zinc-100 flex items-center gap-2">
          <span>{t('Export Modpack')}</span>
          <span class="text-[10px] font-mono px-2 py-0.5 rounded-full bg-violet-500/20 text-violet-300 border border-violet-500/30">
            .morgpack
          </span>
        </h3>
        <p class="text-[11px] text-zinc-400">
          {t('Bundle mods, configs, resourcepacks, shaders, and worlds for easy sharing')}
        </p>
      </div>
    </div>

    <button
      onclick={onclose}
      class="p-1.5 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 transition-colors"
    >
      <X class="w-4 h-4" />
    </button>
  </div>

  <!-- Body -->
  <div class="flex-1 overflow-y-auto p-6 space-y-5">
    <!-- File Destination -->
    <div class="space-y-1.5">
      <span class="text-[11px] font-semibold text-zinc-400 uppercase">{t('Output Archive Path')}</span>
      <input
        type="text"
        bind:value={outputPath}
        class="w-full px-3 py-2 bg-zinc-900 border border-zinc-700 rounded-lg text-xs font-mono text-zinc-200 focus:outline-none focus:border-violet-500"
      />
    </div>

    <!-- Package Contents Options -->
    <div class="space-y-2.5">
      <span class="text-[11px] font-semibold text-zinc-400 uppercase">{t('Included Components')}</span>

      <!-- Mods -->
      <label class="flex items-center justify-between p-3 rounded-xl bg-zinc-900/60 border border-zinc-800 cursor-default">
        <div class="flex items-center gap-3">
          <Boxes class="w-4 h-4 text-violet-400" />
          <div>
            <div class="font-semibold text-zinc-200">{t('Enabled Mods')}</div>
            <div class="text-[11px] text-zinc-500">{profile.enabled_mod_hashes.length} {t('mods from current profile')}</div>
          </div>
        </div>
        <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/30 font-semibold">
          {t('Always included')}
        </span>
      </label>

      <!-- Configs -->
      <label class="flex items-center justify-between p-3 rounded-xl bg-zinc-900/60 border border-zinc-800 hover:border-zinc-700 cursor-pointer">
        <div class="flex items-center gap-3">
          <FileText class="w-4 h-4 text-blue-400" />
          <div>
            <div class="font-semibold text-zinc-200">{t('Mod Configurations')}</div>
            <div class="text-[11px] text-zinc-500">{t('Game configuration files from')} config/</div>
          </div>
        </div>
        <input
          type="checkbox"
          bind:checked={includeConfigs}
          class="w-4 h-4 rounded border-zinc-700 bg-zinc-900 text-violet-600 focus:ring-0 cursor-pointer accent-violet-600"
        />
      </label>

      <!-- Resourcepacks -->
      <label class="flex items-center justify-between p-3 rounded-xl bg-zinc-900/60 border border-zinc-800 hover:border-zinc-700 cursor-pointer">
        <div class="flex items-center gap-3">
          <Palette class="w-4 h-4 text-emerald-400" />
          <div>
            <div class="font-semibold text-zinc-200">{t('Resource Packs')}</div>
            <div class="text-[11px] text-zinc-500">
              {assets.resourcepacks.length} {t('pack(s) found in environment')}
            </div>
          </div>
        </div>
        <input
          type="checkbox"
          bind:checked={includeResourcepacks}
          class="w-4 h-4 rounded border-zinc-700 bg-zinc-900 text-violet-600 focus:ring-0 cursor-pointer accent-violet-600"
        />
      </label>

      <!-- Shaderpacks -->
      <label class="flex items-center justify-between p-3 rounded-xl bg-zinc-900/60 border border-zinc-800 hover:border-zinc-700 cursor-pointer">
        <div class="flex items-center gap-3">
          <Sun class="w-4 h-4 text-amber-400" />
          <div>
            <div class="font-semibold text-zinc-200">{t('Shader Packs')}</div>
            <div class="text-[11px] text-zinc-500">
              {assets.shaderpacks.length} {t('shader(s) found in environment')}
            </div>
          </div>
        </div>
        <input
          type="checkbox"
          bind:checked={includeShaderpacks}
          class="w-4 h-4 rounded border-zinc-700 bg-zinc-900 text-violet-600 focus:ring-0 cursor-pointer accent-violet-600"
        />
      </label>

      <!-- Saves / Worlds -->
      <div class="p-3 rounded-xl bg-zinc-900/60 border border-zinc-800 space-y-3">
        <label class="flex items-center justify-between cursor-pointer">
          <div class="flex items-center gap-3">
            <Map class="w-4 h-4 text-rose-400" />
            <div>
              <div class="font-semibold text-zinc-200">{t('Worlds & Saves')}</div>
              <div class="text-[11px] text-zinc-500">
                {assets.saves.length} {t('world(s) available')}
              </div>
            </div>
          </div>
          <input
            type="checkbox"
            bind:checked={includeSaves}
            class="w-4 h-4 rounded border-zinc-700 bg-zinc-900 text-violet-600 focus:ring-0 cursor-pointer accent-violet-600"
          />
        </label>

        {#if includeSaves && assets.saves.length > 0}
          <div class="pt-2 border-t border-zinc-800/80 space-y-1.5 pl-7">
            <div class="text-[11px] font-semibold text-zinc-400">{t('Select worlds to include:')}</div>
            {#each assets.saves as save}
              <label class="flex items-center justify-between p-2 rounded-lg bg-zinc-900 border border-zinc-800 hover:border-zinc-700 cursor-pointer text-xs">
                <span class="font-mono text-zinc-300">{save.folder_name}</span>
                <input
                  type="checkbox"
                  checked={selectedSaves.includes(save.folder_name)}
                  onchange={() => toggleSaveSelection(save.folder_name)}
                  class="w-3.5 h-3.5 rounded border-zinc-700 bg-zinc-900 text-violet-600 focus:ring-0 accent-violet-600"
                />
              </label>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>

  <!-- Footer -->
  <div class="px-6 py-4 border-t border-zinc-800 bg-[#0c0d10]/60 flex items-center justify-end gap-3 shrink-0">
    <button
      onclick={onclose}
      disabled={isExporting}
      class="px-4 py-2 rounded-lg text-zinc-400 hover:text-zinc-200 transition-colors"
    >
      {t('Cancel')}
    </button>

    <button
      onclick={handleExport}
      disabled={isExporting}
      class="flex items-center gap-2 px-5 py-2 bg-violet-600 hover:bg-violet-500 text-white rounded-lg text-xs font-semibold transition-all shadow-md shadow-violet-950/40"
    >
      <Download class="w-4 h-4 {isExporting ? 'animate-bounce' : ''}" />
      <span>{isExporting ? t('Exporting...') : t('Export Modpack')}</span>
    </button>
  </div>
</div>
