/**
 * MCP Tool: get_recommendations
 * Personalized content recommendations using SONA engine
 */

import { z } from 'zod';
import { MCPTool, RecommendationResult, UserContext } from '../types/index.js';
import { config } from '../config.js';
import { fetchWithRetry } from '../utils/retry.js';

export const getRecommendationsSchema = z.object({
  basedOn: z
    .object({
      contentId: z.string().describe('Content ID to find similar items for'),
      mediaType: z.enum(['movie', 'tv']),
    })
    .optional(),
  preferences: z
    .object({
      genres: z.array(z.number()).optional().describe('Preferred genre IDs'),
      mood: z.string().optional().describe('Current mood (e.g., "relaxed", "excited")'),
      excludeContentIds: z.array(z.string()).optional().describe('Content to exclude'),
    })
    .optional(),
  limit: z.number().int().min(1).max(50).default(10),
});

export type GetRecommendationsInput = z.infer<typeof getRecommendationsSchema>;

export const getRecommendationsTool: MCPTool = {
  name: 'get_recommendations',
  description: 'Get personalized content recommendations based on user preferences or similar content',
  inputSchema: {
    type: 'object',
    properties: {
      basedOn: {
        type: 'object',
        properties: {
          contentId: {
            type: 'string',
            description: 'Content ID to find similar items for',
          },
          mediaType: {
            type: 'string',
            enum: ['movie', 'tv'],
          },
        },
        description: 'Get recommendations similar to specific content',
      },
      preferences: {
        type: 'object',
        properties: {
          genres: {
            type: 'array',
            items: { type: 'number' },
            description: 'Preferred genre IDs',
          },
          mood: {
            type: 'string',
            description: 'Current mood (e.g., "relaxed", "excited", "thoughtful")',
          },
          excludeContentIds: {
            type: 'array',
            items: { type: 'string' },
            description: 'Content IDs to exclude from recommendations',
          },
        },
        description: 'User preferences for personalized recommendations',
      },
      limit: {
        type: 'number',
        minimum: 1,
        maximum: 50,
        default: 10,
      },
    },
  },
};

export async function executeGetRecommendations(
  input: GetRecommendationsInput,
  userContext?: UserContext
): Promise<{
  recommendations: RecommendationResult[];
  modelVersion: string;
  generatedAt: string;
}> {
  try {
    const requestBody = {
      ...input,
      userContext: userContext
        ? {
            userId: userContext.userId,
            tier: userContext.tier,
          }
        : undefined,
    };

    const response = await fetchWithRetry(
      `${config.services.recommendation}/api/v1/recommendations/for-you`,
      {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(requestBody),
      },
      {
        timeout: 100,
        maxRetries: 2,
        baseDelay: 50,
      }
    );

    if (!response.ok) {
      throw new Error(`Recommendation service returned ${response.status}`);
    }

    const data = await response.json();

    return {
      recommendations: data.recommendations || [],
      modelVersion: data.model_version || 'sona-v2.1',
      generatedAt: data.generated_at || new Date().toISOString(),
    };
  } catch (error) {
    console.error('Recommendations error:', error);
    throw new Error(`Recommendations failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}
