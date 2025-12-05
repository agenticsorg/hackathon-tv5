/**
 * Agentics Foundation TV5 Hackathon - Main Module
 *
 * This module exports the core functionality for programmatic use.
 */

// Types
export * from './types.js';

// Constants
export {
  HACKATHON_NAME,
  HACKATHON_TAGLINE,
  TRACKS,
  AVAILABLE_TOOLS,
  DISCORD_URL,
  WEBSITE_URL,
  GITHUB_URL,
  CONFIG_FILE
} from './constants.js';

// Utilities
export {
  logger,
  loadConfig,
  saveConfig,
  configExists,
  createDefaultConfig,
  updateConfig,
  checkToolInstalled,
  installTool,
  checkPrerequisites
} from './utils/index.js';

// MCP Server
export { McpServer, startSseServer } from './mcp/index.js';
