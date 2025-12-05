/**
 * Agentics Foundation TV5 Hackathon CLI Types
 */

export interface HackathonConfig {
  projectName: string;
  teamName?: string;
  track?: HackathonTrack;
  tools: ToolSelection;
  mcpEnabled: boolean;
  discordLinked: boolean;
  initialized: boolean;
  createdAt: string;
}

export type HackathonTrack =
  | 'entertainment-discovery'
  | 'multi-agent-systems'
  | 'agentic-workflows'
  | 'open-innovation';

export interface ToolSelection {
  // AI Assistants
  claudeCode: boolean;
  geminiCli: boolean;
  // Orchestration
  claudeFlow: boolean;
  agenticFlow: boolean;
  flowNexus: boolean;
  adk: boolean;
  // Cloud Platform
  googleCloudCli: boolean;
  vertexAi: boolean;
  // Databases
  ruvector: boolean;
  agentDb: boolean;
  // Synthesis
  agenticSynth: boolean;
  strangeLoops: boolean;
  sparc: boolean;
  // Python Frameworks
  lionpride: boolean;
  agenticFramework: boolean;
  openaiAgents: boolean;
}

export interface Tool {
  name: string;
  displayName: string;
  description: string;
  installCommand: string;
  verifyCommand: string;
  docUrl: string;
  required: boolean;
  category: ToolCategory;
}

export type ToolCategory =
  | 'ai-assistants'
  | 'orchestration'
  | 'databases'
  | 'cloud-platform'
  | 'synthesis'
  | 'python-frameworks';

export interface InstallProgress {
  tool: string;
  status: 'pending' | 'installing' | 'success' | 'failed' | 'skipped';
  message?: string;
}

export interface McpServerConfig {
  transport: 'stdio' | 'sse';
  port?: number;
  host?: string;
}

export interface McpRequest {
  jsonrpc: '2.0';
  id: string | number;
  method: string;
  params?: Record<string, unknown>;
}

export interface McpResponse {
  jsonrpc: '2.0';
  id: string | number;
  result?: unknown;
  error?: {
    code: number;
    message: string;
    data?: unknown;
  };
}
