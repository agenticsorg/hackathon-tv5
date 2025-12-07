import { useState, useCallback } from 'react';
import { useAppStore, FileTreeItem } from '../stores/appStore';
import './Sidebar.css';

export function Sidebar() {
  const {
    sidebarVisible,
    vaultName,
    fileTree,
    openNote,
    createNote,
    deleteNote,
    renameNote,
    setActiveView,
    activeView,
    openSearch,
  } = useAppStore();

  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set());
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    item: FileTreeItem;
  } | null>(null);
  const [renamingPath, setRenamingPath] = useState<string | null>(null);
  const [newName, setNewName] = useState('');

  const toggleFolder = useCallback((path: string) => {
    setExpandedFolders(prev => {
      const next = new Set(prev);
      if (next.has(path)) {
        next.delete(path);
      } else {
        next.add(path);
      }
      return next;
    });
  }, []);

  const handleContextMenu = useCallback((e: React.MouseEvent, item: FileTreeItem) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY, item });
  }, []);

  const closeContextMenu = useCallback(() => {
    setContextMenu(null);
  }, []);

  const handleNewNote = useCallback(() => {
    if (contextMenu) {
      const folder = contextMenu.item.isFolder
        ? contextMenu.item.path
        : contextMenu.item.path.substring(0, contextMenu.item.path.lastIndexOf('/'));
      createNote(folder, 'Untitled');
    }
    closeContextMenu();
  }, [contextMenu, createNote, closeContextMenu]);

  const handleDelete = useCallback(() => {
    if (contextMenu && !contextMenu.item.isFolder) {
      deleteNote(contextMenu.item.path);
    }
    closeContextMenu();
  }, [contextMenu, deleteNote, closeContextMenu]);

  const startRename = useCallback(() => {
    if (contextMenu) {
      setRenamingPath(contextMenu.item.path);
      setNewName(contextMenu.item.name.replace('.md', ''));
    }
    closeContextMenu();
  }, [contextMenu, closeContextMenu]);

  const submitRename = useCallback((path: string) => {
    if (newName.trim()) {
      renameNote(path, newName.trim());
    }
    setRenamingPath(null);
    setNewName('');
  }, [newName, renameNote]);

  if (!sidebarVisible) {
    return null;
  }

  return (
    <div className="sidebar" onClick={closeContextMenu}>
      <div className="sidebar-header">
        <div className="vault-name">{vaultName || 'No vault'}</div>
        <div className="sidebar-actions">
          <button
            className="sidebar-action-btn"
            onClick={openSearch}
            title="Search (Cmd+O)"
          >
            <SearchIcon />
          </button>
          <button
            className="sidebar-action-btn"
            onClick={() => createNote('', 'Untitled')}
            title="New note"
          >
            <PlusIcon />
          </button>
        </div>
      </div>

      <div className="nav-tabs">
        <button
          className={`nav-tab ${activeView === 'editor' ? 'active' : ''}`}
          onClick={() => setActiveView('editor')}
        >
          <FileIcon /> Files
        </button>
        <button
          className={`nav-tab ${activeView === 'graph' ? 'active' : ''}`}
          onClick={() => setActiveView('graph')}
        >
          <GraphIcon /> Graph
        </button>
      </div>

      <div className="file-tree">
        {fileTree.map(item => (
          <FileTreeNode
            key={item.path}
            item={item}
            level={0}
            expandedFolders={expandedFolders}
            toggleFolder={toggleFolder}
            onContextMenu={handleContextMenu}
            renamingPath={renamingPath}
            newName={newName}
            setNewName={setNewName}
            submitRename={submitRename}
            openNote={openNote}
          />
        ))}
      </div>

      {contextMenu && (
        <div
          className="context-menu"
          style={{ left: contextMenu.x, top: contextMenu.y }}
          onClick={e => e.stopPropagation()}
        >
          <button onClick={handleNewNote}>New note</button>
          {!contextMenu.item.isFolder && (
            <>
              <button onClick={startRename}>Rename</button>
              <button onClick={handleDelete} className="danger">Delete</button>
            </>
          )}
        </div>
      )}
    </div>
  );
}

