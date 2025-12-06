/**
 * Content Profiler Type Definitions
 * EmotiStream Nexus - MVP Phase 4
 */

export interface ContentMetadata {
  contentId: string;
  title: string;
  description: string;
  platform: 'mock';
  genres: string[];
  category: 'movie' | 'series' | 'documentary' | 'music' | 'meditation' | 'short';
  tags: string[];
  duration: number; // minutes
}

export interface TargetState {
  currentValence: number;   // -1 to +1
  currentArousal: number;   // -1 to +1
  description: string;
}

export interface EmotionalContentProfile {
  contentId: string;
  primaryTone: string;
  valenceDelta: number;     // -1 to +1
  arousalDelta: number;     // -1 to +1
  intensity: number;        // 0 to 1
  complexity: number;       // 0 to 1
  targetStates: TargetState[];
  embeddingId: string;
  timestamp: number;
}

export interface SearchResult {
  contentId: string;
  title: string;
  similarityScore: number;
  profile: EmotionalContentProfile;
  metadata: ContentMetadata;
  relevanceReason: string;
}

export interface EmotionalState {
  valence: number;
  arousal: number;
  primaryEmotion: string;
  stressLevel: number;
  confidence: number;
  timestamp: number;
}
