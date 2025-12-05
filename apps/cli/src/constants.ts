/**
 * Agentics Foundation TV5 Hackathon CLI Constants
 */

import type { Tool, HackathonTrack } from './types.js';

export const HACKATHON_NAME = 'Agentics Foundation TV5 Hackathon';
export const HACKATHON_TAGLINE = 'Building the Future of Agentic AI';
export const HACKATHON_SPONSOR = 'Google Cloud';
export const HACKATHON_DESCRIPTION = 'Every night, millions spend up to 45 minutes deciding what to watch — billions of hours lost every day. Not from lack of content, but from fragmentation. Join us to build the future of agentic AI systems.';
export const DISCORD_URL = 'https://discord.agentics.org';
export const WEBSITE_URL = 'https://agentics.org/hackathon';
export const GITHUB_URL = 'https://github.com/agenticsorg/hackathon-tv5';
export const CONFIG_FILE = '.hackathon.json';

export const TRACKS: Record<HackathonTrack, { name: string; description: string }> = {
  'entertainment-discovery': {
    name: 'Entertainment Discovery',
    description: 'Solve the 45-minute decision problem - help users find what to watch across fragmented content'
  },
  'multi-agent-systems': {
    name: 'Multi-Agent Systems',
    description: 'Build collaborative AI agents that work together using Google ADK and Vertex AI'
  },
  'agentic-workflows': {
    name: 'Agentic Workflows',
    description: 'Create autonomous workflows with Claude, Gemini, and orchestration tools'
  },
  'open-innovation': {
    name: 'Open Innovation',
    description: 'Bring your own idea - any agentic AI solution that makes an impact'
  }
};

