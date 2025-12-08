'use client';

import { useState } from 'react';
import { motion } from 'framer-motion';
import { Loader2, Sparkles } from 'lucide-react';

interface EmotionInputProps {
  onAnalyze: (text: string) => void;
  isLoading?: boolean;
  error?: string;
}

export function EmotionInput({ onAnalyze, isLoading, error }: EmotionInputProps) {
  const [text, setText] = useState('');
  const minChars = 10;
  const maxChars = 1000;
  const isValid = text.length >= minChars;

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      className="w-full max-w-2xl mx-auto p-6 bg-white dark:bg-gray-800 rounded-2xl shadow-lg"
    >
      <h2 className="text-xl font-semibold mb-4 text-gray-800 dark:text-white">
        How are you feeling right now?
      </h2>

      <textarea
        value={text}
        onChange={(e) => setText(e.target.value.slice(0, maxChars))}
        placeholder="Share your thoughts... I'm feeling stressed about work and anxious about an upcoming presentation..."
        className="w-full h-32 p-4 text-lg border rounded-xl resize-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
        disabled={isLoading}
      />

      <div className="flex justify-between items-center mt-2">
        <span className={`text-sm ${text.length < minChars ? 'text-red-500' : 'text-green-500'}`}>
          {text.length} / {maxChars} characters
          {text.length < minChars && ` (min ${minChars})`}
        </span>

        <button
          onClick={() => onAnalyze(text)}
          disabled={!isValid || isLoading}
          className="px-6 py-3 bg-gradient-to-r from-blue-500 to-purple-600 text-white rounded-xl
                     font-medium disabled:opacity-50 disabled:cursor-not-allowed
                     hover:shadow-lg transition-all flex items-center gap-2"
        >
          {isLoading ? (
            <>
              <Loader2 className="w-5 h-5 animate-spin" />
              Analyzing...
            </>
          ) : (
            <>
              <Sparkles className="w-5 h-5" />
              Analyze Emotion
            </>
          )}
        </button>
      </div>

      {error && (
        <p className="mt-2 text-red-500 text-sm">{error}</p>
      )}
    </motion.div>
  );
}
