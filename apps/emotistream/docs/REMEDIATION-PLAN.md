# EmotiStream Remediation Plan

**Based on:** Sherlock Investigation Report
**Date:** 2025-12-07
**Scope:** Backend (`apps/emotistream`) and Frontend (`apps/emotistream-web`)

---

## Summary of Real Issues

After evidence-based investigation, these are the **actual** issues requiring fixes:

| Priority | Issue | Impact | Effort |
|----------|-------|--------|--------|
| P0 | Session stateBefore never stored | RL training data corrupted | 2h |
| P0 | Content profiles use random values | Recommendations semi-random | 3h |
| P1 | Legacy stub endpoints not deprecated | API confusion | 1h |
| P1 | `any` types in production code | Type safety issues | 2h |
| P2 | Duplicate Q-calculation logic | Code maintenance | 1h |
| P2 | Console.log in production | Log noise | 1h |

---

## P0 - Critical Fixes

### Fix 1: Store Emotional State Before Watching

**Problem:** The `userSessionStore` in `feedback.ts` is never populated, causing all "before" states to be estimated from "after" states.

**Files to Modify:**
- `src/api/routes/recommend.ts`
- `src/api/routes/feedback.ts`

**Implementation:**

```typescript
// In recommend.ts - After generating recommendations
// Store the current emotional state for later feedback

import { sessionStore } from './session-store.js'; // New shared module

router.post('/', async (req, res, next) => {
  const { userId, currentState, desiredState } = req.body;

  // ... existing recommendation logic ...

  // Store session for each recommended content
  for (const rec of recommendations) {
    sessionStore.set(`${userId}:${rec.contentId}`, {
      stateBefore: currentState,
      desiredState: desiredState,
      contentId: rec.contentId,
      timestamp: Date.now()
    });
  }

  // ... return response ...
});
```

**New File: `src/api/routes/session-store.ts`**
```typescript
import { FileStore } from '../../persistence/file-store.js';

interface SessionData {
  stateBefore: { valence: number; arousal: number; stress: number };
  desiredState: { targetValence: number; targetArousal: number; targetStress: number };
  contentId: string;
  timestamp: number;
}

// Use FileStore for persistence across restarts
export const sessionStore = new FileStore<SessionData>('sessions.json');

// Clean up old sessions (>24h)
export function cleanupSessions(): void {
  const oneDayAgo = Date.now() - 24 * 60 * 60 * 1000;
  for (const [key, session] of sessionStore.entries()) {
    if (session.timestamp < oneDayAgo) {
      sessionStore.delete(key);
    }
  }
}
```

**Test:**
```bash
# 1. Call /recommend with currentState
# 2. Check data/sessions.json is created
# 3. Call /feedback for same content
# 4. Verify stateBefore matches original currentState
```

---

### Fix 2: Deterministic Content Emotional Profiles

**Problem:** `profiler.ts` uses `Math.random()` for valenceDelta, arousalDelta, intensity even when real genre data is available.

**Files to Modify:**
- `src/content/profiler.ts`
- `src/content/tmdb-catalog.ts`

**Implementation:**

```typescript
// In profiler.ts - Replace random profile generation

private GENRE_EMOTIONAL_PROFILES: Record<string, {
  valenceDelta: number;
  arousalDelta: number;
  intensity: number
}> = {
  'comedy': { valenceDelta: 0.5, arousalDelta: 0.2, intensity: 0.6 },
  'horror': { valenceDelta: -0.3, arousalDelta: 0.7, intensity: 0.9 },
  'romance': { valenceDelta: 0.4, arousalDelta: -0.1, intensity: 0.5 },
  'action': { valenceDelta: 0.3, arousalDelta: 0.6, intensity: 0.8 },
  'drama': { valenceDelta: 0.1, arousalDelta: 0.1, intensity: 0.7 },
  'documentary': { valenceDelta: 0.2, arousalDelta: -0.2, intensity: 0.4 },
  'thriller': { valenceDelta: -0.1, arousalDelta: 0.5, intensity: 0.8 },
  'animation': { valenceDelta: 0.4, arousalDelta: 0.3, intensity: 0.5 },
  'family': { valenceDelta: 0.5, arousalDelta: 0.1, intensity: 0.4 },
  'sci-fi': { valenceDelta: 0.2, arousalDelta: 0.4, intensity: 0.7 },
};

async profile(content: ContentMetadata): Promise<EmotionalContentProfile> {
  // Calculate emotional profile from genres (deterministic)
  const genreProfiles = content.genres
    .map(g => this.GENRE_EMOTIONAL_PROFILES[g.toLowerCase()])
    .filter(Boolean);

  // Average the genre profiles or use defaults
  const avgProfile = genreProfiles.length > 0
    ? {
        valenceDelta: average(genreProfiles.map(p => p.valenceDelta)),
        arousalDelta: average(genreProfiles.map(p => p.arousalDelta)),
        intensity: average(genreProfiles.map(p => p.intensity)),
      }
    : { valenceDelta: 0, arousalDelta: 0, intensity: 0.5 }; // Neutral default

  const profile: EmotionalContentProfile = {
    contentId: content.contentId,
    primaryTone: this.inferTone(content),
    valenceDelta: avgProfile.valenceDelta,
    arousalDelta: avgProfile.arousalDelta,
    intensity: avgProfile.intensity,
    complexity: this.calculateComplexity(content), // New deterministic method
    // ...
  };

  return profile;
}

private calculateComplexity(content: ContentMetadata): number {
  // Base complexity on genre count and content type
  const genreComplexity = Math.min(content.genres.length / 4, 1);
  const isDocumentary = content.category === 'documentary' ? 0.2 : 0;
  return Math.min(0.3 + genreComplexity + isDocumentary, 1);
}
```

