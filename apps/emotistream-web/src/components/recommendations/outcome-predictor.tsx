import React from 'react';
import { motion } from 'framer-motion';
import { ArrowRight, TrendingUp, TrendingDown, Minus } from 'lucide-react';

interface PredictedOutcome {
  expectedValence: number;
  expectedArousal: number;
  expectedStress: number;
  confidence: number;
}

interface OutcomePredictorProps {
  currentState: {
    valence: number;
    arousal: number;
    stress: number;
  };
  predictedOutcome: PredictedOutcome;
  compact?: boolean;
}

export function OutcomePredictor({ currentState, predictedOutcome, compact = false }: OutcomePredictorProps) {
  const getChangeIcon = (current: number, predicted: number) => {
    const diff = predicted - current;
    if (Math.abs(diff) < 0.1) return <Minus className="w-3 h-3" />;
    return diff > 0 ? <TrendingUp className="w-3 h-3" /> : <TrendingDown className="w-3 h-3" />;
  };

  const getChangeColor = (current: number, predicted: number, isStress = false) => {
    const diff = predicted - current;
    if (Math.abs(diff) < 0.1) return 'text-gray-400';

    if (isStress) {
      // For stress, down is good
      return diff < 0 ? 'text-green-500' : 'text-red-500';
    }
    // For valence/arousal, up is good
    return diff > 0 ? 'text-green-500' : 'text-red-500';
  };

  const formatChange = (current: number, predicted: number): string => {
    const diff = predicted - current;
    const sign = diff > 0 ? '+' : '';
    return `${sign}${(diff * 100).toFixed(0)}%`;
  };

  if (compact) {
    return (
      <div className="flex items-center gap-2 text-xs">
        <div className={`flex items-center gap-1 ${getChangeColor(currentState.valence, predictedOutcome.expectedValence)}`}>
          {getChangeIcon(currentState.valence, predictedOutcome.expectedValence)}
          <span>Mood</span>
        </div>
        <div className={`flex items-center gap-1 ${getChangeColor(currentState.arousal, predictedOutcome.expectedArousal)}`}>
          {getChangeIcon(currentState.arousal, predictedOutcome.expectedArousal)}
          <span>Energy</span>
        </div>
        <div className={`flex items-center gap-1 ${getChangeColor(currentState.stress, predictedOutcome.expectedStress, true)}`}>
          {getChangeIcon(currentState.stress, predictedOutcome.expectedStress)}
          <span>Stress</span>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between text-sm text-gray-400">
        <span>Expected Emotional Transition</span>
        <span className="text-xs">
          {(predictedOutcome.confidence * 100).toFixed(0)}% confidence
        </span>
      </div>

      <div className="space-y-2">
        {/* Valence (Mood) */}
        <motion.div
          className="flex items-center gap-3"
          initial={{ opacity: 0, x: -20 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ delay: 0.1 }}
        >
          <div className="flex-1">
            <div className="flex items-center justify-between mb-1">
              <span className="text-xs text-gray-400">Mood</span>
              <span className={`text-xs font-medium ${getChangeColor(currentState.valence, predictedOutcome.expectedValence)}`}>
                {formatChange(currentState.valence, predictedOutcome.expectedValence)}
              </span>
            </div>
            <div className="flex items-center gap-2">
              <div className="flex-1 h-1.5 bg-gray-700 rounded-full overflow-hidden">
                <motion.div
                  className="h-full bg-purple-500"
                  initial={{ width: `${currentState.valence * 100}%` }}
                  animate={{ width: `${predictedOutcome.expectedValence * 100}%` }}
                  transition={{ duration: 0.8, ease: "easeInOut" }}
                />
              </div>
              <div className={getChangeColor(currentState.valence, predictedOutcome.expectedValence)}>
                {getChangeIcon(currentState.valence, predictedOutcome.expectedValence)}
              </div>
            </div>
          </div>
        </motion.div>

        {/* Arousal (Energy) */}
        <motion.div
          className="flex items-center gap-3"
          initial={{ opacity: 0, x: -20 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ delay: 0.2 }}
        >
          <div className="flex-1">
            <div className="flex items-center justify-between mb-1">
              <span className="text-xs text-gray-400">Energy</span>
              <span className={`text-xs font-medium ${getChangeColor(currentState.arousal, predictedOutcome.expectedArousal)}`}>
                {formatChange(currentState.arousal, predictedOutcome.expectedArousal)}
              </span>
            </div>
            <div className="flex items-center gap-2">
              <div className="flex-1 h-1.5 bg-gray-700 rounded-full overflow-hidden">
                <motion.div
                  className="h-full bg-blue-500"
                  initial={{ width: `${currentState.arousal * 100}%` }}
                  animate={{ width: `${predictedOutcome.expectedArousal * 100}%` }}
                  transition={{ duration: 0.8, ease: "easeInOut" }}
                />
              </div>
              <div className={getChangeColor(currentState.arousal, predictedOutcome.expectedArousal)}>
                {getChangeIcon(currentState.arousal, predictedOutcome.expectedArousal)}
              </div>
            </div>
          </div>
        </motion.div>

        {/* Stress */}
        <motion.div
          className="flex items-center gap-3"
          initial={{ opacity: 0, x: -20 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ delay: 0.3 }}
        >
          <div className="flex-1">
            <div className="flex items-center justify-between mb-1">
              <span className="text-xs text-gray-400">Stress</span>
              <span className={`text-xs font-medium ${getChangeColor(currentState.stress, predictedOutcome.expectedStress, true)}`}>
                {formatChange(currentState.stress, predictedOutcome.expectedStress)}
              </span>
            </div>
            <div className="flex items-center gap-2">
              <div className="flex-1 h-1.5 bg-gray-700 rounded-full overflow-hidden">
                <motion.div
                  className="h-full bg-red-500"
                  initial={{ width: `${currentState.stress * 100}%` }}
                  animate={{ width: `${predictedOutcome.expectedStress * 100}%` }}
                  transition={{ duration: 0.8, ease: "easeInOut" }}
                />
              </div>
              <div className={getChangeColor(currentState.stress, predictedOutcome.expectedStress, true)}>
                {getChangeIcon(currentState.stress, predictedOutcome.expectedStress)}
              </div>
            </div>
          </div>
        </motion.div>
      </div>

      <div className="flex items-center justify-center pt-2">
        <ArrowRight className="w-4 h-4 text-gray-500" />
      </div>
    </div>
  );
}
