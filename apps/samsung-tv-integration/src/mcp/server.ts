import { z } from 'zod';
import { discoverTVs } from '../lib/discovery.js';
import { createTVClient, createTVClientFromIP, SamsungTVClient } from '../lib/tv-client.js';
import {
  SamsungTVDevice,
  TVCommandSchema,
  RemoteKeySchema,
  STREAMING_APPS,
  MCPToolResult,
} from '../lib/types.js';
import {
  getDevices,
  saveDevice,
  removeDevice,
  getDefaultDevice,
  setDefaultDevice,
  updateDeviceToken,
  getDeviceByIP,
} from '../utils/config.js';
import { LEARNING_TOOLS, handleLearningToolCall } from './learning-tools.js';

// Active TV client instances
const clients = new Map<string, SamsungTVClient>();

/**
 * Get or create a TV client for a device
 */
function getClient(deviceId?: string): SamsungTVClient | null {
  if (deviceId && clients.has(deviceId)) {
    return clients.get(deviceId)!;
  }

  // Try default device
  const device = deviceId
    ? getDevices().find(d => d.id === deviceId)
    : getDefaultDevice();

  if (!device) {
    return null;
  }

  const client = createTVClient(device);
  clients.set(device.id, client);
  return client;
}

// Core TV MCP Tool definitions
const TV_TOOLS = [
  {
    name: 'samsung_tv_discover',
    description: 'Discover Samsung Smart TVs on the local network using SSDP',
    inputSchema: {
      type: 'object',
      properties: {
        timeout: {
          type: 'number',
          description: 'Discovery timeout in milliseconds (default: 5000)',
        },
      },
    },
  },
  {
    name: 'samsung_tv_list',
    description: 'List all saved/known Samsung TVs',
    inputSchema: {
      type: 'object',
      properties: {},
    },
  },
  {
    name: 'samsung_tv_connect',
    description: 'Connect to a Samsung TV and get authentication token. The TV will show a pairing dialog on first connection.',
    inputSchema: {
      type: 'object',
      properties: {
        ip: {
          type: 'string',
          description: 'IP address of the TV',
        },
        mac: {
          type: 'string',
          description: 'MAC address of the TV (required for Wake-on-LAN)',
        },
        deviceId: {
          type: 'string',
          description: 'ID of a saved device to connect to',
        },
      },
    },
  },
  {
    name: 'samsung_tv_power',
    description: 'Control TV power (on/off/toggle). Requires MAC address for power on.',
    inputSchema: {
      type: 'object',
      properties: {
        action: {
          type: 'string',
          enum: ['on', 'off', 'toggle'],
          description: 'Power action to perform',
        },
        deviceId: {
          type: 'string',
          description: 'Device ID (uses default if not specified)',
        },
      },
      required: ['action'],
    },
  },
  {
    name: 'samsung_tv_volume',
    description: 'Control TV volume',
    inputSchema: {
      type: 'object',
      properties: {
        action: {
          type: 'string',
          enum: ['up', 'down', 'mute', 'unmute'],
          description: 'Volume action',
        },
        steps: {
          type: 'number',
          description: 'Number of steps for up/down (default: 1)',
        },
        deviceId: {
          type: 'string',
          description: 'Device ID (uses default if not specified)',
        },
      },
      required: ['action'],
    },
  },
  {
    name: 'samsung_tv_navigate',
    description: 'Navigate TV interface with arrow keys',
    inputSchema: {
      type: 'object',
      properties: {
        direction: {
          type: 'string',
          enum: ['up', 'down', 'left', 'right', 'enter', 'back'],
          description: 'Navigation direction',
        },
        deviceId: {
          type: 'string',
          description: 'Device ID (uses default if not specified)',
        },
      },
      required: ['direction'],
    },
  },
  {
    name: 'samsung_tv_key',
    description: 'Send a specific remote key press to the TV',
    inputSchema: {
      type: 'object',
      properties: {
        key: {
          type: 'string',
          description: 'Remote key name (e.g., KEY_HOME, KEY_MENU, KEY_PLAY)',
        },
        deviceId: {
          type: 'string',
          description: 'Device ID (uses default if not specified)',
        },
      },
      required: ['key'],
    },
  },
  {
    name: 'samsung_tv_apps',
    description: 'List installed apps on the TV',
    inputSchema: {
      type: 'object',
      properties: {
        deviceId: {
          type: 'string',
          description: 'Device ID (uses default if not specified)',
        },
      },
    },
  },
  {
    name: 'samsung_tv_launch_app',
    description: 'Launch an app on the TV. Supports app IDs or names like YOUTUBE, NETFLIX, PRIME_VIDEO, DISNEY_PLUS, SPOTIFY, etc.',
    inputSchema: {
      type: 'object',
      properties: {
        app: {
          type: 'string',
          description: 'App ID or name (e.g., "YOUTUBE", "NETFLIX", "111299001912")',
        },
        deviceId: {
          type: 'string',
          description: 'Device ID (uses default if not specified)',
        },
      },
      required: ['app'],
    },
  },
  {
    name: 'samsung_tv_home',
    description: 'Go to TV home screen',
    inputSchema: {
      type: 'object',
      properties: {
        deviceId: {
          type: 'string',
          description: 'Device ID (uses default if not specified)',
        },
      },
    },
  },
  {
    name: 'samsung_tv_status',
    description: 'Get current TV status (power state, etc.)',
    inputSchema: {
      type: 'object',
      properties: {
        deviceId: {
          type: 'string',
          description: 'Device ID (uses default if not specified)',
        },
      },
    },
  },
  {
    name: 'samsung_tv_set_default',
    description: 'Set a device as the default TV',
    inputSchema: {
      type: 'object',
      properties: {
        deviceId: {
          type: 'string',
          description: 'Device ID to set as default',
        },
      },
      required: ['deviceId'],
    },
  },
  {
    name: 'samsung_tv_remove',
    description: 'Remove a saved TV from the configuration',
    inputSchema: {
      type: 'object',
      properties: {
        deviceId: {
          type: 'string',
          description: 'Device ID to remove',
        },
      },
      required: ['deviceId'],
    },
  },
];

