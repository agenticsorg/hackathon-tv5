import apiClient from './client';
import type { Recommendation, ContentItem } from '../../types';

export interface GetRecommendationsRequest {
  userId: string;
  currentState: {
    valence: number;
    arousal: number;
    stressLevel: number;
  };
  desiredState: {
    valence: number;
    arousal: number;
    stressLevel: number;
  };
  limit?: number;
}

// What the frontend components expect
export interface RecommendationItem {
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
}

export interface GetRecommendationsResponse {
  recommendations: RecommendationItem[];
  exploration: {
    epsilon: number;
    wasRandom: boolean;
  };
}

export interface RecommendationHistoryResponse {
  recommendations: Recommendation[];
  total: number;
}

// Backend response format
interface BackendRecommendation {
  contentId: string;
  title: string;
  qValue: number;
  similarityScore: number;
  combinedScore: number;
  predictedOutcome: {
    expectedValence: number;
    expectedArousal: number;
    expectedStress: number;
    confidence: number;
  };
  reasoning: string;
  isExploration: boolean;
}

interface BackendRecommendResponse {
  success: boolean;
  data: {
    userId: string;
    recommendations: BackendRecommendation[];
    explorationRate: number;
    timestamp: number;
  };
  error: null | { code: string; message: string };
  timestamp: string;
}

/**
 * Get content recommendations based on current and desired emotional state
 */
export const getRecommendations = async (
  data: GetRecommendationsRequest
): Promise<GetRecommendationsResponse> => {
  const response = await apiClient.post<BackendRecommendResponse>(
    '/recommend',
    data
  );

  const backendData = response.data.data;

  // Transform backend response to frontend format
  return {
    recommendations: backendData.recommendations.map(rec => ({
      contentId: rec.contentId,
      title: rec.title,
      category: getCategoryFromContentId(rec.contentId),
      duration: getDurationFromContentId(rec.contentId),
      combinedScore: rec.combinedScore,
      predictedOutcome: rec.predictedOutcome,
      reasoning: rec.reasoning,
      isExploration: rec.isExploration,
    })),
    exploration: {
      epsilon: backendData.explorationRate,
      wasRandom: backendData.recommendations.some(r => r.isExploration),
    },
  };
};

// Helper to derive category from contentId
function getCategoryFromContentId(contentId: string): string {
  const prefix = contentId.split('-')[0];
  const categoryMap: Record<string, string> = {
    peaceful: 'Wellness',
    short: 'Entertainment',
    uplifting: 'Comedy',
    drama: 'Drama',
    exciting: 'Action',
    thriller: 'Thriller',
    calming: 'Relaxation',
    energizing: 'Fitness',
  };
  return categoryMap[prefix] || 'Content';
}

// Helper to derive duration from contentId
function getDurationFromContentId(contentId: string): number {
  const prefix = contentId.split('-')[0];
  const durationMap: Record<string, number> = {
    short: 5,
    peaceful: 30,
    uplifting: 45,
    drama: 60,
    exciting: 120,
    thriller: 90,
    calming: 20,
    energizing: 15,
  };
  return durationMap[prefix] || 30;
}

/**
 * Get recommendation history for a user
 */
export const getRecommendationHistory = async (
  userId: string,
  limit = 20,
  offset = 0
): Promise<RecommendationHistoryResponse> => {
  const response = await apiClient.get<RecommendationHistoryResponse>(
    `/recommend/history/${userId}`,
    {
      params: { limit, offset },
    }
  );
  return response.data;
};

/**
 * Get content by ID
 */
export const getContent = async (contentId: string): Promise<ContentItem> => {
  const response = await apiClient.get<ContentItem>(`/content/${contentId}`);
  return response.data;
};

/**
 * Search content by tags or emotion
 */
export const searchContent = async (
  query: string,
  filters?: {
    type?: string[];
    emotion?: string;
    minValence?: number;
    maxValence?: number;
  }
): Promise<ContentItem[]> => {
  const response = await apiClient.get<ContentItem[]>('/content/search', {
    params: { q: query, ...filters },
  });
  return response.data;
};
