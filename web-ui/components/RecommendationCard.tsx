'use client';

import { useState } from 'react';
import type { Recommendation } from './RecommendationSection';

interface RecommendationCardProps {
  recommendation: Recommendation;
}

export default function RecommendationCard({ recommendation }: RecommendationCardProps) {
  const [showReasoning, setShowReasoning] = useState(false);

  const getConfidenceColor = (confidence: Recommendation['confidence']) => {
    switch (confidence) {
      case 'Very High':
        return 'bg-semantic-success text-white';
      case 'High':
        return 'bg-semantic-info text-white';
      case 'Medium':
        return 'bg-semantic-warning text-white';
      case 'Low':
        return 'bg-text-dark-secondary text-white';
    }
  };

  return (
    <div className="bg-bg-dark-card rounded-card overflow-hidden shadow-card hover:shadow-card-hover transition-all group">
      {/* Poster Image */}
      <div className="relative aspect-[2/3] overflow-hidden bg-bg-dark-border">
        <img
          src={recommendation.poster}
          alt={recommendation.title}
          className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
        />

        {/* Confidence Badge */}
        <div className={`absolute top-3 right-3 px-3 py-1 rounded-pill text-xs font-semibold ${getConfidenceColor(recommendation.confidence)}`}>
          {recommendation.confidence}
        </div>

        {/* Platform Badge */}
        <div className="absolute top-3 left-3 px-3 py-1 rounded-pill text-xs font-semibold bg-black/70 text-white backdrop-blur-sm">
          {recommendation.platform}
        </div>

        {/* Tags */}
        {recommendation.tags && recommendation.tags.length > 0 && (
          <div className="absolute bottom-3 left-3 flex gap-2">
            {recommendation.tags.map((tag, idx) => (
              <span
                key={idx}
                className="px-2 py-1 rounded text-xs font-medium bg-youtube-red text-white"
              >
                {tag}
              </span>
            ))}
          </div>
        )}
      </div>

      {/* Content */}
      <div className="p-4">
        {/* Title */}
        <h3 className="text-lg font-bold text-text-dark-primary mb-1 line-clamp-2">
          {recommendation.title}
        </h3>

        {/* Year & Genres */}
        <div className="flex items-center text-sm text-text-dark-secondary mb-3">
          <span>{recommendation.year}</span>
          <span className="mx-2">•</span>
          <span className="line-clamp-1">{recommendation.genres.join(', ')}</span>
        </div>

        {/* Reviews */}
        <div className="flex items-center gap-3 mb-3">
          {recommendation.reviews.map((review, idx) => (
            <div key={idx} className="flex items-center">
              <span className="text-xs text-text-dark-secondary mr-1">
                {review.source}
              </span>
              <div className="flex items-center">
                <svg className="w-4 h-4 text-semantic-warning mr-1" fill="currentColor" viewBox="0 0 20 20">
                  <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                </svg>
                <span className="text-sm font-semibold text-text-dark-primary">
                  {review.score.toFixed(1)}
                </span>
              </div>
            </div>
          ))}
        </div>

        {/* Social Proof */}
        {recommendation.socialProof && (
          <p className="text-xs text-text-dark-secondary mb-3 flex items-center">
            <svg className="w-4 h-4 mr-1" fill="currentColor" viewBox="0 0 20 20">
              <path d="M9 6a3 3 0 11-6 0 3 3 0 016 0zM17 6a3 3 0 11-6 0 3 3 0 016 0zM12.93 17c.046-.327.07-.66.07-1a6.97 6.97 0 00-1.5-4.33A5 5 0 0119 16v1h-6.07zM6 11a5 5 0 015 5v1H1v-1a5 5 0 015-5z" />
            </svg>
            {recommendation.socialProof}
          </p>
        )}

        {/* Why This? Button */}
        <button
          onClick={() => setShowReasoning(!showReasoning)}
          className="w-full mt-2 px-4 py-2 bg-youtube-red hover:bg-youtube-red-dark text-white
                   rounded-lg text-sm font-medium transition-colors flex items-center justify-center"
        >
          <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2}
                  d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          Why this?
        </button>

        {/* Reasoning Panel */}
        {showReasoning && (
          <div className="mt-3 p-3 bg-bg-dark rounded-lg border border-bg-dark-border">
            <p className="text-sm text-text-dark-secondary leading-relaxed">
              {recommendation.reasoning}
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
