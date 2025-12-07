/**
 * STDIO Transport for MCP Server
 * Used by Claude Desktop and other desktop integrations
 */

import * as readline from 'readline';
import { MCPRequest, MCPResponse, MCPErrorCode } from '../types/index.js';
import { handleMCPRequest } from '../server.js';

export class MCPStdioTransport {
  private readline: readline.Interface;

  constructor() {
    this.readline = readline.createInterface({
      input: process.stdin,
      output: process.stdout,
      terminal: false,
    });
  }

  async start(): Promise<void> {
    console.error('[STDIO] MCP Server starting on STDIO transport');

    this.readline.on('line', async (line) => {
      try {
        const request: MCPRequest = JSON.parse(line);
        const response = await this.handleRequest(request);
        console.log(JSON.stringify(response));
      } catch (error) {
        const errorResponse: MCPResponse = {
          jsonrpc: '2.0',
          error: {
            code: MCPErrorCode.PARSE_ERROR,
            message: 'Parse error',
            data: error instanceof Error ? error.message : 'Unknown error',
          },
          id: null,
        };
        console.log(JSON.stringify(errorResponse));
      }
    });

    this.readline.on('close', () => {
      console.error('[STDIO] MCP Server stopped');
      process.exit(0);
    });

    // Handle process termination
    process.on('SIGINT', () => {
      console.error('[STDIO] Received SIGINT, shutting down');
      this.readline.close();
    });

    process.on('SIGTERM', () => {
      console.error('[STDIO] Received SIGTERM, shutting down');
      this.readline.close();
    });
  }

  private async handleRequest(request: MCPRequest): Promise<MCPResponse> {
    const { method, params, id } = request;

    // Validate JSON-RPC 2.0 format
    if (request.jsonrpc !== '2.0') {
      return {
        jsonrpc: '2.0',
        error: {
          code: MCPErrorCode.INVALID_REQUEST,
          message: 'Invalid JSON-RPC version',
        },
        id,
      };
    }

    try {
      const result = await handleMCPRequest(method, params);
      return {
        jsonrpc: '2.0',
        result,
        id,
      };
    } catch (error) {
      console.error('[STDIO] Request error:', error);
      return {
        jsonrpc: '2.0',
        error: {
          code: MCPErrorCode.INTERNAL_ERROR,
          message: error instanceof Error ? error.message : 'Internal error',
          data: error,
        },
        id,
      };
    }
  }
}
