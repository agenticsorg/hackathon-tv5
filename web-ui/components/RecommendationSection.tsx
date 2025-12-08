'use client';

import RecommendationCard from './RecommendationCard';

export interface Recommendation {
  id: string;
  title: string;
  year: number;
  platform: string;
  poster: string;
  rating: number;
  confidence: 'Very High' | 'High' | 'Medium' | 'Low';
  genres: string[];
  reasoning: string;
  reviews: {
    source: string;
    score: number;
  }[];
  tags?: string[];
  socialProof?: string;
}

interface RecommendationSectionProps {
  title: string;
  subtitle: string;
  recommendations: Recommendation[];
  layout?: 'grid' | 'scroll';
}

export default function RecommendationSection({
  title,
  subtitle,
  recommendations,
  layout = 'grid',
}: RecommendationSectionProps) {
  return (
    <section className="py-8">
      {/* Section Header */}
      <div className="mb-6">
        <h2 className="text-2xl md:text-3xl font-bold text-text-dark-primary mb-2">
          {title}
        </h2>
        <p className="text-text-dark-secondary">{subtitle}</p>
      </div>

      {/* Recommendations */}
      {layout === 'grid' ? (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
          {recommendations.map((rec) => (
            <RecommendationCard key={rec.id} recommendation={rec} />
          ))}
        </div>
      ) : (
        <div className="flex overflow-x-auto space-x-4 pb-4 scrollbar-thin scrollbar-thumb-bg-dark-border scrollbar-track-bg-dark">
          {recommendations.map((rec) => (
            <div key={rec.id} className="flex-shrink-0 w-[280px]">
              <RecommendationCard recommendation={rec} />
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
