/**
 * MCP Resources
 * Resources provide structured data that AI agents can query
 */

import { MCPResource } from '../types/index.js';
import { config } from '../config.js';

export const mcpResources: MCPResource[] = [
  {
    uri: 'hackathon://config',
    name: 'Hackathon Configuration',
    description: 'Media Gateway hackathon configuration and metadata',
    mimeType: 'application/json',
  },
  {
    uri: 'hackathon://tracks',
    name: 'Hackathon Tracks',
    description: 'Available hackathon submission tracks and criteria',
    mimeType: 'application/json',
  },
  {
    uri: 'media://trending',
    name: 'Trending Content',
    description: 'Currently trending movies and TV shows',
    mimeType: 'application/json',
  },
  {
    uri: 'media://genres',
    name: 'Content Genres',
    description: 'List of all available content genres',
    mimeType: 'application/json',
  },
  {
    uri: 'media://platforms',
    name: 'Streaming Platforms',
    description: 'Supported streaming platforms and their capabilities',
    mimeType: 'application/json',
  },
  {
    uri: 'llm://home',
    name: 'Homepage Machine View',
    description: 'ARW-optimized homepage content for LLM consumption (85% token reduction)',
    mimeType: 'text/markdown',
  },
  {
    uri: 'llm://search',
    name: 'Search Machine View',
    description: 'ARW-optimized search interface for LLM consumption',
    mimeType: 'text/markdown',
  },
];

/**
 * Retrieve resource content by URI
 */
export async function getResourceContent(uri: string): Promise<any> {
  switch (uri) {
    case 'hackathon://config':
      return {
        name: 'Media Gateway',
        version: config.apiVersion,
        description: 'Unified cross-platform TV and movie discovery engine',
        hackathon: {
          name: 'ARW Hackathon 2025',
          track: 'Model Context Protocol',
          features: ['MCP Server', 'ARW Manifest', 'AI Agent Integration'],
        },
        capabilities: {
          semantic_search: true,
          recommendations: true,
          cross_platform_availability: true,
          deep_linking: true,
          real_time_sync: true,
        },
      };

    case 'hackathon://tracks':
      return {
        tracks: [
          {
            id: 'mcp',
            name: 'Model Context Protocol',
            description: 'Implementations using MCP for AI agent integration',
            criteria: ['Tool implementation', 'Resource quality', 'Documentation'],
          },
          {
            id: 'arw',
            name: 'ARW Manifest',
            description: 'Machine-readable web content for AI agents',
            criteria: ['Token efficiency', 'Semantic quality', 'Coverage'],
          },
        ],
      };

    case 'media://trending':
      return fetchTrendingContent();

    case 'media://genres':
      return fetchGenres();

    case 'media://platforms':
      return fetchPlatforms();

    case 'llm://home':
      return generateHomepageMachineView();

    case 'llm://search':
      return generateSearchMachineView();

    default:
      throw new Error(`Resource not found: ${uri}`);
  }
}

async function fetchTrendingContent() {
  try {
    const response = await fetch(`${config.services.content}/api/v1/content/trending`, {
      headers: { 'Content-Type': 'application/json' },
    });
    if (!response.ok) throw new Error('Failed to fetch trending content');
    return await response.json();
  } catch (error) {
    console.error('Fetch trending error:', error);
    return { trending: [] };
  }
}

async function fetchGenres() {
  try {
    const response = await fetch(`${config.services.content}/api/v1/genres`, {
      headers: { 'Content-Type': 'application/json' },
    });
    if (!response.ok) throw new Error('Failed to fetch genres');
    return await response.json();
  } catch (error) {
    console.error('Fetch genres error:', error);
    return { genres: [] };
  }
}

async function fetchPlatforms() {
  try {
    const response = await fetch(`${config.services.content}/api/v1/platforms`, {
      headers: { 'Content-Type': 'application/json' },
    });
    if (!response.ok) throw new Error('Failed to fetch platforms');
    return await response.json();
  } catch (error) {
    console.error('Fetch platforms error:', error);
    return { platforms: [] };
  }
}

function generateHomepageMachineView(): string {
  return `# Media Gateway - AI-Optimized Homepage

## Quick Actions
- Search for content: Use semantic_search tool
- Get recommendations: Use get_recommendations tool
- Check availability: Use check_availability tool

## Trending Now
[Query media://trending resource for current trending content]

## Available Platforms
[Query media://platforms resource for streaming platforms]

## Genres
[Query media://genres resource for available genres]

*This view is optimized for AI agents with 85% token reduction vs HTML*
`;
}

function generateSearchMachineView(): string {
  return `# Media Gateway Search - AI-Optimized Interface

## Search Capabilities

### Semantic Search
Use natural language to find content:
- "mind-bending thrillers with unreliable narrators"
- "feel-good comedies for family movie night"
- "critically acclaimed sci-fi from the 2010s"

### Filters
- Media type: movie, tv, all
- Rating: 0-10 scale
- Release year: min/max
- Genres: by ID

### Example Query
\`\`\`json
{
  "query": "psychological thrillers like Inception",
  "filters": {
    "mediaType": "movie",
    "ratingMin": 7.5,
    "releaseYearMin": 2000
  },
  "limit": 10,
  "explain": true
}
\`\`\`

*Use semantic_search tool for actual searches*
`;
}
