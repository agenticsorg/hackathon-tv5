# EmotiStream

**Emotion-aware content recommendation system using Q-Learning and Gemini AI**

## Overview

EmotiStream is an intelligent recommendation system that learns user emotional preferences and recommends content based on desired emotional states. It combines:

- **Emotion Detection**: Gemini AI analyzes user emotional state
- **Q-Learning**: Reinforcement learning policy for personalized recommendations
- **Vector Search**: AgentDB HNSW index for emotional profile similarity
- **REST API**: Express-based API for integration

## Architecture

```
┌─────────────────┐
│ User Input      │
│ (Text/Context)  │
└────────┬────────┘
         │
         ▼
┌─────────────────────┐
│ EmotionDetector     │
│ (Gemini 2.0)        │
│ - Russell Model     │
│ - Plutchik Wheel    │
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐         ┌─────────────────┐
│ RLPolicyEngine      │◄────────┤ ReplayBuffer    │
│ (Q-Learning)        │         │ (SQLite)        │
│ - State Buckets     │         └─────────────────┘
│ - ε-greedy + UCB    │
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐         ┌─────────────────┐
│ RecommendationEngine│◄────────┤ ContentProfiler │
│ - Ranked List       │         │ (AgentDB HNSW)  │
│ - Predicted Outcome │         └─────────────────┘
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│ FeedbackProcessor   │
│ - Reward Calc       │
│ - Q-value Update    │
└─────────────────────┘
```

## Project Structure

```
apps/emotistream/
├── src/
│   ├── types/              # Shared TypeScript interfaces
│   ├── emotion/            # EmotionDetector module
│   ├── rl/                 # RLPolicyEngine module
│   ├── content/            # ContentProfiler module
│   ├── recommendations/    # RecommendationEngine module
│   ├── feedback/           # FeedbackProcessor module
│   ├── api/                # Express REST API
│   ├── cli/                # CLI Demo
│   ├── db/                 # Database clients
│   └── utils/              # Utilities (config, errors, logger)
├── tests/
│   ├── unit/               # Unit tests
│   └── integration/        # Integration tests
├── data/                   # Database files (gitignored)
├── package.json
├── tsconfig.json
└── jest.config.js
```

## Installation

```bash
cd apps/emotistream
npm install
```

## Configuration

1. Copy `.env.example` to `.env`:
```bash
cp .env.example .env
```

2. Add your Gemini API key:
```bash
GEMINI_API_KEY=your_api_key_here
```

## Usage

### CLI Demo
```bash
npm run start:cli
```

### API Server
```bash
npm run start:api
```

### Development
```bash
npm run dev
```

### Testing
```bash
npm test                  # Run all tests
npm run test:watch        # Watch mode
npm run test:coverage     # Coverage report
npm run test:integration  # Integration tests only
```

## API Endpoints

### Emotion Detection
```
POST /api/v1/emotion/detect
Body: { userId, input, context? }
Response: { emotionalState, desiredState }
```

### Get Recommendations
```
POST /api/v1/recommendations
Body: { userId, currentState, desiredState, limit? }
Response: { recommendations[] }
```

### Submit Feedback
```
POST /api/v1/feedback
Body: { userId, contentId, actualPostState, watchDuration, completed, explicitRating? }
Response: { reward, policyUpdated, newQValue, learningProgress }
```

### Profile Content
```
POST /api/v1/content/profile
Body: { contentId, metadata, emotionalJourney }
Response: { profile }
```

### Health Check
```
GET /api/v1/health
Response: { status, uptime, components, timestamp }
```

## Key Concepts

### Emotional State (Russell's Circumplex)
- **Valence**: -1 (negative) to +1 (positive)
- **Arousal**: -1 (calm) to +1 (excited)
- **Stress**: 0 (relaxed) to 1 (stressed)

### Q-Learning
- **State Space**: Discretized emotional states (75 buckets)
- **Actions**: Content recommendations
- **Reward**: Alignment with desired emotional outcome
- **Policy**: ε-greedy + UCB exploration

### Recommendation Ranking
- **Combined Score**: 0.7 * Q-value + 0.3 * Emotional Similarity
- **Predicted Outcome**: Expected emotional state after consumption

## Hyperparameters

See `src/utils/config.ts` for all configuration options:

- Learning rate (α): 0.1
- Discount factor (γ): 0.95
- Exploration rate (ε): 0.15 → 0.10
- UCB constant: 2.0
- State buckets: 5 (valence) × 5 (arousal) × 3 (stress) = 75 states

## Development

### Type Checking
```bash
npm run typecheck
```

### Linting
```bash
npm run lint
```

### Build
```bash
npm run build
```

## License

MIT
