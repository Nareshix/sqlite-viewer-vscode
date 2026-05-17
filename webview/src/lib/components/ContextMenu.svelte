<script lang="ts">
  import { store } from '../store.svelte';
</script>

{#if store.contextMenu.visible}
  <div
    class="fixed bg-[var(--vscode-menu-background)] text-[var(--vscode-menu-foreground)] border border-[var(--vscode-menu-border)] rounded z-[1000] min-w-[140px] shadow-lg shadow-black/40"
    style="left: {store.contextMenu.x}px; top: {store.contextMenu.y}px"
  >
    <div
      class="px-3.5 py-2 text-xs cursor-pointer hover:bg-[var(--vscode-menu-selectionBackground)] hover:text-[var(--vscode-menu-selectionForeground)]"
      onclick={() => { store.browseTable(store.contextMenu.tableName); store.hideContextMenu(); }}
    >
      Open
    </div>
    <div
      class="px-3.5 py-2 text-xs cursor-pointer hover:bg-[var(--vscode-menu-selectionBackground)] hover:text-[var(--vscode-menu-selectionForeground)]"
      onclick={() => { store.browseTable(store.contextMenu.tableName, true); store.hideContextMenu(); }}
    >
      Open in New Tab
    </div>
  </div>
{/if}