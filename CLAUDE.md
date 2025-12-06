# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

The **Agentics Foundation TV5 Hackathon** repository provides:
- CLI tools and MCP server for hackathon participants
- Reference implementations of the Agent-Ready Web (ARW) specification
- Demo applications showcasing agentic AI solutions

## Repository Structure

```
hackathon-tv5/
├── apps/
│   ├── cli/                    # Main hackathon CLI (published as agentics-hackathon)
│   ├── media-discovery/        # Next.js app demonstrating ARW implementation
│   ├── agentdb/               # Vector database for AI agents (RuVector-powered)
│   ├── agentic-flow/          # AI agent orchestration platform
│   ├── agentic-synth/         # Synthetic data generator for AI/ML
│   └── arw-chrome-extension/  # Browser extension for ARW compliance inspection
└── .claude/                    # Claude Code agents and commands
```

## Build & Development Commands

### CLI (apps/cli)
```bash
cd apps/cli
npm install
npm run build          # Compile TypeScript
npm run dev            # Watch mode
npm start              # Run CLI
npm run lint           # ESLint
npm run mcp:stdio      # Start MCP server (STDIO transport)
npm run mcp:sse        # Start MCP server (SSE transport)
```

### Media Discovery App (apps/media-discovery)
```bash
cd apps/media-discovery
npm install
npm run dev            # Next.js dev server
npm run build          # Production build
npm run lint           # ESLint
npm run typecheck      # TypeScript check
npm run test           # Vitest
npm run test:coverage  # Coverage report
npm run deploy         # Deploy to Cloud Run
```

### AgentDB (apps/agentdb)
```bash
cd apps/agentdb
npm install
npm run build          # Build (TS + schemas + browser bundle)
npm run test           # Vitest
npm run test:unit      # Unit tests only
npm run dev            # Run CLI in dev mode
npm run benchmark      # Run benchmarks
```

### Agentic Flow (apps/agentic-flow)
```bash
cd apps/agentic-flow
npm install
npm run build          # Build (includes WASM)
npm run dev            # Run in dev mode
npm run test           # Run tests
npm run validate       # Run validation suite
npm run mcp:stdio      # Start MCP server
```

### Agentic Synth (apps/agentic-synth)
```bash
cd apps/agentic-synth
npm install
npm run build:all      # Build all modules
npm run test           # Vitest
npm run typecheck      # TypeScript check
npm run lint           # ESLint
```

## Architecture

### CLI (agentics-hackathon)
- Entry: `apps/cli/src/cli.ts`
- Commands in `apps/cli/src/commands/` (init, tools, status, info, discord, help)
- MCP server in `apps/cli/src/mcp/` with STDIO and SSE transports
- Constants/config in `apps/cli/src/constants.ts`

### Media Discovery (ARW Reference Implementation)
- Next.js 15 App Router in `apps/media-discovery/src/app/`
- Components in `apps/media-discovery/src/components/`
- ARW manifest at `apps/media-discovery/public/.well-known/arw-manifest.json`
- Uses Vercel AI SDK with Google/OpenAI providers
- RuVector for vector search
- TMDB API for movie data

### AgentDB
- Core database in `apps/agentdb/src/core/AgentDB.ts`
- Controllers for various memory patterns: CausalMemoryGraph, ReflexionMemory, SkillLibrary
- Multiple backends: HNSWLib, RuVector, Graph
- MCP server integration in `apps/agentdb/src/mcp/`
- CLI in `apps/agentdb/src/cli/`

### Agentic Flow
- ReasoningBank learning memory system with WASM support
- 66 specialized agents, 213 MCP tools
- Consensus protocols: Byzantine, Raft, Gossip
- Transport: QUIC, WebSocket

## Key Technologies

- **TypeScript** with ES modules across all apps
- **Node.js 18+** (20+ for media-discovery)
- **Next.js 15** for media-discovery app
- **Vitest** for testing
- **MCP (Model Context Protocol)** for AI assistant integration
- **RuVector** for vector embeddings
- **sql.js/SQLite** for AgentDB persistence

## File Organization Rules

- Never save files to the root folder
- Source code in `/src`
- Tests in `/tests` or alongside source files
- Configuration in `/config`
- Documentation in `/docs`

## Running the Project Locally

```bash
# Quick start with CLI
npx agentics-hackathon init
npx agentics-hackathon tools
npx agentics-hackathon status

# Or for development
cd apps/cli && npm install && npm run build && npm start
```

## MCP Integration

Add to Claude Desktop config:
```json
{
  "mcpServers": {
    "agentics-hackathon": {
      "command": "npx",
      "args": ["agentics-hackathon", "mcp"]
    }
  }
}
```
