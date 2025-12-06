import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { Sparkles, ChevronLeft, ChevronRight } from 'lucide-react';
import { RecommendationCard } from './recommendation-card';
import { RecommendationSkeletonGrid } from './recommendation-skeleton';
import { RecommendationDetail } from './recommendation-detail';

interface Recommendation {
  contentId: string;
  title: string;
  category: string;
  duration: number;
  combinedScore: number;
  predictedOutcome: {
    expectedValence: number;
    expectedArousal: number;
    expectedStress: number;
    confidence: number;
  };
  reasoning: string;
  isExploration: boolean;
  qValueHistory?: Array<{
    timestamp: number;
    qValue: number;
    reward?: number;
  }>;
}

interface RecommendationGridProps {
  recommendations?: Recommendation[];
  isLoading?: boolean;
  error?: string | null;
  currentState?: {
    valence: number;
    arousal: number;
    stress: number;
  };
  onWatch: (contentId: string) => void;
  onSave?: (contentId: string) => void;
}

export function RecommendationGrid({
  recommendations = [],
  isLoading = false,
  error = null,
  currentState,
  onWatch,
  onSave,
}: RecommendationGridProps) {
  const [selectedRecommendation, setSelectedRecommendation] = useState<Recommendation | null>(null);
  const scrollContainerRef = React.useRef<HTMLDivElement>(null);

  const scroll = (direction: 'left' | 'right') => {
    if (scrollContainerRef.current) {
      const scrollAmount = 300;
      const currentScroll = scrollContainerRef.current.scrollLeft;
      const targetScroll = direction === 'left'
        ? currentScroll - scrollAmount
        : currentScroll + scrollAmount;

      scrollContainerRef.current.scrollTo({
        left: targetScroll,
        behavior: 'smooth',
      });
    }
  };

  // Loading state
  if (isLoading) {
    return (
      <div className="space-y-4">
        <div className="flex items-center gap-2">
          <Sparkles className="w-5 h-5 text-purple-400 animate-pulse" />
          <h2 className="text-xl font-bold text-white">Finding Perfect Content...</h2>
        </div>
        <RecommendationSkeletonGrid count={5} />
      </div>
    );
  }

  // Error state
  if (error) {
    return (
      <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-6 text-center">
        <p className="text-red-400">{error}</p>
      </div>
    );
  }

  // Empty state
  if (recommendations.length === 0) {
    return (
      <div className="bg-gray-800/50 border border-gray-700 rounded-lg p-12 text-center">
        <Sparkles className="w-12 h-12 text-gray-600 mx-auto mb-4" />
        <h3 className="text-lg font-semibold text-white mb-2">
          Ready to Discover Content
        </h3>
        <p className="text-gray-400 max-w-md mx-auto">
          Describe your current mood and desired emotional state to get personalized recommendations
          powered by reinforcement learning.
        </p>
      </div>
    );
  }

  return (
    <>
      <div className="space-y-4">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Sparkles className="w-5 h-5 text-purple-400" />
            <h2 className="text-xl font-bold text-white">Personalized For You</h2>
            <span className="text-sm text-gray-400">
              ({recommendations.length} recommendations)
            </span>
          </div>

          {/* Scroll Controls - Desktop Only */}
          <div className="hidden md:flex gap-2">
            <button
              onClick={() => scroll('left')}
              className="p-2 bg-gray-800 hover:bg-gray-700 rounded-full transition-colors"
              aria-label="Scroll left"
            >
              <ChevronLeft className="w-5 h-5" />
            </button>
            <button
              onClick={() => scroll('right')}
              className="p-2 bg-gray-800 hover:bg-gray-700 rounded-full transition-colors"
              aria-label="Scroll right"
            >
              <ChevronRight className="w-5 h-5" />
            </button>
          </div>
        </div>

        {/* Horizontal Scrolling Grid */}
        <div
          ref={scrollContainerRef}
          className="flex gap-4 overflow-x-auto pb-4 scrollbar-thin scrollbar-thumb-gray-700 scrollbar-track-transparent scroll-smooth"
          style={{
            scrollbarWidth: 'thin',
            scrollbarColor: '#374151 transparent',
          }}
        >
          {recommendations.map((recommendation, index) => (
            <motion.div
              key={recommendation.contentId}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: index * 0.05, duration: 0.3 }}
            >
              <RecommendationCard
                {...recommendation}
                currentState={currentState}
                onWatch={() => onWatch(recommendation.contentId)}
                onDetails={() => setSelectedRecommendation(recommendation)}
              />
            </motion.div>
          ))}
        </div>

        {/* Mobile Swipe Hint */}
        <div className="md:hidden text-center text-xs text-gray-500">
          ← Swipe to explore more →
        </div>
      </div>

      {/* Detail Modal */}
      {selectedRecommendation && (
        <RecommendationDetail
          isOpen={!!selectedRecommendation}
          onClose={() => setSelectedRecommendation(null)}
          recommendation={selectedRecommendation}
          currentState={currentState}
          onWatch={() => {
            onWatch(selectedRecommendation.contentId);
            setSelectedRecommendation(null);
          }}
          onSave={onSave ? () => {
            onSave(selectedRecommendation.contentId);
            setSelectedRecommendation(null);
          } : undefined}
        />
      )}
    </>
  );
}
