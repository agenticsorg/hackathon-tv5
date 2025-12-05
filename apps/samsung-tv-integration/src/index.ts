// Samsung TV Integration for Agentics TV5 Hackathon
// Main entry point - exports all public APIs

// Core types
export {
  SamsungTVDevice,
  SamsungTVDeviceSchema,
  TVState,
  TVStateSchema,
  TVApp,
  TVAppSchema,
  RemoteKey,
  RemoteKeySchema,
  TVCommand,
  TVCommandSchema,
  TVConfig,
  TVConfigSchema,
  TVEvent,
  TVEventType,
  MCPToolResult,
  STREAMING_APPS,
  StreamingAppId,
} from './lib/types.js';

// Discovery
export {
  discoverTVs,
  checkTVOnline,
  getTVInfo,
  TVDiscoveryService,
} from './lib/discovery.js';

// TV Client
export {
  SamsungTVClient,
  createTVClient,
  createTVClientFromIP,
  KEYS,
} from './lib/tv-client.js';

// Configuration
export {
  getConfig,
  saveDevice,
  removeDevice,
  getDevices,
  getDevice,
  getDeviceByIP,
  setDefaultDevice,
  getDefaultDevice,
  updateDeviceToken,
  clearConfig,
  getConfigPath,
} from './utils/config.js';

// Utilities
export {
  generateDeviceId,
  isValidIP,
  isValidMAC,
  normalizeMAC,
  formatDuration,
  sleep,
  retry,
  truncate,
  parseDeviceString,
} from './utils/helpers.js';

// MCP Server
export {
  MCP_TOOLS,
  handleToolCall,
  handlers as mcpHandlers,
  processRequest,
} from './mcp/server.js';
