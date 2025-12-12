import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { Movie, TVShow } from '@/types/media';

export interface UserPreferences {
    age?: 'under_18' | '18_30' | '30_50' | '50_plus';
    contentType?: 'movie' | 'tv' | 'animation' | 'anime' | 'spectacle' | 'short_film';
    favoriteMovieId?: number;
    favoriteMediaType?: 'movie' | 'tv';
    intent?: 'learn' | 'kill_time';
    social?: 'alone' | 'friends' | 'family' | 'partner';
    era?: 'new' | 'classic';
    userCountry?: string;
    isOnboarded: boolean;
}

export interface ChatMessage {
    role: 'user' | 'assistant';
    content: string;
    recommendations?: (Movie | TVShow)[];
}

interface MatchState {
    preferences: UserPreferences;
    likedContent: (Movie | TVShow)[];
    dislikedContentIds: number[];
    lastLikedContentId?: number;
    genreWeights: Record<number, number>; // Genre ID -> Weight count
    totalLikes: number;
    recommendations: (Movie | TVShow)[];
    currentIndex: number;
    page: number;
    incrementPage: () => void;
    chatHistory: ChatMessage[];

    // Actions
    setPreferences: (prefs: Partial<UserPreferences>) => void;
    completeOnboarding: () => void;
    likeContent: (content: Movie | TVShow) => void;
    unlikeContent: (contentId: number) => void;
    dislikeContent: (contentId: number) => void;
    setRecommendations: (content: (Movie | TVShow)[]) => void;
    appendRecommendations: (content: (Movie | TVShow)[]) => void;
    nextCard: () => void;
    reset: () => void;
    addChatMessage: (message: ChatMessage) => void;
    clearChatHistory: () => void;
}

export const useMatchStore = create<MatchState>()(
    persist(
        (set) => ({
            preferences: {
                isOnboarded: false,
                userCountry: 'FR',
            },
            likedContent: [],
            dislikedContentIds: [],
            genreWeights: {},
            totalLikes: 0,
            recommendations: [],
            currentIndex: 0,
            page: 1,
            chatHistory: [],

            setPreferences: (prefs) =>
                set((state) => ({
                    preferences: { ...state.preferences, ...prefs },
                })),

            completeOnboarding: () =>
                set((state) => ({
                    preferences: { ...state.preferences, isOnboarded: true },
                })),

            likeContent: (content) =>
                set((state) => {
                    // Update genre weights
                    const newWeights = { ...state.genreWeights };
                    content.genreIds.forEach(id => {
                        newWeights[id] = (newWeights[id] || 0) + 1;
                    });

                    return {
                        likedContent: [...state.likedContent, content],
                        lastLikedContentId: content.id,
                        genreWeights: newWeights,
                        totalLikes: state.totalLikes + 1,
                        currentIndex: state.currentIndex + 1,
                    };
                }),

            dislikeContent: (contentId) =>
                set((state) => ({
                    dislikedContentIds: [...state.dislikedContentIds, contentId],
                    currentIndex: state.currentIndex + 1,
                })),

            unlikeContent: (contentId) =>
                set((state) => ({
                    likedContent: state.likedContent.filter((c) => c.id !== contentId),
                    totalLikes: state.totalLikes - 1,
                })),

            setRecommendations: (content) =>
                set({ recommendations: content, currentIndex: 0, page: 1 }),

            appendRecommendations: (content) =>
                set((state) => ({
                    recommendations: [...state.recommendations, ...content],
                    page: state.page + 1,
                })),

            incrementPage: () =>
                set((state) => ({
                    page: state.page + 1,
                })),

            nextCard: () =>
                set((state) => ({
                    currentIndex: state.currentIndex + 1,
                })),

            reset: () =>
                set({
                    preferences: { isOnboarded: false },
                    likedContent: [],
                    dislikedContentIds: [],
                    genreWeights: {},
                    totalLikes: 0,
                    recommendations: [],
                    currentIndex: 0,
                    page: 1,
                    chatHistory: [],
                }),

            addChatMessage: (message) =>
                set((state) => ({
                    chatHistory: [...state.chatHistory, message],
                })),

            clearChatHistory: () =>
                set({ chatHistory: [] }),
        }),
        {
            name: 'movie-match-storage',
        }
    )
);
