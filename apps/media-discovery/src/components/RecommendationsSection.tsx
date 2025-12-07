'use client';

import { useQuery } from '@tanstack/react-query';
import { MediaCard } from './MediaCard';
import type { MediaContent } from '@/types/media';

interface RecommendationsSectionProps {
  genreIds?: number[];
}

async function fetchContent(genreIds?: number[]): Promise<MediaContent[]> {
  const url = genreIds?.length
    ? `/api/discover?category=discover&type=all&genres=${genreIds.join(',')}`
    : '/api/discover?category=popular&type=all';
  const response = await fetch(url);
  if (!response.ok) throw new Error('Failed to fetch recommendations');
  const data = await response.json();
  return data.results;
}

export function RecommendationsSection({ genreIds }: RecommendationsSectionProps) {
  const { data: content, isLoading, error } = useQuery({
    queryKey: ['recommendations', genreIds],
    queryFn: () => fetchContent(genreIds),
  });

  if (isLoading) {
    return (
      <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
        {Array.from({ length: 6 }).map((_, i) => (
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
        Failed to load recommendations
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
