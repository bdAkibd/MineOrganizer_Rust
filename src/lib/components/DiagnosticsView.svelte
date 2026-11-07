<script lang="ts">
  import { appState } from '$lib/store.svelte';
  import { t } from '$lib/i18n';
  import {
    analyzeCrashLogs,
    analyzeRawLog,
    listInstanceLogFiles,
    readLogFileContent,
  } from '$lib/api';
  import type { CrashAnalysis } from '$lib/types';
  import {
    Activity,
    AlertTriangle,
    CheckCircle2,
    Copy,
    FileText,
    RefreshCw,
    XCircle,
    Info,
    ShieldAlert,
    Cpu,
  } from '@lucide/svelte';

  let selectedInstancePath = $state(
    appState.selectedInstance?.path || appState.activeProfile?.instance_path || ''
  );
  let availableLogFiles = $state<[string, string][]>([]);
  let selectedLogFilePath = $state<string>('');
  let rawLogText = $state<string>('');
  let currentAnalysis = $state<CrashAnalysis | null>(null);
  let isAnalyzing = $state(false);

  async function handleLoadLogFiles() {
    if (!selectedInstancePath) return;
    try {
      const files = await listInstanceLogFiles(selectedInstancePath);
      availableLogFiles = files;
      if (files.length > 0) {
        selectedLogFilePath = files[0][1];
        await handleSelectLogFile(files[0][1]);
      }
    } catch (e: any) {
      appState.showToast(`Failed to load log files: ${e?.message || e}`, 'error');
    }
  }

  async function handleSelectLogFile(path: string) {
    selectedLogFilePath = path;
    isAnalyzing = true;
    try {
      const text = await readLogFileContent(path, 1024 * 1024);
      rawLogText = text;

      const fileType = path.includes('crash-') ? 'crash_report' : 'log';
      const analysis = await analyzeRawLog(text, path, fileType);
      currentAnalysis = analysis;
    } catch (e: any) {
      appState.showToast(`Failed to analyze log: ${e?.message || e}`, 'error');
    } finally {
      isAnalyzing = false;
    }
  }

  function copyRecommendation() {
    if (!currentAnalysis?.recommendation) return;
    navigator.clipboard.writeText(currentAnalysis.recommendation);
    appState.showToast('Recommendation copied to clipboard!', 'success');
  }

  $effect(() => {
    if (selectedInstancePath) {
      handleLoadLogFiles();
    }
  });
</script>

