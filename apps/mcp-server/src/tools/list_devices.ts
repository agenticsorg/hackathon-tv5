/**
 * MCP Tool: list_devices
 * List user's registered devices
 */

import { z } from 'zod';
import { MCPTool, Device, UserContext } from '../types/index.js';
import { config } from '../config.js';
import { fetchWithRetry } from '../utils/retry.js';

export const listDevicesSchema = z.object({
  includeOffline: z.boolean().default(false).describe('Include offline devices'),
});

export type ListDevicesInput = z.infer<typeof listDevicesSchema>;

export const listDevicesTool: MCPTool = {
  name: 'list_devices',
  description: "List user's registered devices with capabilities and online status",
  inputSchema: {
    type: 'object',
    properties: {
      includeOffline: {
        type: 'boolean',
        description: 'Include offline devices in the list',
        default: false,
      },
    },
  },
};

export async function executeListDevices(
  input: ListDevicesInput,
  userContext?: UserContext
): Promise<{ devices: Device[] }> {
  if (!userContext?.userId) {
    throw new Error('Authentication required to list devices');
  }

  try {
    const params = new URLSearchParams({
      includeOffline: input.includeOffline.toString(),
    });

    const response = await fetchWithRetry(
      `${config.services.user}/api/v1/user/devices?${params}`,
      {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
          'X-User-ID': userContext.userId,
        },
      },
      {
        timeout: 100,
        maxRetries: 2,
        baseDelay: 50,
      }
    );

    if (!response.ok) {
      throw new Error(`User service returned ${response.status}`);
    }

    const data = await response.json();
    return { devices: data.devices || [] };
  } catch (error) {
    console.error('List devices error:', error);
    throw new Error(`Failed to list devices: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}
