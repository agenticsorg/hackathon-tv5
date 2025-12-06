/**
 * Type definitions for recommendation components
 */

export interface PredictedOutcome {
  expectedValence: number;
  expectedArousal: number;
  expectedStress: number;
  confidence: number;
}

export interface QValueHistoryEntry {
  timestamp: number;
  qValue: number;
  reward?: number;
}

export interface Recommendation {
  contentId: string;
  title: string;
  category: string;
  duration: number;
  combinedScore: number;
  predictedOutcome: PredictedOutcome;
  reasoning: string;
  isExploration: boolean;
  qValueHistory?: QValueHistoryEntry[];
}

export interface EmotionalState {
  valence: number;
  arousal: number;
  stress: number;
}

export interface RecommendationGridProps {
  recommendations?: Recommendation[];
  isLoading?: boolean;
  error?: string | null;
  currentState?: EmotionalState;
  onWatch: (contentId: string) => void;
  onSave?: (contentId: string) => void;
}

export interface RecommendationCardProps {
  contentId: string;
  title: string;
  category: string;
  duration: number;
  combinedScore: number;
  predictedOutcome: PredictedOutcome;
  reasoning: string;
  isExploration: boolean;
  onWatch: () => void;
  onDetails?: () => void;
  currentState?: EmotionalState;
}

export interface RecommendationDetailProps {
  isOpen: boolean;
  onClose: () => void;
  recommendation: Recommendation;
  currentState?: EmotionalState;
  onWatch: () => void;
  onSave?: () => void;
}

export interface OutcomePredictorProps {
  currentState: EmotionalState;
  predictedOutcome: PredictedOutcome;
  compact?: boolean;
}
