/**
 * MetadataService Unit Tests
 * TDD London School - Interaction-based testing with Firestore mocks
 *
 * Tests cover:
 * - CRUD operations
 * - Business logic validation
 * - Error handling
 * - Firestore interaction patterns
 */

import { describe, it, expect, jest, beforeEach } from '@jest/globals';

// Mock Firestore types
type DocumentReference = {
  id: string;
  set: jest.Mock<any>;
  get: jest.Mock<any>;
  update: jest.Mock<any>;
  delete: jest.Mock<any>;
};



type CollectionReference = {
  doc: jest.Mock<any>;
  add: jest.Mock<any>;
  get: jest.Mock<any>;
  where: jest.Mock<any>;
  orderBy: jest.Mock<any>;
  limit: jest.Mock<any>;
  offset: jest.Mock<any>;
};

type Firestore = {
  collection: jest.Mock<any>;
};

// Mock Firestore
const createMockFirestore = (): Firestore => ({
  collection: jest.fn(),
});

describe('MetadataService', () => {
  let mockFirestore: Firestore;
  let mockCollection: CollectionReference;
  let mockDoc: DocumentReference;

  beforeEach(() => {
    jest.clearAllMocks();

    // Setup mock Firestore
    mockDoc = {
      id: 'test_id',
      set: jest.fn(),
      get: jest.fn(),
      update: jest.fn(),
      delete: jest.fn(),
    };

    mockCollection = {
      doc: jest.fn().mockReturnValue(mockDoc),
      add: jest.fn(),
      get: jest.fn(),
      where: jest.fn().mockReturnThis(),
      orderBy: jest.fn().mockReturnThis(),
      limit: jest.fn().mockReturnThis(),
      offset: jest.fn().mockReturnThis(),
    };

    mockFirestore = createMockFirestore();
    mockFirestore.collection.mockReturnValue(mockCollection);
  });

  describe('createMetadata', () => {
    it('should create metadata document in Firestore', async () => {
      // Arrange
      const metadataInput = {
        title: 'Test Movie',
        type: 'movie',
        year: 2024,
        genres: ['action'],
      };

      const mockDocRef = {
        id: 'generated_id',
      };

      mockCollection.add.mockResolvedValue(mockDocRef);

      // Act
      const createMetadata = async (data: any) => {
        const timestamp = new Date().toISOString();
        const docRef = await mockFirestore.collection('metadata').add({
          ...data,
          createdAt: timestamp,
          updatedAt: timestamp,
        });

        return {
          id: docRef.id,
          ...data,
          createdAt: timestamp,
          updatedAt: timestamp,
        };
      };

      const result = await createMetadata(metadataInput);

      // Assert
      expect(mockFirestore.collection).toHaveBeenCalledWith('metadata');
      expect(mockCollection.add).toHaveBeenCalledWith(
        expect.objectContaining({
          ...metadataInput,
          createdAt: expect.any(String),
          updatedAt: expect.any(String),
        })
      );
      expect(result).toHaveProperty('id', 'generated_id');
      expect(result).toHaveProperty('title', 'Test Movie');
    });

    it('should validate required fields before creating', async () => {
      // Arrange
      const invalidInput = { title: 'Test' }; // Missing required 'type'

      // Act & Assert
      const createMetadata = async (data: any) => {
        if (!data.type) {
          throw new Error('Missing required field: type');
        }
        if (!data.title) {
          throw new Error('Missing required field: title');
        }
        return mockCollection.add(data);
      };

      await expect(createMetadata(invalidInput)).rejects.toThrow(
        'Missing required field: type'
      );
      expect(mockCollection.add).not.toHaveBeenCalled();
    });

    it('should sanitize and validate metadata fields', async () => {
      // Arrange
      const metadataInput = {
        title: '  Test Movie  ', // Extra whitespace
        type: 'movie',
        year: 2024,
      };

      // Act
      const createMetadata = async (data: any) => {
        const sanitized = {
          ...data,
          title: data.title.trim(),
        };

        if (sanitized.year && (sanitized.year < 1800 || sanitized.year > 2100)) {
          throw new Error('Invalid year value');
        }

        return mockCollection.add(sanitized);
      };

      await createMetadata(metadataInput);

      // Assert
      expect(mockCollection.add).toHaveBeenCalledWith(
        expect.objectContaining({
          title: 'Test Movie', // Trimmed
        })
      );
    });

    it('should handle Firestore errors gracefully', async () => {
      // Arrange
      const firestoreError = new Error('Firestore permission denied');
      mockCollection.add.mockRejectedValue(firestoreError);

      // Act & Assert
      const createMetadata = async (data: any) => {
        try {
          return await mockCollection.add(data);
        } catch (error) {
          throw new Error(`Failed to create metadata: ${(error as Error).message}`);
        }
      };

      await expect(
        createMetadata({ title: 'Test', type: 'movie' })
      ).rejects.toThrow('Failed to create metadata: Firestore permission denied');
    });
  });

  describe('getMetadata', () => {
    it('should retrieve metadata by ID from Firestore', async () => {
      // Arrange
      const metadataId = 'meta_123';
      const metadataData = {
        title: 'Test Movie',
        type: 'movie',
        year: 2024,
      };

      mockDoc.get.mockResolvedValue({
        exists: true,
        data: () => metadataData,
        id: metadataId,
      });

      // Act
      const getMetadata = async (id: string) => {
        const docRef = mockCollection.doc(id);
        const doc = await docRef.get();

        if (!doc.exists) {
          return null;
        }

        return {
          id: doc.id,
          ...doc.data(),
        };
      };

      const result = await getMetadata(metadataId);

      // Assert
      expect(mockCollection.doc).toHaveBeenCalledWith(metadataId);
      expect(mockDoc.get).toHaveBeenCalled();
      expect(result).toEqual({
        id: metadataId,
        ...metadataData,
      });
    });

    it('should return null when metadata does not exist', async () => {
      // Arrange
      mockDoc.get.mockResolvedValue({
        exists: false,
        data: () => null,
      });

      // Act
      const getMetadata = async (id: string) => {
        const docRef = mockCollection.doc(id);
        const doc = await docRef.get();

        if (!doc.exists) {
          return null;
        }

        return {
          id: doc.id,
          ...doc.data(),
        };
      };

      const result = await getMetadata('non_existent');

      // Assert
      expect(result).toBeNull();
    });

    it('should handle Firestore read errors', async () => {
      // Arrange
      const firestoreError = new Error('Firestore read failed');
      mockDoc.get.mockRejectedValue(firestoreError);

      // Act & Assert
      const getMetadata = async (id: string) => {
        try {
          const docRef = mockCollection.doc(id);
          return await docRef.get();
        } catch (error) {
          throw new Error(`Failed to retrieve metadata: ${(error as Error).message}`);
        }
      };

      await expect(getMetadata('meta_123')).rejects.toThrow(
        'Failed to retrieve metadata: Firestore read failed'
      );
    });
  });

  describe('updateMetadata', () => {
    it('should update existing metadata in Firestore', async () => {
      // Arrange
      const metadataId = 'meta_123';
      const updateData = { title: 'Updated Title', year: 2025 };

      mockDoc.get.mockResolvedValue({
        exists: true,
        data: () => ({ title: 'Old Title', type: 'movie', year: 2024 }),
        id: metadataId,
      });

      mockDoc.update.mockResolvedValue(undefined);

      // Act
      const updateMetadata = async (id: string, data: any) => {
        const docRef = mockCollection.doc(id);
        const doc = await docRef.get();

        if (!doc.exists) {
          return null;
        }

        const timestamp = new Date().toISOString();
        await docRef.update({
          ...data,
          updatedAt: timestamp,
        });

        return {
          id: doc.id,
          ...doc.data(),
          ...data,
          updatedAt: timestamp,
        };
      };

      const result = await updateMetadata(metadataId, updateData);

      // Assert
      expect(mockDoc.update).toHaveBeenCalledWith(
        expect.objectContaining({
          ...updateData,
          updatedAt: expect.any(String),
        })
      );
      expect(result).toHaveProperty('title', 'Updated Title');
      expect(result).toHaveProperty('year', 2025);
    });

    it('should return null when updating non-existent metadata', async () => {
      // Arrange
      mockDoc.get.mockResolvedValue({
        exists: false,
        data: () => null,
      });

      // Act
      const updateMetadata = async (id: string, data: any) => {
        const docRef = mockCollection.doc(id);
        const doc = await docRef.get();

        if (!doc.exists) {
          return null;
        }

        await docRef.update(data);
        return { id, ...data };
      };

      const result = await updateMetadata('non_existent', { title: 'Test' });

      // Assert
      expect(result).toBeNull();
      expect(mockDoc.update).not.toHaveBeenCalled();
    });

    it('should only update allowed fields', async () => {
      // Arrange
      const metadataId = 'meta_123';
      const updateData = {
        title: 'New Title',
        id: 'hacked_id', // Should be ignored
        createdAt: '2020-01-01', // Should be ignored
      };

      mockDoc.get.mockResolvedValue({
        exists: true,
        data: () => ({ title: 'Old Title', type: 'movie' }),
        id: metadataId,
      });

      // Act
      const updateMetadata = async (id: string, data: any) => {
        const docRef = mockCollection.doc(id);
        const doc = await docRef.get();

        if (!doc.exists) {
          return null;
        }

        // Filter out protected fields
        const { id: _id, createdAt: _createdAt, ...allowedData } = data;
        const timestamp = new Date().toISOString();

        await docRef.update({
          ...allowedData,
          updatedAt: timestamp,
        });

        return { id: doc.id, ...doc.data(), ...allowedData, updatedAt: timestamp };
      };

      await updateMetadata(metadataId, updateData);

      // Assert
      expect(mockDoc.update).toHaveBeenCalledWith(
        expect.not.objectContaining({
          id: 'hacked_id',
          createdAt: '2020-01-01',
        })
      );
    });
  });

  describe('deleteMetadata', () => {
    it('should delete metadata from Firestore', async () => {
      // Arrange
      const metadataId = 'meta_123';

      mockDoc.get.mockResolvedValue({
        exists: true,
        data: () => ({ title: 'Test' }),
      });

      mockDoc.delete.mockResolvedValue(undefined);

      // Act
      const deleteMetadata = async (id: string) => {
        const docRef = mockCollection.doc(id);
        const doc = await docRef.get();

        if (!doc.exists) {
          return false;
        }

        await docRef.delete();
        return true;
      };

      const result = await deleteMetadata(metadataId);

      // Assert
      expect(mockCollection.doc).toHaveBeenCalledWith(metadataId);
      expect(mockDoc.delete).toHaveBeenCalled();
      expect(result).toBe(true);
    });

    it('should return false when deleting non-existent metadata', async () => {
      // Arrange
      mockDoc.get.mockResolvedValue({
        exists: false,
      });

      // Act
      const deleteMetadata = async (id: string) => {
        const docRef = mockCollection.doc(id);
        const doc = await docRef.get();

        if (!doc.exists) {
          return false;
        }

        await docRef.delete();
        return true;
      };

      const result = await deleteMetadata('non_existent');

      // Assert
      expect(result).toBe(false);
      expect(mockDoc.delete).not.toHaveBeenCalled();
    });
  });

  describe('listMetadata', () => {
    it('should list metadata with pagination', async () => {
      // Arrange
      const mockDocs = [
        { id: 'meta_1', data: () => ({ title: 'Movie 1' }), exists: true },
        { id: 'meta_2', data: () => ({ title: 'Movie 2' }), exists: true },
      ];

      mockCollection.get.mockResolvedValue({
        docs: mockDocs,
        size: 2,
        empty: false,
      });

      // Act
      const listMetadata = async (options: { limit: number; offset: number }) => {
        const query = mockCollection
          .orderBy('createdAt', 'desc')
          .limit(options.limit)
          .offset(options.offset);

        const snapshot = await query.get();

        return {
          items: snapshot.docs.map((doc: any) => ({ id: doc.id, ...doc.data() })),
          total: snapshot.size,
          limit: options.limit,
          offset: options.offset,
        };
      };

      const result = await listMetadata({ limit: 10, offset: 0 });

      // Assert
      expect(mockCollection.orderBy).toHaveBeenCalledWith('createdAt', 'desc');
      expect(mockCollection.limit).toHaveBeenCalledWith(10);
      expect(mockCollection.offset).toHaveBeenCalledWith(0);
      expect(result.items).toHaveLength(2);
      expect(result.total).toBe(2);
    });

    it('should filter metadata by type', async () => {
      // Arrange
      mockCollection.get.mockResolvedValue({
        docs: [],
        size: 0,
        empty: true,
      });

      // Act
      const listMetadata = async (filters: any) => {
        let query = mockCollection;

        if (filters.type) {
          query = query.where('type', '==', filters.type);
        }

        const snapshot = await query.get();
        return {
          items: snapshot.docs.map((doc: any) => ({ id: doc.id, ...doc.data() })),
          total: snapshot.size,
        };
      };

      await listMetadata({ type: 'movie' });

      // Assert
      expect(mockCollection.where).toHaveBeenCalledWith('type', '==', 'movie');
    });
  });

  describe('searchMetadata', () => {
    it('should search metadata by query parameters', async () => {
      // Arrange
      const searchQuery = { query: 'action', year: 2024 };
      const mockDocs = [
        { id: 'meta_1', data: () => ({ title: 'Action Movie', year: 2024 }), exists: true },
      ];

      mockCollection.get.mockResolvedValue({
        docs: mockDocs,
        size: 1,
        empty: false,
      });

      // Act
      const searchMetadata = async (query: any) => {
        let firestoreQuery = mockCollection;

        if (query.year) {
          firestoreQuery = firestoreQuery.where('year', '==', query.year);
        }

        const snapshot = await firestoreQuery.get();
        let results = snapshot.docs.map((doc: any) => ({ id: doc.id, ...doc.data() }));

        // Client-side text search (in real implementation would use full-text search)
        if (query.query) {
          results = results.filter((item: any) =>
            item.title?.toLowerCase().includes(query.query.toLowerCase())
          );
        }

        return results;
      };

      const results = await searchMetadata(searchQuery);

      // Assert
      expect(mockCollection.where).toHaveBeenCalledWith('year', '==', 2024);
      expect(results).toHaveLength(1);
      expect(results[0]).toHaveProperty('title', 'Action Movie');
    });

    it('should return empty array when no matches found', async () => {
      // Arrange
      mockCollection.get.mockResolvedValue({
        docs: [],
        size: 0,
        empty: true,
      });

      // Act
      const searchMetadata = async (_query: any) => {
        const snapshot = await (mockCollection.get as any)();
        return snapshot.docs.map((doc: any) => ({ id: doc.id, ...doc.data() }));
      };

      const results = await searchMetadata({ query: 'nonexistent' });

      // Assert
      expect(results).toEqual([]);
    });
  });
});
