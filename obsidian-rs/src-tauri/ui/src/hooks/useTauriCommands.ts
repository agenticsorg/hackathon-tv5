import { invoke } from '@tauri-apps/api/core';

// Type definitions matching Rust backend
export interface VaultInfo {
  path: string;
  name: string;
  noteCount: number;
}

export interface NoteMetadata {
  title: string;
  tags: string[];
  aliases: string[];
  created?: string;
  modified?: string;
}

export interface SearchResult {
  path: string;
  title: string;
  score: number;
  snippet?: string;
  matchType: string;
}

export interface GraphData {
  nodes: GraphNode[];
  edges: GraphEdge[];
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

export interface BacklinkInfo {
  path: string;
  name: string;
  context?: string;
}

export interface PluginInfo {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  state: string;
  isEnabled: boolean;
}

export interface FrontmatterResponse {
  title?: string;
  tags: string[];
  aliases: string[];
  created?: string;
  modified?: string;
  extra: Record<string, unknown>;
}

// Vault commands
export const openVault = (path: string) =>
  invoke<VaultInfo>('open_vault', { path });

export const createVault = (path: string, name: string) =>
  invoke<VaultInfo>('create_vault', { path, name });

export const closeVault = () =>
  invoke<void>('close_vault');

export const getVaultInfo = () =>
  invoke<VaultInfo>('get_vault_info');

// Note commands
export const readNote = (path: string) =>
  invoke<string>('read_note', { path });

export const writeNote = (path: string, content: string) =>
  invoke<void>('write_note', { path, content });

export const createNote = (folder: string, name: string, template?: string) =>
  invoke<string>('create_note', { folder, name, template });

export const deleteNote = (path: string) =>
  invoke<void>('delete_note', { path });

export const renameNote = (path: string, newName: string) =>
  invoke<string>('rename_note', { path, newName });

// Search commands
export const searchNotes = (query: string, limit?: number) =>
  invoke<SearchResult[]>('search_notes', { query, limit });

export const searchContent = (query: string, limit?: number) =>
  invoke<SearchResult[]>('search_content', { query, limit });

export const quickSwitch = (query: string, limit?: number) =>
  invoke<SearchResult[]>('quick_switch', { query, limit });

// Graph commands
export const getGraphData = () =>
  invoke<GraphData>('get_graph_data');

export const getLocalGraph = (path: string, depth?: number) =>
  invoke<GraphData>('get_local_graph', { path, depth });

export const getBacklinks = (path: string) =>
  invoke<BacklinkInfo[]>('get_backlinks', { path });

// Markdown commands
export const renderMarkdown = (content: string, basePath?: string) =>
  invoke<string>('render_markdown', { content, basePath });

export const parseFrontmatter = (content: string) =>
  invoke<FrontmatterResponse | null>('parse_frontmatter', { content });

// Plugin commands
export const listPlugins = () =>
  invoke<PluginInfo[]>('list_plugins');

export const enablePlugin = (id: string) =>
  invoke<void>('enable_plugin', { id });

export const disablePlugin = (id: string) =>
  invoke<void>('disable_plugin', { id });

// Settings commands
export const getSettings = () =>
  invoke<Record<string, unknown>>('get_settings');

export const setSettings = (settings: Record<string, unknown>) =>
  invoke<void>('set_settings', { settings });
