/**
 * MCP Tool: initiate_playback
 * Start content playback on a target device
 */

import { z } from 'zod';
import { MCPTool, UserContext } from '../types/index.js';
import { config } from '../config.js';
import { fetchWithRetry } from '../utils/retry.js';

export const initiatePlaybackSchema = z.object({
  contentId: z.string().describe('Content ID to play'),
  deviceId: z.string().describe('Target device ID'),
  platformId: z.string().optional().describe('Preferred platform (if available on multiple)'),
  startPosition: z.number().int().min(0).optional().describe('Start position in seconds'),
});

export type InitiatePlaybackInput = z.infer<typeof initiatePlaybackSchema>;

export const initiatePlaybackTool: MCPTool = {
  name: 'initiate_playback',
  description: 'Start content playback on a target device with deep link generation',
  inputSchema: {
    type: 'object',
    properties: {
      contentId: {
        type: 'string',
        description: 'Content ID to play',
      },
      deviceId: {
        type: 'string',
        description: 'Target device ID',
      },
      platformId: {
        type: 'string',
        description: 'Preferred platform (if available on multiple)',
      },
      startPosition: {
        type: 'number',
        description: 'Start position in seconds (optional)',
        minimum: 0,
      },
    },
    required: ['contentId', 'deviceId'],
  },
};

export async function executeInitiatePlayback(
  input: InitiatePlaybackInput,
  userContext?: UserContext
): Promise<{
  success: boolean;
  deepLink: string;
  platform: string;
  message: string;
}> {
  if (!userContext?.userId) {
    throw new Error('Authentication required to initiate playback');
  }

  if (!userContext.scopes?.includes('playback:control')) {
    throw new Error('Insufficient permissions. playback:control scope required');
  }

  try {
    const response = await fetchWithRetry(
      `${config.services.user}/api/v1/playback/initiate`,
      {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-User-ID': userContext.userId,
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
      throw new Error(`Playback service returned ${response.status}`);
    }

    const data = await response.json();
    return {
      success: true,
      deepLink: data.deepLink || '',
      platform: data.platform || '',
      message: data.message || 'Playback initiated successfully',
    };
  } catch (error) {
    console.error('Initiate playback error:', error);
    throw new Error(`Failed to initiate playback: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}
