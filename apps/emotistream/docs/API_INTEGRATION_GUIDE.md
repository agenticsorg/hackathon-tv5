# API Integration Guide

## Quick Start

This guide shows how to integrate the feedback and progress routes into the main EmotiStream API.

## Step 1: Update Main API Index

**File**: `/apps/emotistream/src/api/index.ts`

```typescript
import express, { Application } from 'express';
import cors from 'cors';
import helmet from 'helmet';
import compression from 'compression';
import rateLimit from 'express-rate-limit';

// Existing routes
import emotionRoutes from './routes/emotion.js';
import recommendRoutes from './routes/recommend.js';
import feedbackRoutes from './routes/feedback.js';
import authRoutes from './routes/auth.js';

// NEW: Import feedback and progress routes
import watchRoutes from './routes/watch.js';
import feedbackEnhancedRoutes from './routes/feedback-enhanced.js';
import progressRoutes from './routes/progress.js';

import { errorHandler } from './middleware/error-handler.js';
import { apiResponse } from './middleware/response.js';

export function createApp(): Application {
  const app = express();

  // Middleware
  app.use(helmet());
  app.use(cors());
  app.use(compression());
  app.use(express.json());

  // Rate limiting
  const limiter = rateLimit({
    windowMs: 15 * 60 * 1000, // 15 minutes
    max: 100 // limit each IP to 100 requests per windowMs
  });
  app.use('/api/', limiter);

  // Health check
  app.get('/api/v1/health', (req, res) => {
    res.json(apiResponse({
      status: 'ok',
      uptime: process.uptime(),
      timestamp: new Date().toISOString()
    }));
  });

  // Existing routes
  app.use('/api/v1/emotion', emotionRoutes);
  app.use('/api/v1/recommendations', recommendRoutes);
  app.use('/api/v1/feedback', feedbackRoutes); // Original feedback endpoint
  app.use('/api/v1/auth', authRoutes);

  // NEW: Mount feedback and progress routes
  app.use('/api/v1/watch', watchRoutes);           // Watch tracking
  app.use('/api/v1/feedback', feedbackEnhancedRoutes); // Enhanced feedback (will merge with existing)
  app.use('/api/v1/progress', progressRoutes);     // Progress analytics

  // Error handling
  app.use(errorHandler);

  return app;
}
```

## Step 2: Merge Feedback Routes (Optional)

You can either:
1. Keep both routes separate (existing + enhanced)
2. Merge them into one file

**Option 1: Keep Separate (Recommended)**

```typescript
// Keep existing: /api/v1/feedback (POST)
app.use('/api/v1/feedback', feedbackRoutes);

// Add enhanced: /api/v1/feedback/submit (POST), /api/v1/feedback/:id (GET), etc.
app.use('/api/v1/feedback', feedbackEnhancedRoutes);
```

Express will match routes in order, so:
- `POST /api/v1/feedback` → Original route (existing RL integration)
- `POST /api/v1/feedback/submit` → Enhanced route (new detailed feedback)
- `GET /api/v1/feedback/:id` → Enhanced route

**Option 2: Merge into Single File**

Combine routes from both files into `/api/routes/feedback.ts`:

```typescript
// Existing POST /feedback
router.post('/', async (req, res) => { /* ... */ });

// Enhanced POST /feedback/submit
router.post('/submit', async (req, res) => { /* ... */ });

// Enhanced GET /feedback/:feedbackId
router.get('/:feedbackId', async (req, res) => { /* ... */ });

// etc.
```

## Step 3: Initialize Stores on Server Start

**File**: `/apps/emotistream/src/server.ts`

```typescript
import { createApp } from './api/index.js';
import { FeedbackStore } from './persistence/feedback-store.js';
import { logger } from './utils/logger.js';
import dotenv from 'dotenv';

dotenv.config();

const PORT = process.env.PORT || 3000;

async function startServer() {
  try {
    // Initialize feedback store
    const feedbackStore = new FeedbackStore();
    await feedbackStore.initialize();
    logger.info('Feedback store initialized');

    // Create and start Express app
    const app = createApp();

    app.listen(PORT, () => {
      logger.info(`EmotiStream API running on port ${PORT}`);
      logger.info(`Health check: http://localhost:${PORT}/api/v1/health`);
    });
  } catch (error) {
    logger.error('Failed to start server', { error });
    process.exit(1);
  }
}

startServer();
```

## Step 4: Add Environment Variables

**File**: `.env`

```bash
# Server
PORT=3000
NODE_ENV=development

# Gemini AI
GEMINI_API_KEY=your_api_key_here

# JWT (existing)
JWT_SECRET=your_jwt_secret_here

# NEW: Feedback system configuration (optional)
FEEDBACK_PERSISTENCE_PATH=./data/feedback.json
WATCH_SESSION_TTL=86400000  # 24 hours in ms

# NEW: Progress analytics configuration (optional)
CONVERGENCE_THRESHOLD_LOW=30
CONVERGENCE_THRESHOLD_HIGH=70
```

## Step 5: Test the Integration

### 1. Start the Server

```bash
npm run dev
```

### 2. Test Watch Tracking

```bash
# Start watch session
curl -X POST http://localhost:3000/api/v1/watch/start \
  -H "Content-Type: application/json" \
  -d '{
    "userId": "test_user_123",
    "contentId": "content_matrix",
    "contentTitle": "The Matrix"
  }'

