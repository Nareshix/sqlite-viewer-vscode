
declare function acquireVsCodeApi(): any;

let vscodeApi: any;
try {
  vscodeApi = acquireVsCodeApi();
} catch (e) {
  vscodeApi = { postMessage: () => {} };
}

export const vscode = vscodeApi;

export type Tab = {
  id: string;
  title: string;
  query: string;
  results: any[];
  columns: string[];
  error: string | null;
};

class AppState {
  tabs: Tab[] = $state([
    { id: '1', title: 'Query 1', query: '-- Open a table from the sidebar or write a query\n', results: [], columns: [], error: null }
  ]);
  activeTabId: string = $state('1');
  schema: { tables: any[]; views: string[] } = $state({ tables: [], views: [] });
  dbName: string = $state('');

  tabCounter = 1;
  isRestored = false;
  contextMenu = $state({ visible: false, x: 0, y: 0, tableName: '' });

  // A hook for the editor component to register its execution method
  executeRunQuery: () => void = () => {};

  get activeTab() {
    return this.tabs.find(t => t.id === this.activeTabId)!;
  }

  switchTab(id: string) {
    this.activeTabId = id;
  }

  newTab() {
    this.tabCounter++;
    const id = String(this.tabCounter);
    this.tabs.push({ id, title: `Query ${this.tabCounter}`, query: '-- Write a query\n', results: [], columns: [], error: null });
    this.switchTab(id);
  }

  closeTab(id: string) {
    const idx = this.tabs.findIndex(t => t.id === id);
    this.tabs = this.tabs.filter(t => t.id !== id);
    if (this.activeTabId === id && this.tabs.length > 0) {
      this.switchTab(this.tabs[Math.min(idx, this.tabs.length - 1)].id);
    }
  }

  browseTable(name: string, forceNew = false) {
    const query = `SELECT * FROM "${name}" LIMIT 100;`;
    if (!forceNew) {
      const existing = this.tabs.find(t => t.title === name);
      if (existing) {
        this.switchTab(existing.id);
        return;
      }
    }
    this.tabCounter++;
    const id = String(this.tabCounter);
    this.tabs.push({ id, title: name, query, results: [], columns: [], error: null });
    this.switchTab(id);
    setTimeout(() => this.executeRunQuery(), 0);
  }

  saveState() {
    if (!this.isRestored) return;
    const cleanTabs = this.tabs.map(t => ({ id: t.id, title: t.title, query: t.query }));
    vscode.postMessage({ command: 'saveState', state: { tabs: cleanTabs, activeTabId: this.activeTabId, tabCounter: this.tabCounter } });
  }

  showContextMenu(e: MouseEvent, name: string) {
    e.preventDefault();
    this.contextMenu = { visible: true, x: e.clientX, y: e.clientY, tableName: name };
  }

  hideContextMenu() {
    this.contextMenu.visible = false;
  }
}

export const store = new AppState();