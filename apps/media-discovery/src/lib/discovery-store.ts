import { create } from 'zustand';

export interface Genre {
  id: string;
  name: string;
  color: string;
  src: string;
  tmdbId: number;
  hoverTime: number;
  hoverCount: number;
  clickOrder: number | null;
}

export interface DiscoveryMetrics {
  genres: Record<string, Genre>;
  clickSequence: string[];
  mouseVelocity: number[];
  skipEvents: number;
  totalHovers: number;
  startTime: number | null;
  totalTime: number;
  currentScreen: 'signin' | 'discovery' | 'results' | 'app';
  isTransitioning: boolean;
  currentHover: string | null;
  hoverStartTime: number | null;
  profileComplete: boolean;
}

// 8 genre configuration with Unsplash images and TMDB genre IDs
export const GENRES: Omit<Genre, 'hoverTime' | 'hoverCount' | 'clickOrder'>[] = [
  {
    id: 'romance',
    name: 'Romance',
    color: '#FF6B6B',
    tmdbId: 10749,
    src: 'https://images.unsplash.com/photo-1516589178581-6cd7833ae3b2?w=300&h=533&fit=crop'
  },
  {
    id: 'thriller',
    name: 'Thriller',
    color: '#4ECDC4',
    tmdbId: 53,
    src: 'https://images.unsplash.com/photo-1478720568477-152d9b164e26?w=300&h=533&fit=crop'
  },
  {
    id: 'comedy',
    name: 'Comedy',
    color: '#FFE66D',
    tmdbId: 35,
    src: 'https://images.unsplash.com/photo-1517604931442-7e0c8ed2963c?w=300&h=533&fit=crop'
  },
  {
    id: 'scifi',
    name: 'Sci-Fi',
    color: '#A855F7',
    tmdbId: 878,
    src: 'https://images.unsplash.com/photo-1446776811953-b23d57bd21aa?w=300&h=533&fit=crop'
  },
  {
    id: 'drama',
    name: 'Drama',
    color: '#F97316',
    tmdbId: 18,
    src: 'https://images.unsplash.com/photo-1440404653325-ab127d49abc1?w=300&h=533&fit=crop'
  },
  {
    id: 'action',
    name: 'Action',
    color: '#EF4444',
    tmdbId: 28,
    src: 'https://images.unsplash.com/photo-1536440136628-849c177e76a1?w=300&h=533&fit=crop'
  },
  {
    id: 'horror',
    name: 'Horror',
    color: '#6B7280',
    tmdbId: 27,
    src: 'https://images.unsplash.com/photo-1626544827763-d516dce335e2?w=300&h=533&fit=crop'
  },
  {
    id: 'documentary',
    name: 'Documentary',
    color: '#10B981',
    tmdbId: 99,
    src: 'https://images.unsplash.com/photo-1492724441997-5dc865305da7?w=300&h=533&fit=crop'
  },
];

interface DiscoveryStore extends DiscoveryMetrics {
  // Actions
  initializeGenres: () => void;
  setScreen: (screen: DiscoveryMetrics['currentScreen']) => void;
  setTransitioning: (value: boolean) => void;
  startDiscovery: () => void;
  onHoverEnter: (id: string) => void;
  onHoverLeave: (id: string) => void;
  onThumbnailClick: (id: string) => void;
  trackVelocity: (speed: number) => void;
  incrementSkipEvents: () => void;
  updateTotalTime: (time: number) => void;
  getGenreScores: () => Genre[];
  resetMetrics: () => void;
  completeProfile: () => void;
}

const initialState: DiscoveryMetrics = {
  genres: {},
  clickSequence: [],
  mouseVelocity: [],
  skipEvents: 0,
  totalHovers: 0,
  startTime: null,
  totalTime: 0,
  currentScreen: 'signin',
  isTransitioning: false,
  currentHover: null,
  hoverStartTime: null,
  profileComplete: false,
};

export const useDiscoveryStore = create<DiscoveryStore>((set, get) => ({
  ...initialState,

  initializeGenres: () => {
    const genresMap: Record<string, Genre> = {};
    GENRES.forEach(g => {
      genresMap[g.id] = {
        ...g,
        tmdbId: g.tmdbId,
        hoverTime: 0,
        hoverCount: 0,
        clickOrder: null,
      };
    });
    set({ genres: genresMap });
  },

  setScreen: (screen) => set({ currentScreen: screen }),

  setTransitioning: (value) => set({ isTransitioning: value }),

  startDiscovery: () => {
    get().initializeGenres();
    set({
      startTime: performance.now(),
      clickSequence: [],
      mouseVelocity: [],
      skipEvents: 0,
      totalHovers: 0,
      totalTime: 0,
    });
  },

  onHoverEnter: (id) => {
    set((state) => ({
      currentHover: id,
      hoverStartTime: performance.now(),
      totalHovers: state.totalHovers + 1,
      genres: {
        ...state.genres,
        [id]: {
          ...state.genres[id],
          hoverCount: state.genres[id].hoverCount + 1,
        },
      },
    }));
  },

  onHoverLeave: (id) => {
    const state = get();
    if (state.currentHover === id && state.hoverStartTime) {
      const hoverDuration = performance.now() - state.hoverStartTime;
      set((s) => ({
        currentHover: null,
        hoverStartTime: null,
        genres: {
          ...s.genres,
          [id]: {
            ...s.genres[id],
            hoverTime: s.genres[id].hoverTime + hoverDuration,
          },
        },
      }));
    }
  },

  onThumbnailClick: (id) => {
    const state = get();
    if (state.genres[id]?.clickOrder !== null) return;

    const clickNum = state.clickSequence.length + 1;
    set((s) => ({
      clickSequence: [...s.clickSequence, id],
      genres: {
        ...s.genres,
        [id]: {
          ...s.genres[id],
          clickOrder: clickNum,
        },
      },
    }));
  },

  trackVelocity: (speed) => {
    set((state) => ({
      mouseVelocity: [...state.mouseVelocity, speed],
    }));
    if (speed > 2000) {
      set((state) => ({ skipEvents: state.skipEvents + 1 }));
    }
  },

  incrementSkipEvents: () => set((state) => ({ skipEvents: state.skipEvents + 1 })),

  updateTotalTime: (time) => set({ totalTime: time }),

  getGenreScores: () => {
    const state = get();
    return Object.values(state.genres)
      .map(g => ({
        ...g,
        score: g.hoverTime + (g.clickOrder ? (9 - g.clickOrder) * 800 : 0),
      }))
      .sort((a, b) => (b as Genre & { score: number }).score - (a as Genre & { score: number }).score);
  },

  resetMetrics: () => {
    set(initialState);
    get().initializeGenres();
  },

  completeProfile: () => set({ profileComplete: true, currentScreen: 'app' }),
}));
