<script lang="ts">
  import { appState } from '$lib/store.svelte';
  import { t } from '$lib/i18n';
  import { exportModpack, previewProfile } from '$lib/api';
  import type { Profile } from '$lib/types';
  import {
    FolderHeart,
    Plus,
    Play,
    Download,
    Trash2,
    Edit2,
    Check,
    Layers,
    Boxes,
    HardDrive,
  } from '@lucide/svelte';

  let newProfileModalOpen = $state(false);
  let newProfileName = $state('');
  let selectedInstanceId = $state<string>('');

  function handleCreateProfile() {
    if (!newProfileName.trim()) return;
    const inst = appState.instances.find((i) => i.id === selectedInstanceId) || null;
    appState.createNewProfile(newProfileName.trim(), inst);
    newProfileName = '';
    newProfileModalOpen = false;
  }

  async function handleExportPack(prof: Profile) {
    const libraryDir = appState.userData.settings.library_dir || './mods_library';
    const outputPath = `./exports/${prof.name.replace(/[^a-zA-Z0-9_\-]/g, '_')}.morgpack`;

    try {
      const res = await exportModpack(prof.id, libraryDir, outputPath, true, true, true, false);
      appState.showToast(`Modpack exported! ${res.mods_count} mods included (${(res.archive_size / (1024 * 1024)).toFixed(1)} MB)`, 'success');
    } catch (e: any) {
      appState.showToast(`Export failed: ${e?.message || e}`, 'error');
    }
  }

  async function handleDeploy(prof: Profile) {
    const instancePath = prof.instance_path || appState.selectedInstance?.path;
    if (!instancePath) {
      appState.showToast('Link an instance to this profile first', 'error');
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
      appState.showToast(`Preview failed: ${e?.message || e}`, 'error');
      appState.virtualizationModalOpen = false;
    } finally {
      appState.virtualizationLoading = false;
    }
  }
</script>

