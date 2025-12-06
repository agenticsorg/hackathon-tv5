/**
 * VectorStore Unit Tests
 */

import { VectorStore } from '../../../src/content/vector-store';

describe('VectorStore', () => {
  let store: VectorStore;

  beforeEach(() => {
    store = new VectorStore();
  });

  describe('upsert', () => {
    it('should store vector with metadata', async () => {
      const id = 'test_001';
      const vector = new Float32Array(1536);
      vector.fill(0.5);
      const metadata = { title: 'Test', category: 'movie' };

      await store.upsert(id, vector, metadata);

      const results = await store.search(vector, 1);
      expect(results.length).toBe(1);
      expect(results[0].id).toBe(id);
    });

    it('should update existing vector', async () => {
      const id = 'test_001';
      const vector1 = new Float32Array(1536);
      vector1.fill(0.5);
      const vector2 = new Float32Array(1536);
      vector2.fill(0.8);

      await store.upsert(id, vector1, {});
      await store.upsert(id, vector2, { updated: true });

      const results = await store.search(vector2, 1);
      expect(results.length).toBe(1);
      expect(results[0].metadata.updated).toBe(true);
    });
  });

  describe('search', () => {
    beforeEach(async () => {
      // Seed some vectors
      for (let i = 0; i < 10; i++) {
        const vector = new Float32Array(1536);
        vector.fill(i * 0.1);
        await store.upsert(`test_${i}`, vector, { index: i });
      }
    });

    it('should return results with cosine similarity', async () => {
      const queryVector = new Float32Array(1536);
      queryVector.fill(0.5);

      const results = await store.search(queryVector, 5);

      expect(results.length).toBeLessThanOrEqual(5);
      results.forEach(result => {
        expect(result.score).toBeGreaterThanOrEqual(0);
        expect(result.score).toBeLessThanOrEqual(1);
      });
    });

    it('should return results sorted by similarity', async () => {
      const queryVector = new Float32Array(1536);
      queryVector.fill(0.5);

      const results = await store.search(queryVector, 5);

      for (let i = 0; i < results.length - 1; i++) {
        expect(results[i].score).toBeGreaterThanOrEqual(results[i + 1].score);
      }
    });

    it('should limit results to topK', async () => {
      const queryVector = new Float32Array(1536);
      queryVector.fill(0.5);

      const results = await store.search(queryVector, 3);

      expect(results.length).toBeLessThanOrEqual(3);
    });
  });
});
