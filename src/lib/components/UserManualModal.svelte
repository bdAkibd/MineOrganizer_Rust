<script lang="ts">
  import { t, i18n, SUPPORTED_LOCALES, LOCALE_NAMES } from '$lib/i18n.svelte';
  import { getUserManual } from '$lib/api';
  import { BookOpen, X, Globe, Printer } from '@lucide/svelte';

  let { onclose }: { onclose: () => void } = $props();

  let selectedLocale = $state(i18n.currentLocale || 'en');
  let manualHtml = $state('');
  let isLoading = $state(false);

  $effect(() => {
    loadManual(selectedLocale);
  });

  async function loadManual(loc: string) {
    isLoading = true;
    try {
      manualHtml = await getUserManual(loc);
    } catch (e) {
      console.error('Failed to load manual', e);
      manualHtml = `<p>Error loading manual: ${e}</p>`;
    } finally {
      isLoading = false;
    }
  }

  function handlePrint() {
    const printWindow = window.open('', '_blank');
    if (printWindow) {
      printWindow.document.write(manualHtml);
      printWindow.document.close();
      printWindow.focus();
      printWindow.print();
    }
  }
</script>

<!-- Backdrop -->
<button
  type="button"
  aria-label={t('Close')}
  class="fixed inset-0 bg-black/70 backdrop-blur-xs z-50 flex items-center justify-center p-4 w-full h-full border-0 cursor-default"
  onclick={onclose}
></button>

<!-- Modal Container -->
<div
  role="dialog"
  aria-modal="true"
  tabindex="-1"
  onkeydown={(e) => e.stopPropagation()}
  class="fixed inset-4 md:inset-8 lg:inset-12 z-50 bg-[#121316] border border-zinc-700/80 rounded-2xl shadow-2xl flex flex-col overflow-hidden text-xs text-zinc-300"
>
  <!-- Header Bar -->
  <div class="px-6 py-3.5 border-b border-zinc-800 bg-[#0e0f12] flex items-center justify-between shrink-0">
    <div class="flex items-center gap-3">
      <div class="p-2 rounded-lg bg-violet-600/15 border border-violet-500/30 text-violet-400">
        <BookOpen class="w-5 h-5" />
      </div>
      <div>
        <h3 class="text-sm font-bold text-zinc-100 flex items-center gap-2">
          <span>{t('User Manual')}</span>
        </h3>
        <p class="text-[11px] text-zinc-400">
          {t('Complete guide for MineOrganizer features, workflows, and troubleshooting')}
        </p>
      </div>
    </div>

    <!-- Controls -->
    <div class="flex items-center gap-2.5">
      <!-- Language Picker -->
      <div class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-zinc-900 border border-zinc-700">
        <Globe class="w-3.5 h-3.5 text-zinc-400" />
        <select
          bind:value={selectedLocale}
          class="bg-transparent text-xs text-zinc-200 focus:outline-none cursor-pointer pr-1"
        >
          {#each SUPPORTED_LOCALES as loc}
            <option value={loc} class="bg-[#18181b] text-zinc-200">
              {LOCALE_NAMES[loc] || loc}
            </option>
          {/each}
        </select>
      </div>

      <!-- Print Button -->
      <button
        onclick={handlePrint}
        title={t('Print')}
        class="p-2 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 border border-zinc-800 transition-colors"
      >
        <Printer class="w-4 h-4" />
      </button>

      <!-- Close Button -->
      <button
        onclick={onclose}
        class="p-2 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 border border-zinc-800 transition-colors"
      >
        <X class="w-4 h-4" />
      </button>
    </div>
  </div>

  <!-- Content Reader -->
  <div class="flex-1 overflow-y-auto p-8 bg-[#09090b] text-zinc-300 leading-relaxed font-sans select-text">
    {#if isLoading}
      <div class="h-64 flex items-center justify-center text-zinc-500">
        {t('Loading...')}
      </div>
    {:else}
      <article class="prose prose-invert prose-zinc max-w-4xl mx-auto space-y-4">
        {@html manualHtml}
      </article>
    {/if}
  </div>

  <!-- Footer -->
  <div class="px-6 py-3 border-t border-zinc-800 bg-[#0e0f12] flex items-center justify-between text-[11px] text-zinc-500 shrink-0">
    <span>MineOrganizer &copy; 2026</span>
    <button
      onclick={onclose}
      class="px-4 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-200 font-medium transition-colors"
    >
      {t('Close')}
    </button>
  </div>
</div>

<style>
  :global(article h1) {
    font-size: 1.5rem !important;
    font-weight: 700 !important;
    color: #f4f4f5 !important;
    margin-bottom: 0.75rem !important;
  }
  :global(article h2) {
    font-size: 1.15rem !important;
    font-weight: 600 !important;
    color: #a78bfa !important;
    border-bottom: 1px solid #27272a !important;
    padding-bottom: 0.35rem !important;
    margin-top: 1.5rem !important;
    margin-bottom: 0.5rem !important;
  }
  :global(article h3) {
    font-size: 0.95rem !important;
    font-weight: 600 !important;
    color: #e4e4e7 !important;
    margin-top: 1rem !important;
    margin-bottom: 0.35rem !important;
  }
  :global(article p, article li) {
    font-size: 0.825rem !important;
    line-height: 1.6 !important;
    color: #d4d4d8 !important;
  }
  :global(article ul, article ol) {
    padding-left: 1.25rem !important;
    margin: 0.5rem 0 !important;
  }
  :global(article li) {
    margin: 0.35rem 0 !important;
  }
  :global(article code) {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace !important;
    background-color: #27272a !important;
    color: #c084fc !important;
    padding: 0.15rem 0.35rem !important;
    border-radius: 0.25rem !important;
    font-size: 0.75rem !important;
  }
  :global(article .note) {
    border-left: 4px solid #8b5cf6 !important;
    background-color: rgba(139, 92, 246, 0.1) !important;
    padding: 0.75rem 1rem !important;
    border-radius: 0.375rem !important;
    margin: 1rem 0 !important;
  }
</style>
