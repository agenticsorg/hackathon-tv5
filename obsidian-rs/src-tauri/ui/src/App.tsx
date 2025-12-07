import { useEffect } from 'react';
import { Sidebar } from './components/Sidebar';
import { EditorPane } from './components/EditorPane';
import { GraphView } from './components/GraphView';
import { SearchModal } from './components/SearchModal';
import { SettingsModal } from './components/SettingsModal';
import { StatusBar } from './components/StatusBar';
import { useAppStore } from './stores/appStore';
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts';
import './styles/app.css';

function App() {
  const {
    activeView,
    isSearchOpen,
    isSettingsOpen,
    vaultPath,
    initializeApp
  } = useAppStore();

  useKeyboardShortcuts();

  useEffect(() => {
    initializeApp();
  }, [initializeApp]);

  return (
    <div className="app-container">
      <div className="app-layout">
        <Sidebar />
        <div className="main-content">
          {activeView === 'editor' && <EditorPane />}
          {activeView === 'graph' && <GraphView />}
        </div>
      </div>
      <StatusBar />

      {isSearchOpen && <SearchModal />}
      {isSettingsOpen && <SettingsModal />}

      {!vaultPath && (
        <div className="welcome-overlay">
          <WelcomeScreen />
        </div>
      )}
    </div>
  );
}

function WelcomeScreen() {
  const { openVault, createVault } = useAppStore();

  return (
    <div className="welcome-screen">
      <div className="welcome-logo">
        <svg viewBox="0 0 100 100" className="logo-icon">
          <circle cx="50" cy="50" r="45" fill="var(--interactive-accent)" />
          <path d="M30 50 L50 30 L70 50 L50 70 Z" fill="var(--background-primary)" />
        </svg>
        <h1>Obsidian-RS</h1>
      </div>
      <p className="welcome-tagline">A second brain, written in Rust</p>
      <div className="welcome-actions">
        <button className="mod-cta" onClick={openVault}>
          Open vault
        </button>
        <button onClick={createVault}>
          Create new vault
        </button>
      </div>
    </div>
  );
}

export default App;