**Test:**
```bash
# Profile same content twice
# Verify valenceDelta, arousalDelta, intensity are identical
```

---

## P1 - High Priority Fixes

### Fix 3: Deprecate Legacy Stub Endpoints

**Problem:** Duplicate endpoints exist - legacy stubs in `feedback.ts` vs working ones in `progress.ts`

**Files to Modify:**
- `src/api/routes/feedback.ts`

**Implementation:**

```typescript
// Add deprecation warnings to legacy endpoints

/**
 * @deprecated Use GET /api/v1/progress/:userId instead
 */
router.get('/progress/:userId', async (req, res, next) => {
  console.warn('DEPRECATED: /feedback/progress/:userId - Use /progress/:userId instead');
  // Redirect or return deprecation notice
  res.status(301).json({
    success: false,
    error: {
      code: 'DEPRECATED',
      message: 'This endpoint is deprecated. Use GET /api/v1/progress/:userId instead',
      redirect: `/api/v1/progress/${req.params.userId}`
    }
  });
});
```

**Also add to:**
- `GET /feedback/experiences/:userId` → Use `GET /progress/:userId/experiences`
- `GET /recommend/history/:userId` → Implement or deprecate
- `GET /emotion/history/:userId` → Implement or deprecate

---

### Fix 4: Add Type Definitions for `any` Usage

**Files to Modify:**
- `src/api/routes/feedback.ts`
- `src/api/routes/recommend.ts`
- `src/api/routes/emotion.ts`
- `src/content/vector-store.ts`

**Implementation:**

```typescript
// feedback.ts:16 - Replace any with proper types
interface SessionData {
  stateBefore: EmotionalStateSnapshot;
  desiredState: DesiredStateSnapshot;
  contentId: string;
}

interface EmotionalStateSnapshot {
  valence: number;
  arousal: number;
  stress: number;
  confidence: number;
}

const userSessionStore = new Map<string, SessionData>();

// Response types - Replace ApiResponse<any>
interface ProgressResponse {
  userId: string;
  totalExperiences: number;
  avgReward: number;
  explorationRate: number;
  convergenceScore: number;
  recentRewards: number[];
}

router.get('/progress/:userId',
  async (req: Request, res: Response<ApiResponse<ProgressResponse>>, next) => {
    // ...
  }
);
```

---

## P2 - Medium Priority Fixes

### Fix 5: Remove Duplicate Q-Calculation

**Problem:** `FeedbackProcessor.process()` has a simplified Q-calculation that's not used.

**File:** `src/feedback/processor.ts`

**Implementation:**

```typescript
// Remove lines 74-77 (simplified Q-calculation)
// The return value should use the Q-value from policyEngine.updatePolicy()

process(request, stateBefore, desiredState): FeedbackResponse {
  // ... existing reward calculation ...

  // Remove this:
  // const oldQValue = 0;
  // const learningRate = 0.1;
  // const newQValue = oldQValue + learningRate * (finalReward - oldQValue);

  return {
    reward: finalReward,
    policyUpdated: false, // Changed - actual update happens in route handler
    newQValue: 0, // Placeholder - real value comes from PolicyEngine
    learningProgress,
  };
}
```

---

### Fix 6: Configure Logging Levels

**Files to Modify:**
- `src/utils/logger.ts`
- All files with `console.log`

**Implementation:**

```typescript
// In logger.ts - Add environment-based log levels
const LOG_LEVEL = process.env.LOG_LEVEL || (
  process.env.NODE_ENV === 'production' ? 'warn' : 'debug'
);

// Replace console.log calls with logger
// Before:
console.log('Initializing RecommendationEngine...');

// After:
logger.info('Initializing RecommendationEngine...');
```

---

## Implementation Order

```
Week 1:
├── Day 1-2: Fix 1 (Session storage) - Critical for RL training
├── Day 3-4: Fix 2 (Deterministic profiles) - Critical for recommendations
└── Day 5: Fix 3 (Deprecate stubs) - API clarity

Week 2:
├── Day 1-2: Fix 4 (Type safety) - Code quality
├── Day 3: Fix 5 (Remove duplicate Q-calc) - Maintenance
└── Day 4-5: Fix 6 (Logging) - Production readiness
```

---

## Verification Checklist

After implementing fixes, verify:

- [ ] `data/sessions.json` created when calling `/recommend`
- [ ] Feedback uses stored `stateBefore`, not estimated
- [ ] Same content always gets same emotional profile values
- [ ] Legacy endpoints return deprecation notices
- [ ] No `any` types in production route handlers
- [ ] No `console.log` in production builds

---

## What NOT to Fix (Validated as Working)

These were claimed as broken but are actually working:

| Claimed Issue | Reality |
|---------------|---------|
| Q-table not persisting | Working via FileStore → data/qtable.json |
| Progress always mock | Working via /progress/:userId routes |
| Experiences always empty | Working via /progress/:userId/experiences |
| Math.random in exploration | Intentional - standard RL practice |

---

*Generated by Sherlock Investigation Process*
