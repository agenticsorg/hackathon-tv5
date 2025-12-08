# Frontend Components Specification

## Overview

This document specifies the React/Next.js components for the EmotiStream feedback collection and progress dashboard interface.

## Tech Stack

- **Framework**: Next.js 14 (App Router)
- **UI Library**: shadcn/ui (Radix UI + Tailwind CSS)
- **Charts**: Recharts
- **Animations**: Framer Motion
- **Icons**: Lucide React
- **State**: React Context + Hooks

## Project Structure

```
src/
├── app/
│   ├── (app)/
│   │   ├── progress/
│   │   │   └── page.tsx              # Progress dashboard
│   │   └── watch/
│   │       └── [contentId]/
│   │           └── page.tsx          # Watch page
│   └── layout.tsx
│
├── components/
│   ├── feedback/
│   │   ├── feedback-modal.tsx        # Main feedback dialog
│   │   ├── emotion-comparison.tsx    # Before/after emotions
│   │   ├── star-rating.tsx           # 5-star rating input
│   │   ├── reward-display.tsx        # Animated reward
│   │   └── emotion-selector.tsx      # Emotion input (before/after)
│   │
│   ├── progress/
│   │   ├── metric-card.tsx           # Hero metric display
│   │   ├── reward-timeline.tsx       # Line chart
│   │   ├── convergence-indicator.tsx # Progress bar
│   │   ├── emotional-journey.tsx     # Scatter plot
│   │   └── experience-list.tsx       # Recent experiences
│   │
│   ├── recommendations/
│   │   └── recommendation-card.tsx   # Content card with "Watch Now"
│   │
│   └── ui/                           # shadcn/ui components
│       ├── dialog.tsx
│       ├── button.tsx
│       ├── card.tsx
│       ├── badge.tsx
│       └── progress.tsx
│
├── lib/
│   ├── hooks/
│   │   ├── use-watch-tracker.ts      # Watch session management
│   │   ├── use-feedback.ts           # Feedback submission
│   │   └── use-progress.ts           # Progress data fetching
│   │
│   ├── api/
│   │   └── client.ts                 # API client
│   │
│   └── utils/
│       ├── emotion-colors.ts         # Color mapping
│       └── confetti.ts               # Confetti animation
│
└── types/
    └── index.ts                      # TypeScript types
```

## Part 1: Feedback Collection

### 1. Feedback Modal (`src/components/feedback/feedback-modal.tsx`)

**Component:**
```tsx
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { EmotionComparison } from './emotion-comparison';
import { StarRating } from './star-rating';
import { RewardDisplay } from './reward-display';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { useFeedback } from '@/lib/hooks/use-feedback';

interface FeedbackModalProps {
  isOpen: boolean;
  onClose: () => void;
  contentId: string;
  contentTitle: string;
  sessionId: string;
  emotionBefore: EmotionalState;
  desiredState: EmotionalState;
}

export function FeedbackModal({
  isOpen,
  onClose,
  contentId,
  contentTitle,
  sessionId,
  emotionBefore,
  desiredState
}: FeedbackModalProps) {
  // Component implementation
}
```

**Features:**
- Opens after "Watch Now" click
- Sections:
  - Title: "How was [Content Title]?"
  - Before/After emotion comparison
  - Post-viewing emotion input
  - Star rating (1-5)
  - "Did you complete it?" checkbox
  - Submit button
  - Reward preview (shown after submit)

**User Flow:**
1. User clicks "Watch Now" → Watch session starts
2. User watches content
3. User returns → Feedback modal opens automatically
4. User inputs emotions and ratings
5. User clicks "Submit"
6. Reward calculation displayed with animation
7. Modal closes after 3 seconds (or user closes)

### 2. Emotion Comparison (`src/components/feedback/emotion-comparison.tsx`)

**Component:**
```tsx
interface EmotionComparisonProps {
  before: EmotionalState;
  after: EmotionalState;
  showDelta?: boolean;
}

export function EmotionComparison({
  before,
  after,
  showDelta = true
}: EmotionComparisonProps) {
  // Render side-by-side mood rings
  // Show arrow indicating transition
  // Display delta values if showDelta=true
}
```

