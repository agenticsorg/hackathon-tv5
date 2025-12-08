import apiClient from './client';

// Backend response wrapper
interface BackendResponse<T> {
  success: boolean;
  data: T;
  error: { code: string; message: string } | null;
  timestamp: string;
}

// Progress data types
export interface ProgressData {
  totalExperiences: number;
  completedContent: number;
  completionRate: string;
  averageReward: string;
  rewardTrend: 'improving' | 'stable' | 'declining';
  recentRewards: number[];
  explorationRate: string;
  explorationCount: number;
  exploitationCount: number;
  convergence: {
    score: number;
    stage: 'exploring' | 'learning' | 'confident';
    percentage: number;
  };
  summary: {
    level: string;
    description: string;
    nextMilestone: { count: number; description: string } | null;
  };
  timestamp: string;
}

export interface ConvergenceData {
  score: number;
  stage: string;
  explanation: string;
  metrics: {
    qValueStability: string;
    rewardVariance: string;
    explorationRate: string;
    policyChanges: number;
  };
  recommendations: string[];
  progressBar: {
    percentage: number;
    color: string;
    label: string;
  };
}

export interface JourneyPoint {
  experienceNumber: number;
  timestamp: string;
  contentId: string;
  contentTitle: string;
  emotionBefore: { valence: number; arousal: number; stress: number };
  emotionAfter: { valence: number; arousal: number; stress: number };
  delta: { valence: number; arousal: number; stress: number };
  reward: number;
  completed: boolean;
  quadrant: string;
}

export interface RewardTimelinePoint {
  experienceNumber: number;
  timestamp: string;
  reward: number;
  contentTitle: string;
  contentId: string;
  completed: boolean;
  starRating: number;
}

export interface ContentPerformance {
  contentId: string;
  contentTitle: string;
  timesWatched: number;
  averageReward: string;
  completionRate: string;
  averageRating: string;
  lastWatched: string;
}

export interface Experience {
  experienceId: string;
  experienceNumber: number;
  timestamp: string;
  contentId: string;
  contentTitle: string;
  emotionChange: {
    before: { valence: number; arousal: number; stressLevel: number };
    after: { valence: number; arousal: number; stressLevel: number };
    delta: { valence: number; arousal: number; stress: number };
    improvement: number;
  };
  reward: number;
  starRating: number;
  completed: boolean;
  watchDuration: number;
  completionPercentage: string;
}

/**
 * Get comprehensive learning progress for user
 */
export const getProgress = async (userId: string): Promise<ProgressData> => {
  const response = await apiClient.get<BackendResponse<{ progress: ProgressData }>>(
    `/progress/${userId}`
  );
  return response.data.data.progress;
};

/**
 * Get detailed convergence analysis
 */
export const getConvergence = async (userId: string): Promise<ConvergenceData> => {
  const response = await apiClient.get<BackendResponse<{ convergence: ConvergenceData }>>(
    `/progress/${userId}/convergence`
  );
  return response.data.data.convergence;
};

/**
 * Get emotional journey visualization data
 */
export const getJourney = async (userId: string, limit?: number): Promise<{ journey: JourneyPoint[]; totalPoints: number }> => {
  const response = await apiClient.get<BackendResponse<{ journey: JourneyPoint[]; totalPoints: number }>>(
    `/progress/${userId}/journey`,
    { params: limit ? { limit } : undefined }
  );
  return response.data.data;
};

/**
 * Get reward timeline data
 */
export const getRewardTimeline = async (userId: string): Promise<{
  timeline: RewardTimelinePoint[];
  trendLine: number[];
  statistics: { average: number; highest: number; lowest: number; recent: number[] };
}> => {
  const response = await apiClient.get<BackendResponse<{
    timeline: RewardTimelinePoint[];
    trendLine: number[];
    statistics: { average: number; highest: number; lowest: number; recent: number[] };
  }>>(`/progress/${userId}/rewards`);
  return response.data.data;
};

/**
 * Get content performance rankings
 */
export const getContentPerformance = async (userId: string): Promise<{
  bestContent: ContentPerformance[];
  worstContent: ContentPerformance[];
}> => {
  const response = await apiClient.get<BackendResponse<{
    bestContent: ContentPerformance[];
    worstContent: ContentPerformance[];
  }>>(`/progress/${userId}/content`);
  return response.data.data;
};

/**
 * Get recent experiences list
 */
export const getExperiences = async (userId: string, limit = 10): Promise<{
  experiences: Experience[];
  total: number;
  showing: number;
}> => {
  const response = await apiClient.get<BackendResponse<{
    experiences: Experience[];
    total: number;
    showing: number;
  }>>(`/progress/${userId}/experiences`, { params: { limit } });
  return response.data.data;
};
