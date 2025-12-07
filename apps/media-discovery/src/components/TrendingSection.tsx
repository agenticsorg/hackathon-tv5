'use client';

import { useQuery } from '@tanstack/react-query';
import { MediaCard } from './MediaCard';
import type { MediaContent } from '@/types/media';

interface TrendingSectionProps {
  genreIds?: number[];
}

async function fetchContent(genreIds?: number[]): Promise<MediaContent[]> {
  const url = genreIds?.length
    ? `/api/discover?category=discover&type=all&genres=${genreIds.join(',')}`
    : '/api/discover?category=trending&type=all';
  const response = await fetch(url);
  if (!response.ok) throw new Error('Failed to fetch content');
  const data = await response.json();
  return data.results;
}

export function TrendingSection({ genreIds }: TrendingSectionProps) {
  const { data: content, isLoading, error } = useQuery({
    queryKey: ['trending', genreIds],
    queryFn: () => fetchContent(genreIds),
  });

  if (isLoading) {
    return (
      <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
        {Array.from({ length: 12 }).map((_, i) => (
          <div
            key={i}
            className="aspect-[2/3] bg-bg-elevated rounded-lg animate-pulse"
          />
        ))}
      </div>
    );
  }

  if (error || !content) {
    return (
      <div className="text-center py-8 text-text-secondary">
        Failed to load trending content
      </div>
    );
  }

  return (
    <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
      {content.slice(0, 12).map((item, index) => (
        <MediaCard
          key={`${item.mediaType}-${item.id}`}
          content={item}
          index={index}
        />
      ))}
    </div>
  );
}
