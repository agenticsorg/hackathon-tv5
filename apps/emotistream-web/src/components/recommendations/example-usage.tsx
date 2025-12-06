/**
 * Example usage of recommendation components
 * This file demonstrates integration patterns
 */

import React, { useState } from 'react';
import { RecommendationGrid } from './recommendation-grid';
import { useRecommendations } from '@/hooks/use-recommendations';
import type { EmotionalState } from './types';

// Example 1: Basic usage with mock data
export function BasicExample() {
  const mockRecommendations = [
    {
      contentId: 'meditation-1',
      title: 'Calm Morning Meditation',
      category: 'meditation',
      duration: 15,
      combinedScore: 0.92,
      predictedOutcome: {
        expectedValence: 0.8,
        expectedArousal: 0.4,
        expectedStress: 0.2,
        confidence: 0.88,
      },
      reasoning: 'Based on your high stress levels, this guided meditation will help you relax and find inner calm.',
      isExploration: false,
    },
    {
      contentId: 'music-1',
      title: 'Upbeat Focus Playlist',
      category: 'music',
      duration: 60,
      combinedScore: 0.85,
      predictedOutcome: {
        expectedValence: 0.7,
        expectedArousal: 0.6,
        expectedStress: 0.3,
        confidence: 0.82,
      },
      reasoning: 'Energizing music to boost your mood while maintaining focus.',
      isExploration: false,
    },
    {
      contentId: 'movie-1',
      title: 'Feel-Good Comedy',
      category: 'movie',
      duration: 90,
      combinedScore: 0.78,
      predictedOutcome: {
        expectedValence: 0.85,
        expectedArousal: 0.5,
        expectedStress: 0.1,
        confidence: 0.75,
      },
      reasoning: 'Laughter is the best medicine - this comedy will lift your spirits.',
      isExploration: true,
    },
  ];

  const currentState: EmotionalState = {
    valence: 0.3,
    arousal: 0.7,
    stress: 0.8,
  };

  const handleWatch = (contentId: string) => {
    console.log('Watch content:', contentId);
    // Navigate to player
  };

  return (
    <div className="max-w-7xl mx-auto p-8">
      <h1 className="text-3xl font-bold text-white mb-8">
        Recommendations for You
      </h1>

      <RecommendationGrid
        recommendations={mockRecommendations}
        currentState={currentState}
        onWatch={handleWatch}
      />
    </div>
  );
}

// Example 2: With API integration
export function ApiIntegrationExample() {
  const [currentState, setCurrentState] = useState<EmotionalState>({
    valence: 0.3,
    arousal: 0.7,
    stress: 0.8,
  });

  const [desiredState, setDesiredState] = useState<EmotionalState>({
    valence: 0.8,
    arousal: 0.5,
    stress: 0.2,
  });

  const {
    recommendations,
    isLoading,
    error,
    fetchRecommendations,
  } = useRecommendations({
    currentState,
    desiredState,
    userId: 'user123',
  });

  const handleWatch = (contentId: string) => {
    console.log('Watch content:', contentId);
    // Track analytics
    // Navigate to player
  };

  const handleSave = (contentId: string) => {
    console.log('Save content:', contentId);
    // Add to watchlist
  };

  return (
    <div className="max-w-7xl mx-auto p-8 space-y-8">
      {/* Emotion Input Section */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-bold text-white mb-4">
          How are you feeling?
        </h2>
        {/* Emotion input components would go here */}
        <button
          onClick={fetchRecommendations}
          className="mt-4 px-6 py-3 bg-purple-600 hover:bg-purple-700 text-white rounded-lg font-medium"
        >
          Get Recommendations
        </button>
      </div>

      {/* Recommendations Section */}
      <div>
        <RecommendationGrid
          recommendations={recommendations}
          isLoading={isLoading}
          error={error}
          currentState={currentState}
          onWatch={handleWatch}
          onSave={handleSave}
        />
      </div>
    </div>
  );
}

// Example 3: With auto-fetch on emotion change
export function AutoFetchExample() {
  const [emotion, setEmotion] = useState<EmotionalState | null>(null);
  const [desired, setDesired] = useState<EmotionalState | null>(null);

  const {
    recommendations,
    isLoading,
    error,
  } = useRecommendations({
    currentState: emotion || undefined,
    desiredState: desired || undefined,
    userId: 'user123',
    autoFetch: true, // Auto-fetch when states change
  });

  const handleEmotionAnalyzed = (analyzed: EmotionalState) => {
    setEmotion(analyzed);
  };

  const handleDesiredStateSelected = (selected: EmotionalState) => {
    setDesired(selected);
  };

  const handleWatch = (contentId: string) => {
    // Record interaction for RL feedback
    recordInteraction({
      contentId,
      userId: 'user123',
      action: 'watch',
      timestamp: Date.now(),
    });
  };

  return (
    <div className="max-w-7xl mx-auto p-8 space-y-8">
      {/* Step 1: Emotion Analysis */}
      <div>
        <h2 className="text-xl font-bold text-white mb-4">
          Step 1: Describe your current mood
        </h2>
        {/* EmotionInput component */}
      </div>

      {/* Step 2: Desired State */}
      {emotion && (
        <div>
          <h2 className="text-xl font-bold text-white mb-4">
            Step 2: How do you want to feel?
          </h2>
          {/* DesiredStateSelector component */}
        </div>
      )}

      {/* Step 3: Recommendations (auto-appears) */}
      {emotion && desired && (
        <div>
          <h2 className="text-xl font-bold text-white mb-4">
            Step 3: Personalized for you
          </h2>
          <RecommendationGrid
            recommendations={recommendations}
            isLoading={isLoading}
            error={error}
            currentState={emotion}
            onWatch={handleWatch}
          />
        </div>
      )}
    </div>
  );
}

// Example 4: Loading states
export function LoadingStateExample() {
  return (
    <div className="max-w-7xl mx-auto p-8">
      <h1 className="text-3xl font-bold text-white mb-8">
        Finding perfect content...
      </h1>

      <RecommendationGrid
        recommendations={[]}
        isLoading={true}
        currentState={{ valence: 0.5, arousal: 0.5, stress: 0.5 }}
        onWatch={() => {}}
      />
    </div>
  );
}

// Example 5: Empty state
export function EmptyStateExample() {
  return (
    <div className="max-w-7xl mx-auto p-8">
      <RecommendationGrid
        recommendations={[]}
        isLoading={false}
        currentState={{ valence: 0.5, arousal: 0.5, stress: 0.5 }}
        onWatch={() => {}}
      />
    </div>
  );
}

// Example 6: Error state
export function ErrorStateExample() {
  return (
    <div className="max-w-7xl mx-auto p-8">
      <RecommendationGrid
        recommendations={[]}
        isLoading={false}
        error="Failed to fetch recommendations. Please try again."
        currentState={{ valence: 0.5, arousal: 0.5, stress: 0.5 }}
        onWatch={() => {}}
      />
    </div>
  );
}

// Helper function (would be in separate utils file)
function recordInteraction(data: {
  contentId: string;
  userId: string;
  action: string;
  timestamp: number;
}) {
  fetch('/api/feedback', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
}
