/**
 * Hook for fetching and managing recommendations
 */

import { useState, useEffect, useCallback } from 'react';
import type { Recommendation, EmotionalState } from '@/components/recommendations/types';

interface UseRecommendationsOptions {
  currentState?: EmotionalState;
  desiredState?: EmotionalState;
  userId?: string;
  autoFetch?: boolean;
}

interface UseRecommendationsReturn {
  recommendations: Recommendation[];
  isLoading: boolean;
  error: string | null;
  fetchRecommendations: () => Promise<void>;
  refresh: () => Promise<void>;
}

export function useRecommendations({
  currentState,
  desiredState,
  userId,
  autoFetch = false,
}: UseRecommendationsOptions = {}): UseRecommendationsReturn {
  const [recommendations, setRecommendations] = useState<Recommendation[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchRecommendations = useCallback(async () => {
    if (!currentState || !desiredState) {
      setError('Current and desired emotional states are required');
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const response = await fetch('/api/recommend', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          userId: userId || 'anonymous',
          currentState,
          desiredState,
          limit: 10,
        }),
      });

      if (!response.ok) {
        throw new Error(`Failed to fetch recommendations: ${response.statusText}`);
      }

      const data = await response.json();
      setRecommendations(data.recommendations || []);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch recommendations';
      setError(errorMessage);
      console.error('Error fetching recommendations:', err);
    } finally {
      setIsLoading(false);
    }
  }, [currentState, desiredState, userId]);

  const refresh = useCallback(async () => {
    await fetchRecommendations();
  }, [fetchRecommendations]);

  // Auto-fetch on mount if enabled and states are available
  useEffect(() => {
    if (autoFetch && currentState && desiredState) {
      fetchRecommendations();
    }
  }, [autoFetch, currentState, desiredState, fetchRecommendations]);

  return {
    recommendations,
    isLoading,
    error,
    fetchRecommendations,
    refresh,
  };
}
