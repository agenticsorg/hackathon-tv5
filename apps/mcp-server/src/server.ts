/**
 * MCP Server Core
 * Handles tool calls, resource requests, and prompt generation
 */

import {
  MCPErrorCode,
  UserContext,
  LogLevel,
  LoggingSetLevelParams,
  LoggingSetLevelResult,
  CancelledNotificationParams,
  CompletionCompleteParams,
  CompletionResult
} from './types/index.js';
import { mcpTools, toolExecutors } from './tools/index.js';
import { mcpResources, getResourceContent } from './resources/index.js';
import { mcpPrompts, getPromptContent } from './prompts/index.js';

/**
 * Server state management
 */
let currentLogLevel: LogLevel = 'info';
const activeRequests = new Map<string, AbortController>();

/**
 * Main request handler for MCP operations
 */
export async function handleMCPRequest(method: string, params: any, requestId?: string): Promise<any> {
  // Log request based on current log level
  logMessage('debug', `[Server] Handling request: ${method}`, { requestId, params });

  switch (method) {
    case 'initialize':
      return handleInitialize();

    case 'ping':
      return handlePing();

    case 'tools/list':
      return handleToolsList();

    case 'tools/call':
      return handleToolCall(params, requestId);

    case 'resources/list':
      return handleResourcesList();

    case 'resources/read':
      return handleResourceRead(params);

    case 'prompts/list':
      return handlePromptsList();

    case 'prompts/get':
      return handlePromptGet(params);

    case 'logging/setLevel':
      return handleLoggingSetLevel(params);

    case 'completion/complete':
      return handleCompletionComplete(params);

    case 'notifications/initialized':
      return handleNotificationInitialized();

    case 'notifications/cancelled':
      return handleNotificationCancelled(params);

    default:
      throw {
        code: MCPErrorCode.METHOD_NOT_FOUND,
        message: `Method not found: ${method}`,
      };
  }
}

/**
 * Initialize MCP server capabilities
 */
function handleInitialize() {
  logMessage('info', '[Server] Initializing MCP server');
  return {
    protocolVersion: '2024-11-05',
    capabilities: {
      tools: {
        listChanged: false,
      },
      resources: {
        listChanged: false,
      },
      prompts: {
        listChanged: false,
      },
      logging: {},
    },
    serverInfo: {
      name: 'media-gateway-mcp',
      version: '1.0.0',
    },
  };
}

/**
 * Ping handler for connection health checks
 * MCP Protocol 2024-11-05
 */
function handlePing(): {} {
  logMessage('debug', '[Server] Ping received');
  return {};
}

/**
 * List available tools
 */
function handleToolsList() {
  return {
    tools: mcpTools,
  };
}

/**
 * Execute a tool
 */
async function handleToolCall(
  params: {
    name: string;
    arguments: any;
    userContext?: UserContext;
  },
  requestId?: string
): Promise<any> {
  const { name, arguments: args, userContext } = params;

  const executor = (toolExecutors as any)[name];
  if (!executor) {
    throw {
      code: MCPErrorCode.TOOL_NOT_FOUND,
      message: `Tool not found: ${name}`,
    };
  }

  // Register request for potential cancellation
  if (requestId) {
    const abortController = new AbortController();
    activeRequests.set(requestId, abortController);
  }

  try {
    logMessage('debug', `[Server] Executing tool: ${name}`, { requestId, args });
    const result = await executor(args, userContext);

    // Clean up request tracking
    if (requestId) {
      activeRequests.delete(requestId);
    }

    return {
      content: [
        {
          type: 'text',
          text: JSON.stringify(result, null, 2),
        },
      ],
    };
  } catch (error) {
    // Clean up request tracking on error
    if (requestId) {
      activeRequests.delete(requestId);
    }

    logMessage('error', `[Server] Tool execution error for ${name}`, { error });
    throw {
      code: MCPErrorCode.TOOL_EXECUTION_ERROR,
      message: error instanceof Error ? error.message : 'Tool execution failed',
      data: error,
    };
  }
}

/**
 * List available resources
 */
function handleResourcesList() {
  return {
    resources: mcpResources,
  };
}

/**
 * Read a resource
 */
