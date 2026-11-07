<script lang="ts">
  import { appState } from '$lib/store.svelte';
  import { t } from '$lib/i18n.svelte';
  import {
    Boxes,
    Gamepad2,
    FolderHeart,
    Store,
    Activity,
    Settings,
    Layers,
    ChevronDown,
    Plus,
    Check,
  } from '@lucide/svelte';

  let profileDropdownOpen = $state(false);
  const current = $derived(appState.activeProfile);

  const navItems = [
    { id: 'mods' as const, label: 'Mod Library', icon: Boxes },
    { id: 'instances' as const, label: 'Environments', icon: Gamepad2 },
    { id: 'profiles' as const, label: 'Profiles', icon: FolderHeart },
    { id: 'modrinth' as const, label: 'Modrinth Browser', icon: Store },
    { id: 'diagnostics' as const, label: 'Diagnostics & Logs', icon: Activity },
    { id: 'settings' as const, label: 'Settings', icon: Settings },
  ];
</script>

<aside class="w-64 bg-[#0c0d10] border-r border-[#27272a] flex flex-col select-none shrink-0 z-20">
  <!-- Brand Header -->
  <div class="h-16 flex items-center px-5 border-b border-[#27272a]/60 gap-3">
    <div class="w-9 h-9 rounded-xl bg-gradient-to-tr from-violet-600 via-indigo-600 to-emerald-500 p-0.5 shadow-lg shadow-violet-500/20">
      <div class="w-full h-full bg-[#0c0d10] rounded-[10px] flex items-center justify-center">
        <Layers class="w-5 h-5 text-violet-400" />
      </div>
    </div>
    <div>
      <h1 class="font-bold text-sm tracking-wide text-zinc-100">
        MineOrganizer
      </h1>
      <p class="text-[11px] text-zinc-400">Next-Gen Mod Virtualizer</p>
    </div>
  </div>

  <!-- Navigation items -->
  <nav class="flex-1 p-3 space-y-1 overflow-y-auto">
    {#each navItems as item}
      {@const Icon = item.icon}
      {@const isActive = appState.activeTab === item.id}
      <button
        onclick={() => (appState.activeTab = item.id)}
        class="w-full flex items-center gap-3 px-3.5 py-2.5 rounded-lg text-xs font-medium transition-all duration-150 group text-left {isActive
          ? 'bg-violet-600/15 text-violet-300 border border-violet-500/30 shadow-sm shadow-violet-950/40'
          : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/40 border border-transparent'}"
      >
        <Icon class="w-4 h-4 transition-transform group-hover:scale-110 {isActive ? 'text-violet-400' : 'text-zinc-400 group-hover:text-zinc-300'}" />
        <span class="truncate">{t(item.label)}</span>
        {#if isActive}
          <div class="ml-auto w-1.5 h-1.5 rounded-full bg-violet-400 shadow-sm shadow-violet-400"></div>
        {/if}
      </button>
    {/each}
  </nav>

  <!-- Active Profile Selector Widget -->
  <div class="p-3 border-t border-[#27272a]/60 bg-[#09090b]/50 relative">
    <div class="text-[11px] font-semibold text-zinc-400 mb-1.5 uppercase tracking-wider px-1">
      {t('Active Profile')}
    </div>

    <div class="relative">
      <button
        onclick={() => (profileDropdownOpen = !profileDropdownOpen)}
        class="w-full flex items-center justify-between px-3 py-2 bg-zinc-900/90 hover:bg-zinc-800/90 border border-zinc-700/60 rounded-lg text-xs text-zinc-200 transition-all shadow-inner"
      >
        <div class="flex items-center gap-2 truncate">
          <div class="w-2 h-2 rounded-full {current ? 'bg-emerald-400' : 'bg-amber-400'}"></div>
          <span class="font-medium truncate">{current?.name || t('No Profile')}</span>
        </div>
        <ChevronDown class="w-3.5 h-3.5 text-zinc-400 transition-transform {profileDropdownOpen ? 'rotate-180' : ''}" />
      </button>

      {#if profileDropdownOpen}
        <div class="absolute bottom-full left-0 right-0 mb-2 bg-[#18181b] border border-zinc-700 rounded-xl shadow-2xl p-1.5 z-30 space-y-1">
          <div class="max-h-48 overflow-y-auto space-y-0.5">
            {#each appState.userData.profiles as prof}
              <button
                onclick={() => {
                  appState.setActiveProfile(prof.id);
                  profileDropdownOpen = false;
                }}
                class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-md text-xs text-left hover:bg-zinc-800 transition-colors {prof.id === current?.id ? 'text-violet-400 font-medium bg-violet-500/10' : 'text-zinc-300'}"
              >
                <span class="truncate">{prof.name}</span>
                {#if prof.id === current?.id}
                  <Check class="w-3.5 h-3.5 text-violet-400 shrink-0" />
                {/if}
              </button>
            {/each}
          </div>

          <div class="border-t border-zinc-700/60 pt-1 mt-1">
            <button
              onclick={() => {
                profileDropdownOpen = false;
                appState.activeTab = 'profiles';
              }}
              class="w-full flex items-center gap-1.5 px-2.5 py-1.5 rounded-md text-xs text-violet-400 hover:bg-violet-500/10 font-medium"
            >
              <Plus class="w-3.5 h-3.5" />
              <span>{t('Manage Profiles')}</span>
            </button>
          </div>
        </div>
      {/if}
    </div>

    <!-- Quick stats -->
    {#if current}
      <div class="mt-2 text-[10px] text-zinc-400 flex items-center justify-between px-1">
        <span>{t('Enabled Mods')}:</span>
        <span class="font-mono text-zinc-300 font-semibold">{current.enabled_mod_hashes.length}</span>
      </div>
      <div class="text-[10px] text-zinc-400 flex items-center justify-between px-1">
        <span>Minecraft:</span>
        <span class="font-mono text-zinc-300 font-semibold">{current.minecraft_version || '1.20.1'}</span>
      </div>
    {/if}
  </div>
</aside>
