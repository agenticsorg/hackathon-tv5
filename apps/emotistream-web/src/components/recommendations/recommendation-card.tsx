import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { Play, Info, Search, Clock, TrendingUp } from 'lucide-react';
import { getCategoryGradient, getCategoryIcon, formatDuration, getScoreColor, getScoreBgColor } from '@/lib/utils/category-thumbnails';
import { OutcomePredictor } from './outcome-predictor';

interface RecommendationCardProps {
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
  onWatch: () => void;
  onDetails?: () => void;
  currentState?: {
    valence: number;
    arousal: number;
    stress: number;
  };
}

export function RecommendationCard({
  contentId,
  title,
  category,
  duration,
  combinedScore,
  predictedOutcome,
  reasoning,
  isExploration,
  onWatch,
  onDetails,
  currentState,
}: RecommendationCardProps) {
  const [isHovered, setIsHovered] = useState(false);
  const scorePercentage = Math.round(combinedScore * 100);
  const gradient = getCategoryGradient(category);
  const icon = getCategoryIcon(category);

  return (
    <motion.div
      className="group relative flex-shrink-0 w-72 cursor-pointer"
      onHoverStart={() => setIsHovered(true)}
      onHoverEnd={() => setIsHovered(false)}
      whileHover={{ scale: 1.05 }}
      transition={{ duration: 0.2 }}
    >
      <div className="relative overflow-hidden rounded-lg bg-gray-800 shadow-lg transition-shadow duration-200 group-hover:shadow-2xl">
        {/* Thumbnail */}
        <div className={`relative h-40 bg-gradient-to-br ${gradient} flex items-center justify-center`}>
          <span className="text-6xl opacity-50">{icon}</span>

          {/* Overlay on hover */}
          <motion.div
            className="absolute inset-0 bg-black/60 flex items-center justify-center gap-2"
            initial={{ opacity: 0 }}
            animate={{ opacity: isHovered ? 1 : 0 }}
            transition={{ duration: 0.2 }}
          >
            <motion.button
              onClick={(e) => {
                e.stopPropagation();
                onWatch();
              }}
              className="p-3 bg-white rounded-full text-black hover:bg-gray-200 transition-colors"
              whileHover={{ scale: 1.1 }}
              whileTap={{ scale: 0.95 }}
            >
              <Play className="w-5 h-5 fill-current" />
            </motion.button>
            {onDetails && (
              <motion.button
                onClick={(e) => {
                  e.stopPropagation();
                  onDetails();
                }}
                className="p-3 bg-gray-700 rounded-full text-white hover:bg-gray-600 transition-colors"
                whileHover={{ scale: 1.1 }}
                whileTap={{ scale: 0.95 }}
              >
                <Info className="w-5 h-5" />
              </motion.button>
            )}
          </motion.div>

          {/* Exploration Badge */}
          {isExploration && (
            <div className="absolute top-2 right-2 px-2 py-1 bg-blue-500/90 backdrop-blur-sm rounded-full flex items-center gap-1 text-xs font-medium">
              <Search className="w-3 h-3" />
              <span>Exploring</span>
            </div>
          )}

          {/* Score Badge */}
          <div className={`absolute top-2 left-2 px-2 py-1 ${getScoreBgColor(scorePercentage)} backdrop-blur-sm rounded-full`}>
            <span className={`text-sm font-bold ${getScoreColor(scorePercentage)}`}>
              {scorePercentage}%
            </span>
          </div>
        </div>

        {/* Content Info */}
        <div className="p-4 space-y-3">
          {/* Title and Category */}
          <div>
            <h3 className="font-semibold text-white line-clamp-1 mb-1">{title}</h3>
            <div className="flex items-center gap-2 text-xs text-gray-400">
              <span className="px-2 py-0.5 bg-gray-700 rounded-full capitalize">{category}</span>
              <span className="flex items-center gap-1">
                <Clock className="w-3 h-3" />
                {formatDuration(duration)}
              </span>
            </div>
          </div>

          {/* Predicted Outcome - Compact View */}
          {currentState && (
            <motion.div
              initial={{ opacity: 0, height: 0 }}
              animate={{
                opacity: isHovered ? 1 : 0,
                height: isHovered ? 'auto' : 0
              }}
              transition={{ duration: 0.2 }}
              className="overflow-hidden"
            >
              <OutcomePredictor
                currentState={currentState}
                predictedOutcome={predictedOutcome}
                compact
              />
            </motion.div>
          )}

          {/* Reasoning Preview */}
          <motion.p
            className="text-xs text-gray-400 line-clamp-2"
            initial={{ opacity: 0, height: 0 }}
            animate={{
              opacity: isHovered ? 1 : 0,
              height: isHovered ? 'auto' : 0
            }}
            transition={{ duration: 0.2 }}
          >
            {reasoning}
          </motion.p>

          {/* Watch Button */}
          <motion.button
            onClick={onWatch}
            className="w-full py-2 bg-purple-600 hover:bg-purple-700 text-white rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
          >
            <Play className="w-4 h-4" />
            <span>Watch Now</span>
          </motion.button>
        </div>
      </div>

      {/* Confidence Indicator */}
      <div className="absolute -bottom-1 left-0 right-0 h-1 bg-gray-700 rounded-full overflow-hidden">
        <motion.div
          className="h-full bg-gradient-to-r from-purple-500 to-blue-500"
          initial={{ width: 0 }}
          animate={{ width: `${predictedOutcome.confidence * 100}%` }}
          transition={{ duration: 0.8, delay: 0.2 }}
        />
      </div>
    </motion.div>
  );
}
