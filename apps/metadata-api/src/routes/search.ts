/**
 * Search Routes
 * RESTful API endpoints for semantic search operations
 */

import { Router, Request, Response } from 'express';
import { query, param, validationResult } from 'express-validator';
import { HybridSearchService, SearchOptions } from '../search';
import { ApiResponse } from '../types';

const router = Router();

// Initialize search service (singleton)
let searchService: HybridSearchService | null = null;

/**
 * Initialize search service
 */
async function getSearchService(): Promise<HybridSearchService> {
  if (!searchService) {
    searchService = new HybridSearchService({
      ruvectorConfig: {
        dimension: parseInt(process.env.VECTOR_DIMENSION || '384'),
        metric: 'cosine',
        maxElements: 100000
      },
      enableCache: true,
      cacheTTL: 300 // 5 minutes
    });

    await searchService.initialize();
  }

  return searchService;
}

/**
 * Helper function to send API response
 */
function sendResponse<T>(res: Response, data?: T, error?: any): void {
  const response: ApiResponse<T> = {
    success: !error,
    data,
    error: error
      ? {
          code: error.code || 'INTERNAL_ERROR',
          message: error.message || 'An error occurred',
          details: error.details
        }
      : undefined,
    metadata: {
      timestamp: new Date(),
      requestId: Math.random().toString(36).substr(2, 9)
    }
  };

  const statusCode = error ? error.statusCode || 500 : 200;
  res.status(statusCode).json(response);
}

/**
 * GET /api/v1/search
 * Semantic search for content
 *
 * Query params:
 * - q: Search query (required)
 * - limit: Number of results (default: 10, max: 100)
 * - threshold: Similarity threshold (0-1)
 * - backends: Comma-separated list of backends (ruvector,vertex-ai)
 * - type: Filter by content type
 * - genres: Filter by genres (comma-separated)
 */
router.get(
  '/',
  [
    query('q').notEmpty().trim().withMessage('Query parameter "q" is required'),
    query('limit').optional().isInt({ min: 1, max: 100 }).toInt(),
    query('threshold').optional().isFloat({ min: 0, max: 1 }).toFloat()
  ],
  async (req: Request, res: Response) => {
    try {
      const errors = validationResult(req);
      if (!errors.isEmpty()) {
        return sendResponse(res, undefined, {
          code: 'VALIDATION_ERROR',
          message: 'Invalid search parameters',
          details: errors.array(),
          statusCode: 400
        });
      }

      const searchQuery = req.query.q as string;
      const limit = parseInt(req.query.limit as string) || 10;
      const threshold = req.query.threshold ? parseFloat(req.query.threshold as string) : undefined;

      // Parse backends
      const backends = req.query.backends
        ? (req.query.backends as string).split(',').map(b => b.trim() as 'ruvector' | 'vertex-ai')
        : ['ruvector'];

      // Parse filters
      const filters: SearchOptions['filters'] = {};

      if (req.query.type) {
        filters.type = (req.query.type as string).split(',') as any;
      }

      if (req.query.genres) {
        filters.genres = (req.query.genres as string).split(',');
      }

      if (req.query.minPopularity) {
        filters.minPopularity = parseFloat(req.query.minPopularity as string);
      }

      // Perform search
      const service = await getSearchService();
      const results = await service.search(searchQuery, {
        limit,
        threshold,
        backends,
        filters
      });

      sendResponse(res, {
        query: searchQuery,
        results,
        count: results.length,
        backends: backends
      });
    } catch (error: any) {
      console.error('[SearchAPI] Search error:', error);
      sendResponse(res, undefined, {
        code: 'SEARCH_ERROR',
        message: error.message || 'Search failed',
        statusCode: 500
      });
    }
  }
);

/**
 * GET /api/v1/search/similar/:itemId
 * Find similar content based on item ID
 *
 * Path params:
 * - itemId: Asset ID
 *
 * Query params:
 * - limit: Number of results (default: 10, max: 100)
 * - threshold: Similarity threshold (0-1)
 */
