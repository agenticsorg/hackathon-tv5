#!/usr/bin/env node
import * as readline from 'readline';
import { processRequest } from './server.js';

/**
 * MCP Server - STDIO Transport
 *
 * This transport is used when running as a subprocess (e.g., Claude Desktop)
 * Messages are sent as newline-delimited JSON over stdin/stdout
 */

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false,
});

let buffer = '';

process.stdin.on('data', (chunk) => {
  buffer += chunk.toString();

  // Process complete messages (newline-delimited)
  const lines = buffer.split('\n');
  buffer = lines.pop() || '';

  for (const line of lines) {
    if (line.trim()) {
      handleMessage(line.trim());
    }
  }
});

async function handleMessage(message: string) {
  try {
    const request = JSON.parse(message);
    const response = await processRequest(request);
    sendResponse(response);
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : 'Parse error';
    sendResponse({
      jsonrpc: '2.0',
      error: { code: -32700, message: `Parse error: ${errorMessage}` },
    });
  }
}

function sendResponse(response: unknown) {
  process.stdout.write(JSON.stringify(response) + '\n');
}

// Handle process termination
process.on('SIGINT', () => {
  process.exit(0);
});

process.on('SIGTERM', () => {
  process.exit(0);
});

// Log startup (to stderr so it doesn't interfere with STDIO transport)
process.stderr.write('Samsung TV MCP Server (STDIO) started\n');