**Visual Design:**
```
┌─────────────────────────────────────────────────┐
│  Before              →              After       │
│ ┌────────┐         ──→          ┌────────┐    │
│ │ 😔     │                      │ 😊     │    │
│ │Mood    │                      │Mood    │    │
│ │Ring    │                      │Ring    │    │
│ └────────┘                      └────────┘    │
│                                                 │
│  Valence:  -0.3  →  0.5  (+0.8) ✓             │
│  Arousal:  -0.2  →  0.3  (+0.5) ✓             │
│  Stress:    0.6  →  0.2  (-0.4) ✓             │
└─────────────────────────────────────────────────┘
```

**Implementation:**
- Mood rings as circular gradient backgrounds
- Colors based on Russell's Circumplex:
  - Valence: Red (negative) → Green (positive)
  - Arousal: Blue (low) → Yellow (high)
- Animated path between states (SVG curve)
- Change indicators with checkmarks/crosses

### 3. Star Rating (`src/components/feedback/star-rating.tsx`)

**Component:**
```tsx
import { Star } from 'lucide-react';
import { motion } from 'framer-motion';

interface StarRatingProps {
  value: number;
  onChange: (value: number) => void;
  size?: 'sm' | 'md' | 'lg';
}

export function StarRating({
  value,
  onChange,
  size = 'md'
}: StarRatingProps) {
  // Interactive star rating with hover preview
}
```

**Features:**
- 5 clickable stars
- Hover preview (highlight stars on hover)
- Click to select rating
- Framer Motion scale animation on click
- Optional: Half-star support (0.5 increments)

**Visual:**
```
★ ★ ★ ★ ☆  (4 stars selected)
```

### 4. Reward Display (`src/components/feedback/reward-display.tsx`)

**Component:**
```tsx
import { motion, AnimatePresence } from 'framer-motion';
import Confetti from 'react-confetti';

interface RewardDisplayProps {
  reward: number;
  explanation: string;
  showConfetti?: boolean;
}

export function RewardDisplay({
  reward,
  explanation,
  showConfetti = false
}: RewardDisplayProps) {
  // Animated reward display
}
```

**Features:**
- Animated number counting up (0 → reward)
- Color coding:
  - Green (positive): reward > 0.5
  - Yellow (neutral): 0 < reward < 0.5
  - Red (negative): reward < 0
- Confetti animation for high rewards (>0.7)
- Message: "Great choice! You felt 45% more relaxed"
- Components breakdown display (optional collapse)

**Visual:**
```
┌────────────────────────────────────┐
│   Reward: +0.75  🎉                │
│                                     │
│   ✨ Great choice!                 │
│   You felt significantly better!   │
│                                     │
│   Components:                       │
│   ■ Emotional: 82%                 │
│   ■ Completion: 100%               │
│   ■ Rating: 100%                   │
└────────────────────────────────────┘
```

### 5. Watch Tracker Hook (`src/lib/hooks/use-watch-tracker.ts`)

**Hook:**
```tsx
import { useState, useEffect, useCallback } from 'react';

export function useWatchTracker(userId: string) {
  const startSession = useCallback(async (contentId: string, contentTitle: string) => {
    // POST /api/v1/watch/start
  }, [userId]);

  const pauseSession = useCallback(async (sessionId: string) => {
    // POST /api/v1/watch/pause
  }, []);

  const resumeSession = useCallback(async (sessionId: string) => {
    // POST /api/v1/watch/resume
  }, []);

  const endSession = useCallback(async (sessionId: string, completed: boolean) => {
    // POST /api/v1/watch/end
  }, []);

  const getElapsedTime = useCallback((sessionId: string) => {
    // GET /api/v1/watch/:sessionId
  }, []);

  return {
    startSession,
    pauseSession,
    resumeSession,
    endSession,
    getElapsedTime
  };
}
```

## Part 2: Learning Progress Dashboard

