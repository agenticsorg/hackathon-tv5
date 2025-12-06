# EmotiStream REST API

## Quick Start

### 1. Install dependencies
```bash
npm install
```

### 2. Configure environment
```bash
cp .env.example .env
# Edit .env with your configuration
```

### 3. Start the server
```bash
# Development mode (with auto-reload)
npm run dev

# Production mode
npm run build
npm start
```

### 4. Test the API
```bash
# Health check
curl http://localhost:3000/health

# Analyze emotion
curl -X POST http://localhost:3000/api/v1/emotion/analyze \
  -H "Content-Type: application/json" \
  -d '{"userId":"user-123","text":"I feel stressed and need to relax"}'
```

## API Documentation

See [docs/API.md](./docs/API.md) for complete API documentation.

## Implementation Status

See [docs/API-IMPLEMENTATION-SUMMARY.md](./docs/API-IMPLEMENTATION-SUMMARY.md) for implementation details.

## Architecture

```
src/api/
├── index.ts                 # Express app setup
├── middleware/
│   ├── error-handler.ts     # Global error handling
│   ├── logger.ts            # Request logging
│   └── rate-limiter.ts      # Rate limiting
└── routes/
    ├── emotion.ts           # Emotion detection endpoints
    ├── recommend.ts         # Recommendation endpoints
    └── feedback.ts          # Feedback endpoints
```

## Available Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check |
| POST | `/api/v1/emotion/analyze` | Analyze emotional state |
| GET | `/api/v1/emotion/history/:userId` | Get emotion history |
| POST | `/api/v1/recommend` | Get recommendations |
| GET | `/api/v1/recommend/history/:userId` | Get recommendation history |
| POST | `/api/v1/feedback` | Submit feedback |
| GET | `/api/v1/feedback/progress/:userId` | Get learning progress |
| GET | `/api/v1/feedback/experiences/:userId` | Get experiences |

## Rate Limits

- General API: 100 requests/minute
- Emotion Analysis: 30 requests/minute
- Recommendations: 60 requests/minute

## Development

```bash
# Run development server
npm run dev

# Build for production
npm run build

# Run tests
npm test

# Type check
npm run typecheck
```

## Integration TODOs

The API is complete but currently returns mock data. Integration needed with:

1. EmotionDetector module (for emotion analysis)
2. RecommendationEngine module (for recommendations)
3. FeedbackProcessor module (for feedback processing)
4. Storage layer (for history endpoints)

See source code comments marked with `// TODO:` for exact integration points.
