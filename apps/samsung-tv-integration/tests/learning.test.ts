import { describe, it, expect, beforeEach } from 'vitest';
import {
  cosineSimilarity,
  batchSimilarity,
  generateContentEmbedding,
  generatePreferenceEmbedding,
  generateStateEmbedding,
  ContentEmbeddingCache,
} from '../src/learning/embeddings.js';
import { PreferenceLearningSystem } from '../src/learning/preference-learning.js';
import { ContentMetadata, Genre, ContentType, ViewingSession, LearningAction } from '../src/learning/types.js';

describe('Content Embeddings', () => {
  describe('cosineSimilarity', () => {
    it('should return 1 for identical vectors', () => {
      const vec = new Float32Array([1, 0, 0, 0]);
      expect(cosineSimilarity(vec, vec)).toBeCloseTo(1, 5);
    });

    it('should return 0 for orthogonal vectors', () => {
      const a = new Float32Array([1, 0, 0, 0]);
      const b = new Float32Array([0, 1, 0, 0]);
      expect(cosineSimilarity(a, b)).toBeCloseTo(0, 5);
    });

    it('should return -1 for opposite vectors', () => {
      const a = new Float32Array([1, 0, 0, 0]);
      const b = new Float32Array([-1, 0, 0, 0]);
      expect(cosineSimilarity(a, b)).toBeCloseTo(-1, 5);
    });

    it('should handle vectors of different magnitudes', () => {
      const a = new Float32Array([1, 0, 0, 0]);
      const b = new Float32Array([100, 0, 0, 0]);
      expect(cosineSimilarity(a, b)).toBeCloseTo(1, 5);
    });

    it('should throw for vectors of different lengths', () => {
      const a = new Float32Array([1, 0, 0]);
      const b = new Float32Array([1, 0, 0, 0]);
      expect(() => cosineSimilarity(a, b)).toThrow('Vectors must have same length');
    });
  });

  describe('batchSimilarity', () => {
    it('should return top-k similar vectors', () => {
      const query = new Float32Array([1, 0, 0, 0]);
      const vectors = [
        new Float32Array([1, 0, 0, 0]),  // Most similar
        new Float32Array([0.9, 0.1, 0, 0]),
        new Float32Array([0, 1, 0, 0]),
        new Float32Array([0, 0, 1, 0]),
      ];

      const results = batchSimilarity(query, vectors, 2);
      expect(results).toHaveLength(2);
      expect(results[0].index).toBe(0);
      expect(results[0].similarity).toBeCloseTo(1, 5);
    });
  });

  describe('generateContentEmbedding', () => {
    it('should generate fixed-size embedding', () => {
      const content: ContentMetadata = {
        id: 'test-1',
        title: 'Test Movie',
        type: 'movie',
        genres: ['action', 'adventure'],
        duration: 120,
        rating: 8.5,
        popularity: 75,
        keywords: ['hero', 'explosion'],
        actors: [],
        directors: [],
      };

      const embedding = generateContentEmbedding(content);
      expect(embedding).toBeInstanceOf(Float32Array);
      expect(embedding.length).toBe(64);
    });

    it('should generate normalized embedding', () => {
      const content: ContentMetadata = {
        id: 'test-1',
        title: 'Test Movie',
        type: 'movie',
        genres: ['action'],
        actors: [],
        directors: [],
        keywords: [],
      };

      const embedding = generateContentEmbedding(content);
      // Check normalized (magnitude ≈ 1)
      let magnitude = 0;
      for (let i = 0; i < embedding.length; i++) {
        magnitude += embedding[i] * embedding[i];
      }
      expect(Math.sqrt(magnitude)).toBeCloseTo(1, 3);
    });

    it('should generate similar embeddings for similar content', () => {
      const content1: ContentMetadata = {
        id: 'test-1',
        title: 'Action Movie 1',
        type: 'movie',
        genres: ['action', 'adventure'],
        actors: [],
        directors: [],
        keywords: [],
      };

      const content2: ContentMetadata = {
        id: 'test-2',
        title: 'Action Movie 2',
        type: 'movie',
        genres: ['action', 'thriller'],
        actors: [],
        directors: [],
        keywords: [],
      };

      const content3: ContentMetadata = {
        id: 'test-3',
        title: 'Romantic Comedy',
        type: 'movie',
        genres: ['romance', 'comedy'],
        actors: [],
        directors: [],
        keywords: [],
      };

      const emb1 = generateContentEmbedding(content1);
      const emb2 = generateContentEmbedding(content2);
      const emb3 = generateContentEmbedding(content3);

      // Action movies should be more similar to each other than to romantic comedy
      const sim12 = cosineSimilarity(emb1, emb2);
      const sim13 = cosineSimilarity(emb1, emb3);

      expect(sim12).toBeGreaterThan(sim13);
    });
  });

  describe('generatePreferenceEmbedding', () => {
    it('should generate embedding from preferences', () => {
      const embedding = generatePreferenceEmbedding(
        ['action', 'thriller'] as Genre[],
        ['movie', 'tv_show'] as ContentType[],
        8.0,
        90
      );

      expect(embedding).toBeInstanceOf(Float32Array);
      expect(embedding.length).toBe(64);
    });
  });

  describe('generateStateEmbedding', () => {
    it('should generate state embedding', () => {
      const embedding = generateStateEmbedding(
        'evening',
        'weekend',
        ['action', 'comedy'] as Genre[],
        ['movie'] as ContentType[],
        10,
        0.75
      );

      expect(embedding).toBeInstanceOf(Float32Array);
      expect(embedding.length).toBe(32);
    });
  });

  describe('ContentEmbeddingCache', () => {
    it('should cache embeddings', () => {
      const cache = new ContentEmbeddingCache(100);
      const content: ContentMetadata = {
        id: 'test-1',
        title: 'Test',
        type: 'movie',
        genres: ['action'],
        actors: [],
        directors: [],
        keywords: [],
      };

      const emb1 = cache.getOrCompute(content);
      const emb2 = cache.getOrCompute(content);

      // Should be the same reference (cached)
      expect(emb1).toBe(emb2);
      expect(cache.size()).toBe(1);
    });

    it('should evict oldest entries when full', () => {
      const cache = new ContentEmbeddingCache(2);

      for (let i = 0; i < 3; i++) {
        cache.getOrCompute({
          id: `test-${i}`,
          title: `Test ${i}`,
          type: 'movie',
          genres: ['action'],
          actors: [],
          directors: [],
          keywords: [],
        });
      }

      expect(cache.size()).toBe(2);
      // First entry should have been evicted
      expect(cache.get('test-0')).toBeUndefined();
    });
  });
});

