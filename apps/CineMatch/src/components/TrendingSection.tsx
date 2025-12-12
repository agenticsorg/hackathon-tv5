'use client';

import { useQuery } from '@tanstack/react-query';
import { MediaCard } from './MediaCard';
import type { MediaContent } from '@/types/media';

async function fetchTrending(): Promise<MediaContent[]> {
  const response = await fetch('/api/discover?category=trending&type=all');
  if (!response.ok) throw new Error('Failed to fetch trending');
  const data = await response.json();
  return data.results;
}

export function TrendingSection() {
  const { data: content, isLoading, error } = useQuery({
    queryKey: ['trending'],
    queryFn: fetchTrending,
  });

  if (isLoading) {
    return (
      <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
        {Array.from({ length: 12 }).map((_, i) => (
          <div
            key={i}
            className="aspect-[2/3] bg-gray-200 dark:bg-gray-800 rounded-lg animate-pulse"
          />
        ))}
      </div>
    );
  }

  if (error || !content) {
    return (
      <div className="text-center py-8 text-gray-500">
        Failed to load trending content
      </div>
    );
  }

  return (
    <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
      {content.slice(0, 12).map((item) => (
        <MediaCard key={`${item.mediaType}-${item.id}`} content={item} />
      ))}
    </div>
  );
}
