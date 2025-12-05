#!/usr/bin/env node
/**
 * MCP Server - STDIO Transport
 * Run with: npx @agenticsorg/hackathon mcp stdio
 */

import * as readline from 'readline';
import { McpServer } from './server.js';
import type { McpRequest } from '../types.js';

const server = new McpServer();

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false
});

let buffer = '';

rl.on('line', async (line) => {
  buffer += line;

  try {
    const request = JSON.parse(buffer) as McpRequest;
    buffer = '';

    const response = await server.handleRequest(request);
    console.log(JSON.stringify(response));
  } catch (e) {
    if (e instanceof SyntaxError) {
      // Incomplete JSON, continue buffering
      return;
    }

    // Other error
    console.log(JSON.stringify({
      jsonrpc: '2.0',
      id: null,
      error: {
        code: -32700,
        message: 'Parse error'
      }
    }));
    buffer = '';
  }
});

rl.on('close', () => {
  process.exit(0);
});

// Handle errors
process.on('uncaughtException', (error) => {
  console.error('Uncaught exception:', error);
  process.exit(1);
});

process.on('unhandledRejection', (reason) => {
  console.error('Unhandled rejection:', reason);
  process.exit(1);
});

// Log startup to stderr (not stdout, which is for MCP messages)
console.error('Agentics Hackathon MCP Server (STDIO) started');
