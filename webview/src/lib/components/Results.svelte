<script lang="ts">
  import { store } from '../store.svelte';
</script>

<div class="flex-1 bg-[#1e1e1e] border-t border-[#444] overflow-auto p-2.5 flex flex-col">
  {#if store.activeTab?.error}
    <div class="text-[#f48771] font-mono whitespace-pre-wrap p-2.5 bg-red-500/10 border border-[#f48771]">
      {store.activeTab.error}
    </div>
  {:else if store.activeTab?.results.length > 0}
    <table class="w-full border-collapse font-mono text-[13px]">
      <thead>
        <tr>
          {#each store.activeTab.columns as col}
            <th class="border border-[#444] px-2.5 py-1.5 text-left bg-[#2d2d2d] text-white sticky top-0">{col}</th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each store.activeTab.results as row}
          <tr>
            {#each store.activeTab.columns as col}
              <td class="border border-[#444] px-2.5 py-1.5 text-left">{row[col] === null ? 'NULL' : row[col]}</td>
            {/each}
          </tr>
        {/each}
      </tbody>
    </table>
  {:else}
    <div class="text-[#888] italic p-2.5">No results to display.</div>
  {/if}
</div>