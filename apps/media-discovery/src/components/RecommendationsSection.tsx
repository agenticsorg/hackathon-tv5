'use client';

import { useQuery } from '@tanstack/react-query';
import { MediaCard } from './MediaCard';
import type { Recommendation } from '@/types/media';

async function fetchRecommendations(): Promise<Recommendation[]> {
  const response = await fetch('/api/recommendations');
  if (!response.ok) throw new Error('Failed to fetch recommendations');
  const data = await response.json();
  return data.recommendations;
}

export function RecommendationsSection() {
  const { data: recommendations, isLoading, error } = useQuery({
    queryKey: ['recommendations'],
    queryFn: fetchRecommendations,
  });

  if (isLoading) {
    return (
      <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
        {Array.from({ length: 6 }).map((_, i) => (
          <div
            key={i}
            className="aspect-[2/3] bg-gray-200 dark:bg-gray-800 rounded-lg animate-pulse"
          />
        ))}
      </div>
    );
  }

  if (error || !recommendations) {
    return (
      <div className="text-center py-8 text-gray-500">
        Failed to load recommendations
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
        {recommendations.slice(0, 12).map((rec) => (
          <MediaCard
            key={`${rec.content.mediaType}-${rec.content.id}`}
            content={rec.content}
            reason={rec.reasons[0]}
          />
        ))}
      </div>
    </div>
  );
}
