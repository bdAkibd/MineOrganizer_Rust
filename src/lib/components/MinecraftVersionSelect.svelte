<script lang="ts">
  import { t } from '$lib/i18n.svelte';
  import {
    MINECRAFT_VERSIONS,
    RELEASE_VERSIONS,
    BETA_VERSIONS,
    ALPHA_VERSIONS,
    SNAPSHOT_VERSIONS,
    getVersionType,
    type MinecraftVersionItem,
  } from '$lib/minecraftVersions';
  import { ChevronDown, Search, X, Check, Tag } from '@lucide/svelte';

  let {
    value = $bindable('all'),
    onchange,
  }: {
    value?: string;
    onchange?: (ver: string) => void;
  } = $props();

  let isOpen = $state(false);
  let searchQuery = $state('');
  let activeTab = $state<'all' | 'release' | 'beta' | 'alpha' | 'snapshot'>('all');
  let searchInputEl = $state<HTMLInputElement | null>(null);

  const currentType = $derived(
    value && value !== 'all' ? getVersionType(value) : null
  );

  const filteredVersions = $derived.by(() => {
    let list: MinecraftVersionItem[] = MINECRAFT_VERSIONS;
    if (activeTab === 'release') list = RELEASE_VERSIONS;
    else if (activeTab === 'beta') list = BETA_VERSIONS;
    else if (activeTab === 'alpha') list = ALPHA_VERSIONS;
    else if (activeTab === 'snapshot') list = SNAPSHOT_VERSIONS;

    const q = searchQuery.trim().toLowerCase();
    if (!q) return list;

    return list.filter(
      (v) =>
        v.version.toLowerCase().includes(q) ||
        v.type.toLowerCase().includes(q)
    );
  });

  function toggleDropdown() {
    isOpen = !isOpen;
    if (isOpen) {
      searchQuery = '';
      setTimeout(() => {
        searchInputEl?.focus();
      }, 50);
    }
  }

  function handleSelect(ver: string) {
    value = ver;
    isOpen = false;
    onchange?.(ver);
  }

  function handleClear() {
    handleSelect('all');
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && isOpen) {
      isOpen = false;
    }
  }

  function getTypeBadgeClass(type: string): string {
    switch (type) {
      case 'release':
        return 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30';
      case 'beta':
        return 'bg-amber-500/10 text-amber-400 border-amber-500/30';
      case 'alpha':
        return 'bg-rose-500/10 text-rose-400 border-rose-500/30';
      case 'snapshot':
        return 'bg-sky-500/10 text-sky-400 border-sky-500/30';
      default:
        return 'bg-zinc-800 text-zinc-400 border-zinc-700';
    }
  }

  function formatTypeLabel(type: string): string {
    switch (type) {
      case 'release':
        return t('Release');
      case 'beta':
        return t('Beta');
      case 'alpha':
        return t('Alpha');
      case 'snapshot':
        return t('Snapshot');
      default:
        return type;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="relative inline-block text-left select-none">
  <!-- Trigger Button -->
  <button
    type="button"
    onclick={toggleDropdown}
    class="flex items-center gap-2 px-3 py-2 bg-zinc-900 hover:bg-zinc-800/80 border border-zinc-700/80 hover:border-zinc-600 rounded-lg text-xs transition-all shadow-sm focus:outline-none focus:border-violet-500"
  >
    <Tag class="w-3.5 h-3.5 text-zinc-400 shrink-0" />

    {#if value === 'all' || !value}
      <span class="text-zinc-300 font-medium">{t('All Versions')}</span>
    {:else}
      <span class="font-mono font-bold text-zinc-100">{value}</span>
      {#if currentType}
        <span
          class="px-1.5 py-0.2 rounded text-[10px] uppercase font-semibold border {getTypeBadgeClass(
            currentType
          )}"
        >
          {formatTypeLabel(currentType)}
        </span>
      {/if}
    {/if}

    <ChevronDown
      class="w-3.5 h-3.5 text-zinc-400 ml-1 transition-transform {isOpen
        ? 'rotate-180'
        : ''}"
    />
  </button>

  <!-- Backdrop for click outside -->
  {#if isOpen}
    <button
      type="button"
      aria-label="Close version dropdown"
      class="fixed inset-0 z-40 bg-transparent w-full h-full border-0 cursor-default"
      onclick={() => (isOpen = false)}
    ></button>

    <!-- Dropdown Menu -->
    <div
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      class="absolute right-0 mt-2 w-80 max-w-[90vw] bg-[#121316] border border-zinc-700 rounded-xl shadow-2xl z-50 overflow-hidden flex flex-col text-xs animate-in fade-in zoom-in-95 duration-100"
    >
      <!-- Search Input Header -->
      <div class="p-2.5 border-b border-zinc-800 bg-[#0c0d10]/60 space-y-2">
        <div class="relative">
          <Search
            class="w-3.5 h-3.5 text-zinc-400 absolute left-2.5 top-1/2 -translate-y-1/2"
          />
          <input
            bind:this={searchInputEl}
            type="text"
            placeholder={t('Filter versions (e.g. 1.20, beta, snapshot)...')}
            bind:value={searchQuery}
            class="w-full pl-8 pr-7 py-1.5 bg-zinc-900 border border-zinc-700/80 rounded-lg text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none focus:border-violet-500"
          />
          {#if searchQuery}
            <button
              type="button"
              onclick={() => (searchQuery = '')}
              class="absolute right-2 top-1/2 -translate-y-1/2 text-zinc-500 hover:text-zinc-300"
            >
              <X class="w-3 h-3" />
            </button>
          {/if}
        </div>

        <!-- Category Tabs -->
        <div class="flex items-center gap-1 overflow-x-auto pb-0.5 text-[11px]">
          <button
            type="button"
            onclick={() => (activeTab = 'all')}
            class="px-2 py-0.5 rounded-md font-medium transition-colors shrink-0 {activeTab ===
            'all'
              ? 'bg-violet-600 text-white shadow-xs'
              : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800'}"
          >
            {t('All')} ({MINECRAFT_VERSIONS.length})
          </button>

          <button
            type="button"
            onclick={() => (activeTab = 'release')}
            class="px-2 py-0.5 rounded-md font-medium transition-colors shrink-0 {activeTab ===
            'release'
              ? 'bg-emerald-600 text-white shadow-xs'
              : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800'}"
          >
            {t('Releases')} ({RELEASE_VERSIONS.length})
          </button>

          <button
            type="button"
            onclick={() => (activeTab = 'beta')}
            class="px-2 py-0.5 rounded-md font-medium transition-colors shrink-0 {activeTab ===
            'beta'
              ? 'bg-amber-600 text-white shadow-xs'
              : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800'}"
          >
            {t('Beta')} ({BETA_VERSIONS.length})
          </button>

          <button
            type="button"
            onclick={() => (activeTab = 'alpha')}
            class="px-2 py-0.5 rounded-md font-medium transition-colors shrink-0 {activeTab ===
            'alpha'
              ? 'bg-rose-600 text-white shadow-xs'
              : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800'}"
          >
            {t('Alpha')} ({ALPHA_VERSIONS.length})
          </button>

          <button
            type="button"
            onclick={() => (activeTab = 'snapshot')}
            class="px-2 py-0.5 rounded-md font-medium transition-colors shrink-0 {activeTab ===
            'snapshot'
              ? 'bg-sky-600 text-white shadow-xs'
              : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800'}"
          >
            {t('Snapshots')} ({SNAPSHOT_VERSIONS.length})
          </button>
        </div>
      </div>

      <!-- Quick Reset Option -->
      <div class="px-2 pt-2 pb-1 border-b border-zinc-800/60">
        <button
          type="button"
          onclick={handleClear}
          class="w-full flex items-center justify-between px-3 py-1.5 rounded-lg text-left transition-colors {value ===
          'all'
            ? 'bg-violet-600/15 text-violet-300 font-semibold border border-violet-500/30'
            : 'text-zinc-300 hover:bg-zinc-800/70 hover:text-white'}"
        >
          <span class="flex items-center gap-2">
            <span class="w-1.5 h-1.5 rounded-full bg-violet-400"></span>
            <span>{t('All Versions')} ({t('No filter')})</span>
          </span>
          {#if value === 'all'}
            <Check class="w-3.5 h-3.5 text-violet-400" />
          {/if}
        </button>
      </div>

      <!-- Versions List -->
      <div class="max-h-64 overflow-y-auto p-1 space-y-0.5">
        {#if filteredVersions.length === 0}
          <div class="py-6 text-center text-zinc-500 text-xs">
            {t('No versions found')}
          </div>
        {:else}
          {#each filteredVersions as item}
            {@const isSelected = value === item.version}
            <button
              type="button"
              onclick={() => handleSelect(item.version)}
              class="w-full flex items-center justify-between px-3 py-1.5 rounded-lg text-left transition-colors group {isSelected
                ? 'bg-violet-600/20 text-violet-200 font-semibold border border-violet-500/30'
                : 'text-zinc-300 hover:bg-zinc-800/70 hover:text-white'}"
            >
              <div class="flex items-center gap-2">
                <span class="font-mono">{item.version}</span>
                <span
                  class="px-1.5 py-0.2 rounded text-[10px] uppercase font-semibold border {getTypeBadgeClass(
                    item.type
                  )}"
                >
                  {formatTypeLabel(item.type)}
                </span>
              </div>

              {#if isSelected}
                <Check class="w-3.5 h-3.5 text-violet-400 shrink-0" />
              {/if}
            </button>
          {/each}
        {/if}
      </div>

      <!-- Footer Info -->
      <div
        class="px-3 py-1.5 bg-[#0a0a0c] border-t border-zinc-800 text-[11px] text-zinc-500 flex items-center justify-between"
      >
        <span>{filteredVersions.length} {t('versions')}</span>
        {#if value && value !== 'all'}
          <button
            type="button"
            onclick={handleClear}
            class="text-violet-400 hover:text-violet-300 transition-colors"
          >
            {t('Reset filter')}
          </button>
        {/if}
      </div>
    </div>
  {/if}
</div>
