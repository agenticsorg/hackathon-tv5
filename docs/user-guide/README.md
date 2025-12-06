# User Guide

Complete guide for using the Samsung Smart TV Integration with AI assistants.

## Table of Contents

1. [Installation](#installation)
2. [Connecting Your TV](#connecting-your-tv)
3. [Basic TV Control](#basic-tv-control)
4. [Content Discovery](#content-discovery)
5. [Learning System](#learning-system)
6. [Using with AI Assistants](#using-with-ai-assistants)

---

## Installation

### Prerequisites

- Node.js 18 or later
- Samsung Smart TV (2016 or newer) on the same network
- TMDb API key (free at [themoviedb.org](https://www.themoviedb.org/settings/api))

### Setup

```bash
cd apps/samsung-tv-integration
npm install
npm run build
```

### Environment

Create a `.env` file or export:

```bash
export TMDB_API_KEY=your_api_key_here
```

---

## Connecting Your TV

### 1. Discover TVs

First, find Samsung TVs on your network:

```json
{ "tool": "samsung_tv_discover", "timeout": 5000 }
```

Returns:
```json
{
  "success": true,
  "data": {
    "count": 1,
    "devices": [{
      "id": "samsung-tv-192-168-1-100",
      "name": "Living Room TV",
      "ip": "192.168.1.100",
      "model": "UN55NU8000"
    }]
  }
}
```

### 2. Connect to TV

Connect to authenticate (TV will show pairing dialog first time):

```json
{ "tool": "samsung_tv_connect", "ip": "192.168.1.100" }
```

**Important**: Accept the pairing dialog on your TV within 30 seconds.

### 3. Set Default TV

```json
{ "tool": "samsung_tv_set_default", "deviceId": "samsung-tv-192-168-1-100" }
```

---

## Basic TV Control

### Power

```json
// Turn off
{ "tool": "samsung_tv_power", "action": "off" }

// Turn on (requires MAC address)
{ "tool": "samsung_tv_power", "action": "on" }

// Toggle
{ "tool": "samsung_tv_power", "action": "toggle" }
```

### Volume

```json
// Volume up (5 steps)
{ "tool": "samsung_tv_volume", "action": "up", "steps": 5 }

// Volume down
{ "tool": "samsung_tv_volume", "action": "down", "steps": 3 }

// Mute
{ "tool": "samsung_tv_volume", "action": "mute" }

// Unmute
{ "tool": "samsung_tv_volume", "action": "unmute" }
```

### Navigation

```json
// Arrow keys
{ "tool": "samsung_tv_navigate", "direction": "up" }
{ "tool": "samsung_tv_navigate", "direction": "down" }
{ "tool": "samsung_tv_navigate", "direction": "left" }
{ "tool": "samsung_tv_navigate", "direction": "right" }

// Select
{ "tool": "samsung_tv_navigate", "direction": "enter" }

// Go back
{ "tool": "samsung_tv_navigate", "direction": "back" }
```

### Apps

```json
// List installed apps
{ "tool": "samsung_tv_apps" }

// Launch Netflix
{ "tool": "samsung_tv_launch_app", "app": "NETFLIX" }

// Launch YouTube
{ "tool": "samsung_tv_launch_app", "app": "YOUTUBE" }

// Launch by app ID
{ "tool": "samsung_tv_launch_app", "app": "111299001912" }
```

**Supported app shortcuts**: YOUTUBE, NETFLIX, PRIME_VIDEO, DISNEY_PLUS, HBO_MAX, HULU, APPLE_TV, SPOTIFY, PLEX, TWITCH

### Home Screen

```json
{ "tool": "samsung_tv_home" }
```

---

## Content Discovery

Find movies and TV shows using TMDb integration.

### Search

```json
// Search for content
{ "tool": "content_search", "query": "inception", "type": "movie" }

// Search TV shows
{ "tool": "content_search", "query": "breaking bad", "type": "tv" }

// Search all
{ "tool": "content_search", "query": "batman", "type": "all" }
```

### Browse

```json
// Trending this week
{ "tool": "content_trending", "timeWindow": "week" }

// Popular movies
{ "tool": "content_popular", "type": "movie" }

// Top rated TV shows
{ "tool": "content_top_rated", "type": "tv" }

// Now in theaters
{ "tool": "content_now_playing" }

// Coming soon
{ "tool": "content_upcoming" }
```

### Discover with Filters

```json
// Action movies from 2020+, rating 7+
{
  "tool": "content_discover",
  "type": "movie",
  "genres": ["action"],
  "minYear": 2020,
  "minRating": 7,
  "sortBy": "popularity.desc"
}

// Sci-fi TV shows
{
  "tool": "content_discover",
  "type": "tv",
  "genres": ["science_fiction"],
  "minRating": 8
}
```

### Mood-Based

```json
// Relaxing evening
{ "tool": "content_for_mood", "mood": "relaxing", "duration": "medium" }

// Scary movie night
{ "tool": "content_for_mood", "mood": "scary" }

// Family friendly
{ "tool": "content_for_mood", "mood": "family", "count": 10 }
```

**Moods**: relaxing, exciting, romantic, scary, funny, thoughtful, family, nostalgic

---

## Learning System

The system learns your preferences over time for better recommendations.

### Get Recommendations

```json
// Personalized recommendations
{ "tool": "samsung_tv_learn_get_recommendations", "count": 5 }

// Or combined with discovery
{ "tool": "content_personalized", "count": 10 }
```

### Record Viewing

For best results, record what you watch:

```json
{
  "tool": "samsung_tv_learn_record_session",
  "contentId": "tmdb-movie-27205",
  "watchDuration": 148,
  "completionRate": 1.0,
  "userRating": 5
}
```

### Smart Launch (Automatic Learning)

Use smart launch to automatically track viewing:

```json
// Launch with tracking
{
  "tool": "samsung_tv_smart_launch",
  "app": "NETFLIX",
  "contentId": "tmdb-movie-27205"
}

// End session (records learning)
{ "tool": "samsung_tv_smart_end_session", "userRating": 5 }
```

### View Stats

```json
// Learning statistics
{ "tool": "samsung_tv_learn_get_stats" }

// Your preferences
{ "tool": "samsung_tv_learn_get_preferences" }
```

### Save/Load Model

```json
// Save learned model
{ "tool": "samsung_tv_learn_save" }

// Load model
{ "tool": "samsung_tv_learn_load" }
```

---

## Using with AI Assistants

### Claude Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "samsung-tv": {
      "command": "node",
      "args": ["path/to/apps/samsung-tv-integration/dist/cli.js", "mcp"]
    }
  }
}
```

### Example Conversations

**Finding something to watch:**
> "I want to watch something exciting tonight, maybe an action movie from the last few years"

**Controlling TV:**
> "Turn on the TV and open Netflix"

**Getting recommendations:**
> "What should I watch based on my viewing history?"

**Mood-based:**
> "Find me something relaxing to watch before bed"

---

## Troubleshooting

### TV Not Found

- Ensure TV is on and connected to same network
- Try extending timeout: `{ "timeout": 10000 }`
- Check if TV's IP changed

### Connection Refused

- TV may have blocked the app - go to TV Settings > General > External Device Manager > Device Connection Manager
- Delete paired device and try again

### Pairing Dialog Not Showing

- Ensure TV screen is on (not screensaver)
- Try power cycling TV

### TMDb Errors

- Verify `TMDB_API_KEY` is set correctly
- Check API rate limits (40 requests/10 seconds)
