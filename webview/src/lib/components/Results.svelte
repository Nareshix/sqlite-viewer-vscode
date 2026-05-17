<script lang="ts">
  import { store } from '../store.svelte';

  // Virtual Scrolling State
  let viewport = $state<HTMLDivElement | null>(null);
  let scrollTop = $state(0);
  let clientHeight = $state(400);

  const ROW_HEIGHT = 30; // Approximated height of one table row in pixels
  const BUFFER = 15;     // Extra rows to render off-screen so scrolling doesn't flicker

  let totalRows = $derived(store.activeTab?.results?.length || 0);

  // Calculate which slice of the massive array to render based on scroll position
  let startIndex = $derived(Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - BUFFER));
  let endIndex = $derived(Math.min(totalRows, Math.ceil((scrollTop + clientHeight) / ROW_HEIGHT) + BUFFER));

  let visibleRows = $derived(store.activeTab?.results?.slice(startIndex, endIndex) || []);

  // Calculate the invisible "fake" height to push the scrollbar down
  let paddingTop = $derived(startIndex * ROW_HEIGHT);
  let paddingBottom = $derived((totalRows - endIndex) * ROW_HEIGHT);

  function onScroll(e: Event) {
    const target = e.target as HTMLDivElement;
    scrollTop = target.scrollTop;
  }
</script>

<div
  bind:this={viewport}
  bind:clientHeight={clientHeight}
  onscroll={onScroll}
  class="flex-1 min-h-0 bg-[var(--vscode-editor-background)] overflow-auto flex flex-col relative"
>

  <!-- Beautiful Spinner Loading State -->
  {#if store.activeTab?.isLoading}
    <div class="absolute inset-0 bg-[var(--vscode-editor-background)]/60 flex items-center justify-center z-[100] backdrop-blur-sm">
      <div class="flex flex-col items-center gap-4">
        <svg class="animate-spin h-8 w-8 text-[var(--vscode-button-background)]" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        <div class="text-[var(--vscode-descriptionForeground)] font-mono animate-pulse">Running query...</div>
      </div>
    </div>
  {/if}

  {#if store.activeTab?.error}
    <div class="m-2.5 text-[var(--vscode-errorForeground)] font-mono whitespace-pre-wrap p-2.5 border border-[var(--vscode-errorForeground)] bg-[var(--vscode-inputValidation-errorBackground)]">
      {store.activeTab.error}
    </div>
  {:else if store.activeTab?.results.length > 0}

    <table class="w-full border-separate border-spacing-0 font-mono text-[13px]">
      <!-- Sticky Header that NEVER moves! -->
      <thead class="sticky top-0 z-10 bg-[var(--vscode-editorWidget-background)] shadow-[0_1px_0_var(--vscode-panel-border)]">
        <tr>
          {#each store.activeTab.columns as col}
            <th class="border-r border-b border-[var(--vscode-panel-border)] px-2.5 py-1.5 text-left text-[var(--vscode-editorWidget-foreground)] whitespace-nowrap overflow-hidden text-ellipsis max-w-[300px]">
              {col}
            </th>
          {/each}
        </tr>
      </thead>
      <tbody>
        <!-- Fake Top Padding -->
        {#if paddingTop > 0}
          <tr style="height: {paddingTop}px"><td colspan="100%" class="p-0 border-none"></td></tr>
        {/if}

        <!-- Only renders the 30 rows you are actively looking at -->
        {#each visibleRows as row}
          <tr class="hover:bg-[var(--vscode-list-hoverBackground)]" style="height: {ROW_HEIGHT}px">
            {#each store.activeTab.columns as col}
              <td class="border-r border-b border-[var(--vscode-panel-border)] px-2.5 py-1.5 text-left truncate max-w-[300px]" title={row[col]}>
                {row[col] === null ? 'NULL' : row[col]}
              </td>
            {/each}
          </tr>
        {/each}

        <!-- Fake Bottom Padding -->
        {#if paddingBottom > 0}
          <tr style="height: {paddingBottom}px"><td colspan="100%" class="p-0 border-none"></td></tr>
        {/if}
      </tbody>
    </table>
  {:else}
    <div class="text-[var(--vscode-descriptionForeground)] italic p-2.5">No results to display.</div>
  {/if}
</div>