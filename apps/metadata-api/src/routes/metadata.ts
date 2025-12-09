/**
 * Metadata Routes
 * RESTful API endpoints for metadata operations
 */

import { Router, Request, Response } from 'express';
import { body, param, query, validationResult } from 'express-validator';
import { MetadataService } from '../services/MetadataService';
import { ApiResponse, PaginationParams } from '../types';

const router = Router();
const metadataService = new MetadataService();

/**
 * Helper function to send API response
 */
function sendResponse<T>(res: Response, data?: T, error?: any): void {
  const response: ApiResponse<T> = {
    success: !error,
    data,
    error: error ? {
      code: error.code || 'INTERNAL_ERROR',
      message: error.message || 'An error occurred',
      details: error.details
    } : undefined,
    metadata: {
      timestamp: new Date(),
      requestId: Math.random().toString(36).substr(2, 9)
    }
  };

  const statusCode = error ? (error.statusCode || 500) : 200;
  res.status(statusCode).json(response);
}

/**
 * GET /api/v1/metadata
 * Get all metadata with pagination
 */
router.get(
  '/',
  [
    query('page').optional().isInt({ min: 1 }).toInt(),
    query('limit').optional().isInt({ min: 1, max: 100 }).toInt()
  ],
  async (req: Request, res: Response) => {
    try {
      const errors = validationResult(req);
      if (!errors.isEmpty()) {
        return sendResponse(res, undefined, {
          code: 'VALIDATION_ERROR',
          message: 'Invalid query parameters',
          details: errors.array(),
          statusCode: 400
        });
      }

      const params: PaginationParams = {
        page: parseInt(req.query.page as string) || 1,
        limit: parseInt(req.query.limit as string) || 10,
        sortBy: req.query.sortBy as string,
        sortOrder: (req.query.sortOrder as 'asc' | 'desc') || 'desc'
      };

      const result = await metadataService.getAll(params);
      sendResponse(res, result);
    } catch (error: any) {
      sendResponse(res, undefined, error);
    }
  }
);

/**
 * GET /api/v1/metadata/:id
 * Get metadata by ID
 */
router.get(
  '/:id',
  [param('id').notEmpty().trim()],
  async (req: Request, res: Response) => {
    try {
      const errors = validationResult(req);
      if (!errors.isEmpty()) {
        return sendResponse(res, undefined, {
          code: 'VALIDATION_ERROR',
          message: 'Invalid asset ID',
          statusCode: 400
        });
      }

      const { id } = req.params;
      const metadata = await metadataService.getById(id);

      if (!metadata) {
        return sendResponse(res, undefined, {
          code: 'NOT_FOUND',
          message: `Asset not found: ${id}`,
          statusCode: 404
        });
      }

      sendResponse(res, metadata);
    } catch (error: any) {
      sendResponse(res, undefined, error);
    }
  }
);

/**
 * POST /api/v1/metadata
 * Create new metadata
 */
router.post(
  '/',
  [
    body('title').notEmpty().trim().isString(),
    body('type').isIn(['movie', 'series', 'episode', 'documentary', 'short']),
    body('genres').isArray({ min: 1 }),
    body('language').notEmpty().isString()
  ],
  async (req: Request, res: Response) => {
    try {
      const errors = validationResult(req);
      if (!errors.isEmpty()) {
        return sendResponse(res, undefined, {
          code: 'VALIDATION_ERROR',
          message: 'Invalid metadata',
          details: errors.array(),
          statusCode: 400
        });
      }

      const metadata = await metadataService.create(req.body);
      sendResponse(res, metadata);
    } catch (error: any) {
      sendResponse(res, undefined, error);
    }
  }
);

/**
 * PUT /api/v1/metadata/:id
 * Update metadata
 */
