import { useState, useEffect, useRef, useCallback } from 'react';
import { useAppStore, SearchResult } from '../stores/appStore';
import './SearchModal.css';

export function SearchModal() {
  const { closeSearch, search, openNote } = useAppStore();
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [isLoading, setIsLoading] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const debounceRef = useRef<number | null>(null);

  // Focus input on mount
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Search with debounce
  useEffect(() => {
    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }

    if (query.trim()) {
      setIsLoading(true);
      debounceRef.current = window.setTimeout(async () => {
        const searchResults = await search(query);
        setResults(searchResults);
        setSelectedIndex(0);
        setIsLoading(false);
      }, 150);
    } else {
      setResults([]);
      setIsLoading(false);
    }

    return () => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
    };
  }, [query, search]);

  const handleSelect = useCallback((result: SearchResult) => {
    openNote(result.path);
    closeSearch();
  }, [openNote, closeSearch]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex(i => Math.min(i + 1, results.length - 1));
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex(i => Math.max(i - 1, 0));
        break;
      case 'Enter':
        e.preventDefault();
        if (results[selectedIndex]) {
          handleSelect(results[selectedIndex]);
        }
        break;
      case 'Escape':
        e.preventDefault();
        closeSearch();
        break;
    }
  }, [results, selectedIndex, handleSelect, closeSearch]);

  return (
    <div className="modal-overlay" onClick={closeSearch}>
      <div className="search-modal" onClick={e => e.stopPropagation()}>
        <div className="search-input-container">
          <SearchIcon />
          <input
            ref={inputRef}
            type="text"
            className="search-input"
            placeholder="Search notes..."
            value={query}
            onChange={e => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
          />
          {isLoading && <LoadingSpinner />}
        </div>

        <div className="search-results">
          {results.length === 0 && query && !isLoading && (
            <div className="search-empty">No results found</div>
          )}
          {results.map((result, index) => (
            <div
              key={result.path}
              className={`search-result ${index === selectedIndex ? 'selected' : ''}`}
              onClick={() => handleSelect(result)}
              onMouseEnter={() => setSelectedIndex(index)}
            >
              <div className="result-title">{result.title}</div>
              <div className="result-path">{result.path}</div>
              {result.snippet && (
                <div className="result-snippet">{result.snippet}</div>
              )}
              <div className="result-meta">
                <span className={`match-type ${result.matchType}`}>
                  {result.matchType}
                </span>
                <span className="score">{(result.score * 100).toFixed(0)}%</span>
              </div>
            </div>
          ))}
        </div>

        <div className="search-footer">
          <span className="hint">
            <kbd>↑↓</kbd> navigate
            <kbd>Enter</kbd> open
            <kbd>Esc</kbd> close
          </span>
        </div>
      </div>
    </div>
  );
}

function SearchIcon() {
  return (
    <svg className="search-icon" viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
      <path d="M15.5 14h-.79l-.28-.27A6.471 6.471 0 0016 9.5 6.5 6.5 0 109.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
    </svg>
  );
}

function LoadingSpinner() {
  return (
    <div className="loading-spinner">
      <svg viewBox="0 0 24 24" width="18" height="18">
        <circle
          cx="12"
          cy="12"
          r="10"
          stroke="currentColor"
          strokeWidth="2"
          fill="none"
          strokeDasharray="60"
          strokeDashoffset="0"
        >
          <animateTransform
            attributeName="transform"
            type="rotate"
            from="0 12 12"
            to="360 12 12"
            dur="1s"
            repeatCount="indefinite"
          />
        </circle>
      </svg>
    </div>
  );
}
