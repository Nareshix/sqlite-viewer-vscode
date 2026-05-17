<script lang="ts">
  import { store } from '../store.svelte';
</script>

<div class="flex items-center bg-[#1e1e1e] overflow-x-auto shrink-0 border-b border-[#333]">
  {#each store.tabs as tab}
    <div
      class="flex items-center gap-1.5 px-3.5 py-2 text-xs cursor-pointer whitespace-nowrap min-w-[80px] max-w-[160px] border-b-2 transition-colors {tab.id === store.activeTabId ? 'text-white border-[#007acc]' : 'text-[#666] border-transparent hover:text-[#aaa]'}"
      onclick={() => store.switchTab(tab.id)}
    >
      <span class="flex-1 overflow-hidden text-ellipsis">{tab.title}</span>
      {#if store.tabs.length > 1}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <span
          class="text-[#555] text-sm leading-none px-0.5 rounded-sm hover:text-white hover:bg-[#333]"
          onclick={(e) => { e.stopPropagation(); store.closeTab(tab.id); }}
        >×</span>
      {/if}
    </div>
  {/each}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="px-3 py-2 text-[#555] cursor-pointer text-lg leading-none hover:text-[#ccc]"
    onclick={() => store.newTab()}
    title="New tab"
  >+</div>
</div>