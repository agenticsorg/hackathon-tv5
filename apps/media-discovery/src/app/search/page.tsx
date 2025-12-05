'use client';

import { useSearchParams } from 'next/navigation';
import { useQuery } from '@tanstack/react-query';
import { Suspense } from 'react';
import Link from 'next/link';
import { SearchBar } from '@/components/SearchBar';
import { MediaCard } from '@/components/MediaCard';
import type { SearchResult } from '@/types/media';

interface SearchResponse {
  success: boolean;
  query: string;
  intent?: {
    mood?: string[];
    themes?: string[];
    genres?: string[];
  };
  results: SearchResult[];
  totalResults: number;
}

async function fetchSearchResults(query: string): Promise<SearchResponse> {
  const response = await fetch(`/api/search?q=${encodeURIComponent(query)}`);
  if (!response.ok) throw new Error('Search failed');
  return response.json();
}

function SearchResults() {
  const searchParams = useSearchParams();
  const query = searchParams.get('q') || '';

  const { data, isLoading, error } = useQuery({
    queryKey: ['search', query],
    queryFn: () => fetchSearchResults(query),
    enabled: !!query,
  });

  if (!query) {
    return (
      <div className="text-center py-16">
        <p className="text-gray-500 text-lg">Enter a search query to find movies and TV shows</p>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="space-y-8">
        <div className="h-6 w-48 bg-gray-200 dark:bg-gray-800 rounded animate-pulse" />
        <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
          {Array.from({ length: 12 }).map((_, i) => (
            <div
              key={i}
              className="aspect-[2/3] bg-gray-200 dark:bg-gray-800 rounded-lg animate-pulse"
            />
          ))}
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="text-center py-16">
        <p className="text-red-500 text-lg">Failed to search. Please try again.</p>
      </div>
    );
  }

  if (!data?.results.length) {
    return (
      <div className="text-center py-16">
        <p className="text-gray-500 text-lg">No results found for &ldquo;{query}&rdquo;</p>
        <p className="text-gray-400 mt-2">Try a different search or browse trending content</p>
        <Link href="/" className="inline-block mt-4 text-blue-600 hover:underline">
          Back to home
        </Link>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Search summary */}
      <div className="flex flex-wrap items-center gap-4">
        <p className="text-gray-600 dark:text-gray-400">
          Found <span className="font-semibold text-gray-900 dark:text-white">{data.totalResults}</span> results for &ldquo;{query}&rdquo;
        </p>

        {/* Show detected intent */}
        {data.intent && (data.intent.mood?.length || data.intent.themes?.length) && (
          <div className="flex flex-wrap gap-2">
            {data.intent.mood?.map((mood) => (
              <span key={mood} className="px-2 py-0.5 text-xs bg-purple-100 dark:bg-purple-900 text-purple-700 dark:text-purple-300 rounded-full">
                {mood}
              </span>
            ))}
            {data.intent.themes?.map((theme) => (
              <span key={theme} className="px-2 py-0.5 text-xs bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300 rounded-full">
                {theme}
              </span>
            ))}
          </div>
        )}
      </div>

      {/* Results grid */}
      <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
        {data.results.map((result) => (
          <MediaCard
            key={`${result.content.mediaType}-${result.content.id}`}
            content={result.content}
            reason={result.matchReasons?.[0]}
          />
        ))}
      </div>
    </div>
  );
}

export default function SearchPage() {
  return (
    <main className="min-h-screen pb-12">
      {/* Header with search */}
      <header className="sticky top-0 z-10 bg-white/80 dark:bg-gray-950/80 backdrop-blur-lg border-b border-gray-200 dark:border-gray-800">
        <div className="max-w-7xl mx-auto px-4 py-4 flex items-center gap-4">
          <Link href="/" className="flex-shrink-0">
            <svg className="w-8 h-8 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
            </svg>
          </Link>
          <div className="flex-1">
            <Suspense fallback={<div className="h-14 bg-gray-200 dark:bg-gray-800 rounded-xl animate-pulse" />}>
              <SearchBar />
            </Suspense>
          </div>
        </div>
      </header>

      {/* Results */}
      <div className="max-w-7xl mx-auto px-4 py-8">
        <Suspense fallback={<SearchSkeleton />}>
          <SearchResults />
        </Suspense>
      </div>
    </main>
  );
}

function SearchSkeleton() {
  return (
    <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
      {Array.from({ length: 12 }).map((_, i) => (
        <div
          key={i}
          className="aspect-[2/3] bg-gray-200 dark:bg-gray-800 rounded-lg animate-pulse"
        />
      ))}
    </div>
  );
}
