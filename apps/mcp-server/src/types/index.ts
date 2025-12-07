/**
 * MCP Server Type Definitions
 */

export interface MCPTool {
  name: string;
  description: string;
  inputSchema: {
    type: 'object';
    properties: Record<string, any>;
    required?: string[];
  };
}

export interface MCPResource {
  uri: string;
  name: string;
  description: string;
  mimeType?: string;
}

export interface MCPPrompt {
  name: string;
  description: string;
  arguments?: Array<{
    name: string;
    description: string;
    required: boolean;
  }>;
}

export interface MCPRequest {
  jsonrpc: '2.0';
  method: string;
  params?: any;
  id: string | number;
}

export interface MCPResponse {
  jsonrpc: '2.0';
  result?: any;
  error?: MCPError;
  id: string | number | null;
}

export interface MCPError {
  code: number;
  message: string;
  data?: any;
}

export enum MCPErrorCode {
  PARSE_ERROR = -32700,
  INVALID_REQUEST = -32600,
  METHOD_NOT_FOUND = -32601,
  INVALID_PARAMS = -32602,
  INTERNAL_ERROR = -32603,
  TOOL_NOT_FOUND = -32000,
  TOOL_EXECUTION_ERROR = -32001,
  RESOURCE_NOT_FOUND = -32002,
  RATE_LIMIT_EXCEEDED = -32003,
  AUTHORIZATION_ERROR = -32004,
}

export interface UserContext {
  userId?: string;
  role?: string;
  scopes?: string[];
  tier?: 'free' | 'pro' | 'enterprise';
}

export interface ContentItem {
  id: string;
  contentType: 'movie' | 'tv';
  title: string;
  overview?: string;
  releaseDate?: string;
  genres?: Array<{ id: number; name: string }>;
  averageRating?: number;
  popularityScore?: number;
  availability?: PlatformAvailability[];
  thumbnail?: string;
}

export interface PlatformAvailability {
  platformId: string;
  platformName: string;
  region: string;
  type: 'subscription' | 'rent' | 'buy' | 'free';
  price?: number;
  currency?: string;
  deepLink: string;
  updatedAt: string;
}

export interface SearchResult extends ContentItem {
  score: number;
  explanation?: string;
}

export interface RecommendationResult extends ContentItem {
  score: number;
  reasoning?: string;
  sonaConfidence?: number;
}

export interface Device {
  id: string;
  name: string;
  type: 'mobile' | 'tablet' | 'desktop' | 'smart_tv' | 'streaming_device';
  capabilities: string[];
  online: boolean;
  lastSeen?: string;
}

export interface Genre {
  id: number;
  name: string;
  description?: string;
}

export interface Platform {
  id: string;
  name: string;
  type: 'subscription' | 'rental' | 'purchase';
  regions: string[];
  deepLinkPattern?: string;
}

/**
 * MCP Protocol 2024-11-05 - Extended Types
 */

export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

export interface LoggingSetLevelParams {
  level: LogLevel;
}

export interface LoggingSetLevelResult {
  previousLevel: LogLevel;
  currentLevel: LogLevel;
}

export interface CancelledNotificationParams {
  requestId: string;
  reason?: string;
}

export interface CompletionReference {
  type: 'ref/resource' | 'ref/prompt';
  uri?: string;
  name?: string;
}

export interface CompletionArgument {
  name: string;
  value: string;
}

export interface CompletionCompleteParams {
  ref: CompletionReference;
  argument: CompletionArgument;
}

export interface CompletionResult {
  completion: {
    values: string[];
    hasMore: boolean;
    total?: number;
  };
}
