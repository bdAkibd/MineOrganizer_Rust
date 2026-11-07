<script lang="ts">
  import { appState } from '$lib/store.svelte';
  import { t } from '$lib/i18n';
  import { checkModUpdates } from '$lib/api';
  import {
    X,
    Boxes,
    Tag,
    Plus,
    Check,
    ExternalLink,
    RefreshCw,
    FolderOpen,
    Trash2,
    ShieldCheck,
    Layers,
    FileText,
  } from '@lucide/svelte';

  let newTagInput = $state('');
  let isCheckingUpdates = $state(false);
  let updateChangelog = $state<string | null>(null);

  const mod = $derived(appState.selectedMod);
  const modHash = $derived(mod?.sha256 || mod?.sha1 || '');
  const isActive = $derived(mod ? appState.isModActiveInProfile(mod) : false);
  const tags = $derived<string[]>(modHash ? (appState.userData.mod_tags[modHash] || []) : []);

  function handleAddTag() {
    if (!newTagInput.trim() || !modHash) return;
    const current = [...tags];
    if (!current.includes(newTagInput.trim())) {
      current.push(newTagInput.trim());
      appState.setModTagsFor(modHash, current);
    }
    newTagInput = '';
  }

  function handleRemoveTag(tagToRemove: string) {
    if (!modHash) return;
    const current = tags.filter((t: string) => t !== tagToRemove);
    appState.setModTagsFor(modHash, current);
  }

  async function handleCheckUpdate() {
    if (!mod) return;
    isCheckingUpdates = true;
    updateChangelog = null;

    try {
      const hashes = [mod.sha1];
      const updates = await checkModUpdates(hashes);
      if (updates.length > 0) {
        const up = updates[0];
        updateChangelog = `New Version Available: ${up.new_version_number}\n${up.changelog}`;
        appState.showToast(`Found update: ${up.new_version_number}`, 'success');
      } else {
        appState.showToast('You already have the latest version!', 'info');
      }
    } catch (e: any) {
      appState.showToast(`Update check failed: ${e?.message || e}`, 'error');
    } finally {
      isCheckingUpdates = false;
    }
  }
</script>

