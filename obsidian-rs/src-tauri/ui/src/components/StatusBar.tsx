import { useAppStore } from '../stores/appStore';
import './StatusBar.css';

export function StatusBar() {
  const { openNotes, activeNoteIndex, vaultName, vaultPath } = useAppStore();

  const activeNote = activeNoteIndex >= 0 ? openNotes[activeNoteIndex] : null;

  // Calculate word count for active note
  const wordCount = activeNote
    ? activeNote.content.split(/\s+/).filter(Boolean).length
    : 0;

  const charCount = activeNote ? activeNote.content.length : 0;

  return (
    <div className="status-bar">
      <div className="status-left">
        {vaultName && (
          <span className="status-item vault-status" title={vaultPath || ''}>
            <VaultIcon /> {vaultName}
          </span>
        )}
      </div>
      <div className="status-center">
        {activeNote?.modified && (
          <span className="status-item modified-indicator">
            Unsaved changes
          </span>
        )}
      </div>
      <div className="status-right">
        {activeNote && (
          <>
            <span className="status-item">
              {wordCount} words
            </span>
            <span className="status-item">
              {charCount} chars
            </span>
          </>
        )}
      </div>
    </div>
  );
}

function VaultIcon() {
  return (
    <svg viewBox="0 0 24 24" width="12" height="12" fill="currentColor">
      <path d="M20 6h-8l-2-2H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2z"/>
    </svg>
  );
}
