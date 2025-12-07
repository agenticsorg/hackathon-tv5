'use client';

import { useState } from 'react';
import { Spinner } from '@heroui/react';

interface SearchBarProps {
  onSearch: (query: string) => void;
  isLoading?: boolean;
}

const EXAMPLE_QUERIES = [
  { text: 'Something fun for date night', icon: '💑' },
  { text: 'French drama series', icon: '🎭' },
  { text: 'Like Breaking Bad', icon: '🧪' },
  { text: 'Kids can watch', icon: '👶' },
  { text: 'Trending now', icon: '📈' },
];

export function SearchBar({ onSearch, isLoading }: SearchBarProps) {
  const [query, setQuery] = useState('');
  const [isFocused, setIsFocused] = useState(false);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (query.trim()) {
      onSearch(query.trim());
    }
  };

  return (
    <div className="w-full max-w-3xl mx-auto">
      {/* Search input container */}
      <form onSubmit={handleSubmit} className="relative">
        <div className={`
          search-input rounded-2xl p-1 transition-all duration-300
          ${isFocused ? 'ring-2 ring-emerald-500/50 border-emerald-500/50' : ''}
        `}>
          <div className="flex items-center gap-3 px-5 py-3">
            {/* Search icon */}
            <div className={`transition-colors duration-300 ${isFocused ? 'text-emerald-400' : 'text-zinc-500'}`}>
              <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
            </div>

            {/* Input field */}
            <input
              type="text"
              placeholder="What do you want to watch? Try: 'something exciting for tonight'"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onFocus={() => setIsFocused(true)}
              onBlur={() => setIsFocused(false)}
              className="flex-1 bg-transparent text-lg text-white placeholder:text-zinc-500 focus:outline-none"
            />

            {/* Search button */}
            <button
              type="submit"
              disabled={isLoading || !query.trim()}
              className={`
                flex items-center gap-2 px-5 py-2.5 rounded-xl font-medium text-sm transition-all duration-300
                ${query.trim()
                  ? 'bg-emerald-500 hover:bg-emerald-400 text-white shadow-lg shadow-emerald-500/25'
                  : 'bg-zinc-700 text-zinc-400 cursor-not-allowed'
                }
              `}
            >
              {isLoading ? (
                <Spinner size="sm" color="white" />
              ) : (
                <>
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                  </svg>
                  Search
                </>
              )}
            </button>
          </div>
        </div>

        {/* AI badge */}
        <div className="absolute -top-3 left-6 flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-zinc-900 border border-zinc-700">
          <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
          <span className="text-xs text-zinc-400">AI-powered search</span>
        </div>
      </form>

      {/* Example queries */}
      <div className="mt-6">
        <p className="text-center text-xs text-zinc-500 mb-3">Try these searches</p>
        <div className="flex flex-wrap gap-2 justify-center">
          {EXAMPLE_QUERIES.map((example) => (
            <button
              key={example.text}
              onClick={() => {
                setQuery(example.text);
                onSearch(example.text);
              }}
              className="group flex items-center gap-2 px-4 py-2 rounded-xl bg-zinc-800/50 border border-zinc-700/50 hover:border-emerald-500/30 hover:bg-zinc-800 transition-all duration-300"
            >
              <span className="text-base opacity-80 group-hover:scale-110 transition-transform duration-300">
                {example.icon}
              </span>
              <span className="text-sm text-zinc-400 group-hover:text-white transition-colors">
                {example.text}
              </span>
            </button>
          ))}
        </div>
      </div>

      {/* Keyboard hint */}
      <div className="mt-4 flex items-center justify-center gap-2 text-xs text-zinc-600">
        <span>Press</span>
        <kbd className="px-2 py-0.5 rounded bg-zinc-800 border border-zinc-700 text-zinc-400">Enter</kbd>
        <span>to search</span>
      </div>
    </div>
  );
}
