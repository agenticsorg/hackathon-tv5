/**
 * SSE (Server-Sent Events) Transport for MCP Server
 * Used for web integrations and remote connections
 */

import express, { Request, Response, NextFunction } from 'express';
import cors from 'cors';
import helmet from 'helmet';
import rateLimit from 'express-rate-limit';
import { config } from '../config.js';
import { MCPErrorCode } from '../types/index.js';
import { handleMCPRequest } from '../server.js';
import { authenticateRequest } from '../middleware/auth.js';
import { arwManifest } from '../arw-manifest.js';

export class MCPSSETransport {
  private app: express.Application;
  private clients: Map<string, Response>;

  constructor() {
    this.app = express();
    this.clients = new Map();
    this.setupMiddleware();
    this.setupRoutes();
  }

  private setupMiddleware(): void {
    // Security
    this.app.use(helmet());
    this.app.use(
      cors({
        origin: config.cors.allowedOrigins,
        credentials: true,
      })
    );

    // Body parsing
    this.app.use(express.json());
    this.app.use(express.urlencoded({ extended: true }));

    // Rate limiting
    const limiter = rateLimit({
      windowMs: config.rateLimit.windowMs,
      max: config.rateLimit.unauthenticated,
      message: 'Too many requests from this IP',
      standardHeaders: true,
      legacyHeaders: false,
    });
    this.app.use('/mcp', limiter);

    // Request logging
    this.app.use((req: Request, res: Response, next: NextFunction) => {
      console.log(`[SSE] ${req.method} ${req.path}`);
      next();
    });
  }

  private setupRoutes(): void {
    // Health check
    this.app.get('/health', (req: Request, res: Response) => {
      res.json({ status: 'healthy', timestamp: new Date().toISOString() });
    });

    // ARW Manifest
    this.app.get('/.well-known/arw-manifest.json', (req: Request, res: Response) => {
      res.json(arwManifest);
    });

    // SSE event stream
    this.app.get('/mcp/events', (req: Request, res: Response) => {
      const clientId = req.query.clientId as string || `client-${Date.now()}`;

      res.setHeader('Content-Type', 'text/event-stream');
      res.setHeader('Cache-Control', 'no-cache');
      res.setHeader('Connection', 'keep-alive');
      res.setHeader('X-Accel-Buffering', 'no');

      this.clients.set(clientId, res);

      // Send initial connection event
      res.write(`event: connected\n`);
      res.write(`data: ${JSON.stringify({ clientId, timestamp: new Date().toISOString() })}\n\n`);

      req.on('close', () => {
        this.clients.delete(clientId);
        console.log(`[SSE] Client ${clientId} disconnected`);
      });
    });

    // Tool execution endpoint
    this.app.post('/mcp/tools/call', authenticateRequest, async (req: Request, res: Response) => {
      const { toolName, arguments: args } = req.body;

      if (!toolName) {
        return res.status(400).json({
          error: {
            code: MCPErrorCode.INVALID_PARAMS,
            message: 'Missing toolName parameter',
          },
        });
      }

      try {
        const result = await handleMCPRequest('tools/call', {
          name: toolName,
          arguments: args,
          userContext: (req as any).userContext,
        });

        res.json(result);
      } catch (error) {
        console.error('[SSE] Tool execution error:', error);
        res.status(500).json({
          error: {
            code: MCPErrorCode.TOOL_EXECUTION_ERROR,
            message: error instanceof Error ? error.message : 'Tool execution failed',
          },
        });
      }
    });

    // Resource retrieval endpoint
    this.app.get('/mcp/resources/:resourceUri(*)', async (req: Request, res: Response) => {
      const { resourceUri } = req.params;

      try {
        const result = await handleMCPRequest('resources/read', {
          uri: resourceUri,
        });

        res.json(result);
      } catch (error) {
        console.error('[SSE] Resource retrieval error:', error);
        res.status(404).json({
          error: {
            code: MCPErrorCode.RESOURCE_NOT_FOUND,
            message: error instanceof Error ? error.message : 'Resource not found',
          },
        });
      }
    });

    // Prompt retrieval endpoint
    this.app.post('/mcp/prompts/get', async (req: Request, res: Response) => {
      const { name, arguments: args } = req.body;

      if (!name) {
        return res.status(400).json({
          error: {
            code: MCPErrorCode.INVALID_PARAMS,
            message: 'Missing prompt name',
          },
        });
      }

      try {
        const result = await handleMCPRequest('prompts/get', {
          name,
          arguments: args,
        });

        res.json(result);
      } catch (error) {
        console.error('[SSE] Prompt retrieval error:', error);
        res.status(404).json({
          error: {
            code: MCPErrorCode.RESOURCE_NOT_FOUND,
            message: error instanceof Error ? error.message : 'Prompt not found',
          },
        });
      }
    });

    // List available tools
    this.app.get('/mcp/tools/list', async (req: Request, res: Response) => {
      try {
        const result = await handleMCPRequest('tools/list', {});
        res.json(result);
      } catch (error) {
        console.error('[SSE] List tools error:', error);
        res.status(500).json({
          error: {
            code: MCPErrorCode.INTERNAL_ERROR,
            message: 'Failed to list tools',
          },
        });
      }
    });

    // List available resources
    this.app.get('/mcp/resources/list', async (req: Request, res: Response) => {
      try {
        const result = await handleMCPRequest('resources/list', {});
        res.json(result);
      } catch (error) {
        console.error('[SSE] List resources error:', error);
        res.status(500).json({
          error: {
            code: MCPErrorCode.INTERNAL_ERROR,
            message: 'Failed to list resources',
          },
        });
      }
    });

    // Error handling
    this.app.use((err: Error, req: Request, res: Response, next: NextFunction) => {
      console.error('[SSE] Error:', err);
      res.status(500).json({
        error: {
          code: MCPErrorCode.INTERNAL_ERROR,
          message: err.message || 'Internal server error',
        },
      });
    });
  }

  public sendEvent(clientId: string, event: string, data: any): void {
    const client = this.clients.get(clientId);
    if (client) {
      client.write(`event: ${event}\n`);
      client.write(`data: ${JSON.stringify(data)}\n\n`);
    }
  }

  public broadcastEvent(event: string, data: any): void {
    this.clients.forEach((client) => {
      client.write(`event: ${event}\n`);
      client.write(`data: ${JSON.stringify(data)}\n\n`);
    });
  }

  async start(port: number): Promise<void> {
    return new Promise((resolve) => {
      this.app.listen(port, () => {
        console.log(`[SSE] MCP Server listening on port ${port}`);
        console.log(`[SSE] ARW Manifest: http://localhost:${port}/.well-known/arw-manifest.json`);
        resolve();
      });
    });
  }
}