async function handleResourceRead(params: { uri: string }): Promise<any> {
  const { uri } = params;

  try {
    const content = await getResourceContent(uri);
    const resource = mcpResources.find((r) => r.uri === uri);

    if (!resource) {
      throw new Error(`Resource not found: ${uri}`);
    }

    return {
      contents: [
        {
          uri,
          mimeType: resource.mimeType || 'application/json',
          text: typeof content === 'string' ? content : JSON.stringify(content, null, 2),
        },
      ],
    };
  } catch (error) {
    logMessage('error', `[Server] Resource read error for ${uri}`, { error });
    throw {
      code: MCPErrorCode.RESOURCE_NOT_FOUND,
      message: error instanceof Error ? error.message : 'Resource not found',
    };
  }
}

/**
 * List available prompts
 */
function handlePromptsList() {
  return {
    prompts: mcpPrompts,
  };
}

/**
 * Get a prompt
 */
function handlePromptGet(params: { name: string; arguments?: Record<string, string> }): any {
  const { name, arguments: args = {} } = params;

  const prompt = mcpPrompts.find((p) => p.name === name);
  if (!prompt) {
    throw {
      code: MCPErrorCode.RESOURCE_NOT_FOUND,
      message: `Prompt not found: ${name}`,
    };
  }

  // Validate required arguments
  const requiredArgs = prompt.arguments?.filter((arg) => arg.required) || [];
  for (const arg of requiredArgs) {
    if (!args[arg.name]) {
      throw {
        code: MCPErrorCode.INVALID_PARAMS,
        message: `Missing required argument: ${arg.name}`,
      };
    }
  }

  try {
    const content = getPromptContent(name, args);
    return {
      description: prompt.description,
      messages: [
        {
          role: 'user',
          content: {
            type: 'text',
            text: content,
          },
        },
      ],
    };
  } catch (error) {
    logMessage('error', `[Server] Prompt generation error for ${name}`, { error });
    throw {
      code: MCPErrorCode.INTERNAL_ERROR,
      message: error instanceof Error ? error.message : 'Prompt generation failed',
    };
  }
}

/**
 * Handle notifications/initialized
 * Client notification that initialization is complete
 * MCP Protocol 2024-11-05
 */
function handleNotificationInitialized(): void {
  logMessage('info', '[Server] Client initialization complete');
  // No response needed for notifications
}

/**
 * Handle notifications/cancelled
 * Cancel in-flight request and clean up resources
 * MCP Protocol 2024-11-05
 */
function handleNotificationCancelled(params: CancelledNotificationParams): void {
  const { requestId, reason } = params;

  logMessage('info', '[Server] Request cancelled', { requestId, reason });

  const abortController = activeRequests.get(requestId);
  if (abortController) {
    abortController.abort();
    activeRequests.delete(requestId);
    logMessage('debug', '[Server] Aborted in-flight request', { requestId });
  } else {
    logMessage('warn', '[Server] Attempted to cancel non-existent request', { requestId });
  }

  // No response needed for notifications
}

/**
 * Handle logging/setLevel
 * Dynamically adjust server log level
 * MCP Protocol 2024-11-05
 */
function handleLoggingSetLevel(params: LoggingSetLevelParams): LoggingSetLevelResult {
  const { level } = params;

  // Validate log level
  const validLevels: LogLevel[] = ['debug', 'info', 'warn', 'error'];
  if (!validLevels.includes(level)) {
    throw {
      code: MCPErrorCode.INVALID_PARAMS,
      message: `Invalid log level: ${level}. Must be one of: ${validLevels.join(', ')}`,
    };
  }

  const previousLevel = currentLogLevel;
  currentLogLevel = level;

  logMessage('info', '[Server] Log level changed', { previousLevel, currentLevel: level });

  return {
    previousLevel,
    currentLevel: level,
  };
}

/**
 * Handle completion/complete
 * Return autocomplete suggestions for tool arguments
 * MCP Protocol 2024-11-05
 */
