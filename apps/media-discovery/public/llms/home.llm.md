# AI Media Discovery - Home

<!-- chunk: overview -->

## Overview

AI Media Discovery helps you find movies and TV shows through natural language. Just describe what you're in the mood to watch, and we'll find the perfect match.

<!-- chunk: capabilities -->

## What You Can Do

### Natural Language Search

Describe what you want to watch in plain English:
- "exciting sci-fi adventure like Interstellar"
- "cozy romantic comedy for a rainy day"
- "dark psychological thriller with unexpected twists"
- "something inspiring about overcoming challenges"

### Browse by Category

- **Trending**: What's popular this week
- **Top Rated**: Highest rated movies and shows
- **Discover**: Filter by genre, year, and rating

### Personalized Recommendations

Based on your viewing history and preferences, we suggest content you'll love.

<!-- chunk: api-access -->

## API Access for AI Agents

### Semantic Search

```http
POST /api/search
Content-Type: application/json

{
  "query": "heartwarming animated movies for family",
  "explain": true
}
```

### Get Recommendations

```http
POST /api/recommendations
Content-Type: application/json

{
  "basedOn": {
    "contentId": 550,
    "mediaType": "movie"
  }
}
```

### Discover Content

```http
GET /api/discover?category=trending&type=all
```

<!-- chunk: content-types -->

## Content Types

- **Movies**: Feature films from all genres and eras
- **TV Shows**: Series with full season information

All content metadata is sourced from The Movie Database (TMDB).
