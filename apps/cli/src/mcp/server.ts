/**
 * MCP (Model Context Protocol) Server Implementation
 * Provides tools and resources for hackathon projects
 */

import type { McpRequest, McpResponse } from '../types.js';
import {
  HACKATHON_NAME,
  TRACKS,
  AVAILABLE_TOOLS,
  DISCORD_URL,
  WEBSITE_URL
} from '../constants.js';
import { loadConfig, checkToolInstalled } from '../utils/index.js';

export class McpServer {
  private handlers: Map<string, (params?: Record<string, unknown>) => Promise<unknown>>;

  constructor() {
    this.handlers = new Map();
    this.registerHandlers();
  }

  private registerHandlers(): void {
    // Initialize handler
    this.handlers.set('initialize', async () => ({
      protocolVersion: '2024-11-05',
      capabilities: {
        tools: {},
        resources: {},
        prompts: {}
      },
      serverInfo: {
        name: 'agentics-hackathon',
        version: '1.0.0'
      }
    }));

    // List tools
    this.handlers.set('tools/list', async () => ({
      tools: [
        {
          name: 'get_hackathon_info',
          description: 'Get information about the Agentics TV5 Hackathon',
          inputSchema: { type: 'object', properties: {} }
        },
        {
          name: 'get_tracks',
          description: 'Get available hackathon tracks and their descriptions',
          inputSchema: { type: 'object', properties: {} }
        },
        {
          name: 'get_available_tools',
          description: 'List available tools for hackathon development',
          inputSchema: {
            type: 'object',
            properties: {
              category: {
                type: 'string',
                enum: ['ai-assistants', 'orchestration', 'databases', 'cloud-platform', 'synthesis'],
                description: 'Filter by category'
              }
            }
          }
        },
        {
          name: 'get_project_status',
          description: 'Get the current hackathon project status and configuration',
          inputSchema: { type: 'object', properties: {} }
        },
        {
          name: 'check_tool_installed',
          description: 'Check if a specific tool is installed',
          inputSchema: {
            type: 'object',
            properties: {
              toolName: { type: 'string', description: 'Name of the tool to check' }
            },
            required: ['toolName']
          }
        },
        {
          name: 'get_resources',
          description: 'Get hackathon resources and links',
          inputSchema: { type: 'object', properties: {} }
        }
      ]
    }));

    // Call tool
    this.handlers.set('tools/call', async (params) => {
      // Validate required parameters
      if (!params || typeof params !== 'object') {
        throw new Error('Invalid params: params must be an object');
      }

      const name = params.name;
      if (!name || typeof name !== 'string') {
        throw new Error('Invalid params: name is required and must be a string');
      }

      const args = (params.arguments as Record<string, unknown>) || {};
      if (typeof args !== 'object' || args === null) {
        throw new Error('Invalid params: arguments must be an object');
      }

      switch (name) {
        case 'get_hackathon_info':
          return {
            content: [{
              type: 'text',
              text: JSON.stringify({
                name: HACKATHON_NAME,
                tagline: 'Building the Future of Agentic AI',
                sponsor: 'Google Cloud',
                description: 'Every night, millions spend up to 45 minutes deciding what to watch. Join us to build agentic AI solutions.',
                website: WEBSITE_URL,
                discord: DISCORD_URL
              }, null, 2)
            }]
          };

        case 'get_tracks':
          return {
            content: [{
              type: 'text',
              text: JSON.stringify(TRACKS, null, 2)
            }]
          };

        case 'get_available_tools':
          // Validate category parameter if provided
          const category = args.category as string | undefined;
          if (category !== undefined && typeof category !== 'string') {
            throw new Error('Invalid params: category must be a string');
          }
          if (category && !['ai-assistants', 'orchestration', 'databases', 'cloud-platform', 'synthesis'].includes(category)) {
            throw new Error(`Invalid params: category must be one of: ai-assistants, orchestration, databases, cloud-platform, synthesis`);
          }

          const tools = category
            ? AVAILABLE_TOOLS.filter(t => t.category === category)
            : AVAILABLE_TOOLS;
          return {
            content: [{
              type: 'text',
              text: JSON.stringify(tools.map(t => ({
                name: t.name,
                displayName: t.displayName,
                description: t.description,
                installCommand: t.installCommand,
                category: t.category
              })), null, 2)
            }]
          };

        case 'get_project_status':
          const config = loadConfig();
          return {
            content: [{
              type: 'text',
              text: JSON.stringify(config || { initialized: false }, null, 2)
            }]
          };

        case 'check_tool_installed':
          // Validate required toolName parameter
          const toolName = args.toolName as string | undefined;
          if (!toolName || typeof toolName !== 'string') {
            throw new Error('Invalid params: toolName is required and must be a string');
          }

          const tool = AVAILABLE_TOOLS.find(t => t.name === toolName);
          if (!tool) {
            throw new Error(`Unknown tool: ${toolName}. Available tools: ${AVAILABLE_TOOLS.map(t => t.name).join(', ')}`);
          }

          const installed = await checkToolInstalled(tool);
          return {
            content: [{
              type: 'text',
              text: JSON.stringify({ tool: toolName, installed })
            }]
          };

        case 'get_resources':
          return {
            content: [{
              type: 'text',
              text: JSON.stringify({
                website: WEBSITE_URL,
                discord: DISCORD_URL,
                googleAdk: 'https://google.github.io/adk-docs/',
                vertexAi: 'https://cloud.google.com/vertex-ai/docs',
                claudeDocs: 'https://docs.anthropic.com',
                geminiDocs: 'https://ai.google.dev/gemini-api/docs'
              }, null, 2)
            }]
          };

        default:
          throw new Error(`Unknown tool: ${name}. Available tools: get_hackathon_info, get_tracks, get_available_tools, get_project_status, check_tool_installed, get_resources`);
      }
    });

    // List resources
    this.handlers.set('resources/list', async () => ({
      resources: [
        {
          uri: 'hackathon://config',
          name: 'Hackathon Configuration',
          description: 'Current project configuration',
          mimeType: 'application/json'
        },
        {
          uri: 'hackathon://tracks',
          name: 'Hackathon Tracks',
          description: 'Available hackathon tracks',
          mimeType: 'application/json'
        }
      ]
    }));

    // Read resource
    this.handlers.set('resources/read', async (params) => {
      // Validate required parameters
      if (!params || typeof params !== 'object') {
        throw new Error('Invalid params: params must be an object');
      }

      const uri = params.uri;
      if (!uri || typeof uri !== 'string') {
        throw new Error('Invalid params: uri is required and must be a string');
      }

      switch (uri) {
        case 'hackathon://config':
          return {
            contents: [{
              uri,
              mimeType: 'application/json',
              text: JSON.stringify(loadConfig() || { initialized: false }, null, 2)
            }]
          };

        case 'hackathon://tracks':
          return {
            contents: [{
              uri,
              mimeType: 'application/json',
              text: JSON.stringify(TRACKS, null, 2)
            }]
          };

        default:
          throw new Error(`Unknown resource: ${uri}`);
      }
    });

    // List prompts
    this.handlers.set('prompts/list', async () => ({
      prompts: [
        {
          name: 'hackathon_starter',
          description: 'Get started with the hackathon',
          arguments: []
        },
        {
          name: 'choose_track',
          description: 'Help choosing a hackathon track',
          arguments: []
        }
      ]
    }));

    // Get prompt
    this.handlers.set('prompts/get', async (params) => {
      // Validate required parameters
      if (!params || typeof params !== 'object') {
        throw new Error('Invalid params: params must be an object');
      }

      const name = params.name;
      if (!name || typeof name !== 'string') {
        throw new Error('Invalid params: name is required and must be a string');
      }

      switch (name) {
        case 'hackathon_starter':
          return {
            description: 'Get started with the Agentics TV5 Hackathon',
            messages: [{
              role: 'user',
              content: {
                type: 'text',
                text: `I'm participating in the ${HACKATHON_NAME}. Help me get started by:\n` +
                  `1. Understanding the hackathon tracks\n` +
                  `2. Setting up my development environment\n` +
                  `3. Choosing appropriate tools\n` +
                  `4. Creating an initial project structure`
              }
            }]
          };

        case 'choose_track':
          return {
            description: 'Help choosing a hackathon track',
            messages: [{
              role: 'user',
              content: {
                type: 'text',
                text: `Help me choose a hackathon track. The available tracks are:\n` +
                  Object.entries(TRACKS)
                    .map(([key, { name, description }]) => `- ${name}: ${description}`)
                    .join('\n') +
                  `\n\nWhat are my interests and skills?`
              }
            }]
          };

        default:
          throw new Error(`Unknown prompt: ${name}`);
      }
    });
  }

  async handleRequest(request: McpRequest): Promise<McpResponse> {
    const handler = this.handlers.get(request.method);

    if (!handler) {
      return {
        jsonrpc: '2.0',
        id: request.id,
        error: {
          code: -32601,
          message: `Method not found: ${request.method}`
        }
      };
    }

    try {
      const result = await handler(request.params);
      return {
        jsonrpc: '2.0',
        id: request.id,
        result
      };
    } catch (error) {
      return {
        jsonrpc: '2.0',
        id: request.id,
        error: {
          code: -32603,
          message: error instanceof Error ? error.message : 'Internal error'
        }
      };
    }
  }
}