# Response should include sessionId
# Copy the sessionId for next steps
```

### 3. Test Feedback Submission

```bash
# Submit feedback
curl -X POST http://localhost:3000/api/v1/feedback/submit \
  -H "Content-Type: application/json" \
  -d '{
    "userId": "test_user_123",
    "contentId": "content_matrix",
    "contentTitle": "The Matrix",
    "sessionId": "watch_abc123",
    "emotionBefore": {
      "valence": -0.3,
      "arousal": -0.2,
      "stress": 0.6
    },
    "emotionAfter": {
      "valence": 0.5,
      "arousal": 0.3,
      "stress": 0.2
    },
    "desiredState": {
      "valence": 0.6,
      "arousal": 0.0,
      "stress": 0.1
    },
    "starRating": 5,
    "completed": true,
    "totalDuration": 8160000
  }'
```

### 4. Test Progress Dashboard

```bash
# Get overall progress
curl http://localhost:3000/api/v1/progress/test_user_123

# Get convergence analysis
curl http://localhost:3000/api/v1/progress/test_user_123/convergence

# Get emotional journey
curl http://localhost:3000/api/v1/progress/test_user_123/journey

# Get reward timeline
curl http://localhost:3000/api/v1/progress/test_user_123/rewards

# Get recent experiences
curl http://localhost:3000/api/v1/progress/test_user_123/experiences
```

## Step 6: Add to Package Scripts

**File**: `package.json`

```json
{
  "scripts": {
    "dev": "nodemon --exec tsx src/server.ts",
    "start": "node --loader ts-node/esm src/server.ts",
    "build": "tsc",
    "test": "jest --coverage",
    "test:watch": "jest --watch",
    "test:integration": "jest --testPathPattern=integration",

    "demo": "tsx src/cli/index.ts",
    "demo:feedback": "tsx examples/feedback-demo.ts",
    "demo:progress": "tsx examples/progress-demo.ts"
  }
}
```

## Demo Scripts

### Feedback Demo

**File**: `/examples/feedback-demo.ts`

```typescript
import { WatchTracker } from '../src/services/watch-tracker.js';
import { RewardCalculator } from '../src/services/reward-calculator.js';
import { FeedbackStore } from '../src/persistence/feedback-store.js';

async function runFeedbackDemo() {
  console.log('🎬 EmotiStream Feedback Demo\n');

  // Initialize services
  const watchTracker = new WatchTracker();
  const rewardCalculator = new RewardCalculator();
  const feedbackStore = new FeedbackStore();
  await feedbackStore.initialize();

  // 1. Start watch session
  console.log('1. Starting watch session...');
  const session = watchTracker.startSession(
    'demo_user',
    'content_inception',
    'Inception'
  );
  console.log(`   ✓ Session started: ${session.sessionId}\n`);

  // 2. Simulate watching
  console.log('2. Watching content...');
  await new Promise(resolve => setTimeout(resolve, 2000));
  console.log('   ✓ Watched for 2 seconds\n');

  // 3. End session
  console.log('3. Ending session...');
  const endedSession = watchTracker.endSession(session.sessionId, true);
  console.log(`   ✓ Duration: ${endedSession.duration}ms\n`);

  // 4. Calculate reward
  console.log('4. Calculating reward...');
  const emotionBefore = { valence: -0.3, arousal: -0.2, stress: 0.6 };
  const emotionAfter = { valence: 0.5, arousal: 0.3, stress: 0.2 };
  const desiredState = { valence: 0.6, arousal: 0.0, stress: 0.1 };

  const rewardCalc = rewardCalculator.calculate(
    emotionBefore as any,
    emotionAfter as any,
    desiredState as any,
    true,
    5,
    endedSession.duration,
    7200000
  );

  console.log(`   ✓ Reward: ${rewardCalc.reward.toFixed(3)}`);
  console.log(`   ✓ Emotional alignment: ${rewardCalc.components.emotionalAlignment.toFixed(3)}`);
  console.log(`   ✓ Completion bonus: ${rewardCalc.components.completionBonus.toFixed(3)}`);
  console.log(`   ✓ Rating bonus: ${rewardCalc.components.ratingBonus.toFixed(3)}`);
  console.log(`   ✓ ${rewardCalc.explanation}\n`);

  // 5. Store feedback
  console.log('5. Storing feedback...');
  const feedback = await feedbackStore.saveFeedback(
    {
      userId: 'demo_user',
      contentId: 'content_inception',
      contentTitle: 'Inception',
      sessionId: session.sessionId,
      emotionBefore: emotionBefore as any,
      emotionAfter: emotionAfter as any,
      starRating: 5,
      completed: true,
      watchDuration: endedSession.duration,
      totalDuration: 7200000,
      timestamp: new Date()
    },
    rewardCalc.reward,
    0.5,
    0.5 + rewardCalc.reward * 0.1
  );
  console.log(`   ✓ Feedback stored: ${feedback.feedbackId}\n`);

  console.log('✨ Demo complete!');
}

