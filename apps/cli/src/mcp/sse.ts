#!/usr/bin/env node
/**
 * MCP Server - SSE (Server-Sent Events) Transport
 * Run with: npx @agenticsorg/hackathon mcp sse --port 3000
 */

import express from 'express';
import helmet from 'helmet';
import rateLimit from 'express-rate-limit';
import { McpServer } from './server.js';
import type { McpRequest } from '../types.js';

const DEFAULT_PORT = 3000;
const REQUEST_TIMEOUT_MS = 30000; // 30 seconds

// Rate limiting: 100 requests per 15 minutes per IP
const limiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100, // Limit each IP to 100 requests per windowMs
  message: 'Too many requests from this IP, please try again later.'
});

export function startSseServer(port: number = DEFAULT_PORT): void {
  const app = express();
  const server = new McpServer();
  const activeIntervals = new Set<NodeJS.Timeout>();

  // Security headers via helmet
  app.use(helmet({
    contentSecurityPolicy: {
      directives: {
        defaultSrc: ["'self'"],
        scriptSrc: ["'self'"],
        styleSrc: ["'self'"],
        imgSrc: ["'self'"],
      },
    },
  }));

  // Apply rate limiting to all requests
  app.use(limiter);

  app.use(express.json());

  // Request timeout middleware using a timeout wrapper
  // Note: req.setTimeout/res.setTimeout can cause "headers already sent" errors
  // Instead, we use a timeout flag pattern that's checked before sending responses
  app.use((req, res, next) => {
    let timedOut = false;

    const timeout = setTimeout(() => {
      timedOut = true;
      if (!res.headersSent) {
        res.status(408).json({
          jsonrpc: '2.0',
          id: null,
          error: {
            code: -32000,
            message: 'Request timeout - request took too long to process'
          }
        });
      }
    }, REQUEST_TIMEOUT_MS);

    // Store timeout state on request for handlers to check
    (req as any).timedOut = () => timedOut;

    // Clean up timeout when response finishes
    res.on('finish', () => clearTimeout(timeout));
    res.on('close', () => clearTimeout(timeout));

    next();
  });

  // Secure CORS - only allow localhost origins for development
  app.use((req, res, next) => {
    const origin = req.headers.origin;
    const allowedOrigins = [
      'http://localhost:3000',
      'http://localhost:3001',
      'http://localhost:8080',
      'http://127.0.0.1:3000',
      'http://127.0.0.1:3001',
      'http://127.0.0.1:8080',
    ];

    if (origin && allowedOrigins.includes(origin)) {
      res.setHeader('Access-Control-Allow-Origin', origin);
    }

    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
    res.setHeader('Access-Control-Max-Age', '86400'); // 24 hours

    if (req.method === 'OPTIONS') {
      res.sendStatus(200);
      return;
    }
    next();
  });

  // Health check endpoint
  app.get('/health', (req, res) => {
    res.json({ status: 'ok', server: 'agentics-hackathon-mcp' });
  });

  // SSE endpoint for MCP
  app.get('/sse', (req, res) => {
    res.setHeader('Content-Type', 'text/event-stream');
    res.setHeader('Cache-Control', 'no-cache');
    res.setHeader('Connection', 'keep-alive');

    // Send initial connection event
    res.write(`event: connected\ndata: ${JSON.stringify({ server: 'agentics-hackathon-mcp' })}\n\n`);

    // Keep connection alive
    const keepAlive = setInterval(() => {
      res.write(': keepalive\n\n');
    }, 30000);

    // Track interval for cleanup
    activeIntervals.add(keepAlive);

    // Cleanup on connection close
    const cleanup = () => {
      clearInterval(keepAlive);
      activeIntervals.delete(keepAlive);
    };

    req.on('close', cleanup);
    req.on('error', cleanup);
    res.on('finish', cleanup);
  });

  // JSON-RPC endpoint for MCP requests
  app.post('/rpc', async (req, res) => {
    const request = req.body as McpRequest;

    if (!request.jsonrpc || request.jsonrpc !== '2.0') {
      res.status(400).json({
        jsonrpc: '2.0',
        id: null,
        error: {
          code: -32600,
          message: 'Invalid Request: jsonrpc must be "2.0"'
        }
      });
      return;
    }

    const response = await server.handleRequest(request);
    res.json(response);
  });

  // Info endpoint
  app.get('/', (req, res) => {
    res.json({
      name: 'Agentics Hackathon MCP Server',
      version: '1.0.0',
      transport: 'sse',
      endpoints: {
        sse: '/sse',
        rpc: '/rpc',
        health: '/health'
      },
      capabilities: {
        tools: true,
        resources: true,
        prompts: true
      }
    });
  });

  const httpServer = app.listen(port, () => {
    console.log(`
╔═══════════════════════════════════════════════════════════════╗
║  Agentics Hackathon MCP Server (SSE)                          ║
╠═══════════════════════════════════════════════════════════════╣
║  Status:    Running                                           ║
║  Port:      ${String(port).padEnd(50)}║
║  SSE:       http://localhost:${port}/sse                         ║
║  RPC:       http://localhost:${port}/rpc                         ║
║  Health:    http://localhost:${port}/health                      ║
║  Security:  Helmet, Rate Limiting (100/15min), Localhost CORS ║
╚═══════════════════════════════════════════════════════════════╝
    `);
  });

  // Graceful shutdown - cleanup all intervals
  const gracefulShutdown = () => {
    console.log('\nShutting down gracefully...');

    // Clear all active intervals
    activeIntervals.forEach(interval => clearInterval(interval));
    activeIntervals.clear();

    // Close server
    httpServer.close(() => {
      console.log('Server closed');
      process.exit(0);
    });

    // Force close after 10 seconds
    setTimeout(() => {
      console.error('Forcing shutdown...');
      process.exit(1);
    }, 10000);
  };

  process.on('SIGTERM', gracefulShutdown);
  process.on('SIGINT', gracefulShutdown);
}

// Run if called directly
if (process.argv[1]?.endsWith('sse.js')) {
  const port = parseInt(process.env.PORT || String(DEFAULT_PORT), 10);
  startSseServer(port);
}
