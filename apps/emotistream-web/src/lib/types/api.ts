// EmotiStream API Types

// Authentication
export interface LoginRequest {
  email: string;
  password: string;
}

export interface RegisterRequest {
  username: string;
  email: string;
  password: string;
}

export interface AuthResponse {
  token: string;
  user: User;
}

export interface User {
  id: string;
  username: string;
  email: string;
  createdAt: string;
}

// Emotion Detection
export interface EmotionDetectionRequest {
  text?: string;
  imageUrl?: string;
}

export interface EmotionResponse {
  emotions: {
    joy: number;
    sadness: number;
    anger: number;
    fear: number;
    surprise: number;
    neutral: number;
  };
  dominantEmotion: string;
  confidence: number;
  timestamp: string;
}

// Recommendations
export interface RecommendationRequest {
  emotion: string;
  context?: string;
}

export interface Recommendation {
  id: string;
  type: 'music' | 'video' | 'article' | 'activity';
  title: string;
  description: string;
  url?: string;
  thumbnailUrl?: string;
  relevanceScore: number;
  reason: string;
}

export interface RecommendationResponse {
  recommendations: Recommendation[];
  emotion: string;
  generatedAt: string;
}

// Feedback
export interface FeedbackRequest {
  recommendationId: string;
  rating: number; // 1-5
  helpful: boolean;
  comment?: string;
}

export interface FeedbackResponse {
  success: boolean;
  message: string;
}

// Progress/Analytics
export interface ProgressResponse {
  emotionHistory: {
    date: string;
    emotions: Record<string, number>;
  }[];
  topEmotions: {
    emotion: string;
    count: number;
    percentage: number;
  }[];
  totalSessions: number;
  averageMood: number;
  trends: {
    improving: string[];
    declining: string[];
  };
}

// API Error Response
export interface ApiError {
  error: string;
  message: string;
  statusCode: number;
}
