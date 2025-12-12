'use client';

import { useState } from 'react';
import { MediaCard } from '@/components/MediaCard';
import type { Movie, TVShow } from '@/types/media';

interface RelatedSectionProps {
  similar: (Movie | TVShow)[];
  recommendations: (Movie | TVShow)[];
}

export function RelatedSection({ similar, recommendations }: RelatedSectionProps) {
  const [activeTab, setActiveTab] = useState<'recommendations' | 'similar'>(
    recommendations.length > 0 ? 'recommendations' : 'similar'
  );

  const hasContent = similar.length > 0 || recommendations.length > 0;
  if (!hasContent) return null;

  const displayContent = activeTab === 'recommendations' ? recommendations : similar;

  return (
    <section className="py-8 px-4 md:px-8 bg-gray-900/50">
      <div className="max-w-7xl mx-auto">
        {/* Tabs */}
        <div className="flex items-center gap-6 mb-6">
          <h2 className="text-xl md:text-2xl font-bold text-white">More Like This</h2>

          <div className="flex gap-2">
            {recommendations.length > 0 && (
              <button
                onClick={() => setActiveTab('recommendations')}
                className={`px-4 py-1.5 text-sm font-medium rounded-full transition-colors ${
                  activeTab === 'recommendations'
                    ? 'bg-blue-600 text-white'
                    : 'bg-gray-800 text-gray-400 hover:bg-gray-700 hover:text-white'
                }`}
              >
                Recommended
              </button>
            )}
            {similar.length > 0 && (
              <button
                onClick={() => setActiveTab('similar')}
                className={`px-4 py-1.5 text-sm font-medium rounded-full transition-colors ${
                  activeTab === 'similar'
                    ? 'bg-blue-600 text-white'
                    : 'bg-gray-800 text-gray-400 hover:bg-gray-700 hover:text-white'
                }`}
              >
                Similar
              </button>
            )}
          </div>
        </div>

        {/* Content grid */}
        <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
          {displayContent.map((item) => (
            <MediaCard
              key={`${item.mediaType}-${item.id}`}
              content={item}
            />
          ))}
        </div>

        {displayContent.length === 0 && (
          <p className="text-gray-500 text-center py-8">
            No {activeTab === 'recommendations' ? 'recommendations' : 'similar content'} found.
          </p>
        )}
      </div>
    </section>
  );
}
