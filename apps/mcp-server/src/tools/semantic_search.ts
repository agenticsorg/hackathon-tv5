/**
 * MCP Tool: semantic_search
 * Natural language content search using AI-powered discovery service
 */

import { z } from 'zod';
import { MCPTool, SearchResult } from '../types/index.js';
import { config } from '../config.js';
import { fetchWithRetry } from '../utils/retry.js';

export const semanticSearchSchema = z.object({
  query: z.string().min(1).max(500).describe('Natural language search query'),
  filters: z
    .object({
      mediaType: z.enum(['movie', 'tv', 'all']).default('all'),
      ratingMin: z.number().min(0).max(10).optional(),
      releaseYearMin: z.number().int().min(1900).max(2100).optional(),
      releaseYearMax: z.number().int().min(1900).max(2100).optional(),
      genres: z.array(z.number()).optional(),
    })
    .optional(),
  limit: z.number().int().min(1).max(50).default(10),
  explain: z.boolean().default(false).describe('Include AI explanations'),
});

export type SemanticSearchInput = z.infer<typeof semanticSearchSchema>;

export const semanticSearchTool: MCPTool = {
  name: 'semantic_search',
  description: 'Search for movies and TV shows using natural language queries',
  inputSchema: {
    type: 'object',
    properties: {
      query: {
        type: 'string',
        description: 'Natural language search query describing the desired content',
      },
      filters: {
        type: 'object',
        properties: {
          mediaType: {
            type: 'string',
            enum: ['movie', 'tv', 'all'],
            default: 'all',
          },
          ratingMin: {
            type: 'number',
            minimum: 0,
            maximum: 10,
            description: 'Minimum average rating (0-10)',
          },
          releaseYearMin: {
            type: 'number',
            description: 'Minimum release year',
          },
          releaseYearMax: {
            type: 'number',
            description: 'Maximum release year',
          },
          genres: {
            type: 'array',
            items: { type: 'number' },
            description: 'Filter by genre IDs',
          },
        },
      },
      limit: {
        type: 'number',
        minimum: 1,
        maximum: 50,
        default: 10,
      },
      explain: {
        type: 'boolean',
        description: 'Include AI-generated explanations for each result',
        default: false,
      },
    },
    required: ['query'],
  },
};

export async function executeSemanticSearch(
  input: SemanticSearchInput
): Promise<{
  results: SearchResult[];
  total: number;
  processingTimeMs: number;
}> {
  const startTime = Date.now();

  try {
    const response = await fetchWithRetry(
      `${config.services.discovery}/api/v1/search/semantic`,
      {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(input),
      },
      {
        timeout: 100,
        maxRetries: 2,
        baseDelay: 50,
      }
    );

    if (!response.ok) {
      throw new Error(`Discovery service returned ${response.status}`);
    }

    const data = await response.json();
    const processingTimeMs = Date.now() - startTime;

    return {
      results: data.results || [],
      total: data.total || 0,
      processingTimeMs,
    };
  } catch (error) {
    console.error('Semantic search error:', error);
    throw new Error(`Search failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}
