import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import type { Recommendation, ContentItem } from '../../types';

interface RecommendationState {
  recommendations: Recommendation[];
  selectedContent: ContentItem | null;
  currentRecommendation: Recommendation | null;
  explorationMode: boolean;
  setRecommendations: (recommendations: Recommendation[]) => void;
  selectContent: (content: ContentItem, recommendation: Recommendation) => void;
  clearSelected: () => void;
  toggleExplorationMode: () => void;
}

export const useRecommendationStore = create<RecommendationState>()(
  persist(
    (set) => ({
      recommendations: [],
      selectedContent: null,
      currentRecommendation: null,
      explorationMode: false,

      setRecommendations: (recommendations) => {
        set({
          recommendations,
        });
      },

      selectContent: (content, recommendation) => {
        set({
          selectedContent: content,
          currentRecommendation: recommendation,
        });
      },

      clearSelected: () => {
        set({
          selectedContent: null,
          currentRecommendation: null,
        });
      },

      toggleExplorationMode: () => {
        set((state) => ({
          explorationMode: !state.explorationMode,
        }));
      },
    }),
    {
      name: 'recommendation-storage',
      storage: createJSONStorage(() => sessionStorage), // Use session storage for recommendations
      partialize: (state) => ({
        selectedContent: state.selectedContent,
        currentRecommendation: state.currentRecommendation,
      }),
    }
  )
);
