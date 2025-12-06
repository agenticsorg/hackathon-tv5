import apiClient from './client';
import type { EmotionAnalysis } from '../../types';

export interface AnalyzeEmotionRequest {
  userId: string;
  text: string;
}

// What the dashboard expects
export interface AnalyzeEmotionResponse {
  emotionalState: {
    valence: number;
    arousal: number;
    stressLevel: number;
    primaryEmotion: string;
    confidence: number;
  };
  analysis: EmotionAnalysis;
}

export interface EmotionHistoryResponse {
  emotions: EmotionAnalysis[];
  total: number;
}

// Backend response format
interface BackendEmotionResponse {
  success: boolean;
  data: {
    userId: string;
    inputText: string;
    state: {
      valence: number;
      arousal: number;
      stressLevel: number;
      primaryEmotion: string;
      confidence: number;
      emotionVector: Record<string, number>;
      timestamp: number;
    };
    desired: {
      targetValence: number;
      targetArousal: number;
      targetStress: number;
      intensity: string;
      reasoning: string;
    };
  };
  error: null | { code: string; message: string };
  timestamp: string;
}

/**
 * Analyze emotion from text using Gemini AI
 */
export const analyzeEmotion = async (
  data: AnalyzeEmotionRequest
): Promise<AnalyzeEmotionResponse> => {
  const response = await apiClient.post<BackendEmotionResponse>('/emotion/analyze', data);
  const backendData = response.data.data;

  // Transform backend response to frontend format
  return {
    emotionalState: {
      valence: backendData.state.valence,
      arousal: backendData.state.arousal,
      stressLevel: backendData.state.stressLevel,
      primaryEmotion: backendData.state.primaryEmotion,
      confidence: backendData.state.confidence,
    },
    analysis: {
      emotionId: `emotion-${Date.now()}`,
      userId: backendData.userId,
      rawText: backendData.inputText,
      detectedEmotion: backendData.state.primaryEmotion,
      confidence: backendData.state.confidence,
      valence: backendData.state.valence,
      arousal: backendData.state.arousal,
      timestamp: new Date(backendData.state.timestamp).toISOString(),
    },
  };
};

/**
 * Get emotion history for a user
 */
export const getEmotionHistory = async (
  userId: string,
  limit = 20,
  offset = 0
): Promise<EmotionHistoryResponse> => {
  const response = await apiClient.get<EmotionHistoryResponse>(
    `/emotion/history/${userId}`,
    {
      params: { limit, offset },
    }
  );
  return response.data;
};

/**
 * Get latest emotion analysis for a user
 */
export const getLatestEmotion = async (userId: string): Promise<EmotionAnalysis | null> => {
  const response = await getEmotionHistory(userId, 1);
  return response.emotions[0] || null;
};

/**
 * Delete emotion analysis
 */
export const deleteEmotion = async (emotionId: string): Promise<void> => {
  await apiClient.delete(`/emotion/${emotionId}`);
};