// Combined MCP Tools (TV control + Learning system)
export const MCP_TOOLS = [...TV_TOOLS, ...LEARNING_TOOLS];

/**
 * Handle MCP tool calls
 */
export async function handleToolCall(toolName: string, args: Record<string, unknown>): Promise<MCPToolResult> {
  try {
    switch (toolName) {
      case 'samsung_tv_discover': {
        const timeout = typeof args.timeout === 'number' ? args.timeout : 5000;
        const devices = await discoverTVs({ timeout });

        // Save discovered devices
        devices.forEach(device => saveDevice(device));

        return {
          success: true,
          data: {
            count: devices.length,
            devices: devices.map(d => ({
              id: d.id,
              name: d.name,
              ip: d.ip,
              model: d.model,
            })),
          },
        };
      }

      case 'samsung_tv_list': {
        const devices = getDevices();
        const defaultDevice = getDefaultDevice();

        return {
          success: true,
          data: {
            count: devices.length,
            defaultDeviceId: defaultDevice?.id,
            devices: devices.map(d => ({
              id: d.id,
              name: d.name,
              ip: d.ip,
              model: d.model,
              hasToken: !!d.token,
              isDefault: d.id === defaultDevice?.id,
            })),
          },
        };
      }

      case 'samsung_tv_connect': {
        let device: SamsungTVDevice | undefined;

        if (args.ip && typeof args.ip === 'string') {
          // Connect by IP
          device = getDeviceByIP(args.ip);
          if (!device) {
            device = {
              id: `samsung-tv-${args.ip.replace(/\./g, '-')}`,
              name: `Samsung TV (${args.ip})`,
              ip: args.ip,
              port: 8002,
              mac: typeof args.mac === 'string' ? args.mac : undefined,
              isOnline: false,
            };
          }
        } else if (args.deviceId && typeof args.deviceId === 'string') {
          device = getDevices().find(d => d.id === args.deviceId);
        } else {
          device = getDefaultDevice();
        }

        if (!device) {
          return { success: false, error: 'No device specified and no default device configured' };
        }

        const client = createTVClient(device);
        const result = await client.connect();

        if (result.success && result.token) {
          device.token = result.token;
          saveDevice(device);
          clients.set(device.id, client);

          // Set as default if it's the first device
          if (getDevices().length === 1) {
            setDefaultDevice(device.id);
          }
        }

        return {
          success: result.success,
          data: result.success ? { deviceId: device.id, token: result.token } : undefined,
          error: result.error,
        };
      }

      case 'samsung_tv_power': {
        const client = getClient(args.deviceId as string | undefined);
        if (!client) {
          return { success: false, error: 'No TV connected. Run samsung_tv_connect first.' };
        }

        const action = args.action as 'on' | 'off' | 'toggle';
        const result = await client.executeCommand({ type: 'power', action });
        return result;
      }

      case 'samsung_tv_volume': {
        const client = getClient(args.deviceId as string | undefined);
        if (!client) {
          return { success: false, error: 'No TV connected. Run samsung_tv_connect first.' };
        }

        const action = args.action as 'up' | 'down' | 'mute' | 'unmute';
        const steps = typeof args.steps === 'number' ? args.steps : 1;
        const result = await client.setVolume(action, steps);
        return result;
      }

      case 'samsung_tv_navigate': {
        const client = getClient(args.deviceId as string | undefined);
        if (!client) {
          return { success: false, error: 'No TV connected. Run samsung_tv_connect first.' };
        }

        const direction = args.direction as 'up' | 'down' | 'left' | 'right' | 'enter' | 'back';
        const result = await client.navigate(direction);
        return result;
      }

      case 'samsung_tv_key': {
        const client = getClient(args.deviceId as string | undefined);
        if (!client) {
          return { success: false, error: 'No TV connected. Run samsung_tv_connect first.' };
        }

        const key = args.key as string;
        const parseResult = RemoteKeySchema.safeParse(key);
        if (!parseResult.success) {
          return { success: false, error: `Invalid key: ${key}. Use keys like KEY_HOME, KEY_MENU, KEY_PLAY, etc.` };
        }

        const result = await client.sendKey(parseResult.data);
        return result;
      }

      case 'samsung_tv_apps': {
        const client = getClient(args.deviceId as string | undefined);
        if (!client) {
          return { success: false, error: 'No TV connected. Run samsung_tv_connect first.' };
        }

        const result = await client.getApps();
        return {
          success: result.success,
          data: result.apps,
          error: result.error,
        };
      }

      case 'samsung_tv_launch_app': {
        const client = getClient(args.deviceId as string | undefined);
        if (!client) {
          return { success: false, error: 'No TV connected. Run samsung_tv_connect first.' };
        }

        const app = args.app as string;

        // Check if it's a known streaming app
        const upperApp = app.toUpperCase().replace(/[^A-Z]/g, '_');
        if (upperApp in STREAMING_APPS) {
          const result = await client.launchStreamingApp(upperApp as keyof typeof STREAMING_APPS);
          return result;
        }

        const result = await client.launchApp(app);
        return result;
      }

      case 'samsung_tv_home': {
        const client = getClient(args.deviceId as string | undefined);
        if (!client) {
          return { success: false, error: 'No TV connected. Run samsung_tv_connect first.' };
        }

        const result = await client.goHome();
        return result;
      }

      case 'samsung_tv_status': {
        const client = getClient(args.deviceId as string | undefined);
        if (!client) {
          return { success: false, error: 'No TV connected. Run samsung_tv_connect first.' };
        }

        const result = await client.getState();
        return {
          success: result.success,
          data: result.state,
          error: result.error,
        };
      }

      case 'samsung_tv_set_default': {
        const deviceId = args.deviceId as string;
        const device = getDevices().find(d => d.id === deviceId);

        if (!device) {
          return { success: false, error: `Device not found: ${deviceId}` };
        }

        setDefaultDevice(deviceId);
        return { success: true, data: { defaultDeviceId: deviceId } };
      }

      case 'samsung_tv_remove': {
        const deviceId = args.deviceId as string;
        const removed = removeDevice(deviceId);

        if (!removed) {
          return { success: false, error: `Device not found: ${deviceId}` };
        }

        clients.delete(deviceId);
        return { success: true };
      }

      default:
        // Check if it's a learning tool
        if (toolName.startsWith('samsung_tv_learn_') || toolName.startsWith('samsung_tv_smart_')) {
          return handleLearningToolCall(toolName, args);
        }
        return { success: false, error: `Unknown tool: ${toolName}` };
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Unknown error';
    return { success: false, error: message };
  }
}

/**
 * MCP server request handlers
 */
export const handlers = new Map<string, (params?: unknown) => unknown>();

// List available tools
handlers.set('tools/list', () => ({
  tools: MCP_TOOLS,
}));

// Handle tool calls
handlers.set('tools/call', async (params: unknown) => {
  const { name, arguments: args } = params as { name: string; arguments?: Record<string, unknown> };
  const result = await handleToolCall(name, args || {});

  return {
    content: [
      {
        type: 'text',
        text: JSON.stringify(result, null, 2),
      },
    ],
  };
});

// Server info
handlers.set('initialize', () => ({
  protocolVersion: '2024-11-05',
  serverInfo: {
    name: 'samsung-tv-integration',
    version: '1.0.0',
  },
  capabilities: {
    tools: {},
  },
}));

handlers.set('notifications/initialized', () => ({}));

/**
 * Process a JSON-RPC request
 */
export async function processRequest(request: { method: string; params?: unknown; id?: number | string }): Promise<{
  jsonrpc: '2.0';
  id?: number | string;
  result?: unknown;
  error?: { code: number; message: string };
}> {
  const handler = handlers.get(request.method);

  if (!handler) {
    return {
      jsonrpc: '2.0',
      id: request.id,
      error: { code: -32601, message: `Method not found: ${request.method}` },
    };
  }

  try {
    const result = await handler(request.params);
    return {
      jsonrpc: '2.0',
      id: request.id,
      result,
    };
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Internal error';
    return {
      jsonrpc: '2.0',
      id: request.id,
      error: { code: -32603, message },
    };
  }
}
