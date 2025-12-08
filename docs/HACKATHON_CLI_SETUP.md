# Hackathon CLI Setup Complete ✅

## Installation Summary

The Agentics Foundation TV5 Hackathon CLI has been successfully installed and tested.

### Build Information
- **Package**: agentics-hackathon@1.3.4
- **Location**: `/home/user/agentic-pancakes/apps/cli`
- **Dependencies**: 293 packages installed
- **Build Status**: ✅ Successful (TypeScript compiled)
- **Vulnerabilities**: 0 found

### Available Commands

```bash
# Show help
node dist/cli.js --help

# Initialize a new project
node dist/cli.js init

# List all available tools
node dist/cli.js tools --list

# Check installed tools
node dist/cli.js tools --check

# Install specific tools
node dist/cli.js tools --install <tool-name>

# Show hackathon info
node dist/cli.js info

# Start MCP server (STDIO)
node dist/cli.js mcp stdio

# Start MCP server (SSE on port 3000)
node dist/cli.js mcp sse --port 3000

# Open Discord
node dist/cli.js discord

# Show detailed help
node dist/cli.js help
```

### Already Installed Tools (4/16)

✅ **Claude Code CLI** - AI-powered coding assistant
✅ **Flow Nexus** - Competitive agentic platform on MCP
✅ **RuVector** - Vector database and embeddings toolkit
✅ **Strange Loops** - Consciousness exploration SDK

### Available for Installation (12/16)

#### AI Assistants
- Google Gemini CLI

#### Orchestration & Agent Frameworks
- Claude Flow (#1 agent orchestration, 101 MCP tools)
- Agentic Flow (66 agents, 213 MCP tools)
- Google Agent Development Kit (ADK)

#### Cloud Platform
- Google Cloud CLI (gcloud)
- Vertex AI SDK

#### Databases & Memory
- AgentDB

#### Synthesis & Advanced
- Agentic Synth
- SPARC 2.0

#### Python Frameworks
- LionPride
- Agentic Framework
- OpenAI Agents SDK

## Hackathon Tracks

### 1. Entertainment Discovery
Solve the 45-minute decision problem - help users find what to watch

### 2. Multi-Agent Systems
Build collaborative AI agents with Google ADK and Vertex AI

### 3. Agentic Workflows
Create autonomous workflows with Claude, Gemini, and orchestration

### 4. Open Innovation
Build any agentic AI solution that makes an impact

## MCP Server

The CLI includes a full Model Context Protocol (MCP) server with two transports:

### STDIO Transport (for Claude Desktop)
```bash
node dist/cli.js mcp stdio
```

Add to Claude Desktop config:
```json
{
  "mcpServers": {
    "agentics-hackathon": {
      "command": "node",
      "args": ["/home/user/agentic-pancakes/apps/cli/dist/mcp/stdio.js"]
    }
  }
}
```

### SSE Transport (for web integrations)
```bash
node dist/cli.js mcp sse --port 3000
```

## Quick Start Workflows

### Option 1: Interactive Setup
```bash
node dist/cli.js init
# Follow prompts to select track and install tools
```

### Option 2: Manual Tool Installation
```bash
# Install Claude Flow for orchestration
node dist/cli.js tools --install claudeFlow

# Install Google tools
node dist/cli.js tools --install geminiCli adk

# Install database tools
node dist/cli.js tools --install agentdb
```

### Option 3: Explore Media Discovery Demo
```bash
cd /home/user/agentic-pancakes/apps/media-discovery
npm install
npm run dev
# Visit http://localhost:3000
```

## Resources

- **Website**: https://agentics.org/hackathon
- **Discord**: https://discord.agentics.org
- **GitHub**: https://github.com/agenticsorg/hackathon-tv5
- **Google ADK**: https://google.github.io/adk-docs/
- **Vertex AI**: https://cloud.google.com/vertex-ai/docs
- **Claude**: https://docs.anthropic.com

## Next Steps

1. **Choose your track** - Run `node dist/cli.js info` to review options
2. **Install tools** - Use `node dist/cli.js tools` to add what you need
3. **Join Discord** - Connect with the community for support
4. **Start building** - Begin your agentic AI project!

---

**Status**: CLI is fully functional and ready for hackathon development
**Last Updated**: 2025-12-05
