<script lang="ts">
  import { appState } from '$lib/store.svelte';
  import { t } from '$lib/i18n';
  import { discoverInstanceEnvironments } from '$lib/api';
  import type { Instance } from '$lib/types';
  import {
    Gamepad2,
    FolderPlus,
    Activity,
    Check,
    Layers,
    FolderOpen,
    Sparkles,
    RefreshCw,
  } from '@lucide/svelte';

  let customPathInput = $state('');

  async function handleAddCustomPath() {
    if (!customPathInput.trim()) return;
    try {
      const discovered = await discoverInstanceEnvironments(customPathInput.trim());
      if (discovered.length > 0) {
        const currentCustom = [...appState.userData.settings.custom_game_dirs];
        if (!currentCustom.includes(customPathInput.trim())) {
          currentCustom.push(customPathInput.trim());
          await appState.saveSettings({
            ...appState.userData.settings,
            custom_game_dirs: currentCustom,
          });
        }
        await appState.refreshInstances();
        appState.showToast(`Found ${discovered.length} environment(s)`, 'success');
        customPathInput = '';
      } else {
        appState.showToast('No Minecraft environments found in that folder', 'error');
      }
    } catch (e: any) {
      appState.showToast(`Error scanning directory: ${e?.message || e}`, 'error');
    }
  }

  function handleSelectForProfile(inst: Instance) {
    const prof = appState.activeProfile();
    if (!prof) {
      appState.showToast('Please select a profile first', 'info');
      return;
    }
    appState.saveProfileItem({
      ...prof,
      instance_id: inst.id,
      instance_path: inst.path,
      minecraft_version: inst.minecraft_version,
      loader: inst.loader,
      loader_version: inst.loader_version,
    });
    appState.selectedInstance = inst;
    appState.showToast(`Linked "${inst.name}" to profile "${prof.name}"`, 'success');
  }

  function handleOpenDiagnostics(inst: Instance) {
    appState.selectedInstance = inst;
    appState.activeTab = 'diagnostics';
  }
</script>

<div class="flex-1 flex flex-col h-full bg-[#09090b] overflow-hidden">
  <!-- Header Bar -->
  <div class="px-6 py-4 border-b border-[#27272a]/60 bg-[#0c0d10]/40 flex flex-wrap items-center justify-between gap-4 shrink-0">
    <div class="flex items-center gap-3">
      <div class="flex gap-2">
        <input
          type="text"
          placeholder={t('Add game folder path (e.g. .minecraft or custom)...')}
          bind:value={customPathInput}
          onkeydown={(e) => e.key === 'Enter' && handleAddCustomPath()}
          class="w-80 px-3 py-1.5 bg-zinc-900 border border-zinc-700 rounded-lg text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none focus:border-violet-500"
        />
        <button
          onclick={handleAddCustomPath}
          class="flex items-center gap-1.5 px-3 py-1.5 bg-violet-600 hover:bg-violet-500 text-white rounded-lg text-xs font-semibold transition-colors"
        >
          <FolderPlus class="w-3.5 h-3.5" />
          <span>{t('Add Folder')}</span>
        </button>
      </div>
    </div>

    <div class="text-xs text-zinc-400">
      <span>{t('Found')}:</span>
      <span class="font-mono font-semibold text-zinc-200 ml-1">{appState.instances.length}</span>
    </div>
  </div>

  <!-- Instances Grid -->
  <div class="flex-1 overflow-y-auto p-6">
    <div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
      {#each appState.instances as inst}
        {@const isActiveForProfile = appState.activeProfile()?.instance_path === inst.path}
        <div class="p-4 rounded-xl bg-zinc-900/60 border transition-all duration-150 flex flex-col justify-between {isActiveForProfile
          ? 'border-violet-500/60 bg-violet-950/10 shadow-lg shadow-violet-950/20'
          : 'border-[#27272a] hover:border-zinc-700 hover:bg-zinc-900'}">
          <div class="space-y-3">
            <!-- Launcher Tag & Status -->
            <div class="flex items-center justify-between">
              <span class="px-2 py-0.5 rounded-full text-[10px] font-semibold tracking-wide uppercase bg-zinc-800 text-zinc-300 border border-zinc-700/60">
                {t(inst.launcher)}
              </span>
              {#if isActiveForProfile}
                <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/30">
                  <Check class="w-3 h-3" />
                  <span>{t('Active Target')}</span>
                </span>
              {/if}
            </div>

            <!-- Instance Name & Path -->
            <div>
              <h4 class="font-semibold text-sm text-zinc-100 truncate mb-1">
                {inst.name}
              </h4>
              <p class="text-[11px] text-zinc-500 font-mono truncate" title={inst.path}>
                {inst.path}
              </p>
            </div>

            <!-- Badges (MC, Loader, Status) -->
            <div class="flex flex-wrap items-center gap-1.5 text-xs">
              <span class="px-2 py-0.5 rounded bg-zinc-800/80 border border-zinc-700/60 text-zinc-300 font-mono text-[11px]">
                MC {inst.minecraft_version === 'Not detected' ? t('Not detected') : inst.minecraft_version}
              </span>
              <span class="px-2 py-0.5 rounded bg-violet-500/10 border border-violet-500/20 text-violet-300 font-medium text-[11px]">
                {t(inst.loader)} {inst.loader_version ? `(${inst.loader_version})` : ''}
              </span>
            </div>

            <div class="text-[11px] text-zinc-400 line-clamp-2">
              {t(inst.status)}
            </div>
          </div>

          <!-- Actions Footer -->
          <div class="pt-4 border-t border-zinc-800/60 flex items-center justify-between gap-2 mt-4">
            <button
              onclick={() => handleSelectForProfile(inst)}
              class="flex-1 py-1.5 px-3 rounded-lg text-xs font-semibold transition-all {isActiveForProfile
                ? 'bg-zinc-800 text-zinc-300 cursor-default'
                : 'bg-violet-600 hover:bg-violet-500 text-white shadow-sm'}"
            >
              {isActiveForProfile ? t('Selected Target') : t('Select for Profile')}
            </button>

            <button
              onclick={() => handleOpenDiagnostics(inst)}
              title={t('Analyze logs & crashes')}
              class="p-1.5 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 border border-zinc-700/60 transition-colors"
            >
              <Activity class="w-4 h-4 text-amber-400" />
            </button>
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>
