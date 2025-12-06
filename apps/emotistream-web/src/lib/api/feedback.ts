import apiClient from './client';
import type { Feedback, LearningProgress, Experience } from '../../types';

// Backend response wrapper
interface BackendResponse<T> {
  success: boolean;
  data: T;
  error: { code: string; message: string } | null;
  timestamp: string;
}

// What the backend actually expects
export interface ContentFeedbackRequest {
  userId: string;
  contentId: string;
  actualPostState: {
    valence: number;
    arousal: number;
    stressLevel: number;
  };
  watchDuration: number; // in minutes
  completed: boolean;
  explicitRating?: number; // 1-5 stars
}

export interface ContentFeedbackResponse {
  reward: number;
  policyUpdated: boolean;
  newQValue: number;
  learningProgress: {
    totalExperiences: number;
    avgReward: number;
    explorationRate: number;
    convergenceScore: number;
  };
}

/**
 * Submit feedback after watching content (for RL learning)
 */
export const submitContentFeedback = async (
  data: ContentFeedbackRequest
): Promise<ContentFeedbackResponse> => {
  const response = await apiClient.post<BackendResponse<ContentFeedbackResponse>>(
    '/feedback',
    data
  );
  return response.data.data;
};

// Legacy types (keeping for compatibility)
export interface SubmitFeedbackRequest {
  userId: string;
  recommendationId: string;
  rating: number;
  wasHelpful: boolean;
  resultingEmotion?: string;
  comments?: string;
}

export interface SubmitFeedbackResponse {
  feedback: Feedback;
  rewardCalculated: number;
  qValueUpdated: boolean;
}

/**
 * Submit feedback for a recommendation (legacy)
 */
export const submitFeedback = async (
  data: SubmitFeedbackRequest
): Promise<SubmitFeedbackResponse> => {
  const response = await apiClient.post<SubmitFeedbackResponse>('/feedback', data);
  return response.data;
};

/**
 * Get learning progress for a user
 */
export const getLearningProgress = async (
  userId: string
): Promise<LearningProgress> => {
  const response = await apiClient.get<LearningProgress>(
    `/feedback/progress/${userId}`
  );
  return response.data;
};

/**
 * Get Q-learning experiences for a user
 */
export const getExperiences = async (
  userId: string,
  limit = 50,
  offset = 0
): Promise<{ experiences: Experience[]; total: number }> => {
  const response = await apiClient.get<{ experiences: Experience[]; total: number }>(
    `/feedback/experiences/${userId}`,
    {
      params: { limit, offset },
    }
  );
  return response.data;
};

/**
 * Get feedback history for a user
 */
export const getFeedbackHistory = async (
  userId: string,
  limit = 20,
  offset = 0
): Promise<{ feedback: Feedback[]; total: number }> => {
  const response = await apiClient.get<{ feedback: Feedback[]; total: number }>(
    `/feedback/history/${userId}`,
    {
      params: { limit, offset },
    }
  );
  return response.data;
};

/**
 * Get Q-table values for visualization
 */
export const getQTable = async (userId: string): Promise<Record<string, Record<string, number>>> => {
  const response = await apiClient.get<Record<string, Record<string, number>>>(
    `/feedback/qtable/${userId}`
  );
  return response.data;
};
