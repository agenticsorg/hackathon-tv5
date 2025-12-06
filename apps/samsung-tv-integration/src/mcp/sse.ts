#!/usr/bin/env node
import express from 'express';
import { processRequest, MCP_TOOLS } from './server.js';

const app = express();
app.use(express.json());

const PORT = process.env.MCP_PORT || 3456;

/**
 * MCP Server - SSE Transport
 *
 * This transport allows web clients to connect via Server-Sent Events
 * and send commands via POST requests
 */

// Store active SSE connections
const connections = new Map<string, express.Response>();

// SSE endpoint for receiving events
app.get('/sse', (req, res) => {
  const clientId = `client-${Date.now()}-${Math.random().toString(36).slice(2)}`;

  res.setHeader('Content-Type', 'text/event-stream');
  res.setHeader('Cache-Control', 'no-cache');
  res.setHeader('Connection', 'keep-alive');
  res.setHeader('Access-Control-Allow-Origin', '*');

  // Send initial connection event
  res.write(`data: ${JSON.stringify({ type: 'connected', clientId })}\n\n`);

  connections.set(clientId, res);

  // Handle client disconnect
  req.on('close', () => {
    connections.delete(clientId);
  });
});

// POST endpoint for sending MCP requests
app.post('/message', async (req, res) => {
  try {
    const request = req.body;
    const response = await processRequest(request);
    res.json(response);
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Internal error';
    res.status(500).json({
      jsonrpc: '2.0',
      error: { code: -32603, message },
    });
  }
});

// Tool call endpoint (convenience)
app.post('/tools/:toolName', async (req, res) => {
  try {
    const { toolName } = req.params;
    const response = await processRequest({
      method: 'tools/call',
      params: {
        name: `samsung_tv_${toolName}`,
        arguments: req.body,
      },
      id: Date.now(),
    });
    res.json(response);
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Internal error';
    res.status(500).json({ error: message });
  }
});

// List available tools
app.get('/tools', (req, res) => {
  res.json({ tools: MCP_TOOLS });
});

// Health check
app.get('/health', (req, res) => {
  res.json({ status: 'ok', transport: 'sse', port: PORT });
});

// CORS preflight
app.options('*', (req, res) => {
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
  res.sendStatus(204);
});

// Start server
app.listen(PORT, () => {
  console.log(`Samsung TV MCP Server (SSE) running on http://localhost:${PORT}`);
  console.log(`  SSE endpoint: http://localhost:${PORT}/sse`);
  console.log(`  Message endpoint: http://localhost:${PORT}/message`);
  console.log(`  Tools list: http://localhost:${PORT}/tools`);
});

// Broadcast function for sending events to all connected clients
export function broadcast(event: unknown) {
  const message = `data: ${JSON.stringify(event)}\n\n`;
  connections.forEach((res) => {
    res.write(message);
  });
}
