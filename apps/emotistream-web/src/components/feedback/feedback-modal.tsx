'use client';

import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, Star, Check, Sparkles, Loader2 } from 'lucide-react';
import { MoodRing } from '@/components/emotion/mood-ring';
import { analyzeEmotion } from '@/lib/api/emotion';
import { submitContentFeedback, type ContentFeedbackResponse } from '@/lib/api/feedback';

interface FeedbackModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (feedback: FeedbackData, response: ContentFeedbackResponse) => void;
  contentTitle: string;
  contentId: string;
  userId: string;
  emotionBefore: {
    valence: number;
    arousal: number;
    stress: number;
  };
}

interface FeedbackData {
  emotionAfterText: string;
  emotionAfter: {
    valence: number;
    arousal: number;
    stress: number;
  };
  starRating: number;
  completed: boolean;
}

export function FeedbackModal({
  isOpen,
  onClose,
  onSubmit,
  contentTitle,
  contentId,
  userId,
  emotionBefore,
}: FeedbackModalProps) {
  const [step, setStep] = useState<'emotion' | 'rating' | 'success'>('emotion');
  const [emotionText, setEmotionText] = useState('');
  const [emotionAfter, setEmotionAfter] = useState({ valence: 0, arousal: 0, stress: 0.5 });
  const [rating, setRating] = useState(0);
  const [completed, setCompleted] = useState(true);
  const [reward, setReward] = useState<number | null>(null);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [feedbackResponse, setFeedbackResponse] = useState<ContentFeedbackResponse | null>(null);

  const handleEmotionAnalyzed = async () => {
    setIsAnalyzing(true);
    setError(null);
    try {
      const result = await analyzeEmotion({ text: emotionText, userId });
      setEmotionAfter({
        valence: result.emotionalState.valence,
        arousal: result.emotionalState.arousal,
        stress: result.emotionalState.stressLevel,
      });
      setStep('rating');
    } catch (err) {
      console.error('Failed to analyze emotion:', err);
      setError('Failed to analyze emotion. Please try again.');
    } finally {
      setIsAnalyzing(false);
    }
  };

  const handleSubmit = async () => {
    setIsSubmitting(true);
    setError(null);
    try {
      // Submit feedback to backend
      const response = await submitContentFeedback({
        userId,
        contentId,
        contentTitle,
        actualPostState: {
          valence: emotionAfter.valence,
          arousal: emotionAfter.arousal,
          stressLevel: emotionAfter.stress,
        },
        watchDuration: 10, // Default 10 minutes
        completed,
        explicitRating: rating,
      });

      setReward(response.reward);
      setFeedbackResponse(response);
      setStep('success');

      // Notify parent
      onSubmit({
        emotionAfterText: emotionText,
        emotionAfter,
        starRating: rating,
        completed,
      }, response);
    } catch (err) {
      console.error('Failed to submit feedback:', err);
      setError('Failed to submit feedback. Please try again.');
    } finally {
      setIsSubmitting(false);
    }
  };

  if (!isOpen) return null;

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4"
        onClick={onClose}
      >
        <motion.div
          initial={{ scale: 0.9, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          exit={{ scale: 0.9, opacity: 0 }}
          onClick={(e) => e.stopPropagation()}
          className="bg-white dark:bg-gray-800 rounded-2xl shadow-2xl max-w-lg w-full max-h-[90vh] overflow-y-auto"
        >
          {/* Header */}
          <div className="p-6 border-b dark:border-gray-700 flex justify-between items-center">
            <h2 className="text-xl font-semibold">
              {step === 'success' ? '🎉 Thank you!' : `How was "${contentTitle}"?`}
            </h2>
            <button onClick={onClose} className="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg">
              <X className="w-5 h-5" />
            </button>
          </div>

          {/* Content */}
          <div className="p-6">
            <AnimatePresence mode="wait">
              {step === 'emotion' && (
                <motion.div
                  key="emotion"
                  initial={{ opacity: 0, x: 20 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: -20 }}
                  className="space-y-6"
                >
                  <p className="text-gray-600 dark:text-gray-400">
                    How do you feel now after watching?
                  </p>

                  <textarea
                    value={emotionText}
                    onChange={(e) => setEmotionText(e.target.value)}
                    placeholder="I feel more relaxed now, the meditation really helped..."
                    className="w-full h-24 p-4 border rounded-xl resize-none focus:ring-2 focus:ring-blue-500
                             dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                  />

                  {error && (
                    <p className="text-red-500 text-sm">{error}</p>
                  )}

                  <button
                    onClick={handleEmotionAnalyzed}
                    disabled={emotionText.length < 10 || isAnalyzing}
                    className="w-full py-3 bg-gradient-to-r from-blue-500 to-purple-600 text-white rounded-xl
                             font-medium disabled:opacity-50 hover:shadow-lg transition-all flex items-center justify-center gap-2"
                  >
                    {isAnalyzing ? (
                      <>
                        <Loader2 className="w-5 h-5 animate-spin" />
                        Analyzing...
                      </>
                    ) : (
                      'Analyze My Emotion'
                    )}
                  </button>
                </motion.div>
              )}

              {step === 'rating' && (
                <motion.div
                  key="rating"
                  initial={{ opacity: 0, x: 20 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: -20 }}
                  className="space-y-6"
                >
                  {/* Before/After Comparison */}
                  <div className="grid grid-cols-2 gap-4">
                    <div className="text-center">
                      <p className="text-sm text-gray-500 mb-2">Before</p>
                      <div className="flex justify-center">
                        <MoodRing
                          valence={emotionBefore.valence}
                          arousal={emotionBefore.arousal}
                          stress={emotionBefore.stress}
                          size="sm"
                          animate={false}
                        />
                      </div>
                    </div>
                    <div className="text-center">
                      <p className="text-sm text-gray-500 mb-2">After</p>
                      <div className="flex justify-center">
                        <MoodRing
                          valence={emotionAfter.valence}
                          arousal={emotionAfter.arousal}
                          stress={emotionAfter.stress}
                          size="sm"
                        />
                      </div>
                    </div>
                  </div>

                  {/* Star Rating */}
                  <div>
                    <p className="text-gray-600 dark:text-gray-400 mb-3">Rate your experience:</p>
                    <div className="flex gap-2 justify-center">
                      {[1, 2, 3, 4, 5].map((star) => (
                        <motion.button
                          key={star}
                          whileHover={{ scale: 1.1 }}
                          whileTap={{ scale: 0.9 }}
                          onClick={() => setRating(star)}
                          className="p-1"
                        >
                          <Star
                            className={`w-10 h-10 ${
                              star <= rating
                                ? 'fill-yellow-400 text-yellow-400'
                                : 'text-gray-300 dark:text-gray-600'
                            }`}
                          />
                        </motion.button>
                      ))}
                    </div>
                  </div>

                  {/* Completion */}
                  <label className="flex items-center gap-3 cursor-pointer">
                    <div
                      className={`w-6 h-6 rounded border-2 flex items-center justify-center
                                ${completed
                                  ? 'bg-green-500 border-green-500'
                                  : 'border-gray-300 dark:border-gray-600'
                                }`}
                      onClick={() => setCompleted(!completed)}
                    >
                      {completed && <Check className="w-4 h-4 text-white" />}
                    </div>
                    <span className="text-gray-700 dark:text-gray-300">
                      I completed the content
                    </span>
                  </label>

                  {error && (
                    <p className="text-red-500 text-sm">{error}</p>
                  )}

                  <button
                    onClick={handleSubmit}
                    disabled={rating === 0 || isSubmitting}
                    className="w-full py-3 bg-gradient-to-r from-green-500 to-emerald-600 text-white rounded-xl
                             font-medium disabled:opacity-50 hover:shadow-lg transition-all flex items-center justify-center gap-2"
                  >
                    {isSubmitting ? (
                      <>
                        <Loader2 className="w-5 h-5 animate-spin" />
                        Submitting...
                      </>
                    ) : (
                      'Submit Feedback'
                    )}
                  </button>
                </motion.div>
              )}

              {step === 'success' && (
                <motion.div
                  key="success"
                  initial={{ opacity: 0, scale: 0.8 }}
                  animate={{ opacity: 1, scale: 1 }}
                  className="text-center space-y-6"
                >
                  <motion.div
                    initial={{ scale: 0 }}
                    animate={{ scale: 1 }}
                    transition={{ type: "spring", delay: 0.2 }}
                    className="w-24 h-24 mx-auto bg-green-100 dark:bg-green-900/30 rounded-full
                             flex items-center justify-center"
                  >
                    <Sparkles className="w-12 h-12 text-green-500" />
                  </motion.div>

                  <div>
                    <motion.p
                      initial={{ opacity: 0, y: 10 }}
                      animate={{ opacity: 1, y: 0 }}
                      transition={{ delay: 0.4 }}
                      className="text-4xl font-bold text-green-500"
                    >
                      +{((reward || 0) * 100).toFixed(0)}%
                    </motion.p>
                    <motion.p
                      initial={{ opacity: 0 }}
                      animate={{ opacity: 1 }}
                      transition={{ delay: 0.6 }}
                      className="text-gray-600 dark:text-gray-400 mt-2"
                    >
                      Great choice! You moved closer to your goal.
                    </motion.p>
                  </div>

                  <motion.button
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    transition={{ delay: 0.8 }}
                    onClick={onClose}
                    className="px-6 py-3 bg-gray-100 dark:bg-gray-700 rounded-xl
                             hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                  >
                    Continue
                  </motion.button>
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}
