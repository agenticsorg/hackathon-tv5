import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useEmotionStore } from '../stores/emotion-store';
import * as emotionApi from '../api/emotion';
import type { AnalyzeEmotionRequest } from '../api/emotion';

/**
 * Hook to analyze emotion from text
 */
export const useAnalyzeEmotion = () => {
  const { setCurrentEmotion, addToHistory } = useEmotionStore();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: AnalyzeEmotionRequest) => emotionApi.analyzeEmotion(data),
    onSuccess: (response) => {
      setCurrentEmotion(response.analysis);
      addToHistory(response.analysis);
      queryClient.invalidateQueries({ queryKey: ['emotion-history'] });
    },
  });
};

/**
 * Hook to get emotion history
 */
export const useEmotionHistory = (userId: string, limit = 20, offset = 0) => {
  return useQuery({
    queryKey: ['emotion-history', userId, limit, offset],
    queryFn: () => emotionApi.getEmotionHistory(userId, limit, offset),
    enabled: !!userId,
    staleTime: 2 * 60 * 1000, // 2 minutes
  });
};

/**
 * Hook to get latest emotion
 */
export const useLatestEmotion = (userId: string) => {
  return useQuery({
    queryKey: ['emotion-latest', userId],
    queryFn: () => emotionApi.getLatestEmotion(userId),
    enabled: !!userId,
    staleTime: 1 * 60 * 1000, // 1 minute
  });
};

/**
 * Hook to delete emotion
 */
export const useDeleteEmotion = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (emotionId: string) => emotionApi.deleteEmotion(emotionId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['emotion-history'] });
      queryClient.invalidateQueries({ queryKey: ['emotion-latest'] });
    },
  });
};
