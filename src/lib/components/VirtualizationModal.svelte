<script lang="ts">
  import { appState } from '$lib/store.svelte';
  import { t } from '$lib/i18n';
  import { applyProfile } from '$lib/api';
  import {
    Play,
    X,
    CheckCircle2,
    AlertTriangle,
    Layers,
    FileCheck,
    FileMinus,
    FileX,
  } from '@lucide/svelte';

  const plan = $derived(appState.virtualizationPlan);
  let isApplying = $state(false);

  async function handleConfirmApply() {
    if (!plan || !plan.mods_dir) return;

    const prof = appState.activeProfile;
    if (!prof) return;

    const instancePath = prof.instance_path || appState.selectedInstance?.path;
    if (!instancePath) return;

    const libraryDir = appState.userData.settings.library_dir || './mods_library';

    isApplying = true;
    try {
      const res = await applyProfile(
        instancePath,
        prof.name,
        prof.enabled_mod_hashes,
        libraryDir,
        null,
        prof.minecraft_version,
        prof.loader
      );

      if (res.success) {
        appState.showToast(res.message, 'success');
        appState.virtualizationModalOpen = false;
        await appState.refreshInstances();
      } else {
        appState.showToast(res.message, 'error');
      }
    } catch (e: any) {
      appState.showToast(`Application error: ${e?.message || e}`, 'error');
    } finally {
      isApplying = false;
    }
  }

  function getActionBadge(action: string) {
    switch (action) {
      case 'Create link':
        return { label: 'Create link', color: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30' };
      case 'Remove link':
        return { label: 'Remove link', color: 'bg-rose-500/10 text-rose-400 border-rose-500/30' };
      case 'Conflict':
        return { label: 'Conflict', color: 'bg-amber-500/10 text-amber-400 border-amber-500/30' };
      case 'Keep link':
        return { label: 'Keep link', color: 'bg-zinc-800 text-zinc-400 border-zinc-700' };
      case 'Keep external file':
        return { label: 'External file (Safe)', color: 'bg-zinc-800 text-zinc-400 border-zinc-700' };
      default:
        return { label: action, color: 'bg-zinc-800 text-zinc-400 border-zinc-700' };
    }
  }
</script>

{#if appState.virtualizationModalOpen}
  <div class="fixed inset-0 bg-black/70 backdrop-blur-xs z-50 flex items-center justify-center p-4">
    <div class="bg-[#121316] border border-zinc-700 rounded-2xl max-w-xl w-full flex flex-col max-h-[85vh] shadow-2xl text-xs text-zinc-300 overflow-hidden">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-zinc-800 flex items-center justify-between shrink-0 bg-[#0c0d10]/50">
        <div>
          <h3 class="text-sm font-bold text-zinc-100 flex items-center gap-2">
            <Layers class="w-4 h-4 text-violet-400" />
            <span>{t('Deploy Profile: Virtualization Preview')}</span>
          </h3>
          <p class="text-[11px] text-zinc-400 mt-0.5">
            {plan?.profile_name || t('Active Profile')} &rarr; {plan?.instance_path || ''}
          </p>
        </div>

        <button
          onclick={() => (appState.virtualizationModalOpen = false)}
          class="p-1.5 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 transition-colors"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Plan Body -->
      <div class="flex-1 overflow-y-auto p-6 space-y-4">
        {#if appState.virtualizationLoading}
          <div class="h-48 flex items-center justify-center text-zinc-400">
            {t('Analyzing filesystem differences...')}
          </div>
        {:else if plan}
          <!-- Warnings / Errors -->
          {#if plan.errors.length > 0}
            <div class="p-3.5 rounded-xl bg-amber-500/10 border border-amber-500/30 text-amber-300 space-y-1">
              <div class="font-bold flex items-center gap-1.5 text-xs">
                <AlertTriangle class="w-4 h-4" />
                <span>{t('Action Required')}</span>
              </div>
              <ul class="list-disc list-inside space-y-0.5 text-[11px] text-amber-200/80">
                {#each plan.errors as err}
                  <li>{err}</li>
                {/each}
              </ul>
            </div>
          {/if}

          <!-- Actions Table -->
          <div class="space-y-2">
            <div class="font-semibold text-zinc-400 uppercase tracking-wider text-[11px]">
              {t('Planned Operations')} ({plan.actions.length})
            </div>

            <div class="border border-zinc-800 rounded-xl divide-y divide-zinc-800/80 max-h-64 overflow-y-auto bg-zinc-900/40">
              {#each plan.actions as act}
                {@const badge = getActionBadge(act.action)}
                <div class="p-3 flex items-center justify-between gap-3 text-xs">
                  <div class="truncate">
                    <div class="font-medium text-zinc-200 truncate">{act.filename}</div>
                    <div class="text-[10px] text-zinc-500 truncate">{act.details}</div>
                  </div>
                  <span class="inline-flex items-center px-2 py-0.5 rounded-md text-[10px] font-semibold border {badge.color} shrink-0">
                    {t(badge.label)}
                  </span>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="px-6 py-4 border-t border-zinc-800 flex items-center justify-between shrink-0 bg-[#0c0d10]/50">
        <div class="text-[11px] text-zinc-500">
          {t('Changes are recorded and can be undone at any time.')}
        </div>

        <div class="flex items-center gap-2">
          <button
            onclick={() => (appState.virtualizationModalOpen = false)}
            class="px-4 py-2 rounded-lg text-zinc-400 hover:text-zinc-200 transition-colors"
          >
            {t('Cancel')}
          </button>
          <button
            onclick={handleConfirmApply}
            disabled={!plan?.can_apply || isApplying}
            class="flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-violet-600 to-indigo-600 hover:from-violet-500 hover:to-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed text-white font-semibold rounded-lg transition-all shadow-lg shadow-violet-900/40"
          >
            <Play class="w-3.5 h-3.5 fill-current" />
            <span>{isApplying ? t('Applying...') : t('Confirm & Deploy')}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
