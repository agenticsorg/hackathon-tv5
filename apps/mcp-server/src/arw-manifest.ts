/**
 * ARW (AI-Readable Web) Manifest
 * Declares MCP capabilities and provides machine-readable content
 */

export const arwManifest = {
  $schema: 'https://arw.agentics.org/schemas/manifest-v1.json',
  version: '1.0.0',

  site: {
    name: 'Media Gateway',
    description: 'Unified cross-platform TV and movie discovery engine with AI-powered recommendations',
    url: 'https://media-gateway.com',
    contact: {
      email: 'support@media-gateway.com',
    },
  },

  capabilities: {
    mcp: {
      version: '2024-11-05',
      transports: ['stdio', 'sse'],
      endpoints: {
        stdio: {
          command: 'npx',
          args: ['@media-gateway/mcp-server', '--transport', 'stdio'],
        },
        sse: {
          url: 'https://api.media-gateway.com/mcp',
          events: '/events',
          tools: '/tools/call',
          resources: '/resources',
          prompts: '/prompts/get',
        },
      },
    },
  },

  content: {
    machineViews: [
      {
        uri: 'llm://home',
        name: 'Homepage',
        description: 'AI-optimized homepage with trending content and quick actions',
        mimeType: 'text/markdown',
        tokenReduction: '85%',
      },
      {
        uri: 'llm://search',
        name: 'Search Interface',
        description: 'AI-optimized search interface with semantic capabilities',
        mimeType: 'text/markdown',
        tokenReduction: '85%',
      },
    ],
  },

  actions: [
    {
      name: 'semantic_search',
      description: 'Search for movies and TV shows using natural language',
      endpoint: '/mcp/tools/call',
      method: 'POST',
      inputSchema: {
        type: 'object',
        properties: {
          query: { type: 'string' },
          filters: { type: 'object' },
          limit: { type: 'number' },
        },
        required: ['query'],
      },
      rateLimit: {
        authenticated: '1000/15min',
        unauthenticated: '10/15min',
      },
    },
    {
      name: 'get_recommendations',
      description: 'Get personalized content recommendations',
      endpoint: '/mcp/tools/call',
      method: 'POST',
      authentication: 'optional',
      rateLimit: {
        authenticated: '1000/15min',
        unauthenticated: '10/15min',
      },
    },
    {
      name: 'check_availability',
      description: 'Check where content is available to watch',
      endpoint: '/mcp/tools/call',
      method: 'POST',
      rateLimit: {
        authenticated: '1000/15min',
        unauthenticated: '10/15min',
      },
    },
  ],

  protocols: {
    rest: {
      baseUrl: 'https://api.media-gateway.com/api/v1',
      documentation: 'https://docs.media-gateway.com',
      openapi: 'https://api.media-gateway.com/openapi.yaml',
    },
    mcp: {
      version: '2024-11-05',
      tools: 7,
      resources: 7,
      prompts: 3,
    },
  },

  policies: {
    training: {
      allowed: true,
      attribution: 'required',
      restrictions: ['No commercial use without agreement'],
    },
    inference: {
      allowed: true,
      rateLimit: {
        free: '10 requests per 15 minutes',
        authenticated: '1000 requests per 15 minutes',
      },
    },
    attribution: {
      required: true,
      format: 'Powered by Media Gateway (https://media-gateway.com)',
    },
  },

  metadata: {
    version: '1.0.0',
    updated: new Date().toISOString(),
    tags: ['media', 'streaming', 'movies', 'tv', 'ai', 'recommendations', 'mcp'],
  },
};
