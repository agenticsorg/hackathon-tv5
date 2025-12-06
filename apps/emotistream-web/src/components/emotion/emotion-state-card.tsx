'use client';

import { motion } from 'framer-motion';

interface EmotionStateCardProps {
  valence: number;
  arousal: number;
  stress: number;
  primaryEmotion: string;
  confidence: number;
}

export function EmotionStateCard({
  valence, arousal, stress, primaryEmotion, confidence
}: EmotionStateCardProps) {
  return (
    <motion.div
      initial={{ opacity: 0, x: 20 }}
      animate={{ opacity: 1, x: 0 }}
      className="p-6 bg-white dark:bg-gray-800 rounded-2xl shadow-lg"
    >
      <h3 className="text-lg font-semibold mb-4 text-gray-800 dark:text-white">
        Your Emotional State
      </h3>

      <div className="space-y-4">
        <MetricBar
          label="Valence"
          value={valence}
          min={-1}
          max={1}
          leftLabel="Negative 😟"
          rightLabel="Positive 😊"
          colorFrom="red-500"
          colorTo="green-500"
        />

        <MetricBar
          label="Arousal"
          value={arousal}
          min={-1}
          max={1}
          leftLabel="Calm 😌"
          rightLabel="Energized ⚡"
          colorFrom="blue-500"
          colorTo="orange-500"
        />

        <MetricBar
          label="Stress"
          value={stress}
          min={0}
          max={1}
          leftLabel="Relaxed 🧘"
          rightLabel="Stressed 🔥"
          colorFrom="green-500"
          colorTo="red-500"
        />
      </div>

      <div className="mt-6 pt-4 border-t dark:border-gray-700">
        <div className="flex justify-between items-center">
          <span className="text-2xl font-semibold capitalize">
            {getEmoji(primaryEmotion)} {primaryEmotion}
          </span>
          <span className="text-sm text-gray-500">
            {Math.round(confidence * 100)}% confident
          </span>
        </div>
      </div>
    </motion.div>
  );
}

function MetricBar({
  label, value, min, max, leftLabel, rightLabel, colorFrom, colorTo
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  leftLabel: string;
  rightLabel: string;
  colorFrom: string;
  colorTo: string;
}) {
  const percentage = ((value - min) / (max - min)) * 100;

  return (
    <div>
      <div className="flex justify-between text-sm mb-1">
        <span className="text-gray-600 dark:text-gray-400">{label}</span>
        <span className="font-medium">{value.toFixed(2)}</span>
      </div>
      <div className="h-3 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
        <motion.div
          className={`h-full bg-gradient-to-r from-${colorFrom} to-${colorTo}`}
          initial={{ width: 0 }}
          animate={{ width: `${percentage}%` }}
          transition={{ duration: 0.5, ease: "easeOut" }}
        />
      </div>
      <div className="flex justify-between text-xs text-gray-500 mt-1">
        <span>{leftLabel}</span>
        <span>{rightLabel}</span>
      </div>
    </div>
  );
}

function getEmoji(emotion: string): string {
  const emojiMap: Record<string, string> = {
    joy: '😊', happy: '😊', excited: '🤩', content: '🙂',
    sad: '😢', anxious: '😰', stressed: '😫', angry: '😠',
    calm: '😌', relaxed: '🧘', neutral: '😐', fear: '😨'
  };
  return emojiMap[emotion.toLowerCase()] || '🎭';
}
