'use client';

import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { EmotionInput, MoodRing, EmotionStateCard, DesiredStateSelector } from '@/components/emotion';
import { RecommendationGrid } from '@/components/recommendations';
import { FeedbackModal } from '@/components/feedback/feedback-modal';
import { useEmotionStore } from '@/lib/stores/emotion-store';
import { analyzeEmotion } from '@/lib/api/emotion';
import { getRecommendations } from '@/lib/api/recommend';
import { useAuthStore } from '@/lib/stores/auth-store';
import type { ContentFeedbackResponse } from '@/lib/api/feedback';
import type { EmotionState, DesiredState } from '@/types';

interface Recommendation {
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

export default function DashboardPage() {
  const { user } = useAuthStore();
  const { currentEmotion, setCurrentEmotion, desiredState, setDesiredState } = useEmotionStore();

  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [recommendations, setRecommendations] = useState<Recommendation[]>([]);
  const [isLoadingRecs, setIsLoadingRecs] = useState(false);

  // Feedback modal state
  const [feedbackModal, setFeedbackModal] = useState<{
    isOpen: boolean;
    contentId: string;
    contentTitle: string;
  }>({ isOpen: false, contentId: '', contentTitle: '' });

  const handleAnalyze = async (text: string) => {
    if (!user) return;

    setIsAnalyzing(true);
    setError(null);

    try {
      const result = await analyzeEmotion({ text, userId: user.id });

      const emotion: EmotionState = {
        valence: result.emotionalState.valence,
        arousal: result.emotionalState.arousal,
        stressLevel: result.emotionalState.stressLevel,
        primaryEmotion: result.emotionalState.primaryEmotion,
        confidence: result.emotionalState.confidence,
      };

      setCurrentEmotion(emotion);

      // Fetch recommendations
      if (desiredState) {
        await fetchRecommendations(emotion);
      }
    } catch (err) {
      setError('Failed to analyze emotion. Please try again.');
      console.error(err);
    } finally {
      setIsAnalyzing(false);
    }
  };

  const fetchRecommendations = async (emotion: EmotionState) => {
    if (!user || !desiredState) return;

    setIsLoadingRecs(true);
    try {
      const result = await getRecommendations({
        userId: user.id,
        currentState: {
          valence: emotion.valence,
          arousal: emotion.arousal,
          stressLevel: emotion.stressLevel,
        },
        desiredState: {
          valence: desiredState.valence,
          arousal: desiredState.arousal,
          stressLevel: desiredState.stress,
        },
      });

      setRecommendations(result.recommendations || []);
    } catch (err) {
      console.error('Failed to fetch recommendations:', err);
    } finally {
      setIsLoadingRecs(false);
    }
  };

  const handleDesiredStateChange = async (state: DesiredState) => {
    setDesiredState(state);
    if (currentEmotion) {
      await fetchRecommendations(currentEmotion);
    }
  };

  const handleWatch = (contentId: string) => {
    // Find the recommendation to get the title
    const rec = recommendations.find(r => r.contentId === contentId);
    setFeedbackModal({
      isOpen: true,
      contentId,
      contentTitle: rec?.title || contentId,
    });
  };

  const handleFeedbackSubmit = (feedback: any, response: ContentFeedbackResponse) => {
    console.log('Feedback submitted:', { feedback, response });
    // Could show a toast notification here
  };

  const handleFeedbackClose = () => {
    setFeedbackModal({ isOpen: false, contentId: '', contentTitle: '' });
  };

  return (
    <div className="space-y-8">
      {/* Header */}
      <motion.div
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
      >
        <h1 className="text-3xl font-bold text-gray-800 dark:text-white">
          Welcome back, {user?.name?.split(' ')[0] || 'there'}! 👋
        </h1>
        <p className="text-gray-600 dark:text-gray-400 mt-2">
          Tell me how you're feeling and I'll recommend content to help.
        </p>
      </motion.div>

      {/* Emotion Input */}
      <EmotionInput
        onAnalyze={handleAnalyze}
        isLoading={isAnalyzing}
        error={error || undefined}
      />

      {/* Emotion Visualization */}
      <AnimatePresence mode="wait">
        {currentEmotion && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            className="grid grid-cols-1 md:grid-cols-2 gap-6"
          >
            <div className="flex justify-center items-center p-8 bg-white dark:bg-gray-800 rounded-2xl shadow-lg">
              <MoodRing
                valence={currentEmotion.valence}
                arousal={currentEmotion.arousal}
                stress={currentEmotion.stressLevel}
                size="lg"
              />
            </div>

            <EmotionStateCard
              valence={currentEmotion.valence}
              arousal={currentEmotion.arousal}
              stress={currentEmotion.stressLevel}
              primaryEmotion={currentEmotion.primaryEmotion}
              confidence={currentEmotion.confidence}
            />
          </motion.div>
        )}
      </AnimatePresence>

      {/* Desired State */}
      <DesiredStateSelector
        onSelect={handleDesiredStateChange}
      />

      {/* Recommendations */}
      <AnimatePresence mode="wait">
        {(recommendations.length > 0 || isLoadingRecs) && (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -20 }}
          >
            <RecommendationGrid
              recommendations={recommendations}
              isLoading={isLoadingRecs}
              onWatch={handleWatch}
            />
          </motion.div>
        )}
      </AnimatePresence>

      {/* Feedback Modal */}
      {user && currentEmotion && (
        <FeedbackModal
          isOpen={feedbackModal.isOpen}
          onClose={handleFeedbackClose}
          onSubmit={handleFeedbackSubmit}
          contentTitle={feedbackModal.contentTitle}
          contentId={feedbackModal.contentId}
          userId={user.id}
          emotionBefore={{
            valence: currentEmotion.valence,
            arousal: currentEmotion.arousal,
            stress: currentEmotion.stressLevel,
          }}
        />
      )}
    </div>
  );
}