### 6. Progress Page (`src/app/(app)/progress/page.tsx`)

**Layout:**
```
┌──────────────────────────────────────────────────────┐
│  Learning Progress Dashboard                         │
├──────────────────────────────────────────────────────┤
│                                                       │
│  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐             │
│  │Total │  │Avg   │  │Explor│  │Convg │  (Metrics)  │
│  │Exp   │  │Reward│  │Rate  │  │Score │             │
│  └──────┘  └──────┘  └──────┘  └──────┘             │
│                                                       │
│  ┌─────────────────────────────────────────┐         │
│  │  Reward Timeline (Line Chart)           │         │
│  │                                          │         │
│  │  [Chart with trend line]                │         │
│  └─────────────────────────────────────────┘         │
│                                                       │
│  ┌──────────────────┐  ┌────────────────────┐        │
│  │ Convergence      │  │ Emotional Journey  │        │
│  │ Progress Bar     │  │ Scatter Plot       │        │
│  │                  │  │                    │        │
│  └──────────────────┘  └────────────────────┘        │
│                                                       │
│  ┌─────────────────────────────────────────┐         │
│  │  Recent Experiences (List)              │         │
│  │                                          │         │
│  │  [Experience items]                     │         │
│  └─────────────────────────────────────────┘         │
│                                                       │
└──────────────────────────────────────────────────────┘
```

### 7. Metric Card (`src/components/progress/metric-card.tsx`)

**Component:**
```tsx
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { TrendingUp, TrendingDown, Minus } from 'lucide-react';

interface MetricCardProps {
  title: string;
  value: string | number;
  subtitle?: string;
  trend?: 'up' | 'down' | 'stable';
  icon?: React.ReactNode;
}
```

**Display Examples:**
- Total Experiences: "42"
- Average Reward: "0.682" (with ↑ trending up)
- Exploration Rate: "12%" (with 🔍 icon)
- Convergence Score: "73%" (with progress bar)

### 8. Reward Timeline (`src/components/progress/reward-timeline.tsx`)

**Component:**
```tsx
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';

interface RewardTimelineProps {
  data: RewardTimelinePoint[];
}
```

**Features:**
- X-axis: Experience number
- Y-axis: Reward (0-1)
- Main line: Actual rewards
- Trend line: Moving average overlay
- Hover tooltips: Content title, reward, completed status
- Color gradient: Green (high) → Yellow (medium) → Red (low)

### 9. Convergence Indicator (`src/components/progress/convergence-indicator.tsx`)

**Component:**
```tsx
import { Progress } from '@/components/ui/progress';

interface ConvergenceIndicatorProps {
  score: number; // 0-100
  stage: 'exploring' | 'learning' | 'confident';
  explanation: string;
}
```

**Visual:**
```
┌─────────────────────────────────────────┐
│  Convergence: 73% confident             │
│                                          │
│  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░░░  73%       │
│  Still exploring → Learning → Confident  │
│                      ^                   │
│  Good progress! The system is           │
│  developing a solid understanding...     │
└─────────────────────────────────────────┘
```

**Colors:**
- Exploring (0-30): #fbbf24 (yellow)
- Learning (30-70): #3b82f6 (blue)
- Confident (70-100): #10b981 (green)

### 10. Emotional Journey (`src/components/progress/emotional-journey.tsx`)

**Component:**
```tsx
import { ScatterChart, Scatter, XAxis, YAxis, ZAxis, Tooltip, ResponsiveContainer } from 'recharts';

interface EmotionalJourneyProps {
  journey: EmotionalJourneyPoint[];
}
```

**Features:**
- Scatter plot: X=valence, Y=arousal
- Points sized by experience number (larger = more recent)
- Points colored by stress level (gradient)
- Connect points with lines (time order)
- Hover: Content title, reward, emotions
- Quadrant labels:
  - Top-right: "Excited"
  - Top-left: "Stressed"
  - Bottom-left: "Sad"
  - Bottom-right: "Calm"

