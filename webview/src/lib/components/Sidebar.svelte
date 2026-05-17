<script lang="ts">
  import { store } from '../store.svelte';

  let { width = 240 }: { width: number } = $props();

  let expandedTables: Set<string> = $state(new Set());
  let searchQuery: string = $state('');
  let searchOpen: boolean = $state(false);
  let searchInput: HTMLInputElement | null = null;

  function toggleSearch() {
    searchOpen = !searchOpen;
    if (searchOpen) setTimeout(() => searchInput?.focus(), 0);
    else searchQuery = '';
  }

  function toggleTable(name: string) {
    const next = new Set(expandedTables);
    next.has(name) ? next.delete(name) : next.add(name);
    expandedTables = next;
  }

  function fmt(n: number): string {
    if (n >= 1000) return (n / 1000).toFixed(1).replace(/\.0$/, '') + 'k rows';
    return `${n} ${n === 1 ? 'row' : 'rows'}`;
  }

  function shortType(t: string): string {
    if (!t) return 'ANY';
    const u = t.toUpperCase();
    if (u.includes('INTEGER') || u === 'INT') return 'INT';
    if (u.includes('VARCHAR') || u.includes('CHAR') || u === 'TEXT') return 'TEXT';
    if (u.includes('REAL') || u.includes('FLOAT') || u.includes('DOUBLE')) return 'REAL';
    if (u.includes('BOOLEAN') || u.includes('BOOL')) return 'BOOL';
    if (u.includes('NUMERIC') || u.includes('DECIMAL')) return 'NUM';
    if (u.includes('BLOB')) return 'BLOB';
    return t;
  }

  let filteredTables = $derived(
    store.schema.tables.filter((t: any) => t.name.toLowerCase().includes(searchQuery.toLowerCase()))
  );
</script>

<div style="width: {width}px; min-width: {width}px" class="bg-[var(--vscode-editor-background)] border-l border-[var(--vscode-panel-border)] flex flex-col overflow-hidden">
  <div class="flex-1 overflow-y-auto py-2">
    <div class="px-3 py-1 text-[11px] font-semibold text-[var(--vscode-sideBarTitle-foreground)] tracking-wider uppercase flex justify-between items-center">
      <span>Tables <span class="text-[var(--vscode-descriptionForeground)]">{store.schema.tables.length}</span></span>
      <span
        class="cursor-pointer text-[14px] px-1 py-0.5 rounded-sm leading-none shrink-0 select-none {searchOpen ? 'text-[var(--vscode-button-background)]' : 'text-[var(--vscode-icon-foreground)] hover:bg-[var(--vscode-list-hoverBackground)]'}"
        onclick={toggleSearch} title="Search tables"
      >⌕</span>
    </div>

    {#if searchOpen}
      <div class="px-2.5 pb-1.5 pt-1">
        <input
          bind:this={searchInput} type="text" placeholder="search tables..." bind:value={searchQuery}
          onkeydown={(e) => e.key === 'Escape' && toggleSearch()}
          class="w-full box-border bg-[var(--vscode-input-background)] border border-[var(--vscode-input-border)] text-[var(--vscode-input-foreground)] px-2 py-1 text-xs rounded-sm outline-none focus:border-[var(--vscode-focusBorder)]"
        />
      </div>
    {/if}

    {#each filteredTables as table}
      <div
        class="flex items-center px-2 py-1 cursor-pointer gap-1.5 hover:bg-[var(--vscode-list-hoverBackground)] group"
        onclick={() => toggleTable(table.name)}
        oncontextmenu={(e) => store.showContextMenu(e, table.name)}
      >
        <span class="text-[15px] text-[var(--vscode-icon-foreground)] w-4 inline-flex items-center justify-center transition-transform {expandedTables.has(table.name) ? 'rotate-90' : ''}">›</span>
        <span class="flex-1 text-[13px] text-[var(--vscode-sideBar-foreground)]">{table.name}</span>
        <span class="text-[11px] text-[var(--vscode-descriptionForeground)]">{fmt(table.rowCount)}</span>
        <span class="opacity-0 text-[11px] px-1 py-[1px] rounded-sm group-hover:opacity-100 hover:text-[var(--vscode-button-foreground)] hover:bg-[var(--vscode-button-background)]"
          onclick={(e) => { e.stopPropagation(); store.browseTable(table.name); }}
        >↗</span>
      </div>

      {#if expandedTables.has(table.name)}
        <div class="pl-7 pb-1">
          {#each table.columns as col}
            <div class="flex flex-wrap items-center gap-1 px-2 py-0.5 text-[11px] text-[#888]">
              {#if col.pk > 0}
                <span class="bg-[#2d4a1e] text-[#7ec850] text-[9px] px-1 py-[1px] rounded-[2px] font-semibold shrink-0">PK</span>
              {/if}
              {#if col.fk}
                <span class="bg-[#1e3347] text-[#4ec9b0] text-[9px] px-1 py-[1px] rounded-[2px] font-semibold shrink-0">FK → {col.fk.table}({col.fk.to})</span>
              {/if}
              <span class="text-[var(--vscode-foreground)]">{col.name}</span>
              <span class="text-[var(--vscode-descriptionForeground)] font-mono opacity-70">{shortType(col.type)}</span>
              {#if col.pk === 0}
                {#if col.notnull === 1}
                  <span class="bg-[#2d2d1e] text-[#d7ba7d] text-[9px] px-1 py-[1px] rounded-[2px] font-semibold shrink-0">not null</span>
                {:else}
                  <span class="bg-[#333] text-[#888] text-[9px] px-1 py-[1px] rounded-[2px] font-semibold shrink-0">null</span>
                {/if}
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    {/each}
  </div>
</div>