function handleCompletionComplete(params: CompletionCompleteParams): CompletionResult {
  const { ref, argument } = params;

  logMessage('debug', '[Server] Generating completions', { ref, argument });

  let values: string[] = [];

  // Generate completions based on reference type
  if (ref.type === 'ref/resource') {
    // Complete resource URIs
    if (argument.name === 'uri') {
      values = mcpResources
        .filter((r) => r.uri.includes(argument.value))
        .map((r) => r.uri)
        .slice(0, 10); // Limit to 10 suggestions
    }
  } else if (ref.type === 'ref/prompt') {
    // Complete prompt names or arguments
    if (argument.name === 'name') {
      values = mcpPrompts
        .filter((p) => p.name.includes(argument.value))
        .map((p) => p.name)
        .slice(0, 10);
    } else {
      // Complete prompt argument values based on the prompt
      const prompt = mcpPrompts.find((p) => p.name === ref.name);
      if (prompt) {
        const promptArg = prompt.arguments?.find((a) => a.name === argument.name);
        if (promptArg) {
          // Generate context-aware completions based on argument type
          values = generateArgumentCompletions(argument.name, argument.value);
        }
      }
    }
  }

  // For tool arguments
  if (ref.uri && ref.uri.startsWith('tool://')) {
    const toolName = ref.uri.replace('tool://', '');
    const tool = mcpTools.find((t) => t.name === toolName);

    if (tool) {
      const properties = tool.inputSchema.properties;
      const property = properties[argument.name];

      if (property) {
        values = generateToolArgumentCompletions(property, argument.value);
      }
    }
  }

  return {
    completion: {
      values,
      hasMore: false, // Could implement pagination in the future
      total: values.length,
    },
  };
}

/**
 * Generate completions for prompt arguments
 */
function generateArgumentCompletions(argName: string, partialValue: string): string[] {
  const completions: string[] = [];

  // Context-aware completions based on argument name
  switch (argName) {
    case 'genre':
      completions.push(
        'action', 'adventure', 'animation', 'comedy', 'crime',
        'documentary', 'drama', 'fantasy', 'horror', 'mystery',
        'romance', 'sci-fi', 'thriller'
      );
      break;
    case 'platform':
      completions.push(
        'netflix', 'hulu', 'disney-plus', 'amazon-prime',
        'hbo-max', 'apple-tv', 'paramount-plus'
      );
      break;
    case 'region':
      completions.push('US', 'UK', 'CA', 'AU', 'DE', 'FR', 'JP');
      break;
    case 'contentType':
      completions.push('movie', 'tv');
      break;
    default:
      // No specific completions
      break;
  }

  return completions
    .filter((c) => c.toLowerCase().includes(partialValue.toLowerCase()))
    .slice(0, 10);
}

/**
 * Generate completions for tool arguments based on schema
 */
function generateToolArgumentCompletions(property: any, partialValue: string): string[] {
  const completions: string[] = [];

  // If property has enum values, use those
  if (property.enum && Array.isArray(property.enum)) {
    completions.push(...property.enum);
  }

  // If property has examples, use those
  if (property.examples && Array.isArray(property.examples)) {
    completions.push(...property.examples);
  }

  return completions
    .filter((c) => String(c).toLowerCase().includes(partialValue.toLowerCase()))
    .slice(0, 10);
}

/**
 * Logging utility that respects current log level
 */
function logMessage(level: LogLevel, message: string, data?: any): void {
  const levels: LogLevel[] = ['debug', 'info', 'warn', 'error'];
  const currentLevelIndex = levels.indexOf(currentLogLevel);
  const messageLevelIndex = levels.indexOf(level);

  // Only log if message level is >= current log level
  if (messageLevelIndex >= currentLevelIndex) {
    const timestamp = new Date().toISOString();
    const logData = data ? ` ${JSON.stringify(data)}` : '';

    switch (level) {
      case 'debug':
      case 'info':
        console.log(`[${timestamp}] [${level.toUpperCase()}] ${message}${logData}`);
        break;
      case 'warn':
        console.warn(`[${timestamp}] [${level.toUpperCase()}] ${message}${logData}`);
        break;
      case 'error':
        console.error(`[${timestamp}] [${level.toUpperCase()}] ${message}${logData}`);
        break;
    }
  }
}

/**
 * Get current log level (for testing)
 */
export function getCurrentLogLevel(): LogLevel {
  return currentLogLevel;
}

/**
 * Get active requests count (for testing)
 */
export function getActiveRequestsCount(): number {
  return activeRequests.size;
}
