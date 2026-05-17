<script lang="ts">
  import { store } from '../store.svelte';
</script>

{#if store.contextMenu.visible}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed bg-[#2d2d2d] border border-[#444] rounded z-[1000] min-w-[140px] shadow-lg shadow-black/40"
    style="left: {store.contextMenu.x}px; top: {store.contextMenu.y}px"
  >
    <div
      class="px-3.5 py-2 text-xs text-[#ccc] cursor-pointer hover:bg-[#007acc] hover:text-white"
      onclick={() => { store.browseTable(store.contextMenu.tableName); store.hideContextMenu(); }}
    >
      Open
    </div>
    <div
      class="px-3.5 py-2 text-xs text-[#ccc] cursor-pointer hover:bg-[#007acc] hover:text-white"
      onclick={() => { store.browseTable(store.contextMenu.tableName, true); store.hideContextMenu(); }}
    >
      Open in New Tab
    </div>
  </div>
{/if}