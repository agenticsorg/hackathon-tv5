'use client';

import { useState, useCallback, useEffect } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { VoiceSearch } from './VoiceSearch';

export function SearchBar() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [query, setQuery] = useState(searchParams.get('q') || '');
  const [isLoading, setIsLoading] = useState(false);
  const [isListening, setIsListening] = useState(false);

  // Reset loading state when search params change (navigation completed)
  useEffect(() => {
    setIsLoading(false);
  }, [searchParams]);

  const handleSubmit = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();
      if (!query.trim() || isLoading) return;

      setIsLoading(true);
      try {
        router.push(`/search?q=${encodeURIComponent(query.trim())}`);
      } catch (error) {
        console.error('Search navigation failed:', error);
        setIsLoading(false);
      }
    },
    [query, router, isLoading]
  );

  const handleVoiceResult = useCallback(
    (transcript: string) => {
      if (isLoading) return; // Prevent duplicate submissions

      setQuery(transcript);
      // Auto-submit after voice input
      setIsLoading(true);
      try {
        router.push(`/search?q=${encodeURIComponent(transcript.trim())}`);
      } catch (error) {
        console.error('Voice search navigation failed:', error);
        setIsLoading(false);
      }
    },
    [router, isLoading]
  );

  const handleInterimResult = useCallback((transcript: string) => {
    setQuery(transcript);
  }, []);

  const handleVoiceError = useCallback((error: string) => {
    console.error('Voice search error:', error);
    // Reset loading state on voice errors
    setIsLoading(false);
  }, []);

  return (
    <form onSubmit={handleSubmit} className="relative flex items-center gap-3">
      <div className="relative flex-1">
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder={isListening ? "Listening..." : "Describe what you want to watch..."}
          className={`w-full h-14 px-6 pr-14 text-lg bg-white dark:bg-gray-800 border ${isListening ? 'border-red-400 ring-2 ring-red-400/30' : 'border-gray-200 dark:border-gray-700'} rounded-xl shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all`}
          disabled={isLoading}
        />
        <button
          type="submit"
          disabled={isLoading || !query.trim()}
          className="absolute right-2 top-1/2 -translate-y-1/2 p-2.5 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white rounded-lg transition-colors"
        >
          {isLoading ? (
            <svg
              className="w-5 h-5 animate-spin"
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
              className="w-5 h-5"
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
      <VoiceSearch
        onResult={handleVoiceResult}
        onInterimResult={handleInterimResult}
        onError={handleVoiceError}
        onListeningChange={setIsListening}
        disabled={isLoading}
        className="flex-shrink-0"
      />
    </form>
  );
}
