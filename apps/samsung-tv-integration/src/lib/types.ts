import { z } from 'zod';

// Device discovery types
export const SamsungTVDeviceSchema = z.object({
  id: z.string(),
  name: z.string(),
  ip: z.string(),
  mac: z.string().optional(),
  model: z.string().optional(),
  port: z.number().default(8002),
  token: z.string().optional(),
  isOnline: z.boolean().default(false),
  lastSeen: z.string().datetime().optional(),
});

export type SamsungTVDevice = z.infer<typeof SamsungTVDeviceSchema>;

// TV state types
export const TVStateSchema = z.object({
  power: z.enum(['on', 'off', 'unknown']),
  volume: z.number().min(0).max(100).optional(),
  muted: z.boolean().optional(),
  currentApp: z.string().optional(),
  currentAppName: z.string().optional(),
  channel: z.string().optional(),
});

export type TVState = z.infer<typeof TVStateSchema>;

// App info types
export const TVAppSchema = z.object({
  appId: z.string(),
  name: z.string(),
  icon: z.string().optional(),
  isRunning: z.boolean().default(false),
});

export type TVApp = z.infer<typeof TVAppSchema>;

// Remote key types
export const RemoteKeySchema = z.enum([
  // Power
  'KEY_POWER',
  'KEY_POWEROFF',
  // Navigation
  'KEY_UP',
  'KEY_DOWN',
  'KEY_LEFT',
  'KEY_RIGHT',
  'KEY_ENTER',
  'KEY_RETURN',
  'KEY_EXIT',
  // Menu
  'KEY_HOME',
  'KEY_MENU',
  'KEY_SOURCE',
  'KEY_GUIDE',
  'KEY_INFO',
  // Volume
  'KEY_VOLUP',
  'KEY_VOLDOWN',
  'KEY_MUTE',
  // Channel
  'KEY_CHUP',
  'KEY_CHDOWN',
  'KEY_PRECH',
  // Playback
  'KEY_PLAY',
  'KEY_PAUSE',
  'KEY_STOP',
  'KEY_REWIND',
  'KEY_FF',
  'KEY_REC',
  // Numbers
  'KEY_0',
  'KEY_1',
  'KEY_2',
  'KEY_3',
  'KEY_4',
  'KEY_5',
  'KEY_6',
  'KEY_7',
  'KEY_8',
  'KEY_9',
  // Colors
  'KEY_RED',
  'KEY_GREEN',
  'KEY_YELLOW',
  'KEY_BLUE',
  // Smart features
  'KEY_CONTENTS',
  'KEY_SEARCH',
  'KEY_AMBIENT',
]);

export type RemoteKey = z.infer<typeof RemoteKeySchema>;

// Command types
export const TVCommandSchema = z.discriminatedUnion('type', [
  z.object({
    type: z.literal('power'),
    action: z.enum(['on', 'off', 'toggle']),
  }),
  z.object({
    type: z.literal('volume'),
    action: z.enum(['up', 'down', 'set', 'mute', 'unmute']),
    value: z.number().min(0).max(100).optional(),
  }),
  z.object({
    type: z.literal('channel'),
    action: z.enum(['up', 'down', 'set']),
    value: z.string().optional(),
  }),
  z.object({
    type: z.literal('key'),
    key: RemoteKeySchema,
  }),
  z.object({
    type: z.literal('app'),
    action: z.enum(['launch', 'close', 'list']),
    appId: z.string().optional(),
  }),
  z.object({
    type: z.literal('text'),
    text: z.string(),
  }),
]);

export type TVCommand = z.infer<typeof TVCommandSchema>;

// Common streaming app IDs
export const STREAMING_APPS = {
  YOUTUBE: '111299001912',
  NETFLIX: '11101200001',
  PRIME_VIDEO: '3201512006785',
  DISNEY_PLUS: '3201601007250',
  SPOTIFY: '3201606009684',
  APPLE_TV: '3201807016597',
  HBO_MAX: '3201601007230',
  HULU: '3201601007625',
  PLEX: '3201512006963',
  TWITCH: '3201909019271',
} as const;

export type StreamingAppId = (typeof STREAMING_APPS)[keyof typeof STREAMING_APPS];

// Configuration types
export const TVConfigSchema = z.object({
  devices: z.array(SamsungTVDeviceSchema).default([]),
  defaultDeviceId: z.string().optional(),
  discoveryTimeout: z.number().default(5000),
  connectionTimeout: z.number().default(10000),
});

export type TVConfig = z.infer<typeof TVConfigSchema>;

// MCP tool response types
export interface MCPToolResult {
  success: boolean;
  data?: unknown;
  error?: string;
}

// Event types for real-time updates
export type TVEventType =
  | 'device_discovered'
  | 'device_connected'
  | 'device_disconnected'
  | 'state_changed'
  | 'app_launched'
  | 'error';

export interface TVEvent {
  type: TVEventType;
  deviceId: string;
  timestamp: string;
  data?: unknown;
}