runFeedbackDemo().catch(console.error);
```

### Progress Demo

**File**: `/examples/progress-demo.ts`

```typescript
import { ProgressAnalytics } from '../src/services/progress-analytics.js';
import { FeedbackStore } from '../src/persistence/feedback-store.js';

async function runProgressDemo() {
  console.log('📊 EmotiStream Progress Demo\n');

  // Initialize services
  const progressAnalytics = new ProgressAnalytics();
  const feedbackStore = new FeedbackStore();
  await feedbackStore.initialize();

  const userId = 'demo_user';

  // Get feedback history
  const feedbackHistory = feedbackStore.getUserFeedback(userId);
  console.log(`Found ${feedbackHistory.length} feedback records\n`);

  if (feedbackHistory.length === 0) {
    console.log('⚠️  No feedback history. Run feedback-demo first!\n');
    return;
  }

  // Calculate progress
  console.log('Calculating learning progress...\n');
  const progress = progressAnalytics.calculateProgress(userId, feedbackHistory);

  // Display results
  console.log('📈 Learning Progress:');
  console.log(`   Total Experiences: ${progress.totalExperiences}`);
  console.log(`   Completed Content: ${progress.completedContent}`);
  console.log(`   Average Reward: ${progress.averageReward.toFixed(3)}`);
  console.log(`   Reward Trend: ${progress.rewardTrend}`);
  console.log(`   Exploration Rate: ${(progress.explorationRate * 100).toFixed(1)}%`);
  console.log();

  console.log('🎯 Convergence:');
  console.log(`   Score: ${progress.convergenceScore.toFixed(1)}/100`);
  console.log(`   Stage: ${progress.convergenceStage}`);
  console.log();

  console.log('🗺️  Emotional Journey:');
  progress.emotionalJourney.slice(-5).forEach(point => {
    console.log(`   ${point.experienceNumber}. ${point.contentTitle}`);
    console.log(`      Before: V=${point.emotionBefore.valence.toFixed(2)} A=${point.emotionBefore.arousal.toFixed(2)} S=${point.emotionBefore.stress.toFixed(2)}`);
    console.log(`      After:  V=${point.emotionAfter.valence.toFixed(2)} A=${point.emotionAfter.arousal.toFixed(2)} S=${point.emotionAfter.stress.toFixed(2)}`);
    console.log(`      Reward: ${point.reward.toFixed(3)}`);
    console.log();
  });

  console.log('⭐ Best Content:');
  progress.bestContent.slice(0, 3).forEach(content => {
    console.log(`   ${content.contentTitle}`);
    console.log(`      Avg Reward: ${content.averageReward.toFixed(3)}`);
    console.log(`      Times Watched: ${content.timesWatched}`);
    console.log(`      Completion Rate: ${(content.completionRate * 100).toFixed(0)}%`);
    console.log();
  });

  console.log('✨ Demo complete!');
}

runProgressDemo().catch(console.error);
```

## Troubleshooting

### Issue: Routes not found (404)

**Solution**: Check route mounting order in `src/api/index.ts`

```typescript
// Make sure routes are mounted BEFORE error handler
app.use('/api/v1/watch', watchRoutes);
app.use('/api/v1/feedback', feedbackEnhancedRoutes);
app.use('/api/v1/progress', progressRoutes);

// Error handler must be last
app.use(errorHandler);
```

### Issue: TypeScript errors

**Solution**: Rebuild the project

```bash
npm run build
```

Check for missing type definitions:

```bash
npm install --save-dev @types/node @types/express
```

### Issue: Data not persisting

**Solution**: Check data directory exists

```bash
mkdir -p data
chmod 755 data
```

Verify `.env` configuration:

```bash
FEEDBACK_PERSISTENCE_PATH=./data/feedback.json
```

### Issue: CORS errors from frontend

**Solution**: Configure CORS properly

```typescript
import cors from 'cors';

app.use(cors({
  origin: process.env.FRONTEND_URL || 'http://localhost:3001',
  credentials: true
}));
```

## Production Checklist

- [ ] Environment variables configured
- [ ] Database directory created with proper permissions
- [ ] Rate limiting configured
- [ ] CORS configured for production domain
- [ ] Error logging configured
- [ ] Health check endpoint tested
- [ ] All routes tested with integration tests
- [ ] API documentation published
- [ ] Monitoring and alerting setup
- [ ] Backup strategy for feedback data

## Next Steps

1. **Testing**: Run integration tests
2. **Frontend**: Implement React components from spec
3. **Database**: Migrate to AgentDB
4. **Real-time**: Add WebSocket support for live updates
5. **Analytics**: Add tracking and monitoring
6. **Optimization**: Performance tuning and caching

## Support

For issues or questions:
- Check API documentation: `/docs/FEEDBACK_AND_PROGRESS_API.md`
- Review frontend spec: `/docs/FRONTEND_COMPONENTS_SPEC.md`
- See integration examples: `/docs/INTEGRATION_EXAMPLES.md`
