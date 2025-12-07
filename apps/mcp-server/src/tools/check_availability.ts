/**
 * MCP Tool: check_availability
 * Check where specific content is available to watch
 */

import { z } from 'zod';
import { MCPTool, PlatformAvailability } from '../types/index.js';
import { config } from '../config.js';
import { fetchWithRetry } from '../utils/retry.js';

export const checkAvailabilitySchema = z.object({
  contentId: z.string().describe('Content ID to check availability for'),
  region: z.string().regex(/^[A-Z]{2}$/).default('US').describe('Region code (e.g., "US", "UK")'),
  platforms: z.array(z.string()).optional().describe('Specific platforms to check'),
});

export type CheckAvailabilityInput = z.infer<typeof checkAvailabilitySchema>;

export const checkAvailabilityTool: MCPTool = {
  name: 'check_availability',
  description: 'Check where specific content is available to watch',
  inputSchema: {
    type: 'object',
    properties: {
      contentId: {
        type: 'string',
        description: 'Content ID to check availability for',
      },
      region: {
        type: 'string',
        description: 'Region code (e.g., "US", "UK", "CA")',
        default: 'US',
      },
      platforms: {
        type: 'array',
        items: { type: 'string' },
        description: 'Specific platforms to check (optional)',
      },
    },
    required: ['contentId'],
  },
};

export async function executeCheckAvailability(
  input: CheckAvailabilityInput
): Promise<{
  contentId: string;
  availability: PlatformAvailability[];
  region: string;
  updatedAt: string;
}> {
  try {
    const params = new URLSearchParams({
      region: input.region,
      ...(input.platforms && { platforms: input.platforms.join(',') }),
    });

    const response = await fetchWithRetry(
      `${config.services.content}/api/v1/content/${input.contentId}/availability?${params}`,
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

    return {
      contentId: input.contentId,
      availability: data.availability || [],
      region: input.region,
      updatedAt: new Date().toISOString(),
    };
  } catch (error) {
    console.error('Check availability error:', error);
    throw new Error(`Availability check failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}
