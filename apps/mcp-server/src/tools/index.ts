/**
 * MCP Tools Registry
 */

import { MCPTool } from '../types/index.js';
import { semanticSearchTool, executeSemanticSearch } from './semantic_search.js';
import { getRecommendationsTool, executeGetRecommendations } from './get_recommendations.js';
import { checkAvailabilityTool, executeCheckAvailability } from './check_availability.js';
import { getContentDetailsTool, executeGetContentDetails } from './get_content_details.js';
import { listDevicesTool, executeListDevices } from './list_devices.js';
import { initiatePlaybackTool, executeInitiatePlayback } from './initiate_playback.js';
import { getGenresTool, executeGetGenres } from './get_genres.js';

export const mcpTools: MCPTool[] = [
  semanticSearchTool,
  getRecommendationsTool,
  checkAvailabilityTool,
  getContentDetailsTool,
  listDevicesTool,
  initiatePlaybackTool,
  getGenresTool,
];

export const toolExecutors = {
  semantic_search: executeSemanticSearch,
  get_recommendations: executeGetRecommendations,
  check_availability: executeCheckAvailability,
  get_content_details: executeGetContentDetails,
  list_devices: executeListDevices,
  initiate_playback: executeInitiatePlayback,
  get_genres: executeGetGenres,
};

export {
  semanticSearchTool,
  getRecommendationsTool,
  checkAvailabilityTool,
  getContentDetailsTool,
  listDevicesTool,
  initiatePlaybackTool,
  getGenresTool,
};
