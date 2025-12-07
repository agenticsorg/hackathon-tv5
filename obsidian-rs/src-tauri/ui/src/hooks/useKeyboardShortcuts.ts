import { useEffect, useCallback } from 'react';
import { useAppStore } from '../stores/appStore';

export function useKeyboardShortcuts() {
  const {
    openSearch,
    closeSearch,
    isSearchOpen,
    openSettings,
    closeSettings,
    isSettingsOpen,
    toggleSidebar,
    openNotes,
    activeNoteIndex,
    saveNote,
    closeNote,
    setActiveView,
    activeView,
  } = useAppStore();

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    const isMod = e.metaKey || e.ctrlKey;

    // Escape - close modals
    if (e.key === 'Escape') {
      if (isSearchOpen) {
        closeSearch();
        e.preventDefault();
      } else if (isSettingsOpen) {
        closeSettings();
        e.preventDefault();
      }
      return;
    }

    // Cmd/Ctrl + O - Quick switcher / Search
    if (isMod && e.key === 'o') {
      e.preventDefault();
      if (isSearchOpen) {
        closeSearch();
      } else {
        openSearch();
      }
      return;
    }

    // Cmd/Ctrl + P - Command palette (same as search for now)
    if (isMod && e.key === 'p') {
      e.preventDefault();
      if (isSearchOpen) {
        closeSearch();
      } else {
        openSearch();
      }
      return;
    }

    // Cmd/Ctrl + , - Settings
    if (isMod && e.key === ',') {
      e.preventDefault();
      if (isSettingsOpen) {
        closeSettings();
      } else {
        openSettings();
      }
      return;
    }

    // Cmd/Ctrl + \ - Toggle sidebar
    if (isMod && e.key === '\\') {
      e.preventDefault();
      toggleSidebar();
      return;
    }

    // Cmd/Ctrl + S - Save
    if (isMod && e.key === 's') {
      e.preventDefault();
      if (activeNoteIndex >= 0 && openNotes[activeNoteIndex]?.modified) {
        saveNote(activeNoteIndex);
      }
      return;
    }

    // Cmd/Ctrl + W - Close tab
    if (isMod && e.key === 'w') {
      e.preventDefault();
      if (activeNoteIndex >= 0) {
        closeNote(activeNoteIndex);
      }
      return;
    }

    // Cmd/Ctrl + G - Toggle graph view
    if (isMod && e.key === 'g') {
      e.preventDefault();
      setActiveView(activeView === 'graph' ? 'editor' : 'graph');
      return;
    }

    // Cmd/Ctrl + Tab - Next tab
    if (isMod && e.key === 'Tab' && !e.shiftKey) {
      e.preventDefault();
      if (openNotes.length > 1) {
        const nextIndex = (activeNoteIndex + 1) % openNotes.length;
        useAppStore.getState().setActiveNote(nextIndex);
      }
      return;
    }

    // Cmd/Ctrl + Shift + Tab - Previous tab
    if (isMod && e.key === 'Tab' && e.shiftKey) {
      e.preventDefault();
      if (openNotes.length > 1) {
        const prevIndex = (activeNoteIndex - 1 + openNotes.length) % openNotes.length;
        useAppStore.getState().setActiveNote(prevIndex);
      }
      return;
    }

    // Cmd/Ctrl + 1-9 - Switch to tab
    if (isMod && e.key >= '1' && e.key <= '9') {
      e.preventDefault();
      const tabIndex = parseInt(e.key) - 1;
      if (tabIndex < openNotes.length) {
        useAppStore.getState().setActiveNote(tabIndex);
      }
      return;
    }
  }, [
    isSearchOpen,
    isSettingsOpen,
    openSearch,
    closeSearch,
    openSettings,
    closeSettings,
    toggleSidebar,
    openNotes,
    activeNoteIndex,
    saveNote,
    closeNote,
    setActiveView,
    activeView,
  ]);

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);
}
