import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useRecommendationStore } from '../stores/recommendation-store';
import * as recommendApi from '../api/recommend';
import type { GetRecommendationsRequest } from '../api/recommend';

/**
 * Hook to get content recommendations
 */
export const useRecommendations = () => {
  const { setRecommendations } = useRecommendationStore();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: GetRecommendationsRequest) => recommendApi.getRecommendations(data),
    onSuccess: (response) => {
      setRecommendations(response.recommendations);
      queryClient.invalidateQueries({ queryKey: ['recommendation-history'] });
    },
  });
};

/**
 * Hook to get recommendation history
 */
export const useRecommendationHistory = (userId: string, limit = 20, offset = 0) => {
  return useQuery({
    queryKey: ['recommendation-history', userId, limit, offset],
    queryFn: () => recommendApi.getRecommendationHistory(userId, limit, offset),
    enabled: !!userId,
    staleTime: 2 * 60 * 1000, // 2 minutes
  });
};

/**
 * Hook to get content by ID
 */
export const useContent = (contentId: string) => {
  return useQuery({
    queryKey: ['content', contentId],
    queryFn: () => recommendApi.getContent(contentId),
    enabled: !!contentId,
    staleTime: 10 * 60 * 1000, // 10 minutes
  });
};

/**
 * Hook to search content
 */
export const useSearchContent = (
  query: string,
  filters?: {
    type?: string[];
    emotion?: string;
    minValence?: number;
    maxValence?: number;
  }
) => {
  return useQuery({
    queryKey: ['content-search', query, filters],
    queryFn: () => recommendApi.searchContent(query, filters),
    enabled: !!query,
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
};