interface FileTreeNodeProps {
  item: FileTreeItem;
  level: number;
  expandedFolders: Set<string>;
  toggleFolder: (path: string) => void;
  onContextMenu: (e: React.MouseEvent, item: FileTreeItem) => void;
  renamingPath: string | null;
  newName: string;
  setNewName: (name: string) => void;
  submitRename: (path: string) => void;
  openNote: (path: string) => void;
}

function FileTreeNode({
  item,
  level,
  expandedFolders,
  toggleFolder,
  onContextMenu,
  renamingPath,
  newName,
  setNewName,
  submitRename,
  openNote,
}: FileTreeNodeProps) {
  const isExpanded = expandedFolders.has(item.path);
  const isRenaming = renamingPath === item.path;

  const handleClick = () => {
    if (item.isFolder) {
      toggleFolder(item.path);
    } else {
      openNote(item.path);
    }
  };

  return (
    <div className="tree-node">
      <div
        className={`tree-item ${item.isFolder ? 'folder' : 'file'}`}
        style={{ paddingLeft: `${level * 16 + 8}px` }}
        onClick={handleClick}
        onContextMenu={(e) => onContextMenu(e, item)}
      >
        {item.isFolder ? (
          <span className={`folder-icon ${isExpanded ? 'expanded' : ''}`}>
            <ChevronIcon />
          </span>
        ) : (
          <span className="file-icon">
            <NoteIcon />
          </span>
        )}
        {isRenaming ? (
          <input
            className="rename-input"
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            onBlur={() => submitRename(item.path)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') submitRename(item.path);
              if (e.key === 'Escape') {
                setNewName('');
                submitRename(item.path);
              }
            }}
            autoFocus
            onClick={(e) => e.stopPropagation()}
          />
        ) : (
          <span className="tree-item-name">
            {item.name.replace('.md', '')}
          </span>
        )}
      </div>
      {item.isFolder && isExpanded && item.children && (
        <div className="tree-children">
          {item.children.map(child => (
            <FileTreeNode
              key={child.path}
              item={child}
              level={level + 1}
              expandedFolders={expandedFolders}
              toggleFolder={toggleFolder}
              onContextMenu={onContextMenu}
              renamingPath={renamingPath}
              newName={newName}
              setNewName={setNewName}
              submitRename={submitRename}
              openNote={openNote}
            />
          ))}
        </div>
      )}
    </div>
  );
}

// Icons
function SearchIcon() {
  return (
    <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
      <path d="M15.5 14h-.79l-.28-.27A6.471 6.471 0 0016 9.5 6.5 6.5 0 109.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
    </svg>
  );
}

function PlusIcon() {
  return (
    <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
      <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
    </svg>
  );
}

function FileIcon() {
  return (
    <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
      <path d="M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6zm2 16H8v-2h8v2zm0-4H8v-2h8v2zm-3-5V3.5L18.5 9H13z"/>
    </svg>
  );
}

function GraphIcon() {
  return (
    <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
      <circle cx="12" cy="12" r="3"/>
      <circle cx="6" cy="6" r="2"/>
      <circle cx="18" cy="6" r="2"/>
      <circle cx="6" cy="18" r="2"/>
      <circle cx="18" cy="18" r="2"/>
      <line x1="12" y1="9" x2="8" y2="6" stroke="currentColor" strokeWidth="1.5"/>
      <line x1="12" y1="9" x2="16" y2="6" stroke="currentColor" strokeWidth="1.5"/>
      <line x1="12" y1="15" x2="8" y2="18" stroke="currentColor" strokeWidth="1.5"/>
      <line x1="12" y1="15" x2="16" y2="18" stroke="currentColor" strokeWidth="1.5"/>
    </svg>
  );
}

function ChevronIcon() {
  return (
    <svg viewBox="0 0 24 24" width="12" height="12" fill="currentColor">
      <path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/>
    </svg>
  );
}

function NoteIcon() {
  return (
    <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
      <path d="M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6zm-1 7V3.5L18.5 9H13z"/>
    </svg>
  );
}
