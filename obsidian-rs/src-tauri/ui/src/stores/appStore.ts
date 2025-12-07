import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

export interface Note {
  path: string;
  title: string;
  content: string;
  modified: boolean;
}

export interface FileTreeItem {
  name: string;
  path: string;
  isFolder: boolean;
  children?: FileTreeItem[];
}

export interface SearchResult {
  path: string;
  title: string;
  score: number;
  snippet?: string;
  matchType: string;
}

export interface GraphNode {
  id: string;
  name: string;
  exists: boolean;
  tags: string[];
  linkCount: number;
  backlinkCount: number;
  x?: number;
  y?: number;
}

export interface GraphEdge {
  source: string;
  target: string;
  linkType: string;
}

export interface Settings {
  theme: 'light' | 'dark' | 'system';
  fontSize: number;
  lineWidth: number;
  showLineNumbers: boolean;
  spellcheck: boolean;
  vimMode: boolean;
  autoSave: boolean;
  autoSaveInterval: number;
}

interface AppState {
  // Vault
  vaultPath: string | null;
  vaultName: string | null;
  fileTree: FileTreeItem[];

  // Editor
  openNotes: Note[];
  activeNoteIndex: number;

  // Views
  activeView: 'editor' | 'graph';
  sidebarVisible: boolean;

  // Modals
  isSearchOpen: boolean;
  isSettingsOpen: boolean;

  // Graph
  graphData: { nodes: GraphNode[]; edges: GraphEdge[] } | null;

  // Settings
  settings: Settings;

  // Actions
  initializeApp: () => Promise<void>;
  openVault: () => Promise<void>;
  createVault: () => Promise<void>;
  closeVault: () => Promise<void>;

  refreshFileTree: () => Promise<void>;

  openNote: (path: string) => Promise<void>;
  closeNote: (index: number) => void;
  setActiveNote: (index: number) => void;
  saveNote: (index: number) => Promise<void>;
  updateNoteContent: (index: number, content: string) => void;
  createNote: (folderPath: string, name: string) => Promise<void>;
  deleteNote: (path: string) => Promise<void>;
  renameNote: (oldPath: string, newName: string) => Promise<void>;

  setActiveView: (view: 'editor' | 'graph') => void;
  toggleSidebar: () => void;

  openSearch: () => void;
  closeSearch: () => void;
  search: (query: string) => Promise<SearchResult[]>;

  openSettings: () => void;
  closeSettings: () => void;
  updateSettings: (settings: Partial<Settings>) => void;

  loadGraphData: () => Promise<void>;
  loadLocalGraph: (path: string, depth?: number) => Promise<void>;
}

const defaultSettings: Settings = {
  theme: 'dark',
  fontSize: 16,
  lineWidth: 700,
  showLineNumbers: false,
  spellcheck: true,
  vimMode: false,
  autoSave: true,
  autoSaveInterval: 5000,
};

