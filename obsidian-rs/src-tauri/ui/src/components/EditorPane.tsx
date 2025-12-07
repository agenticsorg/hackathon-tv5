import { useCallback, useEffect, useRef, useState } from 'react';
import { useAppStore } from '../stores/appStore';
import { MarkdownEditor } from './MarkdownEditor';
import { MarkdownPreview } from './MarkdownPreview';
import './EditorPane.css';

type EditorMode = 'source' | 'preview' | 'split';

export function EditorPane() {
  const {
    openNotes,
    activeNoteIndex,
    closeNote,
    setActiveNote,
    saveNote,
    updateNoteContent,
  } = useAppStore();

  const [editorMode, setEditorMode] = useState<EditorMode>('source');

  const activeNote = activeNoteIndex >= 0 ? openNotes[activeNoteIndex] : null;

  // Auto-save
  const saveTimeoutRef = useRef<number | null>(null);
  const { settings } = useAppStore();

  useEffect(() => {
    if (activeNote?.modified && settings.autoSave) {
      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current);
      }
      saveTimeoutRef.current = window.setTimeout(() => {
        saveNote(activeNoteIndex);
      }, settings.autoSaveInterval);
    }

    return () => {
      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current);
      }
    };
  }, [activeNote?.modified, activeNote?.content, settings.autoSave, settings.autoSaveInterval, saveNote, activeNoteIndex]);

  const handleContentChange = useCallback((content: string) => {
    if (activeNoteIndex >= 0) {
      updateNoteContent(activeNoteIndex, content);
    }
  }, [activeNoteIndex, updateNoteContent]);

  if (openNotes.length === 0) {
    return (
      <div className="editor-pane empty">
        <div className="empty-state">
          <p>No note open</p>
          <p className="hint">Press Cmd+O to search and open a note</p>
        </div>
      </div>
    );
  }

  return (
    <div className="editor-pane">
      {/* Tab bar */}
      <div className="tab-bar">
        <div className="tabs">
          {openNotes.map((note, index) => (
            <div
              key={note.path}
              className={`tab ${index === activeNoteIndex ? 'active' : ''} ${note.modified ? 'modified' : ''}`}
              onClick={() => setActiveNote(index)}
            >
              <span className="tab-title">{note.title}</span>
              {note.modified && <span className="modified-dot" />}
              <button
                className="tab-close"
                onClick={(e) => {
                  e.stopPropagation();
                  closeNote(index);
                }}
              >
                ×
              </button>
            </div>
          ))}
        </div>
        <div className="tab-bar-actions">
          <div className="editor-mode-toggle">
            <button
              className={editorMode === 'source' ? 'active' : ''}
              onClick={() => setEditorMode('source')}
              title="Source mode"
            >
              <CodeIcon />
            </button>
            <button
              className={editorMode === 'preview' ? 'active' : ''}
              onClick={() => setEditorMode('preview')}
              title="Reading mode"
            >
              <BookIcon />
            </button>
            <button
              className={editorMode === 'split' ? 'active' : ''}
              onClick={() => setEditorMode('split')}
              title="Split view"
            >
              <SplitIcon />
            </button>
          </div>
        </div>
      </div>

      {/* Editor content */}
      {activeNote && (
        <div className={`editor-content mode-${editorMode}`}>
          {(editorMode === 'source' || editorMode === 'split') && (
            <div className="editor-source">
              <MarkdownEditor
                content={activeNote.content}
                onChange={handleContentChange}
                onSave={() => saveNote(activeNoteIndex)}
              />
            </div>
          )}
          {(editorMode === 'preview' || editorMode === 'split') && (
            <div className="editor-preview">
              <MarkdownPreview content={activeNote.content} />
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function CodeIcon() {
  return (
    <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
      <path d="M9.4 16.6L4.8 12l4.6-4.6L8 6l-6 6 6 6 1.4-1.4zm5.2 0l4.6-4.6-4.6-4.6L16 6l6 6-6 6-1.4-1.4z"/>
    </svg>
  );
}

function BookIcon() {
  return (
    <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
      <path d="M18 2H6c-1.1 0-2 .9-2 2v16c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zM6 4h5v8l-2.5-1.5L6 12V4z"/>
    </svg>
  );
}

function SplitIcon() {
  return (
    <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
      <path d="M3 5v14h18V5H3zm8 12H5V7h6v10zm8 0h-6V7h6v10z"/>
    </svg>
  );
}
