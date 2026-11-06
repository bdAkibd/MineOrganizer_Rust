<script lang="ts">
  import { appState } from '$lib/store.svelte';
  import { CheckCircle2, AlertCircle, Info, X } from '@lucide/svelte';
</script>

<div class="fixed bottom-5 right-5 z-50 flex flex-col gap-2 max-w-sm pointer-events-none">
  {#each appState.toasts as toast (toast.id)}
    <div
      class="pointer-events-auto flex items-start gap-3 p-3.5 rounded-xl border shadow-2xl backdrop-blur-md transition-all duration-200 animate-in slide-in-from-bottom-3 {toast.type === 'success'
        ? 'bg-emerald-950/90 border-emerald-500/40 text-emerald-200'
        : toast.type === 'error'
        ? 'bg-rose-950/90 border-rose-500/40 text-rose-200'
        : 'bg-zinc-900/90 border-zinc-700/60 text-zinc-200'}"
    >
      {#if toast.type === 'success'}
        <CheckCircle2 class="w-4 h-4 text-emerald-400 shrink-0 mt-0.5" />
      {:else if toast.type === 'error'}
        <AlertCircle class="w-4 h-4 text-rose-400 shrink-0 mt-0.5" />
      {:else}
        <Info class="w-4 h-4 text-violet-400 shrink-0 mt-0.5" />
      {/if}

      <div class="text-xs leading-relaxed flex-1 select-text">
        {toast.message}
      </div>

      <button
        onclick={() => (appState.toasts = appState.toasts.filter((t) => t.id !== toast.id))}
        class="opacity-60 hover:opacity-100 transition-opacity p-0.5"
      >
        <X class="w-3.5 h-3.5" />
      </button>
    </div>
  {/each}
</div>
