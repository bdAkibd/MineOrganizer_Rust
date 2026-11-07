<script lang="ts">
  import { appState } from '$lib/store.svelte';
  import { t } from '$lib/i18n.svelte';
  import { listInstanceAssets } from '$lib/api';
  import type { InstanceAssets, ModFile, Profile } from '$lib/types';
  import {
    X,
    Search,
    Check,
    Boxes,
    SlidersHorizontal,
    CheckSquare,
    Square,
    CheckCircle2,
    XCircle,
    Server,
    Palette,
    Sun,
    Map,
    Folder,
  } from '@lucide/svelte';

  let { profile, onclose }: { profile: Profile; onclose: () => void } = $props();

  let activeTab = $state<'mods' | 'resourcepacks' | 'shaderpacks' | 'saves'>('mods');
  let searchQuery = $state('');
  let filterMode = $state<'all' | 'enabled' | 'disabled'>('all');

  let assets = $state<InstanceAssets>({
    resourcepacks: [],
    shaderpacks: [],
    saves: [],
  });

  $effect(() => {
    if (profile.instance_path) {
      listInstanceAssets(profile.instance_path)
        .then((res) => {
          assets = res;
        })
        .catch((e) => console.warn('Could not list instance assets', e));
    }
  });

  let enabledHashesSet = $derived(new Set(profile.enabled_mod_hashes));
  let enabledRpsSet = $derived(new Set(profile.enabled_resourcepack_hashes || []));
  let enabledSpsSet = $derived(new Set(profile.enabled_shaderpack_hashes || []));

  let filteredMods = $derived.by<ModFile[]>(() => {
    return appState.libraryMods.filter((mod) => {
      const hash = mod.sha256 || mod.sha1;
      const isEnabled = enabledHashesSet.has(hash);

      if (filterMode === 'enabled' && !isEnabled) return false;
      if (filterMode === 'disabled' && isEnabled) return false;

      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase();
        const matchesName = mod.file_name.toLowerCase().includes(q) || mod.metadata.name.toLowerCase().includes(q);
        const matchesId = mod.metadata.id.toLowerCase().includes(q);
        if (!matchesName && !matchesId) return false;
      }

      return true;
    });
  });

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  async function toggleMod(mod: ModFile) {
    const hash = mod.sha256 || mod.sha1;
    const current = new Set(profile.enabled_mod_hashes);
    if (current.has(hash)) {
      current.delete(hash);
    } else {
      current.add(hash);
    }

    const updated: Profile = {
      ...profile,
      enabled_mod_hashes: Array.from(current),
      active_mods: Array.from(current),
    };
    profile = updated;
    await appState.saveProfileItem(updated);
  }

  async function toggleResourcepack(filename: string) {
    const current = new Set(profile.enabled_resourcepack_hashes || []);
    if (current.has(filename)) {
      current.delete(filename);
    } else {
      current.add(filename);
    }

    const updated: Profile = {
      ...profile,
      enabled_resourcepack_hashes: Array.from(current),
    };
    profile = updated;
    await appState.saveProfileItem(updated);
  }

  async function toggleShaderpack(filename: string) {
    const current = new Set(profile.enabled_shaderpack_hashes || []);
    if (current.has(filename)) {
      current.delete(filename);
    } else {
      current.add(filename);
    }

    const updated: Profile = {
      ...profile,
      enabled_shaderpack_hashes: Array.from(current),
    };
    profile = updated;
    await appState.saveProfileItem(updated);
  }

  async function handleSelectAll() {
    if (activeTab === 'mods') {
      const current = new Set(profile.enabled_mod_hashes);
      for (const mod of filteredMods) {
        current.add(mod.sha256 || mod.sha1);
      }
      const updated: Profile = {
        ...profile,
        enabled_mod_hashes: Array.from(current),
        active_mods: Array.from(current),
      };
      profile = updated;
      await appState.saveProfileItem(updated);
    } else if (activeTab === 'resourcepacks') {
      const current = new Set(assets.resourcepacks.map((r) => r.filename));
      const updated: Profile = {
        ...profile,
        enabled_resourcepack_hashes: Array.from(current),
      };
      profile = updated;
      await appState.saveProfileItem(updated);
    } else if (activeTab === 'shaderpacks') {
      const current = new Set(assets.shaderpacks.map((s) => s.filename));
      const updated: Profile = {
        ...profile,
        enabled_shaderpack_hashes: Array.from(current),
      };
      profile = updated;
      await appState.saveProfileItem(updated);
    }
  }

  async function handleDeselectAll() {
    if (activeTab === 'mods') {
      const current = new Set(profile.enabled_mod_hashes);
      for (const mod of filteredMods) {
        current.delete(mod.sha256 || mod.sha1);
      }
      const updated: Profile = {
        ...profile,
        enabled_mod_hashes: Array.from(current),
        active_mods: Array.from(current),
      };
      profile = updated;
      await appState.saveProfileItem(updated);
    } else if (activeTab === 'resourcepacks') {
      const updated: Profile = {
        ...profile,
        enabled_resourcepack_hashes: [],
      };
      profile = updated;
      await appState.saveProfileItem(updated);
    } else if (activeTab === 'shaderpacks') {
      const updated: Profile = {
        ...profile,
        enabled_shaderpack_hashes: [],
      };
      profile = updated;
      await appState.saveProfileItem(updated);
    }
  }

  function getModStatusBadge(mod: ModFile) {
    if (mod.metadata.kind === 'Plugin') {
      return { label: 'Server plugin', color: 'bg-blue-500/10 text-blue-400 border-blue-500/30' };
    }
    const loader = profile.loader || 'Fabric';
    const mc = profile.minecraft_version || '1.20.1';

    const modLoaders = mod.metadata.loaders.map((l) => l.toLowerCase());
    if (modLoaders.length > 0 && !modLoaders.includes(loader.toLowerCase())) {
      return { label: 'Incompatible', color: 'bg-rose-500/10 text-rose-400 border-rose-500/30' };
    }

    if (mod.metadata.mc_versions.length > 0) {
      const match = mod.metadata.mc_versions.some((v) => mc.includes(v) || v.includes(mc) || v === '*');
      if (!match) {
        return { label: 'Incompatible', color: 'bg-rose-500/10 text-rose-400 border-rose-500/30' };
      }
    }

    return { label: 'Compatible', color: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30' };
  }
</script>

<!-- Backdrop -->
<button
  type="button"
  aria-label="Close modal backdrop"
  class="fixed inset-0 bg-black/60 backdrop-blur-xs z-50 flex items-center justify-center p-4 w-full h-full border-0 cursor-default"
  onclick={onclose}
></button>

<!-- Modal Container -->
<div
  role="dialog"
  aria-modal="true"
  tabindex="-1"
  onkeydown={(e) => e.stopPropagation()}
  class="fixed inset-4 md:inset-x-auto md:top-10 md:bottom-10 md:w-[760px] mx-auto z-50 bg-[#121316] border border-zinc-700 rounded-2xl shadow-2xl flex flex-col overflow-hidden text-xs text-zinc-300"
  onclick={(e) => e.stopPropagation()}
>
  <!-- Header -->
  <div class="px-6 py-4 border-b border-zinc-800 bg-[#0c0d10]/60 flex items-center justify-between shrink-0">
    <div class="flex items-center gap-3">
      <div class="w-8 h-8 rounded-lg bg-violet-600/20 border border-violet-500/40 flex items-center justify-center text-violet-400">
        <SlidersHorizontal class="w-4 h-4" />
      </div>
      <div>
        <h3 class="font-bold text-sm text-zinc-100 flex items-center gap-2">
          <span>{profile.name}</span>
          <span class="text-[10px] font-mono px-2 py-0.5 rounded-full bg-violet-500/20 text-violet-300 border border-violet-500/30">
            {profile.minecraft_version || '1.20.1'} · {profile.loader || 'Fabric'}
          </span>
        </h3>
        <p class="text-[11px] text-zinc-400">
          {profile.enabled_mod_hashes.length} {t('mods')}, {profile.enabled_resourcepack_hashes?.length || 0} {t('resourcepacks')}, {profile.enabled_shaderpack_hashes?.length || 0} {t('shaders')}
        </p>
      </div>
    </div>

    <button
      onclick={onclose}
      class="p-1.5 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 transition-colors"
      title={t('Close')}
    >
      <X class="w-4 h-4" />
    </button>
  </div>

  <!-- Category Tabs -->
  <div class="px-6 border-b border-zinc-800 bg-zinc-900/40 flex items-center gap-4 shrink-0 select-none">
    <button
      onclick={() => (activeTab = 'mods')}
      class="py-3 px-1 border-b-2 flex items-center gap-2 font-medium transition-colors {activeTab === 'mods'
        ? 'border-violet-500 text-violet-300 font-semibold'
        : 'border-transparent text-zinc-400 hover:text-zinc-200'}"
    >
      <Boxes class="w-3.5 h-3.5" />
      <span>{t('Mods')} ({profile.enabled_mod_hashes.length}/{appState.libraryMods.length})</span>
    </button>

    <button
      onclick={() => (activeTab = 'resourcepacks')}
      class="py-3 px-1 border-b-2 flex items-center gap-2 font-medium transition-colors {activeTab === 'resourcepacks'
        ? 'border-violet-500 text-violet-300 font-semibold'
        : 'border-transparent text-zinc-400 hover:text-zinc-200'}"
    >
      <Palette class="w-3.5 h-3.5" />
      <span>{t('Resource Packs')} ({profile.enabled_resourcepack_hashes?.length || 0}/{assets.resourcepacks.length})</span>
    </button>

    <button
      onclick={() => (activeTab = 'shaderpacks')}
      class="py-3 px-1 border-b-2 flex items-center gap-2 font-medium transition-colors {activeTab === 'shaderpacks'
        ? 'border-violet-500 text-violet-300 font-semibold'
        : 'border-transparent text-zinc-400 hover:text-zinc-200'}"
    >
      <Sun class="w-3.5 h-3.5" />
      <span>{t('Shader Packs')} ({profile.enabled_shaderpack_hashes?.length || 0}/{assets.shaderpacks.length})</span>
    </button>

    <button
      onclick={() => (activeTab = 'saves')}
      class="py-3 px-1 border-b-2 flex items-center gap-2 font-medium transition-colors {activeTab === 'saves'
        ? 'border-violet-500 text-violet-300 font-semibold'
        : 'border-transparent text-zinc-400 hover:text-zinc-200'}"
    >
      <Map class="w-3.5 h-3.5" />
      <span>{t('Worlds / Saves')} ({assets.saves.length})</span>
    </button>
  </div>

  <!-- Search & Quick Actions Bar (For Mods, Resource Packs, Shader Packs) -->
  {#if activeTab !== 'saves'}
    <div class="px-6 py-2.5 border-b border-zinc-800/80 bg-zinc-900/30 flex flex-wrap items-center justify-between gap-3 shrink-0">
      {#if activeTab === 'mods'}
        <!-- Search Input -->
        <div class="relative flex-1 min-w-[180px]">
          <Search class="w-3.5 h-3.5 text-zinc-500 absolute left-3 top-1/2 -translate-y-1/2" />
          <input
            type="text"
            placeholder={t('Search mods...')}
            bind:value={searchQuery}
            class="w-full pl-8 pr-3 py-1.5 bg-zinc-900 border border-zinc-700/70 rounded-lg text-xs text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-violet-500"
          />
        </div>

        <!-- Filter Buttons -->
        <div class="flex items-center gap-1 bg-zinc-900 p-0.5 rounded-lg border border-zinc-800">
          <button
            onclick={() => (filterMode = 'all')}
            class="px-2 py-1 rounded-md text-[11px] font-medium transition-colors {filterMode === 'all' ? 'bg-violet-600 text-white' : 'text-zinc-400 hover:text-zinc-200'}"
          >
            {t('All')}
          </button>
          <button
            onclick={() => (filterMode = 'enabled')}
            class="px-2 py-1 rounded-md text-[11px] font-medium transition-colors {filterMode === 'enabled' ? 'bg-violet-600 text-white' : 'text-zinc-400 hover:text-zinc-200'}"
          >
            {t('Enabled')}
          </button>
          <button
            onclick={() => (filterMode = 'disabled')}
            class="px-2 py-1 rounded-md text-[11px] font-medium transition-colors {filterMode === 'disabled' ? 'bg-violet-600 text-white' : 'text-zinc-400 hover:text-zinc-200'}"
          >
            {t('Disabled')}
          </button>
        </div>
      {/if}

      <!-- Bulk Select Actions -->
      <div class="flex items-center gap-1.5 ml-auto">
        <button
          onclick={handleSelectAll}
          class="px-2.5 py-1 rounded-lg border border-zinc-700 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-[11px] font-medium transition-colors"
        >
          {t('Select All')}
        </button>
        <button
          onclick={handleDeselectAll}
          class="px-2.5 py-1 rounded-lg border border-zinc-700 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-[11px] font-medium transition-colors"
        >
          {t('Deselect All')}
        </button>
      </div>
    </div>
  {/if}

  <!-- Tab Contents -->
  <div class="flex-1 overflow-y-auto p-4 space-y-1.5">
    {#if activeTab === 'mods'}
      {#if filteredMods.length === 0}
        <div class="h-48 flex flex-col items-center justify-center text-center p-6 text-zinc-500">
          <Boxes class="w-10 h-10 text-zinc-700 mb-2" />
          <p class="text-xs">{t('No mods match your criteria')}</p>
        </div>
      {:else}
        {#each filteredMods as mod}
          {@const hash = mod.sha256 || mod.sha1}
          {@const isEnabled = enabledHashesSet.has(hash)}
          {@const badge = getModStatusBadge(mod)}

          <div
            role="button"
            tabindex="0"
            onclick={() => toggleMod(mod)}
            onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleMod(mod)}
            class="flex items-center justify-between p-3 rounded-xl border transition-all cursor-pointer select-none {isEnabled
              ? 'bg-violet-950/20 border-violet-500/40 hover:border-violet-500/60'
              : 'bg-zinc-900/50 border-zinc-800/80 hover:border-zinc-700'}"
          >
            <div class="flex items-center gap-3 truncate mr-3">
              <input
                type="checkbox"
                checked={isEnabled}
                onchange={() => toggleMod(mod)}
                onclick={(e) => e.stopPropagation()}
                class="w-4 h-4 rounded border-zinc-700 bg-zinc-900 text-violet-600 focus:ring-0 focus:ring-offset-0 cursor-pointer accent-violet-600 shrink-0"
              />

              {#if mod.metadata.icon_url}
                <img
                  src={mod.metadata.icon_url}
                  alt=""
                  class="w-7 h-7 rounded-lg object-contain bg-zinc-800 shrink-0 border border-zinc-700/60"
                />
              {:else}
                <div class="w-7 h-7 rounded-lg bg-zinc-800 border border-zinc-700 flex items-center justify-center shrink-0 text-zinc-500">
                  <Boxes class="w-3.5 h-3.5" />
                </div>
              {/if}

              <div class="truncate">
                <div class="font-semibold text-zinc-200 text-xs truncate">
                  {mod.metadata.name || mod.file_name}
                </div>
                <div class="text-[10px] text-zinc-500 font-mono truncate">
                  {mod.file_name}
                </div>
              </div>
            </div>

            <div class="flex items-center gap-2 shrink-0">
              <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-zinc-800 text-zinc-400 border border-zinc-700/60">
                v{mod.metadata.version || '1.0'}
              </span>

              <span class="text-[10px] px-2 py-0.5 rounded-full border {badge.color}">
                {t(badge.label)}
              </span>
            </div>
          </div>
        {/each}
      {/if}
    {:else if activeTab === 'resourcepacks'}
      {#if assets.resourcepacks.length === 0}
        <div class="h-48 flex flex-col items-center justify-center text-center p-6 text-zinc-500">
          <Palette class="w-10 h-10 text-zinc-700 mb-2" />
          <p class="text-xs">{t('No resource packs found in instance resourcepacks/ folder')}</p>
        </div>
      {:else}
        {#each assets.resourcepacks as rp}
          {@const isEnabled = enabledRpsSet.has(rp.filename)}
          <div
            role="button"
            tabindex="0"
            onclick={() => toggleResourcepack(rp.filename)}
            onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleResourcepack(rp.filename)}
            class="flex items-center justify-between p-3 rounded-xl border transition-all cursor-pointer select-none {isEnabled
              ? 'bg-violet-950/20 border-violet-500/40 hover:border-violet-500/60'
              : 'bg-zinc-900/50 border-zinc-800/80 hover:border-zinc-700'}"
          >
            <div class="flex items-center gap-3 truncate mr-3">
              <input
                type="checkbox"
                checked={isEnabled}
                onchange={() => toggleResourcepack(rp.filename)}
                onclick={(e) => e.stopPropagation()}
                class="w-4 h-4 rounded border-zinc-700 bg-zinc-900 text-violet-600 focus:ring-0 accent-violet-600 shrink-0"
              />
              <Palette class="w-5 h-5 text-emerald-400 shrink-0" />
              <div class="truncate">
                <div class="font-semibold text-zinc-200 text-xs truncate">{rp.name}</div>
                <div class="text-[10px] text-zinc-500 font-mono truncate">{rp.filename}</div>
              </div>
            </div>

            <span class="text-[11px] font-mono text-zinc-400 shrink-0">
              {formatBytes(rp.size)}
            </span>
          </div>
        {/each}
      {/if}
    {:else if activeTab === 'shaderpacks'}
      {#if assets.shaderpacks.length === 0}
        <div class="h-48 flex flex-col items-center justify-center text-center p-6 text-zinc-500">
          <Sun class="w-10 h-10 text-zinc-700 mb-2" />
          <p class="text-xs">{t('No shader packs found in instance shaderpacks/ folder')}</p>
        </div>
      {:else}
        {#each assets.shaderpacks as sp}
          {@const isEnabled = enabledSpsSet.has(sp.filename)}
          <div
            role="button"
            tabindex="0"
            onclick={() => toggleShaderpack(sp.filename)}
            onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleShaderpack(sp.filename)}
            class="flex items-center justify-between p-3 rounded-xl border transition-all cursor-pointer select-none {isEnabled
              ? 'bg-violet-950/20 border-violet-500/40 hover:border-violet-500/60'
              : 'bg-zinc-900/50 border-zinc-800/80 hover:border-zinc-700'}"
          >
            <div class="flex items-center gap-3 truncate mr-3">
              <input
                type="checkbox"
                checked={isEnabled}
                onchange={() => toggleShaderpack(sp.filename)}
                onclick={(e) => e.stopPropagation()}
                class="w-4 h-4 rounded border-zinc-700 bg-zinc-900 text-violet-600 focus:ring-0 accent-violet-600 shrink-0"
              />
              <Sun class="w-5 h-5 text-amber-400 shrink-0" />
              <div class="truncate">
                <div class="font-semibold text-zinc-200 text-xs truncate">{sp.name}</div>
                <div class="text-[10px] text-zinc-500 font-mono truncate">{sp.filename}</div>
              </div>
            </div>

            <span class="text-[11px] font-mono text-zinc-400 shrink-0">
              {formatBytes(sp.size)}
            </span>
          </div>
        {/each}
      {/if}
    {:else if activeTab === 'saves'}
      {#if assets.saves.length === 0}
        <div class="h-48 flex flex-col items-center justify-center text-center p-6 text-zinc-500">
          <Map class="w-10 h-10 text-zinc-700 mb-2" />
          <p class="text-xs">{t('No worlds / saves found in instance saves/ folder')}</p>
        </div>
      {:else}
        {#each assets.saves as save}
          <div class="flex items-center justify-between p-3 rounded-xl bg-zinc-900/50 border border-zinc-800">
            <div class="flex items-center gap-3 truncate mr-3">
              <Map class="w-5 h-5 text-rose-400 shrink-0" />
              <div class="truncate">
                <div class="font-semibold text-zinc-200 text-xs truncate">{save.folder_name}</div>
                <div class="text-[10px] text-zinc-500 font-mono truncate">{save.path}</div>
              </div>
            </div>

            <span class="text-[11px] font-mono text-zinc-400 shrink-0">
              {formatBytes(save.size)}
            </span>
          </div>
        {/each}
      {/if}
    {/if}
  </div>

  <!-- Footer -->
  <div class="px-6 py-3 border-t border-zinc-800 bg-[#0c0d10]/60 flex items-center justify-between shrink-0">
    <span class="text-[11px] text-zinc-400 font-mono">
      {#if activeTab === 'mods'}
        {profile.enabled_mod_hashes.length} {t('mods selected')}
      {:else if activeTab === 'resourcepacks'}
        {profile.enabled_resourcepack_hashes?.length || 0} {t('resourcepacks active')}
      {:else if activeTab === 'shaderpacks'}
        {profile.enabled_shaderpack_hashes?.length || 0} {t('shaderpacks active')}
      {:else}
        {assets.saves.length} {t('worlds available')}
      {/if}
    </span>

    <button
      onclick={onclose}
      class="px-5 py-2 bg-violet-600 hover:bg-violet-500 text-white rounded-lg text-xs font-semibold transition-all shadow-md shadow-violet-950/40"
    >
      {t('Done')}
    </button>
  </div>
</div>
