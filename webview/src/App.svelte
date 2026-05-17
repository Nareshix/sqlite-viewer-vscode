<script lang="ts">
  import { onMount } from 'svelte';
  import { store, vscode } from './lib/store.svelte';

  import TabBar from './lib/components/TabBar.svelte';
  import Editor from './lib/components/Editor.svelte';
  import Results from './lib/components/Results.svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import ContextMenu from './lib/components/ContextMenu.svelte';

  // Horizontal (sidebar) resize
  let sidebarWidth = $state(240);
  let hDragging = false;
  let startX = 0;
  let startWidth = 0;

  function onHDragStart(e: PointerEvent) {
    hDragging = true;
    startX = e.clientX;
    startWidth = sidebarWidth;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }
  function onHDragMove(e: PointerEvent) {
    if (!hDragging) return;
    // Moving left decreases clientX, which should increase width
    const deltaX = startX - e.clientX;
    sidebarWidth = Math.min(Math.max(startWidth + deltaX, 160), 600);
  }
  function onHDragEnd(e: PointerEvent) {
    hDragging = false;
    (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
  }

  // Vertical (editor/results) resize
  let leftPane: HTMLDivElement;
  let editorHeightPx = $state(window.innerHeight * 0.4);
  let vDragging = false;
  let startY = 0;
  let startHeight = 0;

  function onVDragStart(e: PointerEvent) {
    vDragging = true;
    startY = e.clientY;
    startHeight = editorHeightPx;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }
  function onVDragMove(e: PointerEvent) {
    if (!vDragging) return;
    const deltaY = e.clientY - startY;
    const rect = leftPane.getBoundingClientRect();
    editorHeightPx = Math.min(Math.max(startHeight + deltaY, 80), rect.height - 80);
  }
  function onVDragEnd(e: PointerEvent) {
    vDragging = false;
    (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
  }

  $effect(() => {
    store.tabs;
    store.activeTabId;
    store.saveState();
  });

  onMount(() => {
    vscode.postMessage({ command: 'getSchema' });
    vscode.postMessage({ command: 'ready' });

    const messageHandler = (event: MessageEvent) => {
      const message = event.data;

      if (message.command === 'sqlResult') {
        const data = message.data || [];
        const tab = store.activeTab;
        if (tab) {
          tab.results = data;
          tab.columns = data.length > 0 ? Object.keys(data[0]) : [];
        }
      } else if (message.command === 'sqlError') {
        const tab = store.activeTab;
        if (tab) {
          tab.error = message.error;
          tab.results = [];
          tab.columns = [];
        }
      } else if (message.command === 'schemaResult') {
        store.schema = message.schema;
        store.dbName = message.dbName ?? '';
      } else if (message.command === 'runQuery') {
        store.executeRunQuery();
      } else if (message.command === 'restoreState') {
        if (message.state?.tabs) {
          store.tabs = message.state.tabs.map((t: any) => ({
            ...t, results: [], columns: [], error: null
          }));
          store.activeTabId = message.state.activeTabId;
          store.tabCounter = message.state.tabCounter;
        }
        store.isRestored = true;
        store.saveState();
      }
    };

    window.addEventListener('message', messageHandler);
    window.addEventListener('click', () => store.hideContextMenu());

    return () => {
      window.removeEventListener('message', messageHandler);
      window.removeEventListener('click', () => store.hideContextMenu());
    };
  });
</script>

<!-- Appended dynamically select-none during drag actions to completely prevent lag/text selection -->
<div class="flex flex-col h-screen w-screen text-[#ccc] font-sans bg-[#1e1e1e] {hDragging || vDragging ? 'select-none' : ''}">
  <div class="h-px bg-[#333] shrink-0"></div>

  <div class="flex flex-row flex-1 min-h-0">
    <!-- Main Left Pane: Added min-h-0 here so inner flex items can shrink properly -->
    <div class="flex flex-col flex-1 min-w-0 min-h-0" bind:this={leftPane}>
      <TabBar />
      <Editor height={editorHeightPx} />

      <!-- Vertical drag handle (Made h-1 instead of h-px for easier precise grabbing) -->
      <div
        class="h-1 cursor-row-resize bg-[#333] hover:bg-[#007acc] active:bg-[#007acc] transition-colors shrink-0 z-10"
        onpointerdown={onVDragStart}
        onpointermove={onVDragMove}
        onpointerup={onVDragEnd}
        onpointercancel={onVDragEnd}
      ></div>

      <Results />
    </div>

    <!-- Horizontal drag handle -->
    <div
      class="w-1 cursor-col-resize bg-[#333] hover:bg-[#007acc] active:bg-[#007acc] transition-colors shrink-0 z-10"
      onpointerdown={onHDragStart}
      onpointermove={onHDragMove}
      onpointerup={onHDragEnd}
      onpointercancel={onHDragEnd}
    ></div>

    <Sidebar width={sidebarWidth} />
  </div>

  <ContextMenu />
</div>