router.get(
  '/similar/:itemId',
  [
    param('itemId').notEmpty().trim().withMessage('Item ID is required'),
    query('limit').optional().isInt({ min: 1, max: 100 }).toInt(),
    query('threshold').optional().isFloat({ min: 0, max: 1 }).toFloat()
  ],
  async (req: Request, res: Response) => {
    try {
      const errors = validationResult(req);
      if (!errors.isEmpty()) {
        return sendResponse(res, undefined, {
          code: 'VALIDATION_ERROR',
          message: 'Invalid parameters',
          details: errors.array(),
          statusCode: 400
        });
      }

      const { itemId } = req.params;
      const limit = parseInt(req.query.limit as string) || 10;
      const threshold = req.query.threshold ? parseFloat(req.query.threshold as string) : undefined;

      // Parse filters
      const filters: SearchOptions['filters'] = {};

      if (req.query.type) {
        filters.type = (req.query.type as string).split(',') as any;
      }

      if (req.query.genres) {
        filters.genres = (req.query.genres as string).split(',');
      }

      // Find similar content
      const service = await getSearchService();
      const results = await service.findSimilar(itemId, {
        limit,
        threshold,
        filters
      });

      sendResponse(res, {
        itemId,
        similar: results,
        count: results.length
      });
    } catch (error: any) {
      console.error('[SearchAPI] Similar search error:', error);

      if (error.message.includes('not found')) {
        return sendResponse(res, undefined, {
          code: 'NOT_FOUND',
          message: `Item not found: ${req.params.itemId}`,
          statusCode: 404
        });
      }

      sendResponse(res, undefined, {
        code: 'SEARCH_ERROR',
        message: error.message || 'Similar search failed',
        statusCode: 500
      });
    }
  }
);

/**
 * GET /api/v1/search/trending
 * Get trending content
 *
 * Query params:
 * - window: Time window (7d, 30d, 24h, etc.)
 * - limit: Number of results (default: 20, max: 100)
 * - type: Filter by content type
 * - genres: Filter by genres (comma-separated)
 */
router.get(
  '/trending',
  [
    query('window').optional().isString(),
    query('limit').optional().isInt({ min: 1, max: 100 }).toInt()
  ],
  async (req: Request, res: Response) => {
    try {
      const errors = validationResult(req);
      if (!errors.isEmpty()) {
        return sendResponse(res, undefined, {
          code: 'VALIDATION_ERROR',
          message: 'Invalid parameters',
          details: errors.array(),
          statusCode: 400
        });
      }

      const timeWindow = (req.query.window as string) || '7d';
      const limit = parseInt(req.query.limit as string) || 20;

      // Parse filters
      const filters: SearchOptions['filters'] = {};

      if (req.query.type) {
        filters.type = (req.query.type as string).split(',') as any;
      }

      if (req.query.genres) {
        filters.genres = (req.query.genres as string).split(',');
      }

      if (req.query.minPopularity) {
        filters.minPopularity = parseFloat(req.query.minPopularity as string);
      }

      // Get trending content
      const service = await getSearchService();
      const trending = await service.getTrending(timeWindow, { filters, limit });

      sendResponse(res, {
        timeWindow,
        trending,
        count: trending.length
      });
    } catch (error: any) {
      console.error('[SearchAPI] Trending error:', error);
      sendResponse(res, undefined, {
        code: 'SEARCH_ERROR',
        message: error.message || 'Trending search failed',
        statusCode: 500
      });
    }
  }
);

/**
 * GET /api/v1/search/stats
 * Get search service statistics
 */
router.get('/stats', async (req: Request, res: Response) => {
  try {
    const service = await getSearchService();
    const stats = service.getStats();

    sendResponse(res, stats);
  } catch (error: any) {
    console.error('[SearchAPI] Stats error:', error);
    sendResponse(res, undefined, {
      code: 'STATS_ERROR',
      message: error.message || 'Failed to retrieve stats',
      statusCode: 500
    });
  }
});

export default router;
