#!/usr/bin/env node

/**
 * Media Gateway MCP Server
 * Main entry point - supports both STDIO and SSE transports
 */

import { config } from './config.js';
import { MCPStdioTransport } from './transports/stdio.js';
import { MCPSSETransport } from './transports/sse.js';

async function main() {
  const args = process.argv.slice(2);
  const transportFlag = args.indexOf('--transport');
  const portFlag = args.indexOf('--port');

  const transport = transportFlag !== -1 ? args[transportFlag + 1] : config.transport;
  const port = portFlag !== -1 ? parseInt(args[portFlag + 1], 10) : config.port;

  console.error(`[Main] Starting Media Gateway MCP Server`);
  console.error(`[Main] Environment: ${config.nodeEnv}`);
  console.error(`[Main] Transport: ${transport}`);
  console.error(`[Main] API Version: ${config.apiVersion}`);

  if (transport === 'stdio') {
    const stdioTransport = new MCPStdioTransport();
    await stdioTransport.start();
  } else if (transport === 'sse') {
    const sseTransport = new MCPSSETransport();
    await sseTransport.start(port);
  } else {
    console.error(`[Main] Error: Unknown transport '${transport}'`);
    console.error(`[Main] Supported transports: stdio, sse`);
    process.exit(1);
  }
}

// Handle uncaught errors
process.on('uncaughtException', (error) => {
  console.error('[Main] Uncaught exception:', error);
  process.exit(1);
});

process.on('unhandledRejection', (reason, promise) => {
  console.error('[Main] Unhandled rejection at:', promise, 'reason:', reason);
  process.exit(1);
});

// Start server
main().catch((error) => {
  console.error('[Main] Fatal error:', error);
  process.exit(1);
});