export const AVAILABLE_TOOLS: Tool[] = [
  // AI Assistants
  {
    name: 'claudeCode',
    displayName: 'Claude Code CLI',
    description: 'Anthropic\'s official CLI for Claude - AI-powered coding assistant',
    installCommand: 'npm install -g @anthropic-ai/claude-code',
    verifyCommand: 'claude --version',
    docUrl: 'https://docs.anthropic.com/claude-code',
    required: false,
    category: 'ai-assistants'
  },
  {
    name: 'geminiCli',
    displayName: 'Google Gemini CLI',
    description: 'Command-line interface for Google Gemini models',
    installCommand: 'npm install -g @google/generative-ai-cli',
    verifyCommand: 'gemini --version',
    docUrl: 'https://ai.google.dev/gemini-api/docs',
    required: false,
    category: 'ai-assistants'
  },

  // Orchestration & Agent Frameworks
  {
    name: 'claudeFlow',
    displayName: 'Claude Flow',
    description: '#1 agent orchestration platform - multi-agent swarms, 101 MCP tools, RAG integration',
    installCommand: 'npx claude-flow@alpha init --force',
    verifyCommand: 'npx claude-flow --version',
    docUrl: 'https://github.com/ruvnet/claude-flow',
    required: false,
    category: 'orchestration'
  },
  {
    name: 'agenticFlow',
    displayName: 'Agentic Flow',
    description: 'Production AI orchestration - 66 agents, 213 MCP tools, ReasoningBank memory',
    installCommand: 'npx agentic-flow init',
    verifyCommand: 'npx agentic-flow --version',
    docUrl: 'https://github.com/ruvnet/agentic-flow',
    required: false,
    category: 'orchestration'
  },
  {
    name: 'flowNexus',
    displayName: 'Flow Nexus',
    description: 'Competitive agentic platform on MCP - deploy AI swarms, earn credits',
    installCommand: 'npx flow-nexus init',
    verifyCommand: 'npx flow-nexus --version',
    docUrl: 'https://github.com/ruvnet/flow-nexus',
    required: false,
    category: 'orchestration'
  },
  {
    name: 'adk',
    displayName: 'Google Agent Development Kit',
    description: 'Build multi-agent systems with Google\'s ADK',
    installCommand: 'pip install google-adk',
    verifyCommand: 'python -c "import google.adk"',
    docUrl: 'https://google.github.io/adk-docs/',
    required: false,
    category: 'orchestration'
  },

  // Cloud Platform
  {
    name: 'googleCloudCli',
    displayName: 'Google Cloud CLI (gcloud)',
    description: 'Google Cloud SDK for Vertex AI, Cloud Functions, and more',
    installCommand: 'curl https://sdk.cloud.google.com | bash',
    verifyCommand: 'gcloud --version',
    docUrl: 'https://cloud.google.com/sdk/docs/install',
    required: false,
    category: 'cloud-platform'
  },
  {
    name: 'vertexAi',
    displayName: 'Vertex AI SDK',
    description: 'Google Cloud\'s unified ML platform SDK',
    installCommand: 'pip install google-cloud-aiplatform',
    verifyCommand: 'python -c "import vertexai"',
    docUrl: 'https://cloud.google.com/vertex-ai/docs',
    required: false,
    category: 'cloud-platform'
  },

  // Databases & Memory
  {
    name: 'ruvector',
    displayName: 'RuVector',
    description: 'Vector database and embeddings toolkit for AI applications',
    installCommand: 'npm install ruvector',
    verifyCommand: 'npx ruvector --version',
    docUrl: 'https://ruv.io/projects',
    required: false,
    category: 'databases'
  },
  {
    name: 'agentDb',
    displayName: 'AgentDB',
    description: 'Database designed for agentic AI state management and memory',
    installCommand: 'npx agentdb init',
    verifyCommand: 'npx agentdb --version',
    docUrl: 'https://ruv.io/projects',
    required: false,
    category: 'databases'
  },

  // Synthesis & Advanced Tools
  {
    name: 'agenticSynth',
    displayName: 'Agentic Synth',
    description: 'Synthesis tools for agentic AI development',
    installCommand: 'npx @ruvector/agentic-synth init',
    verifyCommand: 'npx @ruvector/agentic-synth --version',
    docUrl: 'https://ruv.io/projects',
    required: false,
    category: 'synthesis'
  },
  {
    name: 'strangeLoops',
    displayName: 'Strange Loops',
    description: 'Consciousness exploration SDK - emergent intelligence, 500K+ ops/sec nano-agents',
    installCommand: 'npx strange-loops init',
    verifyCommand: 'npx strange-loops --version',
    docUrl: 'https://github.com/ruvnet/strange-loops',
    required: false,
    category: 'synthesis'
  },
  {
    name: 'sparc',
    displayName: 'SPARC 2.0',
    description: 'Autonomous vector coding agent with MCP - intelligent code analysis',
    installCommand: 'npx sparc init',
    verifyCommand: 'npx sparc --version',
    docUrl: 'https://github.com/ruvnet/sparc',
    required: false,
    category: 'synthesis'
  },

  // Python Frameworks
  {
    name: 'lionpride',
    displayName: 'LionPride',
    description: 'Python agentic AI framework for building intelligent agent systems',
    installCommand: 'pip install lionpride',
    verifyCommand: 'python -c "import lionpride"',
    docUrl: 'https://pypi.org/project/lionpride/',
    required: false,
    category: 'python-frameworks'
  },
  {
    name: 'agenticFramework',
    displayName: 'Agentic Framework',
    description: 'Python framework for creating AI agents with natural language & tools',
    installCommand: 'pip install agentic-framework',
    verifyCommand: 'python -c "import agentic"',
    docUrl: 'https://pypi.org/project/agentic-framework/',
    required: false,
    category: 'python-frameworks'
  },
  {
    name: 'openaiAgents',
    displayName: 'OpenAI Agents SDK',
    description: 'Lightweight framework for multi-agent workflows from OpenAI',
    installCommand: 'pip install openai-agents',
    verifyCommand: 'python -c "import agents"',
    docUrl: 'https://github.com/openai/openai-agents-python',
    required: false,
    category: 'python-frameworks'
  }
];

export const BANNER = `
╔═══════════════════════════════════════════════════════════════════════════════╗
║                                                                               ║
║     █████╗  ██████╗ ███████╗███╗   ██╗████████╗██╗ ██████╗███████╗           ║
║    ██╔══██╗██╔════╝ ██╔════╝████╗  ██║╚══██╔══╝██║██╔════╝██╔════╝           ║
║    ███████║██║  ███╗█████╗  ██╔██╗ ██║   ██║   ██║██║     ███████╗           ║
║    ██╔══██║██║   ██║██╔══╝  ██║╚██╗██║   ██║   ██║██║     ╚════██║           ║
║    ██║  ██║╚██████╔╝███████╗██║ ╚████║   ██║   ██║╚██████╗███████║           ║
║    ╚═╝  ╚═╝ ╚═════╝ ╚══════╝╚═╝  ╚═══╝   ╚═╝   ╚═╝ ╚═════╝╚══════╝           ║
║                                                                               ║
║                    🚀 TV5 HACKATHON - Supported by Google 🚀                  ║
║                                                                               ║
║         Building the Future of Agentic AI | Open Source | Global             ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
`;

export const WELCOME_MESSAGE = `
Welcome to the Agentics Foundation TV5 Hackathon!

Every night, millions spend up to 45 minutes deciding what to watch — billions
of hours lost every day. Not from lack of content, but from fragmentation.

Join us to build the future of agentic AI systems that solve real problems.

🔗 Discord: ${DISCORD_URL}
🌐 Website: ${WEBSITE_URL}
📦 GitHub:  ${GITHUB_URL}
`;