export const useAppStore = create<AppState>((set, get) => ({
  // Initial state
  vaultPath: null,
  vaultName: null,
  fileTree: [],
  openNotes: [],
  activeNoteIndex: -1,
  activeView: 'editor',
  sidebarVisible: true,
  isSearchOpen: false,
  isSettingsOpen: false,
  graphData: null,
  settings: defaultSettings,

  initializeApp: async () => {
    try {
      const settings = await invoke<Settings>('get_settings');
      set({ settings: { ...defaultSettings, ...settings } });
    } catch (error) {
      console.error('Failed to initialize app:', error);
    }
  },

  openVault: async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Vault Folder',
      });

      if (selected) {
        const vaultInfo = await invoke<{ path: string; name: string; noteCount: number }>('open_vault', {
          path: selected
        });

        set({
          vaultPath: vaultInfo.path,
          vaultName: vaultInfo.name
        });

        await get().refreshFileTree();
      }
    } catch (error) {
      console.error('Failed to open vault:', error);
    }
  },

  createVault: async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Location for New Vault',
      });

      if (selected) {
        const vaultInfo = await invoke<{ path: string; name: string; noteCount: number }>('create_vault', {
          path: selected,
          name: 'My Vault',
        });

        set({
          vaultPath: vaultInfo.path,
          vaultName: vaultInfo.name
        });

        await get().refreshFileTree();
      }
    } catch (error) {
      console.error('Failed to create vault:', error);
    }
  },

  closeVault: async () => {
    try {
      await invoke('close_vault');
      set({
        vaultPath: null,
        vaultName: null,
        fileTree: [],
        openNotes: [],
        activeNoteIndex: -1,
        graphData: null,
      });
    } catch (error) {
      console.error('Failed to close vault:', error);
    }
  },

  refreshFileTree: async () => {
    try {
      const files = await invoke<FileTreeItem[]>('get_file_tree');
      set({ fileTree: files });
    } catch (error) {
      console.error('Failed to refresh file tree:', error);
    }
  },

  openNote: async (path: string) => {
    const { openNotes } = get();

    // Check if note is already open
    const existingIndex = openNotes.findIndex(n => n.path === path);
    if (existingIndex >= 0) {
      set({ activeNoteIndex: existingIndex });
      return;
    }

    try {
      const content = await invoke<string>('read_note', { path });
      const title = path.split('/').pop()?.replace('.md', '') || 'Untitled';

      const newNote: Note = {
        path,
        title,
        content,
        modified: false,
      };

      set({
        openNotes: [...openNotes, newNote],
        activeNoteIndex: openNotes.length,
        activeView: 'editor',
      });
    } catch (error) {
      console.error('Failed to open note:', error);
    }
  },

  closeNote: (index: number) => {
    const { openNotes, activeNoteIndex } = get();
    const newNotes = openNotes.filter((_, i) => i !== index);

    let newActiveIndex = activeNoteIndex;
    if (index <= activeNoteIndex) {
      newActiveIndex = Math.max(0, activeNoteIndex - 1);
    }
    if (newNotes.length === 0) {
      newActiveIndex = -1;
    }

    set({
      openNotes: newNotes,
      activeNoteIndex: newActiveIndex
    });
  },

  setActiveNote: (index: number) => {
    set({ activeNoteIndex: index });
  },

  saveNote: async (index: number) => {
    const { openNotes } = get();
    const note = openNotes[index];

    if (!note) return;

    try {
      await invoke('write_note', {
        path: note.path,
        content: note.content
      });

      const newNotes = [...openNotes];
      newNotes[index] = { ...note, modified: false };
      set({ openNotes: newNotes });
    } catch (error) {
      console.error('Failed to save note:', error);
    }
  },

  updateNoteContent: (index: number, content: string) => {
    const { openNotes } = get();
    const newNotes = [...openNotes];
    newNotes[index] = {
      ...newNotes[index],
      content,
      modified: true
    };
    set({ openNotes: newNotes });
  },

  createNote: async (folderPath: string, name: string) => {
    try {
      const path = await invoke<string>('create_note', {
        folder: folderPath,
        name
      });

      await get().refreshFileTree();
      await get().openNote(path);
    } catch (error) {
      console.error('Failed to create note:', error);
    }
  },

  deleteNote: async (path: string) => {
    try {
      await invoke('delete_note', { path });

      const { openNotes } = get();
      const index = openNotes.findIndex(n => n.path === path);
      if (index >= 0) {
        get().closeNote(index);
      }

      await get().refreshFileTree();
    } catch (error) {
      console.error('Failed to delete note:', error);
    }
  },

  renameNote: async (oldPath: string, newName: string) => {
    try {
      const newPath = await invoke<string>('rename_note', {
        path: oldPath,
        newName
      });

      const { openNotes } = get();
      const index = openNotes.findIndex(n => n.path === oldPath);
      if (index >= 0) {
        const newNotes = [...openNotes];
        newNotes[index] = {
          ...newNotes[index],
          path: newPath,
          title: newName.replace('.md', ''),
        };
        set({ openNotes: newNotes });
      }

      await get().refreshFileTree();
    } catch (error) {
      console.error('Failed to rename note:', error);
    }
  },

  setActiveView: (view) => {
    set({ activeView: view });
    if (view === 'graph') {
      get().loadGraphData();
    }
  },

  toggleSidebar: () => {
    set(state => ({ sidebarVisible: !state.sidebarVisible }));
  },

  openSearch: () => {
    set({ isSearchOpen: true });
  },

  closeSearch: () => {
    set({ isSearchOpen: false });
  },

  search: async (query: string) => {
    try {
      const results = await invoke<SearchResult[]>('search_notes', {
        query,
        limit: 20
      });
      return results;
    } catch (error) {
      console.error('Failed to search:', error);
      return [];
    }
  },

  openSettings: () => {
    set({ isSettingsOpen: true });
  },

  closeSettings: () => {
    set({ isSettingsOpen: false });
  },

  updateSettings: async (newSettings: Partial<Settings>) => {
    const { settings } = get();
    const updated = { ...settings, ...newSettings };

    try {
      await invoke('set_settings', { settings: updated });
      set({ settings: updated });
    } catch (error) {
      console.error('Failed to update settings:', error);
    }
  },

  loadGraphData: async () => {
    try {
      const data = await invoke<{ nodes: GraphNode[]; edges: GraphEdge[] }>('get_graph_data');
      set({ graphData: data });
    } catch (error) {
      console.error('Failed to load graph data:', error);
    }
  },

  loadLocalGraph: async (path: string, depth = 2) => {
    try {
      const data = await invoke<{ nodes: GraphNode[]; edges: GraphEdge[] }>('get_local_graph', {
        path,
        depth
      });
      set({ graphData: data });
    } catch (error) {
      console.error('Failed to load local graph:', error);
    }
  },
}));
