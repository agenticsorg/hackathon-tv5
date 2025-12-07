'use client';

import { useState, useCallback } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';

export function SearchBar() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [query, setQuery] = useState(searchParams.get('q') || '');
  const [isLoading, setIsLoading] = useState(false);
  const [isFocused, setIsFocused] = useState(false);

  const handleSubmit = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();
      if (!query.trim()) return;

      setIsLoading(true);
      router.push(`/search?q=${encodeURIComponent(query.trim())}`);
    },
    [query, router]
  );

  return (
    <form onSubmit={handleSubmit} className="relative group">
      {/* Animated border wrapper - always animating, more intense on focus */}
      <div
        className={`absolute -inset-[2px] rounded-2xl transition-all duration-300 bg-gradient-to-r from-accent-cyan via-genre-scifi to-accent-cyan bg-[length:200%_100%] ${
          isFocused ? 'opacity-100' : 'opacity-40'
        }`}
        style={{
          animation: 'gradientShift 3s ease infinite',
        }}
      />

      {/* Input container */}
      <div className="relative bg-bg-primary rounded-2xl">
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onFocus={() => setIsFocused(true)}
          onBlur={() => setIsFocused(false)}
          placeholder="Describe what you want to watch..."
          className="w-full h-16 md:h-20 px-6 md:px-8 pr-16 md:pr-20 text-lg md:text-xl bg-transparent text-text-primary placeholder:text-text-secondary rounded-2xl focus:outline-none transition-all"
          disabled={isLoading}
        />
        <button
          type="submit"
          disabled={isLoading || !query.trim()}
          className="absolute right-3 md:right-4 top-1/2 -translate-y-1/2 p-3 md:p-4 bg-accent-cyan hover:bg-accent-cyan-dim disabled:bg-bg-elevated text-bg-deep disabled:text-text-secondary rounded-xl transition-all hover:-translate-y-[calc(50%+2px)]"
        >
          {isLoading ? (
            <svg
              className="w-5 h-5 md:w-6 md:h-6 animate-spin"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle
                className="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                strokeWidth="4"
              />
              <path
                className="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
              />
            </svg>
          ) : (
            <svg
              className="w-5 h-5 md:w-6 md:h-6"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
              />
            </svg>
          )}
        </button>
      </div>

      <style jsx>{`
        @keyframes gradientShift {
          0%, 100% { background-position: 0% 50%; }
          50% { background-position: 100% 50%; }
        }
      `}</style>
    </form>
  );
}