router.put(
  '/:id',
  [
    param('id').notEmpty().trim(),
    body('title').optional().trim().isString(),
    body('type').optional().isIn(['movie', 'series', 'episode', 'documentary', 'short']),
    body('genres').optional().isArray(),
    body('language').optional().isString()
  ],
  async (req: Request, res: Response) => {
    try {
      const errors = validationResult(req);
      if (!errors.isEmpty()) {
        return sendResponse(res, undefined, {
          code: 'VALIDATION_ERROR',
          message: 'Invalid update data',
          details: errors.array(),
          statusCode: 400
        });
      }

      const { id } = req.params;
      const metadata = await metadataService.update(id, req.body);

      if (!metadata) {
        return sendResponse(res, undefined, {
          code: 'NOT_FOUND',
          message: `Asset not found: ${id}`,
          statusCode: 404
        });
      }

      sendResponse(res, metadata);
    } catch (error: any) {
      sendResponse(res, undefined, error);
    }
  }
);

/**
 * DELETE /api/v1/metadata/:id
 * Delete metadata
 */
router.delete(
  '/:id',
  [param('id').notEmpty().trim()],
  async (req: Request, res: Response) => {
    try {
      const errors = validationResult(req);
      if (!errors.isEmpty()) {
        return sendResponse(res, undefined, {
          code: 'VALIDATION_ERROR',
          message: 'Invalid asset ID',
          statusCode: 400
        });
      }

      const { id } = req.params;
      const deleted = await metadataService.delete(id);

      if (!deleted) {
        return sendResponse(res, undefined, {
          code: 'NOT_FOUND',
          message: `Asset not found: ${id}`,
          statusCode: 404
        });
      }

      sendResponse(res, { deleted: true, id });
    } catch (error: any) {
      sendResponse(res, undefined, error);
    }
  }
);

/**
 * GET /api/v1/metadata/search
 * Search metadata
 */
router.get(
  '/search/query',
  [
    query('q').notEmpty().trim(),
    query('limit').optional().isInt({ min: 1, max: 100 }).toInt()
  ],
  async (req: Request, res: Response) => {
    try {
      const errors = validationResult(req);
      if (!errors.isEmpty()) {
        return sendResponse(res, undefined, {
          code: 'VALIDATION_ERROR',
          message: 'Invalid search query',
          details: errors.array(),
          statusCode: 400
        });
      }

      const query = req.query.q as string;
      const limit = parseInt(req.query.limit as string) || 10;

      const results = await metadataService.search(query, limit);
      sendResponse(res, results);
    } catch (error: any) {
      sendResponse(res, undefined, error);
    }
  }
);

/**
 * POST /api/v1/metadata/:id/enrich
 * Enrich metadata with AI
 */
router.post(
  '/:id/enrich',
  [
    param('id').notEmpty().trim(),
    body('fields').optional().isArray(),
    body('model').optional().isIn(['gemini-2.0', 'claude-3.5', 'gpt-4'])
  ],
  async (req: Request, res: Response) => {
    try {
      const errors = validationResult(req);
      if (!errors.isEmpty()) {
        return sendResponse(res, undefined, {
          code: 'VALIDATION_ERROR',
          message: 'Invalid enrichment request',
          details: errors.array(),
          statusCode: 400
        });
      }

      const { id } = req.params;
      const enrichmentRequest = {
        assetId: id,
        fields: req.body.fields,
        model: req.body.model,
        approach: req.body.approach
      };

      const result = await metadataService.enrich(enrichmentRequest);
      sendResponse(res, result);
    } catch (error: any) {
      sendResponse(res, undefined, error);
    }
  }
);

/**
 * POST /api/v1/metadata/:id/validate
 * Validate metadata against platform requirements
 */
router.post(
  '/:id/validate',
  [
    param('id').notEmpty().trim(),
    body('platform').notEmpty().isString()
  ],
  async (req: Request, res: Response) => {
    try {
      const errors = validationResult(req);
      if (!errors.isEmpty()) {
        return sendResponse(res, undefined, {
          code: 'VALIDATION_ERROR',
          message: 'Invalid validation request',
          details: errors.array(),
          statusCode: 400
        });
      }

      const { id } = req.params;
      const { platform } = req.body;

      const result = await metadataService.validate(id, platform);
      sendResponse(res, result);
    } catch (error: any) {
      sendResponse(res, undefined, error);
    }
  }
);

export default router;
