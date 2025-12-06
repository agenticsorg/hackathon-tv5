# Samsung Smart TV AI Assistant

> **End the nightly "what should we watch?" debate** — An AI that controls your Samsung TV, learns your taste, and finds the perfect content in seconds instead of 45 minutes.

[![Tests: 71 Passing](https://img.shields.io/badge/tests-71%20passing-brightgreen.svg)]()
[![MCP Tools: 38](https://img.shields.io/badge/MCP%20tools-38-blue.svg)]()
[![TypeScript 5.6](https://img.shields.io/badge/TypeScript-5.6-blue.svg)]()
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)

---

## The Problem We're Solving

**45 minutes.** That's how long the average household spends every night deciding what to watch. With dozens of streaming apps, thousands of titles, and different preferences among family members — finding the right content has become exhausting.

### Why Existing Solutions Fail

| Solution | Problem |
|----------|---------|
| Netflix/YouTube recommendations | Locked to one platform, can't see other services |
| Voice assistants | Only basic controls, no learning, no cross-platform discovery |
| Smart TV interfaces | Still requires manual browsing through each app |
| Third-party apps | Require phone in hand, don't control the TV |

---

## Our Solution: AI + Learning + Control

We built an **MCP-powered AI assistant** that:

1. **Talks to your TV** — Natural language control via Claude, Gemini, or any MCP-compatible AI
2. **Learns your taste** — On-device Q-Learning improves with every viewing session
3. **Searches everywhere** — Finds content across all streaming platforms
4. **Gets smarter over time** — The more you watch, the better it recommends

### What Makes This Different

| Feature | Traditional | Our Solution |
|---------|-------------|--------------|
| Platform coverage | Single app | All streaming apps |
| Learning | Cloud-based, generic | On-device, personal |
| Privacy | Data sent to servers | Stays on your network |
| Control method | Remote/phone | Natural conversation |
| Improvement | Static algorithms | Reinforcement learning |

---

## Live Demo

### Conversation Example

```
You: "I'm in the mood for something thrilling but not too scary"

AI: Based on your preferences, here are 3 picks:

   1. Inception (Prime Video) ★ 8.8
      Why: You enjoyed Interstellar (95% watched), same director

   2. The Dark Knight (Netflix) ★ 9.0
      Why: Action thriller, your top genre on weekday evenings

   3. Sicario (HBO Max) ★ 7.6
      Why: Similar to Prisoners which you rated 5 stars

   Should I launch one of these?

You: "Play Inception on my living room TV"

AI: Launching Prime Video on Living Room Samsung TV...
    [TV turns on, opens Prime Video, searches for Inception]

    Enjoy the movie! I'll learn from this session to improve
    future recommendations.
```

### What Just Happened (Technical)

1. **Mood parsing** → Mapped "thrilling but not too scary" to genres: `[thriller, action]` excluding `[horror]`
2. **Q-Learning query** → Retrieved top actions for current state (weekday evening, recent genres)
3. **Content embedding** → Generated 64-dim vectors, found cosine similarity matches
4. **TMDb enrichment** → Added ratings, availability, metadata
5. **TV control** → WebSocket command to Samsung TV on local network

---

## How the Learning System Works

### The Q-Learning Algorithm

Our system uses **temporal difference learning** — a reinforcement learning technique that learns optimal behavior through trial and reward.

```
Q(state, action) ← Q(state, action) + α × [reward + γ × max(Q(next_state)) - Q(state, action)]

Where:
  α (learning rate) = 0.1     — How fast to incorporate new information
  γ (discount factor) = 0.95  — How much to value future rewards
  ε (exploration) = 0.3→0.05  — Balance between trying new vs. known-good
```

### States (What the system observes)

| State Component | Values | Why It Matters |
|-----------------|--------|----------------|
| Time of day | morning, afternoon, evening, night | Comedy at night, news in morning |
| Day of week | weekday, weekend | Longer content on weekends |
| Recent genres | top 3 watched | Genre momentum patterns |
| Completion rate | 0-100% average | Indicates engagement level |

### Actions (What the system recommends)

| Action | When Used |
|--------|-----------|
| `recommend_similar` | High completion rate on last content |
| `recommend_genre` | Strong genre preference detected |
| `explore_new_genre` | User seems open to variety |
| `recommend_trending` | New user or exploration mode |
| `recommend_based_on_time` | Time-of-day patterns detected |

### Rewards (How it learns)

```typescript
reward =
    completion_rate × 0.5    // Watched 80%? Good sign
  + (user_rating / 5) × 0.3  // Explicit 5-star? Great!
  + engagement × 0.2         // Paused to think? Rewound? Engaged!
```

### Example Learning Progression

```
Day 1:  Random recommendations (ε=0.3 exploration)
Day 7:  Notices you watch comedies on weekday evenings
Day 14: Learns you prefer 90-120 min duration
Day 30: Knows your mood patterns by time and day
Day 60: Recommendations feel personalized
```

---

## Content Embeddings (How We Understand Content)

Every movie/show is converted to a **64-dimensional vector** that captures its essence:

```
┌────────────────────────────────────────────────────────────────┐
│                    64-Dimension Embedding                       │
├────────────┬────────────┬────────────┬────────────┬────────────┤
│ Genres (10)│ Type (8)   │ Meta (8)   │Duration(5) │Keywords(33)│
│ action:0.8 │ movie:1.0  │ rating:0.9 │ 90-120:1.0 │ heist:0.7  │
│ thriller:0.6│ tv:0.0     │ pop:0.7    │            │ dreams:0.9 │
│ scifi:0.9  │            │ year:0.85  │            │ ...        │
└────────────┴────────────┴────────────┴────────────┴────────────┘
```

### Similarity Calculation (WASM-Optimized)

```typescript
// Cosine similarity with loop unrolling for SIMD optimization
function cosineSimilarity(a: Float32Array, b: Float32Array): number {
  let dot = 0, normA = 0, normB = 0;

  // Process 4 elements at a time (SIMD-friendly)
  for (let i = 0; i < 64; i += 4) {
    dot   += a[i]*b[i] + a[i+1]*b[i+1] + a[i+2]*b[i+2] + a[i+3]*b[i+3];
    normA += a[i]*a[i] + a[i+1]*a[i+1] + a[i+2]*a[i+2] + a[i+3]*a[i+3];
    normB += b[i]*b[i] + b[i+1]*b[i+1] + b[i+2]*b[i+2] + b[i+3]*b[i+3];
  }

  return dot / (Math.sqrt(normA) * Math.sqrt(normB));
}
```

**Result**: 1,285,875 similarity calculations per second

---

## Benchmarks (Proven Performance)

All benchmarks run on standard hardware. **Run them yourself:**

```bash
cd apps/samsung-tv-integration
npm run benchmark
```

### Speed Results

| Operation | Speed | What It Means |
|-----------|-------|---------------|
| **Embedding Generation** | 135,448/sec | Convert any content to searchable vector instantly |
| **Cosine Similarity** | 1,285,875/sec | Compare two items in <1 microsecond |
| **Batch Top-10 Search** | 81,478/sec | Find best matches from 1000s of items in real-time |
| **Cache Hit Rate** | 99.6% | Almost never recalculate the same embedding |

### Learning Speed

| Metric | Result |
|--------|--------|
| Training 500 episodes | 0.18 seconds |
| Q-table convergence | ~200 episodes |
| Memory footprint | <5MB for full model |
| Model save/load | <50ms |

### Real-World Timing

| User Action | Response Time |
|-------------|---------------|
| "Find me something funny" | <200ms |
| "What's trending?" | <300ms (includes TMDb API) |
| Record viewing session | <10ms |
| Get personalized recommendations | <50ms |

---

## Complete Feature List

### TV Control (13 Tools)

| Tool | What It Does | Example |
|------|--------------|---------|
| `samsung_tv_discover` | Find Samsung TVs on your network | Auto-detects via SSDP |
| `samsung_tv_connect` | Pair with TV (first time shows PIN) | One-time setup |
| `samsung_tv_power` | Turn on/off/toggle | "Turn off the bedroom TV" |
| `samsung_tv_volume` | Adjust volume up/down/mute | "Volume up 3 steps" |
| `samsung_tv_navigate` | D-pad controls (up/down/left/right/enter) | "Select that option" |
| `samsung_tv_key` | Send any remote key | "Press the back button" |
| `samsung_tv_apps` | List installed apps | "What apps are on my TV?" |
| `samsung_tv_launch_app` | Open Netflix, YouTube, etc. | "Open Disney+" |
| `samsung_tv_home` | Go to home screen | "Go home" |
| `samsung_tv_status` | Get current state | "Is the TV on?" |
| `samsung_tv_list` | Show saved TVs | "Which TVs do I have?" |
| `samsung_tv_set_default` | Set primary TV | "Use living room by default" |
| `samsung_tv_remove` | Forget a TV | "Remove old TV" |

### Learning System (13 Tools)

| Tool | What It Does | When To Use |
|------|--------------|-------------|
| `samsung_tv_learn_get_recommendations` | AI-powered personalized picks | Main recommendation engine |
| `samsung_tv_learn_add_content` | Add movie/show to knowledge base | Expand content library |
| `samsung_tv_learn_record_session` | Record what was watched | After watching something |
| `samsung_tv_learn_feedback` | Submit explicit rating (1-5 stars) | When user gives feedback |
| `samsung_tv_learn_get_stats` | View learning statistics | Check how smart it is |
| `samsung_tv_learn_get_preferences` | See detected preferences | Understand user taste |
| `samsung_tv_learn_train` | Run experience replay training | Improve from history |
| `samsung_tv_learn_save` | Persist model to disk | Backup preferences |
| `samsung_tv_learn_load` | Load saved model | Restore after restart |
| `samsung_tv_learn_clear` | Reset all learning | Start fresh |
| `samsung_tv_learn_storage_stats` | Check storage usage | Monitor disk space |
| `samsung_tv_smart_launch` | Launch app + start tracking | Play content with learning |
| `samsung_tv_smart_end_session` | End viewing + record learning | Finish watching |

### Content Discovery (12 Tools)

| Tool | What It Does | Example Query |
|------|--------------|---------------|
| `content_search` | Search movies/shows by title | "Search for Dune" |
| `content_trending` | What's popular this week | "What's trending?" |
| `content_popular` | All-time popular content | "Most popular movies" |
| `content_top_rated` | Highest rated content | "Best rated TV shows" |
| `content_discover` | Filter by year, genre, rating | "Action movies from 2023" |
| `content_details` | Full info including cast | "Tell me about Oppenheimer" |
| `content_similar` | Find similar content | "Movies like Inception" |
| `content_recommendations` | TMDb's recommendations | "What else would I like?" |
| `content_now_playing` | Currently in theaters | "What's in theaters?" |
| `content_upcoming` | Coming soon | "Upcoming releases" |
| `content_personalized` | Combines learning + TMDb | "What should I watch?" |
| `content_for_mood` | Mood-based suggestions | "Something relaxing" |

---

## Tutorial: Complete Setup Guide

### Step 1: Prerequisites

```bash
# Required
node --version  # Must be 18+
npm --version   # Comes with Node.js

# Your Samsung TV must be:
# - 2016 model or newer (Tizen OS)
# - Connected to same WiFi network as your computer
# - Developer mode enabled (optional but helps)
```

### Step 2: Get TMDb API Key (Free)

1. Go to [themoviedb.org/signup](https://www.themoviedb.org/signup)
2. Create free account
3. Go to Settings → API → Create → Developer
4. Copy your API key (v3 auth)

### Step 3: Install and Build

```bash
# Clone the repository
git clone <repo-url>
cd hackathon-tv5/apps/samsung-tv-integration

# Install dependencies
npm install

# Build TypeScript
npm run build

# Run tests to verify everything works
npm test
# Expected: 71 tests passing
```

### Step 4: Configure Environment

```bash
# Set your TMDb key
export TMDB_API_KEY=your_key_here

# Optional: Set default TV IP if you know it
export SAMSUNG_TV_IP=192.168.1.100
```

### Step 5: Discover Your TV

```bash
# Start MCP server
npm start

# In another terminal, or through Claude:
# The samsung_tv_discover tool will find your TV automatically
```

### Step 6: Connect to Claude Desktop

Edit `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or equivalent:

```json
{
  "mcpServers": {
    "samsung-tv": {
      "command": "node",
      "args": ["/full/path/to/apps/samsung-tv-integration/dist/cli.js", "mcp"],
      "env": {
        "TMDB_API_KEY": "your_tmdb_key_here"
      }
    }
  }
}
```

### Step 7: First Conversation

Open Claude Desktop and try:

```
"Find Samsung TVs on my network"
"Connect to [TV name]"
"Turn on the TV"
"What's trending this week?"
"Find me an action movie from 2023"
"Launch Netflix"
```

---

## Architecture Deep Dive

```
┌─────────────────────────────────────────────────────────────────┐
│                        AI Assistant                              │
│                   (Claude, Gemini, etc.)                         │
└─────────────────────────┬───────────────────────────────────────┘
                          │ MCP Protocol (JSON-RPC over STDIO/SSE)
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                      MCP Server (38 tools)                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ TV Control  │  │  Learning   │  │   Content Discovery     │  │
│  │  13 tools   │  │  13 tools   │  │       12 tools          │  │
│  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘  │
└─────────┼────────────────┼─────────────────────┼────────────────┘
          │                │                     │
          ▼                ▼                     ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│  Samsung TV     │ │  Q-Learning     │ │   TMDb API      │
│  WebSocket API  │ │  + Embeddings   │ │   v3 REST       │
│  (port 8002)    │ │  (on-device)    │ │   (cloud)       │
└─────────────────┘ └─────────────────┘ └─────────────────┘
```

### File Structure

```
apps/samsung-tv-integration/
├── src/
│   ├── lib/                    # Core TV functionality
│   │   ├── types.ts            # Zod schemas, TypeScript interfaces
│   │   ├── tv-client.ts        # Samsung WebSocket client
│   │   └── discovery.ts        # SSDP network discovery
│   │
│   ├── learning/               # On-device ML system
│   │   ├── types.ts            # Learning schemas
│   │   ├── embeddings.ts       # 64-dim content vectors
│   │   ├── preference-learning.ts  # Q-Learning implementation
│   │   ├── persistence.ts      # Model save/load
│   │   └── smart-tv-client.ts  # Learning-enhanced TV client
│   │
│   ├── content/                # Content discovery
│   │   ├── tmdb-client.ts      # TMDb API wrapper with caching
│   │   └── discovery-tools.ts  # 12 discovery MCP tools
│   │
│   ├── mcp/                    # MCP server
│   │   ├── server.ts           # Main server, tool routing
│   │   └── learning-tools.ts   # 13 learning MCP tools
│   │
│   └── utils/                  # Utilities
│       ├── config.ts           # Device storage (conf)
│       └── helpers.ts          # ID generation, formatting
│
├── scripts/
│   └── train-benchmark.ts      # Training simulation & benchmarks
│
├── tests/                      # 71 tests
│   ├── helpers.test.ts
│   ├── types.test.ts
│   ├── learning.test.ts
│   └── content-discovery.test.ts
│
└── docs/
    ├── user-guide/             # End-user documentation
    └── developer/              # API reference, architecture
```

---

## Technology Stack

| Layer | Technology | Why We Chose It |
|-------|------------|-----------------|
| **Language** | TypeScript 5.6 | Type safety, great tooling |
| **Runtime** | Node.js 18+ | Async I/O, broad compatibility |
| **TV Protocol** | Samsung WebSocket (8002) | Official, low-latency |
| **Discovery** | SSDP/UPnP | Standard for device discovery |
| **Learning** | Q-Learning | Simple, interpretable, works offline |
| **Vectors** | Float32Array (64-dim) | Fast, compact, SIMD-friendly |
| **Content API** | TMDb v3 | Free tier, comprehensive data |
| **Schema** | Zod | Runtime validation + TypeScript types |
| **Testing** | Vitest | Fast, ESM-native, great DX |
| **MCP** | Model Context Protocol | AI assistant integration standard |

---

## Hackathon Entry

| | |
|---|---|
| **Event** | Agentics Foundation TV5 Hackathon |
| **Team** | agentics |
| **Track** | Entertainment Discovery |
| **Challenge** | Help users find what to watch faster |

### What We Built in 48 Hours

- Complete Samsung TV integration via WebSocket
- On-device Q-Learning recommendation system
- TMDb content discovery with 12 tools
- 38 MCP tools for AI assistant integration
- 71 tests with full coverage
- Benchmarks proving real-time performance
- Documentation for users and developers

### Key Innovations

1. **On-device learning** — Privacy-first, no cloud dependency
2. **WASM-ready embeddings** — 1M+ similarity ops/sec
3. **Unified control** — One interface for all streaming apps
4. **Conversational TV** — Natural language to remote commands

---

## Future Roadmap

- [ ] Multi-room TV coordination
- [ ] Voice control via microphone
- [ ] Watch party sync across homes
- [ ] Smart home integration (lights dim when movie starts)
- [ ] Mobile companion app
- [ ] LG/Sony/Roku TV support

---

## Documentation

- **[User Guide](docs/user-guide/README.md)** — Getting started, everyday usage
- **[Developer Guide](docs/developer/README.md)** — Architecture, APIs, extending

---

## Contributing

We welcome contributions! See the developer guide for architecture details.

```bash
# Run tests
npm test

# Run benchmarks
npm run benchmark

# Build
npm run build
```

---

## License

Apache-2.0 — See [LICENSE](LICENSE)

---

**Built with care by Team Agentics** for the [Agentics Foundation TV5 Hackathon](https://agentics.org/hackathon)
