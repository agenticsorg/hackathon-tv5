# Agentics Foundation TV5 Hackathon

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![npm version](https://img.shields.io/badge/npm-agentics--hackathon-red.svg)](https://www.npmjs.com/package/agentics-hackathon)
[![Discord](https://img.shields.io/badge/Discord-Agentics-7289da.svg)](https://discord.agentics.org)

> **Build the future of agentic AI - Supported by Google Cloud**

The **Agentics Foundation TV5 Hackathon** repository provides CLI tools, MCP servers, and reference implementations for building agentic AI solutions. This includes the **AI Media Discovery** demo app showcasing the Agent-Ready Web (ARW) specification.

🌐 **Website:** [agentics.org/hackathon](https://agentics.org/hackathon)
💬 **Discord:** [discord.agentics.org](https://discord.agentics.org)
📦 **npm:** `npx agentics-hackathon`

---

## 🎯 The Challenge

Every night, millions spend up to **45 minutes deciding what to watch** — billions of hours lost every day. Not from lack of content, but from fragmentation across streaming platforms.

Join us to build agentic AI solutions that solve real problems using Google Cloud, Gemini, Claude, and open-source tools.

---

## 🚀 Quick Start

```bash
# Initialize your hackathon project
npx agentics-hackathon init

# Browse and install 17+ AI tools
npx agentics-hackathon tools

# Check project status
npx agentics-hackathon status

# Start MCP server for AI assistant integration
npx agentics-hackathon mcp
```

---

## 🏆 Hackathon Tracks

| Track | Description |
|-------|-------------|
| **Entertainment Discovery** | Solve the 45-minute decision problem - help users find what to watch |
| **Multi-Agent Systems** | Build collaborative AI agents with Google ADK and Vertex AI |
| **Agentic Workflows** | Create autonomous workflows with Claude, Gemini, and orchestration |
| **Open Innovation** | Bring your own idea - any agentic AI solution that makes an impact |

---

## ✨ Features

### 🛠 CLI Tool (`npx agentics-hackathon`)

- **`init`** - Interactive project setup with track selection and tool installation
- **`tools`** - Browse and install 17+ AI development tools across 6 categories
- **`status`** - View project configuration and installed tools
- **`info`** - Hackathon information and resources
- **`mcp`** - Start MCP server (stdio or SSE transport)
- **`discord`** - Join the community
- **`help`** - Detailed guides and examples

### 🤖 MCP Server

Full Model Context Protocol implementation with:
- **Tools**: `get_hackathon_info`, `get_tracks`, `get_available_tools`, `get_project_status`, `check_tool_installed`, `get_resources`
- **Resources**: Project configuration, track information
- **Prompts**: `hackathon_starter`, `choose_track`

### 📱 Demo Applications

| App | Description |
|-----|-------------|
| **[Media Discovery](apps/media-discovery/)** | AI-powered movie/TV discovery with ARW implementation |
| **[ARW Chrome Extension](apps/arw-chrome-extension/)** | Browser extension for inspecting ARW compliance |

### 📐 ARW (Agent-Ready Web) Components

This repository includes reference implementations of the ARW specification:

- **Specification**: [ARW v0.1 Draft](spec/ARW-0.1-draft.md)
- **Schemas**: JSON schemas for validation (`packages/schemas/`)
- **Validators**: Python and Node.js validation tools (`packages/validators/`)
- **Badges**: Compliance level badges (`packages/badges/`)

---

## 📦 Repository Structure

```plaintext
hackathon-tv5/
├── src/                             # Hackathon CLI source
│   ├── cli.ts                      # Main CLI entry point
│   ├── commands/                   # CLI commands (init, tools, status, etc.)
│   ├── mcp/                        # MCP server implementation
│   │   ├── server.ts              # MCP tools, resources, prompts
│   │   ├── stdio.ts               # STDIO transport
│   │   └── sse.ts                 # SSE transport
│   ├── constants.ts               # Tracks, tools, configuration
│   └── utils/                     # Helpers and utilities
│
├── apps/                           # Demo Applications
│   ├── media-discovery/           # AI Media Discovery (Next.js + ARW)
│   │   ├── public/
│   │   │   ├── .well-known/arw-manifest.json  # ARW manifest
│   │   │   └── llms.txt                       # ARW discovery file
│   │   └── src/                   # React components & API routes
│   └── arw-chrome-extension/      # ARW Inspector Chrome Extension
│       ├── manifest.json          # Chrome Manifest V3
│       └── src/                   # Popup, content script, service worker
│
├── packages/                       # Shared Packages
│   ├── @arw/schemas/              # TypeScript ARW schemas with Zod
│   ├── schemas/                   # JSON schemas for ARW validation
│   ├── validators/                # Python & Node.js validators
│   ├── validator/                 # ARW validator CLI tool
│   ├── badges/                    # ARW compliance badges (SVG)
│   ├── cli/                       # Rust ARW CLI (advanced)
│   ├── crawler-sdk/               # TypeScript SDK for ARW crawler service
│   ├── crawler-service/           # High-performance crawler API service
│   ├── nextjs-plugin/             # Next.js plugin for ARW integration
│   └── benchmark/                 # ARW benchmark evaluation
│
├── spec/                           # ARW Specification
│   └── ARW-0.1-draft.md           # Editor's draft specification
│
├── docs/                           # Documentation
├── ai_docs/                        # AI-focused documentation
├── scripts/                        # Build and utility scripts
│
├── .claude/                        # Claude Code configuration
│   ├── commands/                  # Slash commands
│   └── agents/                    # Sub-agent definitions
│
├── CLAUDE.md                       # Claude Code guidance
└── README.md                       # This file
```

---

## 🔧 Available Tools (17+)

The CLI provides access to tools across 6 categories:

### AI Assistants
- **Claude Code CLI** - Anthropic's AI-powered coding assistant
- **Gemini CLI** - Google's Gemini model interface

### Orchestration & Agent Frameworks
- **Claude Flow** - #1 agent orchestration platform with 101 MCP tools
- **Agentic Flow** - Production AI orchestration with 66 agents
- **Flow Nexus** - Competitive agentic platform on MCP
- **Google ADK** - Build multi-agent systems with Google's Agent Development Kit

### Cloud Platform
- **Google Cloud CLI** - gcloud SDK for Vertex AI, Cloud Functions
- **Vertex AI SDK** - Google Cloud's unified ML platform

### Databases & Memory
- **RuVector** - Vector database and embeddings toolkit
- **AgentDB** - Database for agentic AI state management

### Synthesis & Advanced Tools
- **Agentic Synth** - Synthesis tools for agentic development
- **Strange Loops** - Consciousness exploration SDK
- **SPARC 2.0** - Autonomous vector coding agent

### Python Frameworks
- **LionPride** - Python agentic AI framework
- **Agentic Framework** - AI agents with natural language
- **OpenAI Agents SDK** - Multi-agent workflows from OpenAI

---

## 🌐 ARW (Agent-Ready Web)

This repository demonstrates the ARW specification through the **Media Discovery** app.

### What is ARW?

ARW provides infrastructure for efficient agent-web interaction:

- **85% token reduction** - Machine views vs HTML scraping
- **10x faster discovery** - Structured manifests vs crawling
- **OAuth-enforced actions** - Safe agent transactions
- **AI-* headers** - Full observability of agent traffic

### ARW in Media Discovery

The media-discovery app implements ARW with:

```json
// /.well-known/arw-manifest.json
{
  "version": "0.1",
  "profile": "ARW-1",
  "site": {
    "name": "AI Media Discovery",
    "description": "Discover movies and TV shows through natural language"
  },
  "actions": [
    {
      "id": "semantic_search",
      "endpoint": "/api/search",
      "method": "POST"
    }
  ]
}
```

See the [ARW Specification](spec/ARW-0.1-draft.md) for full details.

---

## 💻 Development

### Prerequisites

- Node.js 18+
- npm or pnpm

### Build & Run

```bash
# Install dependencies
npm install

# Build the CLI
npm run build

# Run locally
npm start

# Development mode (watch)
npm run dev

# Run linter
npm run lint
```

### MCP Server

```bash
# STDIO transport (for Claude Desktop, etc.)
npm run mcp:stdio

# SSE transport (for web integrations)
npm run mcp:sse
```

### Media Discovery App

```bash
cd apps/media-discovery
npm install
npm run dev
```

---

## 🔌 MCP Integration

Add to your Claude Desktop config (`claude_desktop_config.json`):

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

Or use SSE transport:

```bash
npx agentics-hackathon mcp sse --port 3000
```

---

## 🤝 Contributing

We welcome contributions! Areas of focus:

1. **CLI Improvements** - New commands, better UX
2. **Tool Integrations** - Add more AI tools
3. **Demo Apps** - Build showcases for hackathon tracks
4. **ARW Implementation** - Expand specification coverage
5. **Documentation** - Guides and tutorials

### Development Workflow

See [CLAUDE.md](CLAUDE.md) for development guidelines including:
- SPARC methodology for systematic development
- Concurrent execution patterns
- File organization rules

---

## 📜 License

This project is licensed under the [Apache License 2.0](LICENSE).

---

## 🔗 Links

- **🌐 Hackathon Website:** [agentics.org/hackathon](https://agentics.org/hackathon)
- **💬 Discord:** [discord.agentics.org](https://discord.agentics.org)
- **📦 GitHub:** [github.com/agenticsorg/hackathon-tv5](https://github.com/agenticsorg/hackathon-tv5)
- **📖 ARW Spec:** [ARW v0.1 Draft](spec/ARW-0.1-draft.md)

---

<div align="center">

**🚀 Agentics Foundation TV5 Hackathon**

*Building the Future of Agentic AI - Supported by Google Cloud*

[Website](https://agentics.org/hackathon) | [Discord](https://discord.agentics.org) | [GitHub](https://github.com/agenticsorg/hackathon-tv5)

</div>
