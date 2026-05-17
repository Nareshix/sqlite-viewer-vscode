<script lang="ts">
  import { store } from '../store.svelte';

  let expandedTables: Set<string> = $state(new Set());
  let searchQuery: string = $state('');
  let searchOpen: boolean = $state(false);
  let searchInput: HTMLInputElement | null = null;

  function toggleSearch() {
    searchOpen = !searchOpen;
    if (searchOpen) {
      setTimeout(() => searchInput?.focus(), 0);
    } else {
      searchQuery = '';
    }
  }

  function toggleTable(name: string) {
    const next = new Set(expandedTables);
    next.has(name) ? next.delete(name) : next.add(name);
    expandedTables = next;
  }

  function fmt(n: number): string {
    if (n >= 1000) return (n / 1000).toFixed(1).replace(/\.0$/, '') + 'k';
    return String(n);
  }

  let filteredTables = $derived(
    store.schema.tables.filter((t: any) =>
      t.name.toLowerCase().includes(searchQuery.toLowerCase())
    )
  );
</script>

<div class="w-[240px] min-w-[240px] bg-[#1e1e1e] border-l border-[#333] flex flex-col overflow-hidden">
  <div class="flex-1 overflow-y-auto py-2">
    <!-- Tables Header -->
    <div class="px-3 py-1 text-[11px] font-semibold text-[#888] tracking-wider uppercase flex justify-between items-center">
      <span>Tables <span class="text-[#555]">{store.schema.tables.length}</span></span>
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <span
        class="cursor-pointer text-[14px] px-1 py-0.5 rounded-sm leading-none shrink-0 select-none {searchOpen ? 'text-[#007acc]' : 'text-[#666] hover:text-[#ccc] hover:bg-[#2a2d2e]'}"
        onclick={toggleSearch}
        title={searchOpen ? 'Close search' : 'Search tables'}
      >⌕</span>
    </div>

    <!-- Search Input -->
    {#if searchOpen}
      <div class="px-2.5 pb-1.5 pt-1">
        <input
          bind:this={searchInput}
          type="text"
          placeholder="search tables..."
          bind:value={searchQuery}
          onkeydown={(e) => e.key === 'Escape' && toggleSearch()}
          class="w-full box-border bg-[#2d2d2d] border border-[#444] text-[#ccc] px-2 py-1 text-xs rounded-sm outline-none focus:border-[#007acc]"
        />
      </div>
    {/if}

    <!-- Tables List -->
    {#each filteredTables as table}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="flex items-center px-2 py-1 cursor-pointer gap-1.5 hover:bg-[#2a2d2e] group"
        onclick={() => toggleTable(table.name)}
        oncontextmenu={(e) => store.showContextMenu(e, table.name)}
        title="right-click for options"
      >
        <span class="text-[15px] text-[#555] w-4 inline-flex items-center justify-center transition-transform {expandedTables.has(table.name) ? 'rotate-90' : ''}">›</span>
        <span class="flex-1 text-[13px] text-[#ccc]">{table.name}</span>
        <span class="text-[11px] text-[#666]">{fmt(table.rowCount)}</span>
        <span
          class="opacity-0 text-[11px] text-[#666] px-1 py-[1px] rounded-sm group-hover:opacity-100 hover:!text-white hover:bg-[#3a3a3a]"
          onclick={(e) => { e.stopPropagation(); store.browseTable(table.name); }}
          title="Open table"
        >↗</span>
      </div>

      <!-- Columns -->
      {#if expandedTables.has(table.name)}
        <div class="pl-7 pb-1">
          {#each table.columns as col}
            <div class="flex items-center gap-1.5 px-2 py-0.5 text-[11px] text-[#888]">
              {#if col.pk > 0}
                <span class="bg-[#2d4a1e] text-[#7ec850] text-[9px] px-1 py-[1px] rounded-[2px] font-semibold">PK</span>
              {/if}
              <span class="text-[#bbb]">{col.name}</span>
              <span class="text-[#666] font-mono">{col.type || 'ANY'}</span>
              {#if col.notnull === 0 && col.pk === 0}
                <span class="bg-[#333] text-[#888] text-[9px] px-1 py-[1px] rounded-[2px] font-semibold">null</span>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    {/each}

    <!-- Views -->
    {#if store.schema.views.length > 0}
      <div class="px-3 py-1 text-[11px] font-semibold text-[#888] tracking-wider uppercase mt-3">
        Views <span class="text-[#555]">{store.schema.views.length}</span>
      </div>
      {#each store.schema.views as view}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="px-3 py-1 text-[13px] text-[#9cdcfe] cursor-pointer hover:bg-[#2a2d2e]"
          ondblclick={() => store.browseTable(view)}
        >{view}</div>
      {/each}
    {/if}
  </div>
</div>