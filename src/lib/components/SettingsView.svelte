<script lang="ts">
  import { appState } from '$lib/store.svelte';
  import { i18n, LOCALE_NAMES, SUPPORTED_LOCALES, t } from '$lib/i18n.svelte';
  import {
    Settings,
    Globe,
    Folder,
    FolderPlus,
    Trash2,
    Save,
    Info,
    Shield,
    Sparkles,
  } from '@lucide/svelte';

  let currentSettings = $state({ ...appState.userData.settings });
  let newPathInput = $state('');

  function handleAddPath() {
    if (!newPathInput.trim()) return;
    const paths = [...currentSettings.custom_game_dirs];
    if (!paths.includes(newPathInput.trim())) {
      paths.push(newPathInput.trim());
      currentSettings.custom_game_dirs = paths;
    }
    newPathInput = '';
  }

  function handleRemovePath(index: number) {
    currentSettings.custom_game_dirs = currentSettings.custom_game_dirs.filter((_, i) => i !== index);
  }

  async function handleSave() {
    await appState.saveSettings(currentSettings);
  }
</script>

<div class="flex-1 flex flex-col h-full bg-[#09090b] overflow-hidden">
  <!-- Header Bar -->
  <div class="px-6 py-4 border-b border-[#27272a]/60 bg-[#0c0d10]/40 flex items-center justify-between shrink-0">
    <div>
      <h3 class="text-sm font-semibold text-zinc-100">{t('Settings & Preferences')}</h3>
      <p class="text-xs text-zinc-400">{t('Customize launcher directories, UI language, and virtualization')}</p>
    </div>

    <button
      onclick={handleSave}
      class="flex items-center gap-2 px-4 py-2 bg-violet-600 hover:bg-violet-500 text-white rounded-lg text-xs font-semibold transition-all shadow-md shadow-violet-950/40"
    >
      <Save class="w-3.5 h-3.5" />
      <span>{t('Save Settings')}</span>
    </button>
  </div>

  <!-- Settings Body -->
  <div class="flex-1 overflow-y-auto p-6 max-w-2xl space-y-6">
    <!-- Language Selection -->
    <div class="p-5 rounded-2xl bg-zinc-900/60 border border-[#27272a] space-y-3">
      <div class="flex items-center gap-2 text-zinc-100 font-semibold text-xs">
        <Globe class="w-4 h-4 text-violet-400" />
        <span>{t('Language')}</span>
      </div>

      <select
        bind:value={currentSettings.locale}
        onchange={() => i18n.setLocale(currentSettings.locale)}
        class="w-full px-3 py-2 bg-zinc-900 border border-zinc-700 rounded-lg text-xs text-zinc-200 focus:outline-none focus:border-violet-500"
      >
        {#each SUPPORTED_LOCALES as loc}
          <option value={loc}>{LOCALE_NAMES[loc] || loc}</option>
        {/each}
      </select>
    </div>

    <!-- Paths Management -->
    <div class="p-5 rounded-2xl bg-zinc-900/60 border border-[#27272a] space-y-4">
      <div class="flex items-center gap-2 text-zinc-100 font-semibold text-xs">
        <Folder class="w-4 h-4 text-violet-400" />
        <span>{t('Game & Library Directories')}</span>
      </div>

      <!-- Prism Path -->
      <div>
        <span class="block text-[11px] font-semibold text-zinc-400 uppercase mb-1">
          {t('Prism Launcher Data Directory')}
        </span>
        <input
          type="text"
          placeholder="C:\Users\...\AppData\Roaming\PrismLauncher"
          bind:value={currentSettings.prism_data_dir}
          class="w-full px-3 py-2 bg-zinc-900 border border-zinc-700 rounded-lg text-xs text-zinc-200 font-mono placeholder-zinc-600 focus:outline-none focus:border-violet-500"
        />
      </div>

      <!-- Mods Library Path -->
      <div>
        <span class="block text-[11px] font-semibold text-zinc-400 uppercase mb-1">
          {t('MineOrganizer Mod Library Storage')}
        </span>
        <input
          type="text"
          placeholder="./mods_library"
          bind:value={currentSettings.library_dir}
          class="w-full px-3 py-2 bg-zinc-900 border border-zinc-700 rounded-lg text-xs text-zinc-200 font-mono placeholder-zinc-600 focus:outline-none focus:border-violet-500"
        />
      </div>

      <!-- Custom Game Dirs -->
      <div class="space-y-2">
        <span class="block text-[11px] font-semibold text-zinc-400 uppercase">
          {t('Custom Environment Paths')}
        </span>

        <div class="space-y-1.5">
          {#each currentSettings.custom_game_dirs as cpath, index}
            <div class="flex items-center justify-between p-2 bg-zinc-900 border border-zinc-800 rounded-lg text-xs font-mono text-zinc-300">
              <span class="truncate">{cpath}</span>
              <button
                onclick={() => handleRemovePath(index)}
                class="hover:text-rose-400 text-zinc-500 transition-colors ml-2"
              >
                <Trash2 class="w-3.5 h-3.5" />
              </button>
            </div>
          {/each}
        </div>

        <div class="flex gap-2">
          <input
            type="text"
            placeholder={t('Add another path...')}
            bind:value={newPathInput}
            onkeydown={(e) => e.key === 'Enter' && handleAddPath()}
            class="flex-1 px-3 py-1.5 bg-zinc-900 border border-zinc-700 rounded-lg text-xs text-zinc-200 font-mono focus:outline-none focus:border-violet-500"
          />
          <button
            onclick={handleAddPath}
            class="px-3 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 rounded-lg text-xs font-medium transition-colors"
          >
            <FolderPlus class="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>

    <!-- Launch & Virtualization Preferences -->
    <div class="p-5 rounded-2xl bg-zinc-900/60 border border-[#27272a] space-y-3">
      <div class="flex items-center gap-2 text-zinc-100 font-semibold text-xs">
        <Shield class="w-4 h-4 text-emerald-400" />
        <span>{t('Virtualization & Safety')}</span>
      </div>

      <label class="flex items-center gap-3 cursor-pointer">
        <input
          type="checkbox"
          bind:checked={currentSettings.close_launcher_on_apply}
          class="w-4 h-4 rounded border-zinc-700 bg-zinc-900 text-violet-600 focus:ring-0 cursor-pointer accent-violet-600"
        />
        <span class="text-xs text-zinc-300">{t('Close running launcher before applying profile links')}</span>
      </label>
    </div>

    <!-- About App -->
    <div class="p-5 rounded-2xl bg-zinc-900/30 border border-[#27272a]/60 text-xs text-zinc-400 space-y-1.5">
      <div class="font-semibold text-zinc-200 flex items-center gap-2">
        <Sparkles class="w-4 h-4 text-violet-400" />
        <span>MineOrganizer 2.0 (Rust + Tauri + Svelte 5)</span>
      </div>
      <p>High-performance transactional Minecraft mod manager with zero-overhead virtualization.</p>
    </div>
  </div>
</div>
