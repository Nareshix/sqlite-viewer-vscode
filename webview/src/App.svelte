<script lang="ts">
  import { onMount } from 'svelte';
  import { store, vscode } from './lib/store.svelte';

  import TabBar from './lib/components/TabBar.svelte';
  import Editor from './lib/components/Editor.svelte';
  import Results from './lib/components/Results.svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import ContextMenu from './lib/components/ContextMenu.svelte';

  // Automatically save state when critical properties change
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

<div class="flex flex-col h-screen w-screen text-[#ccc] font-sans bg-[#1e1e1e]">
  <div class="h-px bg-[#333] shrink-0"></div>

  <div class="flex flex-row flex-1 min-h-0">
    <!-- Main Left Pane -->
    <div class="flex flex-col flex-1 min-w-0">
      <TabBar />
      <Editor />
      <Results />
    </div>

    <!-- Sidebar Right Pane -->
    <Sidebar />
  </div>

  <ContextMenu />
</div>