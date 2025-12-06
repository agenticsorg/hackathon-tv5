import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import type { EmotionAnalysis } from '../../types';

interface EmotionState {
  currentEmotion: EmotionAnalysis | null;
  desiredState: string | null;
  emotionHistory: EmotionAnalysis[];
  setCurrentEmotion: (emotion: EmotionAnalysis) => void;
  setDesiredState: (state: string) => void;
  addToHistory: (emotion: EmotionAnalysis) => void;
  clearHistory: () => void;
}

export const useEmotionStore = create<EmotionState>()(
  persist(
    (set) => ({
      currentEmotion: null,
      desiredState: null,
      emotionHistory: [],

      setCurrentEmotion: (emotion) => {
        set({
          currentEmotion: emotion,
        });
      },

      setDesiredState: (state) => {
        set({
          desiredState: state,
        });
      },

      addToHistory: (emotion) => {
        set((state) => ({
          emotionHistory: [emotion, ...state.emotionHistory].slice(0, 10), // Keep last 10
        }));
      },

      clearHistory: () => {
        set({
          emotionHistory: [],
        });
      },
    }),
    {
      name: 'emotion-storage',
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({
        currentEmotion: state.currentEmotion,
        desiredState: state.desiredState,
        emotionHistory: state.emotionHistory,
      }),
    }
  )
);