{#if mod}
  <!-- Backdrop -->
  <button
    type="button"
    aria-label="Close details"
    class="fixed inset-0 bg-black/50 backdrop-blur-xs z-30 transition-opacity w-full h-full border-0 cursor-default"
    onclick={() => (appState.selectedMod = null)}
  ></button>

  <!-- Slide-over Drawer -->
  <div class="fixed inset-y-0 right-0 w-full max-w-md bg-[#0c0d10] border-l border-[#27272a] shadow-2xl z-40 flex flex-col text-xs text-zinc-300">
    <!-- Header -->
    <div class="h-16 px-6 border-b border-[#27272a] flex items-center justify-between shrink-0 bg-[#09090b]/40">
      <div class="flex items-center gap-3 truncate">
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
        <div class="truncate">
          <h3 class="font-semibold text-zinc-100 truncate text-sm">
            {mod.metadata.name || mod.file_name}
          </h3>
          <p class="text-[11px] text-zinc-500 font-mono truncate">{mod.metadata.id}</p>
        </div>
      </div>

      <button
        onclick={() => (appState.selectedMod = null)}
        class="p-1.5 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 transition-colors"
      >
        <X class="w-4 h-4" />
      </button>
    </div>

    <!-- Scrollable Body -->
    <div class="flex-1 overflow-y-auto p-6 space-y-6">
      <!-- Active Toggle in Profile -->
      <div class="p-3.5 rounded-xl bg-zinc-900/80 border border-zinc-800 flex items-center justify-between">
        <div>
          <div class="font-medium text-zinc-200">{t('Profile Status')}</div>
          <div class="text-[11px] text-zinc-400">
            {isActive ? t('Enabled in active profile') : t('Disabled in active profile')}
          </div>
        </div>

        <button
          onclick={() => appState.toggleModInActiveProfile(mod.sha256 || mod.sha1)}
          class="px-3.5 py-1.5 rounded-lg text-xs font-semibold transition-all {isActive
            ? 'bg-rose-500/15 text-rose-300 border border-rose-500/30 hover:bg-rose-500/20'
            : 'bg-violet-600 text-white hover:bg-violet-500 shadow-md shadow-violet-900/40'}"
        >
          {isActive ? t('Disable Mod') : t('Enable Mod')}
        </button>
      </div>

      <!-- Description -->
      {#if mod.metadata.description}
        <div class="space-y-1.5">
          <div class="text-[11px] font-semibold text-zinc-400 uppercase tracking-wider">{t('Description')}</div>
          <div class="p-3 bg-zinc-900/50 border border-zinc-800/80 rounded-xl text-zinc-300 leading-relaxed">
            {mod.metadata.description}
          </div>
        </div>
      {/if}

      <!-- Metadata Properties Grid -->
      <div class="grid grid-cols-2 gap-3">
        <div class="p-3 bg-zinc-900/40 border border-zinc-800/60 rounded-xl space-y-1">
          <div class="text-[10px] text-zinc-500 uppercase">{t('Version')}</div>
          <div class="font-mono text-zinc-200 font-medium">{mod.metadata.version || '1.0.0'}</div>
        </div>
        <div class="p-3 bg-zinc-900/40 border border-zinc-800/60 rounded-xl space-y-1">
          <div class="text-[10px] text-zinc-500 uppercase">{t('Kind')}</div>
          <div class="text-zinc-200 font-medium capitalize">{mod.metadata.kind}</div>
        </div>
      </div>

      <!-- Loaders -->
      <div class="space-y-1.5">
        <div class="text-[11px] font-semibold text-zinc-400 uppercase tracking-wider">{t('Loaders')}</div>
        <div class="flex flex-wrap gap-1.5">
          {#each mod.metadata.loaders as ldr}
            <span class="px-2 py-0.5 rounded-md bg-violet-500/10 text-violet-300 border border-violet-500/20 font-medium">
              {ldr}
            </span>
          {:else}
            <span class="text-zinc-500">Not specified</span>
          {/each}
        </div>
      </div>

      <!-- Minecraft Versions -->
      <div class="space-y-1.5">
        <div class="text-[11px] font-semibold text-zinc-400 uppercase tracking-wider">{t('Minecraft Versions')}</div>
        <div class="flex flex-wrap gap-1.5">
          {#each mod.metadata.mc_versions as mc}
            <span class="px-2 py-0.5 rounded-md bg-zinc-800 text-zinc-300 border border-zinc-700/60 font-mono text-[11px]">
              {mc}
            </span>
          {:else}
            <span class="text-zinc-500">Not specified</span>
          {/each}
        </div>
      </div>

      <!-- Dependencies -->
      {#if mod.metadata.dependencies.length > 0}
        <div class="space-y-1.5">
          <div class="text-[11px] font-semibold text-zinc-400 uppercase tracking-wider">{t('Dependencies')}</div>
          <div class="space-y-1">
            {#each mod.metadata.dependencies as dep}
              <div class="p-2 bg-zinc-900/60 border border-zinc-800 rounded-lg flex items-center justify-between text-[11px]">
                <span class="font-mono text-zinc-200">{dep.id}</span>
                <span class="px-1.5 py-0.5 rounded text-[10px] {dep.required ? 'bg-amber-500/10 text-amber-400 border border-amber-500/30' : 'bg-zinc-800 text-zinc-400'}">
                  {dep.required ? t('Required') : t('Optional')}
                </span>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Tags Section -->
      <div class="space-y-2">
        <div class="text-[11px] font-semibold text-zinc-400 uppercase tracking-wider flex items-center gap-1.5">
          <Tag class="w-3.5 h-3.5" />
          <span>{t('Tags')}</span>
        </div>

        <div class="flex flex-wrap gap-1.5">
          {#each tags as tag}
            <span class="inline-flex items-center gap-1 px-2.5 py-1 rounded-full bg-zinc-800 text-zinc-300 border border-zinc-700 text-[11px]">
              <span>{tag}</span>
              <button
                onclick={() => handleRemoveTag(tag)}
                class="hover:text-rose-400 transition-colors ml-0.5"
              >
                <X class="w-3 h-3" />
              </button>
            </span>
          {/each}
        </div>

        <div class="flex gap-1.5">
          <input
            type="text"
            placeholder={t('Add tag...')}
            bind:value={newTagInput}
            onkeydown={(e) => e.key === 'Enter' && handleAddTag()}
            class="flex-1 px-3 py-1.5 bg-zinc-900 border border-zinc-700 rounded-lg text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none focus:border-violet-500"
          />
          <button
            onclick={handleAddTag}
            class="px-3 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 rounded-lg text-xs font-medium transition-colors"
          >
            <Plus class="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      <!-- Links & Updates -->
      <div class="pt-4 border-t border-zinc-800/80 space-y-2">
        {#if mod.metadata.homepage}
          <a
            href={mod.metadata.homepage}
            target="_blank"
            class="w-full flex items-center justify-between p-2.5 bg-zinc-900/60 hover:bg-zinc-800/60 border border-zinc-800 rounded-lg text-zinc-300 transition-colors"
          >
            <span class="flex items-center gap-2">
              <ExternalLink class="w-3.5 h-3.5 text-violet-400" />
              <span>{t('Homepage')}</span>
            </span>
            <ExternalLink class="w-3.5 h-3.5 text-zinc-500" />
          </a>
        {/if}

        <button
          onclick={handleCheckUpdate}
          disabled={isCheckingUpdates}
          class="w-full flex items-center justify-center gap-2 px-3 py-2 bg-zinc-900 hover:bg-zinc-800 border border-zinc-700 rounded-lg text-zinc-200 transition-colors"
        >
          <RefreshCw class="w-3.5 h-3.5 {isCheckingUpdates ? 'animate-spin text-violet-400' : ''}" />
          <span>{isCheckingUpdates ? t('Checking...') : t('Check for Updates')}</span>
        </button>

        {#if updateChangelog}
          <div class="p-3 bg-violet-950/20 border border-violet-800/40 rounded-xl text-violet-200 whitespace-pre-wrap text-[11px] font-mono">
            {updateChangelog}
          </div>
        {/if}
      </div>

      <!-- File Details -->
      <div class="text-[10px] font-mono text-zinc-500 space-y-1 pt-2">
        <div>SHA1: {mod.sha1}</div>
        {#if mod.sha256}
          <div class="truncate">SHA256: {mod.sha256}</div>
        {/if}
      </div>
    </div>
  </div>
{/if}