<div class="flex-1 flex flex-col h-full bg-[#09090b] overflow-hidden">
  <!-- Header Bar -->
  <div class="px-6 py-4 border-b border-[#27272a]/60 bg-[#0c0d10]/40 flex items-center justify-between shrink-0">
    <div>
      <h3 class="text-sm font-semibold text-zinc-100">{t('Manage Profiles')}</h3>
      <p class="text-xs text-zinc-400">{t('Configure isolated mod sets for your environments')}</p>
    </div>

    <button
      onclick={() => (newProfileModalOpen = true)}
      class="flex items-center gap-2 px-3.5 py-1.5 bg-violet-600 hover:bg-violet-500 text-white rounded-lg text-xs font-semibold transition-all shadow-md shadow-violet-950/40"
    >
      <Plus class="w-3.5 h-3.5" />
      <span>{t('New Profile')}</span>
    </button>
  </div>

  <!-- Profiles List -->
  <div class="flex-1 overflow-y-auto p-6 space-y-4">
    {#if appState.userData.profiles.length === 0}
      <div class="h-64 flex flex-col items-center justify-center text-center p-8 text-zinc-400">
        <FolderHeart class="w-12 h-12 text-zinc-600 mb-3" />
        <h4 class="text-sm font-semibold text-zinc-300 mb-1">{t('No profiles yet')}</h4>
        <p class="text-xs text-zinc-500 mb-4">{t('Create your first profile to manage mods and deploy links.')}</p>
        <button
          onclick={() => (newProfileModalOpen = true)}
          class="px-4 py-2 rounded-lg bg-violet-600 hover:bg-violet-500 text-xs font-semibold text-white transition-all"
        >
          {t('Create Profile')}
        </button>
      </div>
    {:else}
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        {#each appState.userData.profiles as prof}
          {@const isActive = prof.id === appState.activeProfile?.id}
          <div class="p-5 rounded-xl bg-zinc-900/60 border transition-all flex flex-col justify-between {isActive
            ? 'border-violet-500/60 bg-violet-950/15 shadow-xl shadow-violet-950/30'
            : 'border-[#27272a] hover:border-zinc-700'}">
            <div class="space-y-3">
              <!-- Top Row: Name and Active Pill -->
              <div class="flex items-center justify-between">
                <h4 class="font-bold text-sm text-zinc-100 truncate flex items-center gap-2">
                  <span>{prof.name}</span>
                </h4>
                {#if isActive}
                  <span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/30">
                    <Check class="w-3 h-3" />
                    <span>{t('Active')}</span>
                  </span>
                {:else}
                  <button
                    onclick={() => appState.setActiveProfile(prof.id)}
                    class="text-[11px] font-medium text-zinc-400 hover:text-zinc-200 transition-colors"
                  >
                    {t('Make Active')}
                  </button>
                {/if}
              </div>

              <!-- Stats & Environment info -->
              <div class="grid grid-cols-2 gap-2 text-xs">
                <div class="p-2.5 rounded-lg bg-zinc-900/70 border border-zinc-800">
                  <span class="text-zinc-500 text-[10px] uppercase block">{t('Enabled Mods')}</span>
                  <span class="font-mono font-bold text-zinc-200 text-sm">{prof.enabled_mod_hashes.length}</span>
                </div>
                <div class="p-2.5 rounded-lg bg-zinc-900/70 border border-zinc-800">
                  <span class="text-zinc-500 text-[10px] uppercase block">{t('Game Version')}</span>
                  <span class="font-mono font-bold text-violet-300 text-sm">{prof.minecraft_version || '1.20.1'} ({prof.loader || 'Fabric'})</span>
                </div>
              </div>

              {#if prof.instance_path}
                <div class="text-[11px] text-zinc-500 font-mono truncate" title={prof.instance_path}>
                  {t('Target')}: {prof.instance_path}
                </div>
              {/if}
            </div>

            <!-- Profile Action Buttons -->
            <div class="pt-4 border-t border-zinc-800/60 flex items-center justify-between gap-2 mt-4">
              <div class="flex items-center gap-2 flex-1">
                <button
                  onclick={() => handleDeploy(prof)}
                  class="flex-1 flex items-center justify-center gap-1.5 py-1.5 px-3 rounded-lg text-xs font-semibold text-white bg-violet-600 hover:bg-violet-500 transition-colors shadow-sm"
                >
                  <Play class="w-3.5 h-3.5 fill-current" />
                  <span>{t('Deploy')}</span>
                </button>

                <button
                  onclick={() => handleExportPack(prof)}
                  title={t('Export as modpack')}
                  class="p-2 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 border border-zinc-700/60 transition-colors"
                >
                  <Download class="w-4 h-4" />
                </button>
              </div>

              <button
                onclick={() => appState.deleteProfile(prof.id)}
                title={t('Delete Profile')}
                class="p-2 rounded-lg text-zinc-500 hover:text-rose-400 hover:bg-rose-500/10 transition-colors"
              >
                <Trash2 class="w-4 h-4" />
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<!-- Modal: Create New Profile -->
{#if newProfileModalOpen}
  <div class="fixed inset-0 bg-black/60 backdrop-blur-xs z-50 flex items-center justify-center p-4">
    <div class="bg-[#121316] border border-zinc-700 rounded-2xl max-w-md w-full p-6 space-y-5 shadow-2xl text-xs text-zinc-300">
      <h3 class="text-sm font-bold text-zinc-100">{t('Create New Profile')}</h3>

      <div class="space-y-3">
        <div>
          <span class="block text-[11px] font-semibold text-zinc-400 uppercase mb-1">{t('Profile Name')}</span>
          <input
            type="text"
            placeholder={t('e.g. Fabric 1.20.1 Client')}
            bind:value={newProfileName}
            class="w-full px-3 py-2 bg-zinc-900 border border-zinc-700 rounded-lg text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none focus:border-violet-500"
          />
        </div>

        <div>
          <span class="block text-[11px] font-semibold text-zinc-400 uppercase mb-1">{t('Target Environment')}</span>
          <select
            bind:value={selectedInstanceId}
            class="w-full px-3 py-2 bg-zinc-900 border border-zinc-700 rounded-lg text-xs text-zinc-100 focus:outline-none focus:border-violet-500"
          >
            <option value="">{t('None / Set later')}</option>
            {#each appState.instances as inst}
              <option value={inst.id}>{inst.name} ({inst.minecraft_version} {inst.loader})</option>
            {/each}
          </select>
        </div>
      </div>

      <div class="flex items-center justify-end gap-2 pt-2 border-t border-zinc-800">
        <button
          onclick={() => (newProfileModalOpen = false)}
          class="px-4 py-2 rounded-lg text-zinc-400 hover:text-zinc-200 transition-colors"
        >
          {t('Cancel')}
        </button>
        <button
          onclick={handleCreateProfile}
          class="px-4 py-2 rounded-lg bg-violet-600 hover:bg-violet-500 text-white font-semibold transition-all shadow-md shadow-violet-900/40"
        >
          {t('Create')}
        </button>
      </div>
    </div>
  </div>
{/if}
