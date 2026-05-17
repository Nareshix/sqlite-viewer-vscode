<script lang="ts">
  import { store } from '../store.svelte';
</script>

<!-- Note: bg-[var(--vscode-editor-background)] instead of tab backgrounds for seamless look -->
<div class="flex items-center bg-[var(--vscode-editor-background)] overflow-x-auto shrink-0 border-b border-[var(--vscode-panel-border)]">
  {#each store.tabs as tab}
    <div
      class="flex items-center gap-1.5 px-3.5 py-2 text-xs cursor-pointer whitespace-nowrap min-w-[80px] max-w-[160px] border-b-2 transition-colors {tab.id === store.activeTabId ? 'text-[var(--vscode-tab-activeForeground)] border-[var(--vscode-tab-activeBorderTop)]' : 'text-[var(--vscode-tab-inactiveForeground)] border-transparent hover:bg-[var(--vscode-tab-hoverBackground)]'}"
      onclick={() => store.switchTab(tab.id)}
    >
      <span class="flex-1 overflow-hidden text-ellipsis">{tab.title}</span>
      {#if store.tabs.length > 1}
        <span
          class="text-[var(--vscode-descriptionForeground)] text-sm leading-none px-0.5 rounded-sm hover:bg-[var(--vscode-toolbar-hoverBackground)] hover:text-[var(--vscode-toolbar-hoverOutline)]"
          onclick={(e) => { e.stopPropagation(); store.closeTab(tab.id); }}
        >×</span>
      {/if}
    </div>
  {/each}
  <div
    class="px-3 py-2 text-[var(--vscode-icon-foreground)] cursor-pointer text-lg leading-none hover:text-[var(--vscode-foreground)] hover:bg-[var(--vscode-toolbar-hoverBackground)]"
    onclick={() => store.newTab()} title="New tab"
  >+</div>
</div>