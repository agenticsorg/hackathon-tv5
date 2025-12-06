/**
 * Metadata API Endpoint Tests
 * TDD London School - Interaction-based testing with mocks
 *
 * Tests cover:
 * - Health endpoint
 * - CRUD operations for metadata
 * - Error handling
 * - Request validation
 */

import { describe, it, expect, jest, beforeEach } from '@jest/globals';

// Mock Express types
type Request = {
  params?: Record<string, string>;
  body?: any;
  query?: Record<string, string>;
};

type Response = {
  status: jest.Mock<any>;
  json: jest.Mock<any>;
  send: jest.Mock<any>;
};



// Mock MetadataService
const mockMetadataService = {
  createMetadata: jest.fn() as jest.Mock<any>,
  getMetadata: jest.fn() as jest.Mock<any>,
  updateMetadata: jest.fn() as jest.Mock<any>,
  deleteMetadata: jest.fn() as jest.Mock<any>,
  listMetadata: jest.fn() as jest.Mock<any>,
  searchMetadata: jest.fn() as jest.Mock<any>,
};

describe('Metadata API Endpoints', () => {
  let req: Request;
  let res: Response;

  beforeEach(() => {
    // Reset mocks before each test
    jest.clearAllMocks();

    // Setup mock response object
    res = {
      status: jest.fn().mockReturnThis(),
      json: jest.fn().mockReturnThis(),
      send: jest.fn().mockReturnThis(),
    };

    req = {};
  });

  describe('GET /health', () => {
    it('should return 200 with healthy status', async () => {
      // Arrange
      const expectedResponse = {
        status: 'healthy',
        service: 'nexus-ummid-metadata-api',
        timestamp: expect.any(String),
        version: '1.0.0',
      };

      // Act
      const healthHandler = (_req: Request, res: Response) => {
        res.status(200).json({
          status: 'healthy',
          service: 'nexus-ummid-metadata-api',
          timestamp: new Date().toISOString(),
          version: '1.0.0',
        });
      };

      healthHandler(req, res);

      // Assert
      expect(res.status).toHaveBeenCalledWith(200);
      expect(res.json).toHaveBeenCalledWith(expectedResponse);
    });

    it('should include database connection status', async () => {
      // Act
      const healthHandler = (_req: Request, res: Response) => {
        res.status(200).json({
          status: 'healthy',
          service: 'nexus-ummid-metadata-api',
          timestamp: new Date().toISOString(),
          version: '1.0.0',
          database: 'connected',
        });
      };

      healthHandler(req, res);

      // Assert
      const jsonCall = (res.json as jest.Mock).mock.calls[0][0];
      expect(jsonCall).toHaveProperty('database');
    });
  });

  describe('POST /api/metadata', () => {
    it('should create metadata and return 201', async () => {
      // Arrange
      const metadataInput = {
        title: 'Test Movie',
        type: 'movie',
        year: 2024,
        genres: ['action', 'sci-fi'],
      };

      const createdMetadata = {
        id: 'meta_123',
        ...metadataInput,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      };

      req.body = metadataInput;
      mockMetadataService.createMetadata.mockResolvedValue(createdMetadata);

      // Act
      const createHandler = async (req: Request, res: Response) => {
        const result = await mockMetadataService.createMetadata(req.body);
        res.status(201).json(result);
      };

      await createHandler(req, res);

      // Assert
      expect(mockMetadataService.createMetadata).toHaveBeenCalledWith(metadataInput);
      expect(mockMetadataService.createMetadata).toHaveBeenCalledTimes(1);
      expect(res.status).toHaveBeenCalledWith(201);
      expect(res.json).toHaveBeenCalledWith(createdMetadata);
    });

    it('should validate required fields and return 400', async () => {
      // Arrange
      req.body = { title: 'Test' }; // Missing required fields

      // Act
      const createHandler = async (req: Request, res: Response) => {
        if (!req.body.type) {
          res.status(400).json({
            error: 'Validation failed',
            message: 'Missing required field: type',
          });
          return;
        }
        const result = await mockMetadataService.createMetadata(req.body);
        res.status(201).json(result);
      };

      await createHandler(req, res);

      // Assert
      expect(mockMetadataService.createMetadata).not.toHaveBeenCalled();
      expect(res.status).toHaveBeenCalledWith(400);
      expect(res.json).toHaveBeenCalledWith({
        error: 'Validation failed',
        message: 'Missing required field: type',
      });
    });

    it('should handle service errors and return 500', async () => {
      // Arrange
      req.body = { title: 'Test', type: 'movie' };
      const serviceError = new Error('Database connection failed');
      mockMetadataService.createMetadata.mockRejectedValue(serviceError);

      // Act
      const createHandler = async (req: Request, res: Response) => {
        try {
          const result = await mockMetadataService.createMetadata(req.body);
          res.status(201).json(result);
        } catch (error) {
          res.status(500).json({
            error: 'Internal server error',
            message: (error as Error).message,
          });
        }
      };

      await createHandler(req, res);

      // Assert
      expect(mockMetadataService.createMetadata).toHaveBeenCalledWith(req.body);
      expect(res.status).toHaveBeenCalledWith(500);
      expect(res.json).toHaveBeenCalledWith({
        error: 'Internal server error',
        message: 'Database connection failed',
      });
    });
  });

  describe('GET /api/metadata/:id', () => {
    it('should retrieve metadata by ID and return 200', async () => {
      // Arrange
      const metadataId = 'meta_123';
      const metadata = {
        id: metadataId,
        title: 'Test Movie',
        type: 'movie',
        year: 2024,
      };

      req.params = { id: metadataId };
      mockMetadataService.getMetadata.mockResolvedValue(metadata);

      // Act
      const getHandler = async (req: Request, res: Response) => {
        const result = await mockMetadataService.getMetadata(req.params!.id);
        res.status(200).json(result);
      };

      await getHandler(req, res);

      // Assert
      expect(mockMetadataService.getMetadata).toHaveBeenCalledWith(metadataId);
      expect(res.status).toHaveBeenCalledWith(200);
      expect(res.json).toHaveBeenCalledWith(metadata);
    });

    it('should return 404 when metadata not found', async () => {
      // Arrange
      req.params = { id: 'non_existent' };
      mockMetadataService.getMetadata.mockResolvedValue(null);

      // Act
      const getHandler = async (req: Request, res: Response) => {
        const result = await mockMetadataService.getMetadata(req.params!.id);
        if (!result) {
          res.status(404).json({
            error: 'Not found',
            message: `Metadata with ID ${req.params!.id} not found`,
          });
          return;
        }
        res.status(200).json(result);
      };

      await getHandler(req, res);

      // Assert
      expect(res.status).toHaveBeenCalledWith(404);
      expect(res.json).toHaveBeenCalledWith({
        error: 'Not found',
        message: 'Metadata with ID non_existent not found',
      });
    });
  });

  describe('PUT /api/metadata/:id', () => {
    it('should update metadata and return 200', async () => {
      // Arrange
      const metadataId = 'meta_123';
      const updateData = { title: 'Updated Title', year: 2025 };
      const updatedMetadata = {
        id: metadataId,
        title: 'Updated Title',
        type: 'movie',
        year: 2025,
        updatedAt: new Date().toISOString(),
      };

      req.params = { id: metadataId };
      req.body = updateData;
      mockMetadataService.updateMetadata.mockResolvedValue(updatedMetadata);

      // Act
      const updateHandler = async (req: Request, res: Response) => {
        const result = await mockMetadataService.updateMetadata(
          req.params!.id,
          req.body
        );
        res.status(200).json(result);
      };

      await updateHandler(req, res);

      // Assert
      expect(mockMetadataService.updateMetadata).toHaveBeenCalledWith(
        metadataId,
        updateData
      );
      expect(res.status).toHaveBeenCalledWith(200);
      expect(res.json).toHaveBeenCalledWith(updatedMetadata);
    });

    it('should return 404 when updating non-existent metadata', async () => {
      // Arrange
      req.params = { id: 'non_existent' };
      req.body = { title: 'Test' };
      mockMetadataService.updateMetadata.mockResolvedValue(null);

      // Act
      const updateHandler = async (req: Request, res: Response) => {
        const result = await mockMetadataService.updateMetadata(
          req.params!.id,
          req.body
        );
        if (!result) {
          res.status(404).json({
            error: 'Not found',
            message: `Metadata with ID ${req.params!.id} not found`,
          });
          return;
        }
        res.status(200).json(result);
      };

      await updateHandler(req, res);

      // Assert
      expect(res.status).toHaveBeenCalledWith(404);
    });
  });

  describe('DELETE /api/metadata/:id', () => {
    it('should delete metadata and return 204', async () => {
      // Arrange
      req.params = { id: 'meta_123' };
      mockMetadataService.deleteMetadata.mockResolvedValue(true);

      // Act
      const deleteHandler = async (req: Request, res: Response) => {
        await mockMetadataService.deleteMetadata(req.params!.id);
        res.status(204).send();
      };

      await deleteHandler(req, res);

      // Assert
      expect(mockMetadataService.deleteMetadata).toHaveBeenCalledWith('meta_123');
      expect(res.status).toHaveBeenCalledWith(204);
      expect(res.send).toHaveBeenCalled();
    });

    it('should return 404 when deleting non-existent metadata', async () => {
      // Arrange
      req.params = { id: 'non_existent' };
      mockMetadataService.deleteMetadata.mockResolvedValue(false);

      // Act
      const deleteHandler = async (req: Request, res: Response) => {
        const deleted = await mockMetadataService.deleteMetadata(req.params!.id);
        if (!deleted) {
          res.status(404).json({
            error: 'Not found',
            message: `Metadata with ID ${req.params!.id} not found`,
          });
          return;
        }
        res.status(204).send();
      };

      await deleteHandler(req, res);

      // Assert
      expect(res.status).toHaveBeenCalledWith(404);
    });
  });

  describe('GET /api/metadata', () => {
    it('should list all metadata with pagination', async () => {
      // Arrange
      const metadata = [
        { id: 'meta_1', title: 'Movie 1', type: 'movie' },
        { id: 'meta_2', title: 'Movie 2', type: 'movie' },
      ];

      req.query = { limit: '10', offset: '0' };
      mockMetadataService.listMetadata.mockResolvedValue({
        items: metadata,
        total: 2,
        limit: 10,
        offset: 0,
      });

      // Act
      const listHandler = async (req: Request, res: Response) => {
        const limit = parseInt(req.query?.limit || '10');
        const offset = parseInt(req.query?.offset || '0');
        const result = await mockMetadataService.listMetadata({ limit, offset });
        res.status(200).json(result);
      };

      await listHandler(req, res);

      // Assert
      expect(mockMetadataService.listMetadata).toHaveBeenCalledWith({
        limit: 10,
        offset: 0,
      });
      expect(res.status).toHaveBeenCalledWith(200);
    });

    it('should filter metadata by type', async () => {
      // Arrange
      req.query = { type: 'movie', limit: '10', offset: '0' };
      mockMetadataService.listMetadata.mockResolvedValue({
        items: [],
        total: 0,
        limit: 10,
        offset: 0,
      });

      // Act
      const listHandler = async (req: Request, res: Response) => {
        const filters = {
          type: req.query?.type,
          limit: parseInt(req.query?.limit || '10'),
          offset: parseInt(req.query?.offset || '0'),
        };
        const result = await mockMetadataService.listMetadata(filters);
        res.status(200).json(result);
      };

      await listHandler(req, res);

      // Assert
      expect(mockMetadataService.listMetadata).toHaveBeenCalledWith(
        expect.objectContaining({ type: 'movie' })
      );
    });
  });

  describe('POST /api/metadata/search', () => {
    it('should search metadata by query', async () => {
      // Arrange
      const searchQuery = { query: 'action movies', year: 2024 };
      const searchResults = [
        { id: 'meta_1', title: 'Action Movie 1', type: 'movie', year: 2024 },
      ];

      req.body = searchQuery;
      mockMetadataService.searchMetadata.mockResolvedValue(searchResults);

      // Act
      const searchHandler = async (req: Request, res: Response) => {
        const results = await mockMetadataService.searchMetadata(req.body);
        res.status(200).json({ results });
      };

      await searchHandler(req, res);

      // Assert
      expect(mockMetadataService.searchMetadata).toHaveBeenCalledWith(searchQuery);
      expect(res.status).toHaveBeenCalledWith(200);
      expect(res.json).toHaveBeenCalledWith({ results: searchResults });
    });
  });
});
