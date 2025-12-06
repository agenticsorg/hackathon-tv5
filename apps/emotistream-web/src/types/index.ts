// Core type definitions for EmotiStream

export interface User {
  id: string;
  email: string;
  name: string;
  createdAt: string;
}

export interface EmotionAnalysis {
  emotionId: string;
  userId: string;
  rawText: string;
  detectedEmotion: string;
  confidence: number;
  valence: number;
  arousal: number;
  timestamp: string;
}

export interface ContentItem {
  contentId: string;
  title: string;
  type: 'video' | 'music' | 'article' | 'podcast';
  url: string;
  thumbnailUrl?: string;
  duration?: number;
  description?: string;
  tags: string[];
  emotionalProfile: {
    targetEmotion: string;
    valence: number;
    arousal: number;
  };
}

export interface Recommendation {
  recommendationId: string;
  userId: string;
  content: ContentItem;
  currentState: string;
  desiredState: string;
  confidence: number;
  qValue: number;
  timestamp: string;
}

export interface Feedback {
  feedbackId: string;
  userId: string;
  recommendationId: string;
  rating: number;
  wasHelpful: boolean;
  resultingEmotion?: string;
  comments?: string;
  timestamp: string;
}

export interface LearningProgress {
  userId: string;
  totalFeedback: number;
  averageRating: number;
  successRate: number;
  topTransitions: Array<{
    from: string;
    to: string;
    count: number;
    avgRating: number;
  }>;
  lastUpdated: string;
}

export interface Experience {
  experienceId: string;
  userId: string;
  state: string;
  action: string;
  reward: number;
  nextState: string;
  timestamp: string;
}
