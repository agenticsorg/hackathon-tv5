/**
 * Content Discovery Module
 *
 * Provides content metadata fetching from TMDb for the learning system
 */

export {
  TMDbClient,
  createTMDbClient,
  type TMDbConfig,
  type TMDbMovie,
  type TMDbTVShow,
  type TMDbSearchResult,
  type TMDbCredits,
  type TMDbWatchProviders,
} from './tmdb-client.js';

export {
  DISCOVERY_TOOLS,
  handleDiscoveryToolCall,
  initContentDiscovery,
  getDiscoveryTools,
} from './discovery-tools.js';
