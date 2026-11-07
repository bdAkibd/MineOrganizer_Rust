<script lang="ts">
  import { contextMenu } from '$lib/contextMenu.svelte';
  import { t } from '$lib/i18n.svelte';

  function handleClickOutside() {
    contextMenu.close();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      contextMenu.close();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if contextMenu.isOpen}
  <!-- Backdrop for click outside -->
  <button
    type="button"
    aria-label="Close context menu"
    class="fixed inset-0 z-50 bg-transparent w-full h-full border-0 cursor-default"
    onclick={handleClickOutside}
    oncontextmenu={(e) => {
      e.preventDefault();
      contextMenu.close();
    }}
  ></button>

  <!-- Menu Container -->
  <div
    role="menu"
    tabindex="-1"
    onkeydown={(e) => e.stopPropagation()}
    class="fixed z-50 min-w-[210px] py-1.5 px-1 bg-[#121316] border border-zinc-700/80 rounded-xl shadow-2xl backdrop-blur-md text-xs select-none"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
    onclick={(e) => e.stopPropagation()}
  >
    {#each contextMenu.items as item}
      {#if item.separator}
        <div class="my-1 border-t border-zinc-800"></div>
      {:else}
        {@const Icon = item.icon}
        <button
          onclick={() => {
            contextMenu.close();
            item.action();
          }}
          disabled={item.disabled}
          class="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-left transition-colors font-medium disabled:opacity-40 disabled:cursor-not-allowed {item.danger
            ? 'text-rose-400 hover:bg-rose-500/15 hover:text-rose-300'
            : 'text-zinc-300 hover:bg-zinc-800 hover:text-white'}"
        >
          {#if Icon}
            <Icon class="w-4 h-4 shrink-0 {item.danger ? 'text-rose-400' : 'text-zinc-400'}" />
          {/if}
          <span class="truncate">{t(item.label)}</span>
        </button>
      {/if}
    {/each}
  </div>
{/if}
