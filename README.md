# Samsung Smart TV AI Assistant

> Solving the "45 minutes deciding what to watch" problem with AI-powered TV control and on-device learning

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-71%20passing-brightgreen.svg)]()
[![MCP Tools](https://img.shields.io/badge/MCP%20tools-38-blue.svg)]()
[![TypeScript](https://img.shields.io/badge/TypeScript-5.6-blue.svg)]()

---

## Agentics Foundation TV5 Hackathon Entry

| | |
|---|---|
| **Team** | agentics |
| **Track** | Entertainment Discovery |
| **Challenge** | Help users find what to watch faster |

### The Problem

Every night, millions of people spend **up to 45 minutes deciding what to watch** — that's billions of hours lost globally, not from lack of content, but from fragmentation across streaming platforms.

### Our Solution

An AI-powered Samsung Smart TV integration that:

1. **Controls your TV** via natural language through AI assistants (Claude, Gemini)
2. **Learns your preferences** using on-device Q-Learning that improves over time
3. **Discovers content** across streaming platforms with mood-based recommendations
4. **Reduces decision time** from 45 minutes to seconds

---

## Demo

### Talk to Your TV

```
You: "I want to watch something exciting tonight"

AI: Found 5 action thrillers based on your preferences:
    1. The Dark Knight (Netflix) - 9.0★
    2. Inception (Prime Video) - 8.8★
    3. The Matrix (HBO Max) - 8.7★

    Want me to launch any of these?

You: "Play The Dark Knight"

AI: Launching Netflix on your Living Room TV...
```

### How It Works

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   AI Assistant  │────▶│   MCP Server    │────▶│   Samsung TV    │
│ (Claude/Gemini) │     │   (38 tools)    │     │   (WebSocket)   │
└─────────────────┘     └────────┬────────┘     └─────────────────┘
                                 │
                    ┌────────────┴────────────┐
                    │                         │
              ┌─────▼─────┐           ┌───────▼───────┐
              │ Q-Learning │           │  TMDb API     │
              │  (on-device)│           │ (discovery)   │
              └───────────┘           └───────────────┘
```

---

## Features

### TV Control
- Auto-discover Samsung TVs on your network
- Power, volume, navigation, app launching
- Works with Netflix, YouTube, Disney+, HBO Max, Prime Video, and more

### Self-Learning Recommendations
- On-device Q-Learning that respects privacy
- Learns from viewing patterns (completion rate, ratings, time of day)
- Improves recommendations with each session
- No cloud dependency for preferences

### Content Discovery
- Search movies and TV shows across platforms
- Mood-based suggestions (relaxing, exciting, scary, funny)
- Trending, popular, and personalized feeds
- Streaming availability detection

---

## Benchmarks

Performance tested on standard hardware:

| Operation | Speed | Notes |
|-----------|-------|-------|
| Embedding Generation | **135,448 ops/sec** | 64-dim content vectors |
| Cosine Similarity | **1,285,875 ops/sec** | WASM-optimized |
| Batch Search (Top-10) | **81,478 ops/sec** | Real-time recommendations |
| Cache Hit Rate | **99.6%** | Efficient memory usage |
| Q-Learning Training | **0.18s / 500 episodes** | Fast convergence |

Run benchmarks yourself:
```bash
cd apps/samsung-tv-integration
npm run benchmark
```

---

## Quick Start

### Prerequisites
- Node.js 18+
- Samsung Smart TV (2016+) on same network
- TMDb API key (free at [themoviedb.org](https://themoviedb.org))

### Installation

```bash
# Clone and install
cd apps/samsung-tv-integration
npm install

# Build
npm run build

# Set TMDb key
export TMDB_API_KEY=your_key_here

# Run tests
npm test

# Start MCP server
npm start
```

### Connect to Claude Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "samsung-tv": {
      "command": "node",
      "args": ["/path/to/apps/samsung-tv-integration/dist/cli.js", "mcp"],
      "env": {
        "TMDB_API_KEY": "your_key_here"
      }
    }
  }
}
```

---

## MCP Tools (38 Total)

### TV Control (13 tools)
| Tool | Description |
|------|-------------|
| `samsung_tv_discover` | Find TVs on network |
| `samsung_tv_connect` | Connect and pair |
| `samsung_tv_power` | Power on/off/toggle |
| `samsung_tv_volume` | Volume control |
| `samsung_tv_navigate` | D-pad navigation |
| `samsung_tv_key` | Send remote keys |
| `samsung_tv_apps` | List apps |
| `samsung_tv_launch_app` | Launch streaming apps |
| `samsung_tv_home` | Go to home |
| `samsung_tv_status` | Get TV state |
| `samsung_tv_list` | List saved TVs |
| `samsung_tv_set_default` | Set default TV |
| `samsung_tv_remove` | Remove TV |

### Learning System (13 tools)
| Tool | Description |
|------|-------------|
| `samsung_tv_learn_get_recommendations` | Personalized picks |
| `samsung_tv_learn_add_content` | Add to library |
| `samsung_tv_learn_record_session` | Record viewing |
| `samsung_tv_learn_feedback` | Submit ratings |
| `samsung_tv_learn_get_stats` | Learning stats |
| `samsung_tv_learn_get_preferences` | User preferences |
| `samsung_tv_learn_train` | Experience replay |
| `samsung_tv_learn_save` | Save model |
| `samsung_tv_learn_load` | Load model |
| `samsung_tv_learn_clear` | Reset learning |
| `samsung_tv_learn_storage_stats` | Storage info |
| `samsung_tv_smart_launch` | Launch with tracking |
| `samsung_tv_smart_end_session` | End with learning |

### Content Discovery (12 tools)
| Tool | Description |
|------|-------------|
| `content_search` | Search movies/shows |
| `content_trending` | Trending now |
| `content_popular` | Popular content |
| `content_top_rated` | Highest rated |
| `content_discover` | Filter by criteria |
| `content_details` | Full details + cast |
| `content_similar` | Similar content |
| `content_recommendations` | TMDb recommendations |
| `content_now_playing` | In theaters |
| `content_upcoming` | Coming soon |
| `content_personalized` | AI-powered picks |
| `content_for_mood` | Mood-based |

---

## Architecture

```
apps/samsung-tv-integration/
├── src/
│   ├── lib/           # TV control (WebSocket, SSDP)
│   ├── learning/      # Q-Learning, embeddings
│   ├── content/       # TMDb integration
│   ├── mcp/           # MCP server (38 tools)
│   └── utils/         # Config, helpers
├── scripts/
│   └── train-benchmark.ts  # Training & benchmarks
├── tests/             # 71 tests
└── docs/
    ├── user-guide/    # Usage documentation
    └── developer/     # API reference
```

---

## Tech Stack

| Component | Technology |
|-----------|------------|
| Runtime | Node.js 18+, TypeScript 5.6 |
| TV Protocol | Samsung WebSocket API (port 8002) |
| Discovery | SSDP/UPnP |
| Learning | Q-Learning with experience replay |
| Embeddings | WASM-optimized 64-dim vectors |
| Content API | TMDb v3 |
| MCP | Model Context Protocol (STDIO/SSE) |
| Testing | Vitest (71 tests) |

---

## Documentation

- [User Guide](docs/user-guide/README.md) - Getting started
- [Developer Guide](docs/developer/README.md) - Architecture & APIs

---

## What's Next

- [ ] Multi-room TV coordination
- [ ] Voice control integration
- [ ] Watch party synchronization
- [ ] Smart home integration (lights, thermostat)
- [ ] Cross-platform mobile app

---

## Team

Built by **Team Agentics** for the [Agentics Foundation TV5 Hackathon](https://agentics.org/hackathon)

**Challenge**: Entertainment Discovery — Help users find what to watch

---

## License

Apache-2.0 — See [LICENSE](LICENSE)