describe('PreferenceLearningSystem', () => {
  let learner: PreferenceLearningSystem;

  beforeEach(() => {
    learner = new PreferenceLearningSystem();
  });

  describe('content management', () => {
    it('should add content and generate recommendations', () => {
      const content: ContentMetadata = {
        id: 'movie-1',
        title: 'Test Movie',
        type: 'movie',
        genres: ['action'],
        actors: [],
        directors: [],
        keywords: [],
      };

      learner.addContent(content);
      // Verify content was added by getting recommendations
      const recs = learner.getRecommendations(1);
      expect(recs).toHaveLength(1);
      expect(recs[0].contentId).toBe('movie-1');
    });

    it('should add multiple contents', () => {
      const contents: ContentMetadata[] = [
        { id: 'movie-1', title: 'Test 1', type: 'movie', genres: ['action'], actors: [], directors: [], keywords: [] },
        { id: 'movie-2', title: 'Test 2', type: 'movie', genres: ['comedy'], actors: [], directors: [], keywords: [] },
      ];

      learner.addContents(contents);
      const recs = learner.getRecommendations(2);
      expect(recs).toHaveLength(2);
    });
  });

  describe('Q-Learning', () => {
    it('should select action based on Q-values', () => {
      // Add some content first
      learner.addContent({
        id: 'movie-1',
        title: 'Action Movie',
        type: 'movie',
        genres: ['action'],
        actors: [],
        directors: [],
        keywords: [],
      });

      // Get current state and select action
      const state = learner.getCurrentState();
      const action = learner.selectAction(state);
      expect(action).toBeDefined();
      // Should be one of the valid actions
      const validActions: LearningAction[] = [
        'recommend_similar',
        'recommend_popular',
        'recommend_trending',
        'recommend_genre',
        'recommend_new_release',
        'recommend_continue_watching',
        'recommend_based_on_time',
        'explore_new_genre',
        'explore_new_type',
      ];
      expect(validActions).toContain(action);
    });

    it('should update Q-values after session', () => {
      // Add content
      learner.addContent({
        id: 'movie-1',
        title: 'Test Movie',
        type: 'movie',
        genres: ['action'],
        duration: 120,
        actors: [],
        directors: [],
        keywords: [],
      });

      const session: ViewingSession = {
        id: 'session-1',
        contentId: 'movie-1',
        contentMetadata: {
          id: 'movie-1',
          title: 'Test Movie',
          type: 'movie',
          genres: ['action'],
          actors: [],
          directors: [],
          keywords: [],
        },
        startTime: new Date(Date.now() - 60 * 60 * 1000).toISOString(),
        endTime: new Date().toISOString(),
        watchDuration: 60,
        completionRate: 0.5,
        implicit: { paused: 0, rewound: 0, fastForwarded: 0, volumeChanges: 0 },
      };

      const initialStats = learner.getStats();
      learner.recordSession(session, 'recommend_similar');
      const newStats = learner.getStats();

      expect(newStats.totalSessions).toBe(initialStats.totalSessions + 1);
    });
  });

  describe('recommendations', () => {
    it('should generate recommendations', () => {
      // Add multiple content items
      const contents: ContentMetadata[] = [
        { id: '1', title: 'Action 1', type: 'movie', genres: ['action'], actors: [], directors: [], keywords: [] },
        { id: '2', title: 'Action 2', type: 'movie', genres: ['action'], actors: [], directors: [], keywords: [] },
        { id: '3', title: 'Comedy 1', type: 'movie', genres: ['comedy'], actors: [], directors: [], keywords: [] },
      ];

      contents.forEach(c => learner.addContent(c));

      const recommendations = learner.getRecommendations(2);
      expect(recommendations).toHaveLength(2);
      expect(recommendations[0]).toHaveProperty('contentId');
      expect(recommendations[0]).toHaveProperty('score');
      expect(recommendations[0]).toHaveProperty('reason');
    });

    it('should exclude watched content from recommendations', () => {
      learner.addContent({
        id: 'movie-1',
        title: 'Watched Movie',
        type: 'movie',
        genres: ['action'],
        actors: [],
        directors: [],
        keywords: [],
      });
      learner.addContent({
        id: 'movie-2',
        title: 'Unwatched Movie',
        type: 'movie',
        genres: ['action'],
        actors: [],
        directors: [],
        keywords: [],
      });

      // Record a session for movie-1
      const session: ViewingSession = {
        id: 'session-1',
        contentId: 'movie-1',
        contentMetadata: {
          id: 'movie-1',
          title: 'Watched Movie',
          type: 'movie',
          genres: ['action'],
          actors: [],
          directors: [],
          keywords: [],
        },
        startTime: new Date().toISOString(),
        watchDuration: 120,
        completionRate: 1.0,
        implicit: { paused: 0, rewound: 0, fastForwarded: 0, volumeChanges: 0 },
      };
      learner.recordSession(session, 'recommend_similar');

      const recommendations = learner.getRecommendations(10);
      const watchedInRecs = recommendations.find(r => r.contentId === 'movie-1');
      expect(watchedInRecs).toBeUndefined();
    });
  });

  describe('preferences', () => {
    it('should learn genre preferences', () => {
      // Add content
      learner.addContent({
        id: 'movie-1',
        title: 'Action Movie',
        type: 'movie',
        genres: ['action', 'adventure'],
        actors: [],
        directors: [],
        keywords: [],
      });

      // Record high-engagement session
      const session: ViewingSession = {
        id: 'session-1',
        contentId: 'movie-1',
        contentMetadata: {
          id: 'movie-1',
          title: 'Action Movie',
          type: 'movie',
          genres: ['action', 'adventure'],
          actors: [],
          directors: [],
          keywords: [],
        },
        startTime: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
        endTime: new Date().toISOString(),
        watchDuration: 120,
        completionRate: 1.0,
        userRating: 5,
        implicit: { paused: 0, rewound: 0, fastForwarded: 0, volumeChanges: 0 },
      };

      learner.recordSession(session, 'recommend_similar');

      const prefs = learner.getPreferences();
      // Action or adventure should be in favorites after high-engagement session
      expect(
        prefs.favoriteGenres.includes('action') || prefs.favoriteGenres.includes('adventure')
      ).toBe(true);
    });
  });

  describe('model export/import', () => {
    it('should export and import model', () => {
      // Add content and record sessions
      learner.addContent({
        id: 'movie-1',
        title: 'Test',
        type: 'movie',
        genres: ['action'],
        actors: [],
        directors: [],
        keywords: [],
      });

      const session: ViewingSession = {
        id: 'session-1',
        contentId: 'movie-1',
        contentMetadata: {
          id: 'movie-1',
          title: 'Test',
          type: 'movie',
          genres: ['action'],
          actors: [],
          directors: [],
          keywords: [],
        },
        startTime: new Date().toISOString(),
        watchDuration: 60,
        completionRate: 0.5,
        implicit: { paused: 0, rewound: 0, fastForwarded: 0, volumeChanges: 0 },
      };
      learner.recordSession(session, 'recommend_similar');

      // Export model
      const exported = learner.exportModel();
      expect(exported).toHaveProperty('qTable');
      expect(exported).toHaveProperty('preferences');
      expect(exported).toHaveProperty('patterns');
      expect(exported).toHaveProperty('stats');

      // Verify export contains correct stats
      expect(exported.stats.episodeCount).toBe(1);

      // Import into new learner
      const newLearner = new PreferenceLearningSystem();
      newLearner.importModel(exported);

      // The exported stats are restored correctly
      const newStats = newLearner.getStats();
      // Note: sessions array is not exported/imported, but episodeCount is in stats
      expect(newStats.avgReward).toBeGreaterThan(0);
    });
  });

  describe('experience replay', () => {
    it('should perform experience replay', () => {
      // Add content and record multiple sessions
      learner.addContent({
        id: 'movie-1',
        title: 'Test',
        type: 'movie',
        genres: ['action'],
        actors: [],
        directors: [],
        keywords: [],
      });

      for (let i = 0; i < 10; i++) {
        const session: ViewingSession = {
          id: `session-${i}`,
          contentId: 'movie-1',
          contentMetadata: {
            id: 'movie-1',
            title: 'Test',
            type: 'movie',
            genres: ['action'],
            actors: [],
            directors: [],
            keywords: [],
          },
          startTime: new Date().toISOString(),
          watchDuration: 60 + i * 10,
          completionRate: 0.5 + i * 0.05,
          implicit: { paused: 0, rewound: 0, fastForwarded: 0, volumeChanges: 0 },
        };
        learner.recordSession(session, 'recommend_similar');
      }

      // Should not throw
      expect(() => learner.experienceReplay(5)).not.toThrow();
    });
  });

  describe('stats', () => {
    it('should return correct stats', () => {
      const stats = learner.getStats();

      expect(stats).toHaveProperty('totalSessions');
      expect(stats).toHaveProperty('totalPatterns');
      expect(stats).toHaveProperty('avgReward');
      expect(stats).toHaveProperty('explorationRate');
      expect(stats).toHaveProperty('topActions');
      expect(stats).toHaveProperty('learningProgress');

      expect(stats.explorationRate).toBeGreaterThan(0);
      expect(stats.explorationRate).toBeLessThanOrEqual(1);
    });
  });
});
