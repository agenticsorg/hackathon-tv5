import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import type { EmotionState, DesiredState } from '../../types';

interface EmotionStoreState {
  currentEmotion: EmotionState | null;
  desiredState: DesiredState | null;
  emotionHistory: EmotionState[];
  setCurrentEmotion: (emotion: EmotionState) => void;
  setDesiredState: (state: DesiredState) => void;
  addToHistory: (emotion: EmotionState) => void;
  clearHistory: () => void;
}

export const useEmotionStore = create<EmotionStoreState>()(
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