<div class="flex-1 flex flex-col h-full bg-[#09090b] overflow-hidden">
  <!-- Header Bar -->
  <div class="px-6 py-4 border-b border-[#27272a]/60 bg-[#0c0d10]/40 flex flex-wrap items-center justify-between gap-4 shrink-0">
    <div class="flex flex-wrap items-center gap-3">
      <!-- Instance selector -->
      <select
        bind:value={selectedInstancePath}
        class="bg-zinc-900 border border-zinc-700 text-xs text-zinc-200 rounded-lg px-3 py-2 focus:outline-none focus:border-violet-500"
      >
        <option value="">{t('Select Environment...')}</option>
        {#each appState.instances as inst}
          <option value={inst.path}>{inst.name} ({inst.launcher})</option>
        {/each}
      </select>

      <!-- Log file selector -->
      {#if availableLogFiles.length > 0}
        <select
          bind:value={selectedLogFilePath}
          onchange={() => handleSelectLogFile(selectedLogFilePath)}
          class="bg-zinc-900 border border-zinc-700 text-xs text-zinc-200 rounded-lg px-3 py-2 focus:outline-none focus:border-violet-500 font-mono"
        >
          {#each availableLogFiles as [label, p]}
            <option value={p}>{label}</option>
          {/each}
        </select>
      {/if}

      <button
        onclick={handleLoadLogFiles}
        title={t('Refresh logs')}
        class="p-2 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 border border-zinc-700/60 transition-colors"
      >
        <RefreshCw class="w-4 h-4 {isAnalyzing ? 'animate-spin text-violet-400' : ''}" />
      </button>
    </div>

    <div class="text-xs text-zinc-400 flex items-center gap-2">
      <Activity class="w-4 h-4 text-violet-400" />
      <span>{t('Automated Root Cause Diagnostics')}</span>
    </div>
  </div>

  <!-- Body Content -->
  <div class="flex-1 overflow-y-auto p-6 space-y-6">
    {#if !selectedInstancePath}
      <div class="h-64 flex flex-col items-center justify-center text-center p-8 text-zinc-400">
        <Activity class="w-12 h-12 text-zinc-600 mb-3" />
        <h4 class="text-sm font-semibold text-zinc-300 mb-1">{t('Select an Environment')}</h4>
        <p class="text-xs text-zinc-500 max-w-sm">{t('Choose an environment above to analyze crash reports and game logs.')}</p>
      </div>
    {:else if availableLogFiles.length === 0}
      <div class="h-64 flex flex-col items-center justify-center text-center p-8 text-zinc-400">
        <CheckCircle2 class="w-12 h-12 text-emerald-500/50 mb-3" />
        <h4 class="text-sm font-semibold text-zinc-300 mb-1">{t('No crashes or log files found')}</h4>
        <p class="text-xs text-zinc-500 max-w-sm">{t('This instance has no recorded crash reports or latest.log entries.')}</p>
      </div>
    {:else if currentAnalysis}
      <!-- Diagnosis Card -->
      <div class="p-6 rounded-2xl bg-zinc-900/80 border border-[#27272a] shadow-xl space-y-4">
        <div class="flex items-start justify-between gap-4">
          <div class="space-y-1">
            <div class="flex items-center gap-2">
              <span class="px-2.5 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider bg-rose-500/15 text-rose-400 border border-rose-500/30">
                {currentAnalysis.category}
              </span>
              {#if currentAnalysis.minecraft_version}
                <span class="px-2 py-0.5 rounded bg-zinc-800 text-zinc-400 font-mono text-[10px]">
                  MC {currentAnalysis.minecraft_version}
                </span>
              {/if}
              {#if currentAnalysis.java_version}
                <span class="px-2 py-0.5 rounded bg-zinc-800 text-zinc-400 font-mono text-[10px]">
                  Java {currentAnalysis.java_version}
                </span>
              {/if}
            </div>

            <h3 class="text-base font-bold text-zinc-100">
              {currentAnalysis.title}
            </h3>
            <p class="text-xs text-zinc-300 font-medium leading-relaxed">
              {currentAnalysis.summary}
            </p>
          </div>
        </div>

        <!-- Suspected Mods -->
        {#if currentAnalysis.suspected_mods.length > 0}
          <div class="space-y-1.5 pt-2 border-t border-zinc-800/80">
            <span class="text-[10px] font-semibold text-zinc-400 uppercase tracking-wider">{t('Suspected Mods')}</span>
            <div class="flex flex-wrap gap-1.5">
              {#each currentAnalysis.suspected_mods as smod}
                <span class="px-2.5 py-1 rounded-lg bg-amber-500/10 text-amber-300 border border-amber-500/20 font-semibold text-xs">
                  {smod}
                </span>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Recommended Action Box -->
        {#if currentAnalysis.recommendation}
          <div class="p-4 rounded-xl bg-violet-950/20 border border-violet-800/40 space-y-2">
            <div class="flex items-center justify-between">
              <span class="text-xs font-bold text-violet-300 flex items-center gap-1.5">
                <ShieldAlert class="w-4 h-4 text-violet-400" />
                {t('Recommended Solution')}
              </span>
              <button
                onclick={copyRecommendation}
                title={t('Copy solution')}
                class="flex items-center gap-1 text-[11px] text-violet-400 hover:text-violet-200 transition-colors"
              >
                <Copy class="w-3.5 h-3.5" />
                <span>{t('Copy')}</span>
              </button>
            </div>
            <p class="text-xs text-violet-200/90 leading-relaxed font-sans">
              {currentAnalysis.recommendation}
            </p>
          </div>
        {/if}
      </div>

      <!-- Raw Log Viewer -->
      <div class="space-y-2">
        <div class="flex items-center justify-between text-xs text-zinc-400">
          <span class="font-semibold uppercase tracking-wider text-[11px] flex items-center gap-1.5">
            <FileText class="w-3.5 h-3.5" />
            {t('Raw Log Output')}
          </span>
          <span class="font-mono text-[10px] text-zinc-500 truncate max-w-sm">{selectedLogFilePath}</span>
        </div>

        <div class="p-4 rounded-xl bg-[#070709] border border-zinc-800 font-mono text-[11px] text-zinc-300 overflow-x-auto max-h-96 whitespace-pre leading-relaxed select-text">
          {rawLogText || t('No log text available.')}
        </div>
      </div>
    {/if}
  </div>
</div>
