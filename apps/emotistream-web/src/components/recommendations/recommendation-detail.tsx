import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, Play, Bookmark, TrendingUp, Brain, Clock, Target } from 'lucide-react';
import { getCategoryGradient, getCategoryIcon, formatDuration, getScoreColor } from '@/lib/utils/category-thumbnails';
import { OutcomePredictor } from './outcome-predictor';

interface RecommendationDetailProps {
  isOpen: boolean;
  onClose: () => void;
  recommendation: {
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
    qValueHistory?: Array<{
      timestamp: number;
      qValue: number;
      reward?: number;
    }>;
  };
  currentState?: {
    valence: number;
    arousal: number;
    stress: number;
  };
  onWatch: () => void;
  onSave?: () => void;
}

export function RecommendationDetail({
  isOpen,
  onClose,
  recommendation,
  currentState,
  onWatch,
  onSave,
}: RecommendationDetailProps) {
  const scorePercentage = Math.round(recommendation.combinedScore * 100);
  const gradient = getCategoryGradient(recommendation.category);
  const icon = getCategoryIcon(recommendation.category);

  return (
    <AnimatePresence>
      {isOpen && (
        <>
          {/* Backdrop */}
          <motion.div
            className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            onClick={onClose}
          />

          {/* Modal */}
          <motion.div
            className="fixed inset-x-4 top-20 bottom-20 md:inset-x-auto md:left-1/2 md:top-1/2 md:-translate-x-1/2 md:-translate-y-1/2 md:w-full md:max-w-3xl bg-gray-900 rounded-xl shadow-2xl z-50 overflow-hidden flex flex-col"
            initial={{ opacity: 0, scale: 0.9, y: 20 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.9, y: 20 }}
            transition={{ type: "spring", damping: 25, stiffness: 300 }}
          >
            {/* Header with Thumbnail */}
            <div className={`relative h-48 bg-gradient-to-br ${gradient} flex items-center justify-center`}>
              <span className="text-8xl opacity-30">{icon}</span>

              <button
                onClick={onClose}
                className="absolute top-4 right-4 p-2 bg-black/50 hover:bg-black/70 rounded-full transition-colors"
              >
                <X className="w-5 h-5" />
              </button>

              <div className={`absolute bottom-4 left-4 px-3 py-1.5 bg-black/50 backdrop-blur-sm rounded-full`}>
                <span className={`text-lg font-bold ${getScoreColor(scorePercentage)}`}>
                  {scorePercentage}% Match
                </span>
              </div>
            </div>

            {/* Scrollable Content */}
            <div className="flex-1 overflow-y-auto p-6 space-y-6">
              {/* Title and Meta */}
              <div>
                <h2 className="text-2xl font-bold text-white mb-2">{recommendation.title}</h2>
                <div className="flex items-center gap-3 text-sm text-gray-400">
                  <span className="px-3 py-1 bg-gray-800 rounded-full capitalize">{recommendation.category}</span>
                  <span className="flex items-center gap-1">
                    <Clock className="w-4 h-4" />
                    {formatDuration(recommendation.duration)}
                  </span>
                  {recommendation.isExploration && (
                    <span className="px-3 py-1 bg-blue-500/20 text-blue-400 rounded-full">
                      Exploration Pick
                    </span>
                  )}
                </div>
              </div>

              {/* Why This Content */}
              <div className="space-y-3">
                <div className="flex items-center gap-2 text-white font-semibold">
                  <Brain className="w-5 h-5 text-purple-400" />
                  <span>Why We Recommend This</span>
                </div>
                <p className="text-gray-300 leading-relaxed">{recommendation.reasoning}</p>
              </div>

              {/* Predicted Emotional Transition */}
              {currentState && (
                <div className="space-y-3">
                  <div className="flex items-center gap-2 text-white font-semibold">
                    <Target className="w-5 h-5 text-green-400" />
                    <span>Expected Emotional Impact</span>
                  </div>
                  <div className="bg-gray-800 rounded-lg p-4">
                    <OutcomePredictor
                      currentState={currentState}
                      predictedOutcome={recommendation.predictedOutcome}
                    />
                  </div>
                </div>
              )}

              {/* Q-Value History */}
              {recommendation.qValueHistory && recommendation.qValueHistory.length > 0 && (
                <div className="space-y-3">
                  <div className="flex items-center gap-2 text-white font-semibold">
                    <TrendingUp className="w-5 h-5 text-blue-400" />
                    <span>Learning History</span>
                  </div>
                  <div className="bg-gray-800 rounded-lg p-4">
                    <div className="text-xs text-gray-400 mb-3">
                      Our AI has learned from {recommendation.qValueHistory.length} previous interactions
                    </div>
                    <div className="space-y-2">
                      {recommendation.qValueHistory.slice(-5).reverse().map((entry, i) => (
                        <div key={i} className="flex items-center justify-between text-sm">
                          <span className="text-gray-400">
                            {new Date(entry.timestamp).toLocaleDateString()}
                          </span>
                          <div className="flex items-center gap-2">
                            <span className="text-gray-300">
                              Q-Value: {entry.qValue.toFixed(3)}
                            </span>
                            {entry.reward !== undefined && (
                              <span className={entry.reward > 0 ? 'text-green-400' : 'text-red-400'}>
                                {entry.reward > 0 ? '+' : ''}{entry.reward.toFixed(2)}
                              </span>
                            )}
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
              )}

              {/* Confidence Explanation */}
              <div className="bg-gray-800 rounded-lg p-4">
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm text-gray-400">Prediction Confidence</span>
                  <span className="text-sm font-medium text-white">
                    {(recommendation.predictedOutcome.confidence * 100).toFixed(0)}%
                  </span>
                </div>
                <div className="h-2 bg-gray-700 rounded-full overflow-hidden">
                  <motion.div
                    className="h-full bg-gradient-to-r from-purple-500 to-blue-500"
                    initial={{ width: 0 }}
                    animate={{ width: `${recommendation.predictedOutcome.confidence * 100}%` }}
                    transition={{ duration: 0.8 }}
                  />
                </div>
                <p className="text-xs text-gray-400 mt-2">
                  {recommendation.predictedOutcome.confidence > 0.8
                    ? 'High confidence - similar content has consistently delivered positive outcomes'
                    : recommendation.predictedOutcome.confidence > 0.6
                    ? 'Moderate confidence - we have some data supporting this recommendation'
                    : 'Lower confidence - this is an exploratory recommendation to learn your preferences'}
                </p>
              </div>
            </div>

            {/* Action Buttons */}
            <div className="border-t border-gray-800 p-4 flex gap-3">
              <motion.button
                onClick={onWatch}
                className="flex-1 py-3 bg-purple-600 hover:bg-purple-700 text-white rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
              >
                <Play className="w-5 h-5" />
                <span>Watch Now</span>
              </motion.button>
              {onSave && (
                <motion.button
                  onClick={onSave}
                  className="px-6 py-3 bg-gray-800 hover:bg-gray-700 text-white rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
                  whileHover={{ scale: 1.02 }}
                  whileTap={{ scale: 0.98 }}
                >
                  <Bookmark className="w-5 h-5" />
                  <span>Save</span>
                </motion.button>
              )}
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}