**Visual:**
```
      Arousal (1)
          │
Stressed  │  Excited
    ──────┼──────── Valence
    Sad   │  Calm
          │
      (-1)
```

### 11. Experience List (`src/components/progress/experience-list.tsx`)

**Component:**
```tsx
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { EmotionComparison } from '../feedback/emotion-comparison';

interface ExperienceListProps {
  experiences: ExperienceListItem[];
  onLoadMore?: () => void;
}
```

**Item Layout:**
```
┌──────────────────────────────────────────┐
│  #42 • The Matrix • 2 hours ago          │
│  ⭐⭐⭐⭐⭐  Completed  Reward: +0.75     │
│                                           │
│  😔 → 😊  Valence: +0.8  ✓               │
│           Arousal: +0.5  ✓               │
│           Stress: -0.4   ✓               │
│                                           │
│  [Expand for details]                    │
└──────────────────────────────────────────┘
```

**Features:**
- Experience number, title, timestamp
- Star rating, completion badge, reward
- Emotion change summary (expandable)
- Infinite scroll (load more)
- Click to expand full details

## Design Guidelines

### Colors

**Emotions:**
- Valence: `red-500` (negative) → `green-500` (positive)
- Arousal: `blue-500` (low) → `yellow-500` (high)
- Stress: `gray-500` (low) → `orange-500` (high)

**Convergence:**
- Exploring: `yellow-400` (#fbbf24)
- Learning: `blue-500` (#3b82f6)
- Confident: `green-500` (#10b981)

**Rewards:**
- High (>0.5): `green-600`
- Medium (0-0.5): `yellow-600`
- Low (<0): `red-600`

### Animations

**Framer Motion Variants:**
```tsx
const fadeIn = {
  initial: { opacity: 0, y: 20 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -20 }
};

const scaleIn = {
  initial: { scale: 0.8, opacity: 0 },
  animate: { scale: 1, opacity: 1 },
  transition: { type: 'spring', stiffness: 300 }
};

const countUp = (value: number) => ({
  initial: { value: 0 },
  animate: { value },
  transition: { duration: 1.5, ease: 'easeOut' }
});
```

### Responsiveness

- **Mobile**: Stack cards vertically, hide secondary metrics
- **Tablet**: 2-column grid for metric cards
- **Desktop**: Full layout as specified

## API Integration

### Hooks

**use-feedback.ts:**
```tsx
export function useFeedback() {
  const submitFeedback = async (data: FeedbackSubmission) => {
    return await fetch('/api/v1/feedback/submit', {
      method: 'POST',
      body: JSON.stringify(data)
    }).then(r => r.json());
  };

  return { submitFeedback };
}
```

**use-progress.ts:**
```tsx
export function useProgress(userId: string) {
  const { data, isLoading, error } = useSWR(
    `/api/v1/progress/${userId}`,
    fetcher
  );

  return { progress: data, isLoading, error };
}
```

## State Management

Use React Context for global state:

```tsx
interface AppState {
  userId: string;
  currentSession: WatchSession | null;
  feedback: FeedbackRecord[];
}

export const AppContext = createContext<AppState>(/* ... */);
```

## Testing

**Unit Tests:**
- Component rendering
- User interactions (clicks, inputs)
- Prop validation
- Animation timing

**Integration Tests:**
- API calls
- State updates
- Navigation flows
- Error handling

**E2E Tests (Playwright):**
- Complete feedback submission flow
- Progress dashboard loading
- Chart interactions
- Mobile responsiveness

## Accessibility

- **ARIA labels**: All interactive elements
- **Keyboard navigation**: Tab order, Enter/Space for actions
- **Screen readers**: Announce reward changes, convergence updates
- **Color contrast**: WCAG AA minimum (4.5:1)
- **Focus indicators**: Visible focus rings

## Next Steps

1. Setup Next.js project with shadcn/ui
2. Implement feedback modal flow
3. Build progress dashboard
4. Add Recharts visualizations
5. Integrate with backend API
6. Add animations and polish
7. Write comprehensive tests
8. Deploy to production
