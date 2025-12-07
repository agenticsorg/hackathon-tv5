/**
 * MCP Tool: get_content_details
 * Get detailed information about a specific movie or TV show
 */

import { z } from 'zod';
import { MCPTool, ContentItem } from '../types/index.js';
import { config } from '../config.js';
import { fetchWithRetry } from '../utils/retry.js';

export const getContentDetailsSchema = z.object({
  contentId: z.string().describe('Content ID'),
  include: z
    .array(z.enum(['credits', 'images', 'availability', 'similar']))
    .optional()
    .describe('Additional information to include'),
});

export type GetContentDetailsInput = z.infer<typeof getContentDetailsSchema>;

export const getContentDetailsTool: MCPTool = {
  name: 'get_content_details',
  description: 'Get detailed information about a specific movie or TV show',
  inputSchema: {
    type: 'object',
    properties: {
      contentId: {
        type: 'string',
        description: 'Content ID',
      },
      include: {
        type: 'array',
        items: {
          type: 'string',
          enum: ['credits', 'images', 'availability', 'similar'],
        },
        description: 'Additional information to include',
      },
    },
    required: ['contentId'],
  },
};

export async function executeGetContentDetails(
  input: GetContentDetailsInput
): Promise<ContentItem & Record<string, any>> {
  try {
    const params = new URLSearchParams();
    if (input.include && input.include.length > 0) {
      params.set('include', input.include.join(','));
    }

    const queryString = params.toString();
    const url = `${config.services.content}/api/v1/content/${input.contentId}${queryString ? `?${queryString}` : ''}`;

    const response = await fetchWithRetry(
      url,
      {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      },
      {
        timeout: 100,
        maxRetries: 2,
        baseDelay: 50,
      }
    );

    if (!response.ok) {
      if (response.status === 404) {
        throw new Error(`Content with ID ${input.contentId} not found`);
      }
      throw new Error(`Content service returned ${response.status}`);
    }

    const data = await response.json();
    return data;
  } catch (error) {
    console.error('Get content details error:', error);
    throw new Error(`Failed to get content details: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}
