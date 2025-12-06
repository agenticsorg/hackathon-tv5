import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import * as feedbackApi from '../api/feedback';
import type { SubmitFeedbackRequest } from '../api/feedback';

/**
 * Hook to submit feedback
 */
export const useSubmitFeedback = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: SubmitFeedbackRequest) => feedbackApi.submitFeedback(data),
    onSuccess: (_, variables) => {
      // Invalidate related queries
      queryClient.invalidateQueries({ queryKey: ['learning-progress', variables.userId] });
      queryClient.invalidateQueries({ queryKey: ['feedback-history', variables.userId] });
      queryClient.invalidateQueries({ queryKey: ['experiences', variables.userId] });
      queryClient.invalidateQueries({ queryKey: ['qtable', variables.userId] });
    },
  });
};

/**
 * Hook to get learning progress
 */
export const useLearningProgress = (userId: string) => {
  return useQuery({
    queryKey: ['learning-progress', userId],
    queryFn: () => feedbackApi.getLearningProgress(userId),
    enabled: !!userId,
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
};

/**
 * Hook to get Q-learning experiences
 */
export const useExperiences = (userId: string, limit = 50, offset = 0) => {
  return useQuery({
    queryKey: ['experiences', userId, limit, offset],
    queryFn: () => feedbackApi.getExperiences(userId, limit, offset),
    enabled: !!userId,
    staleTime: 2 * 60 * 1000, // 2 minutes
  });
};

/**
 * Hook to get feedback history
 */
export const useFeedbackHistory = (userId: string, limit = 20, offset = 0) => {
  return useQuery({
    queryKey: ['feedback-history', userId, limit, offset],
    queryFn: () => feedbackApi.getFeedbackHistory(userId, limit, offset),
    enabled: !!userId,
    staleTime: 2 * 60 * 1000, // 2 minutes
  });
};

/**
 * Hook to get Q-table for visualization
 */
export const useQTable = (userId: string) => {
  return useQuery({
    queryKey: ['qtable', userId],
    queryFn: () => feedbackApi.getQTable(userId),
    enabled: !!userId,
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
};
