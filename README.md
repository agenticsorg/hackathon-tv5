# Samsung Smart TV Integration

> AI-powered Samsung Smart TV control with on-device learning and content discovery

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-71%20passing-brightgreen.svg)]()
[![MCP Tools](https://img.shields.io/badge/MCP%20tools-38-blue.svg)]()

Built for the **Agentics Foundation TV5 Hackathon** - solving the "45 minutes deciding what to watch" problem with AI agents.

## Features

- **TV Control** - Discover, connect, and control Samsung Smart TVs via WebSocket
- **Self-Learning** - On-device Q-Learning that improves recommendations over time
- **Content Discovery** - TMDb integration for movies, TV shows, and streaming availability
- **MCP Server** - 38 tools for AI assistant integration (Claude, Gemini, etc.)

## Quick Start

```bash
# Install
cd apps/samsung-tv-integration
npm install

# Build
npm run build

# Run tests
npm test

# Start MCP server
npm start
```

## Architecture

```
apps/samsung-tv-integration/
├── src/
│   ├── lib/           # TV control (WebSocket, SSDP discovery)
│   ├── learning/      # Q-Learning, embeddings, preferences
│   ├── content/       # TMDb API, content discovery
│   ├── mcp/           # MCP server and tools
│   └── utils/         # Configuration, helpers
├── tests/             # 71 tests
└── dist/              # Compiled output
```

## MCP Tools (38 Total)

### TV Control (13 tools)
| Tool | Description |
|------|-------------|
| `samsung_tv_discover` | Find TVs on network via SSDP |
| `samsung_tv_connect` | Connect and authenticate |
| `samsung_tv_power` | Power on/off/toggle |
| `samsung_tv_volume` | Volume up/down/mute |
| `samsung_tv_navigate` | Arrow keys & enter |
| `samsung_tv_key` | Send any remote key |
| `samsung_tv_apps` | List installed apps |
| `samsung_tv_launch_app` | Launch Netflix, YouTube, etc. |
| `samsung_tv_home` | Go to home screen |
| `samsung_tv_status` | Get TV state |
| `samsung_tv_list` | List saved TVs |
| `samsung_tv_set_default` | Set default TV |
| `samsung_tv_remove` | Remove saved TV |

### Learning System (13 tools)
| Tool | Description |
|------|-------------|
| `samsung_tv_learn_get_recommendations` | Get personalized recommendations |
| `samsung_tv_learn_add_content` | Add content to library |
| `samsung_tv_learn_record_session` | Record viewing session |
| `samsung_tv_learn_feedback` | Submit feedback |
| `samsung_tv_learn_get_stats` | Get learning statistics |
| `samsung_tv_learn_get_preferences` | Get user preferences |
| `samsung_tv_learn_train` | Trigger experience replay |
| `samsung_tv_learn_save` | Save learned model |
| `samsung_tv_learn_load` | Load saved model |
| `samsung_tv_learn_clear` | Clear learning data |
| `samsung_tv_learn_storage_stats` | Get storage statistics |
| `samsung_tv_smart_launch` | Launch with learning |
| `samsung_tv_smart_end_session` | End session with learning |

### Content Discovery (12 tools)
| Tool | Description |
|------|-------------|
| `content_search` | Search movies/TV shows |
| `content_trending` | Get trending content |
| `content_popular` | Get popular content |
| `content_top_rated` | Get top-rated content |
| `content_discover` | Filter by genre/rating/year |
| `content_details` | Get full details with cast |
| `content_similar` | Find similar content |
| `content_recommendations` | TMDb recommendations |
| `content_now_playing` | Movies in theaters |
| `content_upcoming` | Upcoming releases |
| `content_personalized` | Learning-based recommendations |
| `content_for_mood` | Mood-based suggestions |

## Documentation

- [User Guide](docs/user-guide/README.md) - Getting started and usage
- [Developer Guide](docs/developer/README.md) - Architecture and API reference

## Tech Stack

- **Runtime**: Node.js 18+, TypeScript
- **TV Protocol**: Samsung WebSocket API (port 8002)
- **Discovery**: SSDP/UPnP
- **Learning**: Q-Learning with experience replay
- **Embeddings**: WASM-optimized cosine similarity
- **Content API**: TMDb v3
- **MCP**: Model Context Protocol (STDIO/SSE)
- **Testing**: Vitest

## Environment Variables

```bash
TMDB_API_KEY=your_tmdb_api_key  # Required for content discovery
```

## License

Apache-2.0 - See [LICENSE](LICENSE)

## Hackathon

Part of [Agentics Foundation TV5 Hackathon](https://agentics.org/hackathon)

- Team: **agentics**
- Track: **Entertainment Discovery**
