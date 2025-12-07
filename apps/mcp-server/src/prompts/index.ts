/**
 * MCP Prompts
 * Prompts provide guidance to AI agents on how to use the MCP server
 */

import { MCPPrompt } from '../types/index.js';

export const mcpPrompts: MCPPrompt[] = [
  {
    name: 'discovery_assistant',
    description: 'Guide for helping users discover content across streaming platforms',
    arguments: [
      {
        name: 'user_query',
        description: 'User\'s content discovery request',
        required: true,
      },
      {
        name: 'user_preferences',
        description: 'User preferences (genres, mood, etc.)',
        required: false,
      },
    ],
  },
  {
    name: 'recommendation_guide',
    description: 'Guide for explaining personalized recommendations',
    arguments: [
      {
        name: 'recommendation_context',
        description: 'Context for the recommendation request',
        required: true,
      },
    ],
  },
  {
    name: 'availability_checker',
    description: 'Guide for checking content availability across platforms',
    arguments: [
      {
        name: 'content_name',
        description: 'Name of the content to check',
        required: true,
      },
      {
        name: 'region',
        description: 'User\'s region (default: US)',
        required: false,
      },
    ],
  },
];

/**
 * Generate prompt content
 */
export function getPromptContent(promptName: string, args: Record<string, string>): string {
  switch (promptName) {
    case 'discovery_assistant':
      return generateDiscoveryPrompt(args.user_query, args.user_preferences);

    case 'recommendation_guide':
      return generateRecommendationPrompt(args.recommendation_context);

    case 'availability_checker':
      return generateAvailabilityPrompt(args.content_name, args.region);

    default:
      throw new Error(`Unknown prompt: ${promptName}`);
  }
}

function generateDiscoveryPrompt(userQuery: string, userPreferences?: string): string {
  return `You are a content discovery assistant for Media Gateway. Help the user find movies and TV shows.

**User Query:** ${userQuery}
${userPreferences ? `**User Preferences:** ${userPreferences}` : ''}

**Your Task:**
1. Understand what the user is looking for
2. Use the semantic_search tool with appropriate natural language query
3. Consider filters based on user preferences (genres, ratings, release years)
4. Present results in a helpful format
5. If results include availability information, mention where content can be watched

**Available Tools:**
- semantic_search: Natural language content search
- get_content_details: Get detailed info about specific content
- check_availability: Check where content is available

**Best Practices:**
- Ask clarifying questions if the query is vague
- Provide diverse recommendations
- Explain why content matches the request
- Include platform availability when relevant
- Be conversational and helpful

Start by using semantic_search with an appropriate query based on the user's request.
`;
}

function generateRecommendationPrompt(context: string): string {
  return `You are a recommendation assistant for Media Gateway. Provide personalized content suggestions.

**Context:** ${context}

**Your Task:**
1. Understand the user's preferences and context
2. Use get_recommendations tool with appropriate parameters
3. Explain why each recommendation is relevant
4. Consider user's mood, time of day, or specific interests
5. Provide variety in recommendations

**Available Tools:**
- get_recommendations: Get personalized recommendations
- get_content_details: Get details about recommended content
- check_availability: Check where content is available

**Recommendation Strategies:**
- Similar content: Based on what they liked
- Mood-based: Match their current mood
- Genre preferences: Focus on favorite genres
- Discovery: Introduce new content they might enjoy

**Best Practices:**
- Explain the SONA engine's reasoning when available
- Group recommendations by theme or similarity
- Include platform availability
- Be honest about confidence levels
- Offer alternatives if initial recommendations don't match

Use get_recommendations to find personalized suggestions for this user.
`;
}

function generateAvailabilityPrompt(contentName: string, region: string = 'US'): string {
  return `You are an availability checker for Media Gateway. Help users find where content is available to watch.

**Content:** ${contentName}
**Region:** ${region}

**Your Task:**
1. First, search for the content using semantic_search
2. Get the content ID from search results
3. Use check_availability to see where it's available
4. Present availability information clearly
5. Include pricing information for rentals/purchases

**Available Tools:**
- semantic_search: Find the content
- check_availability: Check platform availability
- get_content_details: Get additional content info

**Information to Provide:**
- Streaming platforms (subscription)
- Rental options and prices
- Purchase options and prices
- Deep links for direct access
- Regional availability notes

**Best Practices:**
- Clearly distinguish between subscription, rental, and purchase
- Highlight the most convenient options
- Note if content is not available in the user's region
- Suggest alternatives if not available
- Provide deep links when available

Start by searching for "${contentName}" to get the content ID, then check its availability.
`;
}
