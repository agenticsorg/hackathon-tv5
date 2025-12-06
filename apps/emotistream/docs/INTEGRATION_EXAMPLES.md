# EmotiStream Integration Examples

## Overview

Complete examples for integrating the feedback collection and progress analytics system.

## Complete User Flow Example

### Step 1: User Starts Watching Content

```typescript
// Frontend: User clicks "Watch Now"
import { useWatchTracker } from '@/lib/hooks/use-watch-tracker';

function RecommendationCard({ content }) {
  const { startSession } = useWatchTracker('usr_123');
  const [sessionId, setSessionId] = useState<string | null>(null);

  const handleWatchNow = async () => {
    // Start watch session
    const response = await startSession(content.id, content.title);
    setSessionId(response.data.session.sessionId);

    // Store emotion before watching
    const emotionBefore = await detectEmotion(userId);
    localStorage.setItem('emotionBefore', JSON.stringify(emotionBefore));
    localStorage.setItem('desiredState', JSON.stringify(desiredState));

    // Navigate to watch page or open player
    router.push(`/watch/${content.id}?session=${response.data.session.sessionId}`);
  };

  return (
    <Card>
      <h3>{content.title}</h3>
      <Button onClick={handleWatchNow}>Watch Now</Button>
    </Card>
  );
}
```

**API Request:**
```http
POST /api/v1/watch/start
Content-Type: application/json

{
  "userId": "usr_123",
  "contentId": "content_456",
  "contentTitle": "The Matrix"
}
```

**API Response:**
```json
{
  "success": true,
  "data": {
    "session": {
      "sessionId": "watch_abc123",
      "userId": "usr_123",
      "contentId": "content_456",
      "contentTitle": "The Matrix",
      "startTime": "2025-12-06T10:00:00.000Z",
      "status": "active"
    }
  }
}
```

### Step 2: Track Watch Progress

```typescript
// Frontend: Watch page with pause/resume tracking
function WatchPage({ contentId, sessionId }) {
  const { pauseSession, resumeSession, endSession } = useWatchTracker();
  const [isPaused, setIsPaused] = useState(false);

  const handlePause = async () => {
    await pauseSession(sessionId);
    setIsPaused(true);
  };

  const handleResume = async () => {
    await resumeSession(sessionId);
    setIsPaused(false);
  };

  const handleEnd = async (completed: boolean) => {
    const response = await endSession(sessionId, completed);

    // Show feedback modal
    setShowFeedbackModal(true);
  };

  return (
    <VideoPlayer
      onPause={handlePause}
      onResume={handleResume}
      onEnd={() => handleEnd(true)}
      onUserExit={() => handleEnd(false)}
    />
  );
}
```

### Step 3: Collect Feedback

```typescript
// Frontend: Feedback modal appears after watching
import { FeedbackModal } from '@/components/feedback/feedback-modal';
import { useFeedback } from '@/lib/hooks/use-feedback';

function WatchPage() {
  const { submitFeedback } = useFeedback();
  const [showFeedbackModal, setShowFeedbackModal] = useState(false);
  const [feedbackResult, setFeedbackResult] = useState(null);

  const handleSubmitFeedback = async (data) => {
    // Get stored states
    const emotionBefore = JSON.parse(localStorage.getItem('emotionBefore'));
    const desiredState = JSON.parse(localStorage.getItem('desiredState'));

    // Submit feedback
    const result = await submitFeedback({
      userId: 'usr_123',
      contentId: content.id,
      contentTitle: content.title,
      sessionId: sessionId,
      emotionBefore: emotionBefore,
      emotionAfter: data.emotionAfter,
      desiredState: desiredState,
      starRating: data.starRating,
      completed: data.completed,
      totalDuration: content.duration
    });

    setFeedbackResult(result);

    // Show reward animation
    setTimeout(() => {
      setShowFeedbackModal(false);
      router.push('/progress'); // Navigate to progress page
    }, 3000);
  };

  return (
    <>
      <VideoPlayer />

      <FeedbackModal
        isOpen={showFeedbackModal}
        onClose={() => setShowFeedbackModal(false)}
        onSubmit={handleSubmitFeedback}
        contentTitle={content.title}
        emotionBefore={emotionBefore}
        desiredState={desiredState}
        result={feedbackResult}
      />
    </>
  );
}
```

**API Request:**
```http
POST /api/v1/feedback/submit
Content-Type: application/json

{
  "userId": "usr_123",
  "contentId": "content_456",
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
}
```

**API Response:**
```json
{
  "success": true,
  "data": {
    "feedbackId": "fbk_xyz789",
    "reward": {
      "value": 0.75,
      "components": {
        "emotionalAlignment": 0.82,
        "completionBonus": 1.0,
        "ratingBonus": 1.0
      },
      "explanation": "✨ Great choice! You moved significantly closer to your desired emotional state (+82%). You completed the content. You gave a high rating."
    },
    "emotionComparison": {
      "before": { "valence": -0.3, "arousal": -0.2, "stress": 0.6 },
      "after": { "valence": 0.5, "arousal": 0.3, "stress": 0.2 },
      "delta": { "valence": 0.8, "arousal": 0.5, "stress": -0.4 },
      "improvement": 0.875
    },
    "message": "🎉 Excellent choice! You felt significantly better!",
    "confetti": true
  }
}
```

### Step 4: View Progress Dashboard

```typescript
// Frontend: Progress dashboard page
import { useProgress } from '@/lib/hooks/use-progress';
import { MetricCard } from '@/components/progress/metric-card';
import { RewardTimeline } from '@/components/progress/reward-timeline';
import { ConvergenceIndicator } from '@/components/progress/convergence-indicator';
import { EmotionalJourney } from '@/components/progress/emotional-journey';
import { ExperienceList } from '@/components/progress/experience-list';

export default function ProgressPage() {
  const userId = 'usr_123';
  const { progress, isLoading } = useProgress(userId);

  if (isLoading) return <Loading />;

  return (
    <div className="container mx-auto py-8">
      <h1>Learning Progress Dashboard</h1>

      {/* Metric Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
        <MetricCard
          title="Total Experiences"
          value={progress.totalExperiences}
          subtitle="content watched"
        />
        <MetricCard
          title="Average Reward"
          value={progress.averageReward}
          trend={progress.rewardTrend}
        />
        <MetricCard
          title="Exploration Rate"
          value={progress.explorationRate}
          subtitle="discovering new content"
        />
        <MetricCard
          title="Convergence"
          value={progress.convergence.percentage + '%'}
          subtitle={progress.convergence.stage}
        />
      </div>

      {/* Reward Timeline */}
      <Card className="mb-8">
        <CardHeader>
          <CardTitle>Reward Timeline</CardTitle>
        </CardHeader>
        <CardContent>
          <RewardTimeline userId={userId} />
        </CardContent>
      </Card>

      {/* Convergence & Journey */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8 mb-8">
        <Card>
          <CardHeader>
            <CardTitle>Learning Progress</CardTitle>
          </CardHeader>
          <CardContent>
            <ConvergenceIndicator userId={userId} />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Emotional Journey</CardTitle>
          </CardHeader>
          <CardContent>
            <EmotionalJourney userId={userId} />
          </CardContent>
        </Card>
      </div>

      {/* Recent Experiences */}
      <Card>
        <CardHeader>
          <CardTitle>Recent Experiences</CardTitle>
        </CardHeader>
        <CardContent>
          <ExperienceList userId={userId} />
        </CardContent>
      </Card>
    </div>
  );
}
```

**API Requests:**
```http
GET /api/v1/progress/usr_123
GET /api/v1/progress/usr_123/rewards
GET /api/v1/progress/usr_123/convergence
GET /api/v1/progress/usr_123/journey?limit=50
GET /api/v1/progress/usr_123/experiences?limit=10
```

## Component Examples

### Feedback Modal Implementation

```tsx
// src/components/feedback/feedback-modal.tsx
import { useState } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { EmotionSelector } from './emotion-selector';
import { StarRating } from './star-rating';
import { EmotionComparison } from './emotion-comparison';
import { RewardDisplay } from './reward-display';

interface FeedbackModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (data: any) => Promise<void>;
  contentTitle: string;
  emotionBefore: EmotionalState;
  desiredState: EmotionalState;
  result?: any;
}

export function FeedbackModal({
  isOpen,
  onClose,
  onSubmit,
  contentTitle,
  emotionBefore,
  desiredState,
  result
}: FeedbackModalProps) {
  const [emotionAfter, setEmotionAfter] = useState<EmotionalState | null>(null);
  const [starRating, setStarRating] = useState(3);
  const [completed, setCompleted] = useState(true);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSubmit = async () => {
    if (!emotionAfter) return;

    setIsSubmitting(true);
    await onSubmit({
      emotionAfter,
      starRating,
      completed
    });
    setIsSubmitting(false);
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>How was {contentTitle}?</DialogTitle>
        </DialogHeader>

        {!result ? (
          <div className="space-y-6">
            {/* Before/After Comparison */}
            <div>
              <h3 className="text-sm font-medium mb-2">Emotional Journey</h3>
              <EmotionComparison
                before={emotionBefore}
                after={emotionAfter || emotionBefore}
                showDelta={false}
              />
            </div>

            {/* After Emotion Input */}
            <div>
              <h3 className="text-sm font-medium mb-2">How do you feel now?</h3>
              <EmotionSelector
                value={emotionAfter}
                onChange={setEmotionAfter}
              />
            </div>

            {/* Star Rating */}
            <div>
              <h3 className="text-sm font-medium mb-2">How would you rate it?</h3>
              <StarRating
                value={starRating}
                onChange={setStarRating}
                size="lg"
              />
            </div>

            {/* Completion Checkbox */}
            <div className="flex items-center space-x-2">
              <Checkbox
                id="completed"
                checked={completed}
                onCheckedChange={setCompleted}
              />
              <label htmlFor="completed" className="text-sm">
                Did you complete it?
              </label>
            </div>

            {/* Submit Button */}
            <Button
              onClick={handleSubmit}
              disabled={!emotionAfter || isSubmitting}
              className="w-full"
            >
              {isSubmitting ? 'Submitting...' : 'Submit Feedback'}
            </Button>
          </div>
        ) : (
          <div className="space-y-6">
            {/* Reward Display */}
            <RewardDisplay
              reward={result.data.reward.value}
              explanation={result.data.reward.explanation}
              showConfetti={result.data.confetti}
            />

            {/* Emotion Comparison */}
            <EmotionComparison
              before={result.data.emotionComparison.before}
              after={result.data.emotionComparison.after}
              showDelta={true}
            />

            {/* Close Button */}
            <Button onClick={onClose} className="w-full">
              View Progress Dashboard
            </Button>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
```

### Reward Timeline Chart

```tsx
// src/components/progress/reward-timeline.tsx
import { useEffect, useState } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from 'recharts';

interface RewardTimelineProps {
  userId: string;
}

export function RewardTimeline({ userId }: RewardTimelineProps) {
  const [data, setData] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetch(`/api/v1/progress/${userId}/rewards`)
      .then(res => res.json())
      .then(result => {
        const chartData = result.data.timeline.map((point, index) => ({
          experience: point.experienceNumber,
          reward: point.reward,
          trendLine: result.data.trendLine[index],
          contentTitle: point.contentTitle
        }));
        setData(chartData);
        setLoading(false);
      });
  }, [userId]);

  if (loading) return <div>Loading...</div>;

  return (
    <ResponsiveContainer width="100%" height={300}>
      <LineChart data={data}>
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis
          dataKey="experience"
          label={{ value: 'Experience Number', position: 'insideBottom', offset: -5 }}
        />
        <YAxis
          domain={[-1, 1]}
          label={{ value: 'Reward', angle: -90, position: 'insideLeft' }}
        />
        <Tooltip
          content={({ active, payload }) => {
            if (!active || !payload || !payload.length) return null;
            const data = payload[0].payload;
            return (
              <div className="bg-white p-3 border rounded shadow">
                <p className="font-semibold">{data.contentTitle}</p>
                <p className="text-sm">Experience #{data.experience}</p>
                <p className="text-sm">Reward: {data.reward.toFixed(3)}</p>
              </div>
            );
          }}
        />
        <Legend />
        <Line
          type="monotone"
          dataKey="reward"
          stroke="#10b981"
          strokeWidth={2}
          dot={{ r: 4 }}
          name="Actual Reward"
        />
        <Line
          type="monotone"
          dataKey="trendLine"
          stroke="#3b82f6"
          strokeWidth={2}
          strokeDasharray="5 5"
          dot={false}
          name="Trend"
        />
      </LineChart>
    </ResponsiveContainer>
  );
}
```

### Emotional Journey Scatter Plot

```tsx
// src/components/progress/emotional-journey.tsx
import { useEffect, useState } from 'react';
import { ScatterChart, Scatter, XAxis, YAxis, ZAxis, Tooltip, ResponsiveContainer, Cell } from 'recharts';

interface EmotionalJourneyProps {
  userId: string;
}

export function EmotionalJourney({ userId }: EmotionalJourneyProps) {
  const [data, setData] = useState([]);

  useEffect(() => {
    fetch(`/api/v1/progress/${userId}/journey?limit=50`)
      .then(res => res.json())
      .then(result => {
        const chartData = result.data.journey.map(point => ({
          x: point.emotionAfter.valence,
          y: point.emotionAfter.arousal,
          z: point.emotionAfter.stress,
          experienceNumber: point.experienceNumber,
          contentTitle: point.contentTitle,
          reward: point.reward,
          quadrant: point.quadrant
        }));
        setData(chartData);
      });
  }, [userId]);

  const getColor = (stress: number) => {
    if (stress < 0.3) return '#10b981'; // green (low stress)
    if (stress < 0.6) return '#fbbf24'; // yellow (medium)
    return '#ef4444'; // red (high stress)
  };

  return (
    <ResponsiveContainer width="100%" height={400}>
      <ScatterChart>
        <XAxis
          type="number"
          dataKey="x"
          domain={[-1, 1]}
          label={{ value: 'Valence (Negative ← → Positive)', position: 'insideBottom', offset: -5 }}
        />
        <YAxis
          type="number"
          dataKey="y"
          domain={[-1, 1]}
          label={{ value: 'Arousal (Calm ← → Excited)', angle: -90, position: 'insideLeft' }}
        />
        <ZAxis type="number" dataKey="experienceNumber" range={[50, 200]} />
        <Tooltip
          content={({ active, payload }) => {
            if (!active || !payload || !payload.length) return null;
            const data = payload[0].payload;
            return (
              <div className="bg-white p-3 border rounded shadow">
                <p className="font-semibold">{data.contentTitle}</p>
                <p className="text-sm">Experience #{data.experienceNumber}</p>
                <p className="text-sm">Quadrant: {data.quadrant}</p>
                <p className="text-sm">Reward: {data.reward.toFixed(3)}</p>
              </div>
            );
          }}
        />

        {/* Quadrant Labels */}
        <text x="75%" y="25%" fill="#666" fontSize="12">Excited</text>
        <text x="25%" y="25%" fill="#666" fontSize="12">Stressed</text>
        <text x="25%" y="75%" fill="#666" fontSize="12">Sad</text>
        <text x="75%" y="75%" fill="#666" fontSize="12">Calm</text>

        <Scatter data={data}>
          {data.map((entry, index) => (
            <Cell key={index} fill={getColor(entry.z)} />
          ))}
        </Scatter>
      </ScatterChart>
    </ResponsiveContainer>
  );
}
```

## Testing Examples

### Unit Test: Reward Calculator

```typescript
// tests/unit/services/reward-calculator.test.ts
import { RewardCalculator } from '../../../src/services/reward-calculator';

describe('RewardCalculator', () => {
  let calculator: RewardCalculator;

  beforeEach(() => {
    calculator = new RewardCalculator();
  });

  it('should calculate high reward for perfect emotional alignment', () => {
    const before = { valence: -0.5, arousal: 0.0, stress: 0.7 };
    const after = { valence: 0.6, arousal: -0.2, stress: 0.1 };
    const desired = { valence: 0.6, arousal: -0.2, stress: 0.1 };

    const result = calculator.calculate(before, after, desired, true, 5, 7200000, 8160000);

    expect(result.reward).toBeGreaterThan(0.8);
    expect(result.components.emotionalAlignment).toBeGreaterThan(0.7);
    expect(result.components.completionBonus).toBe(1.0);
    expect(result.components.ratingBonus).toBe(1.0);
  });

  it('should penalize incomplete watching', () => {
    const before = { valence: 0.0, arousal: 0.0, stress: 0.5 };
    const after = { valence: 0.5, arousal: 0.0, stress: 0.3 };
    const desired = { valence: 0.5, arousal: 0.0, stress: 0.2 };

    const result = calculator.calculate(before, after, desired, false, 3, 1000000, 8160000);

    expect(result.components.completionBonus).toBeLessThan(0);
  });
});
```

### Integration Test: Feedback Submission

```typescript
// tests/integration/api/feedback-enhanced.test.ts
import request from 'supertest';
import { app } from '../../../src/api/index';

describe('POST /api/v1/feedback/submit', () => {
  it('should submit feedback and return reward', async () => {
    // Start watch session
    const sessionResponse = await request(app)
      .post('/api/v1/watch/start')
      .send({
        userId: 'test_user',
        contentId: 'test_content',
        contentTitle: 'Test Movie'
      });

    const sessionId = sessionResponse.body.data.session.sessionId;

    // Submit feedback
    const response = await request(app)
      .post('/api/v1/feedback/submit')
      .send({
        userId: 'test_user',
        contentId: 'test_content',
        contentTitle: 'Test Movie',
        sessionId,
        emotionBefore: { valence: -0.3, arousal: -0.2, stress: 0.6 },
        emotionAfter: { valence: 0.5, arousal: 0.3, stress: 0.2 },
        desiredState: { valence: 0.6, arousal: 0.0, stress: 0.1 },
        starRating: 5,
        completed: true,
        totalDuration: 8160000
      });

    expect(response.status).toBe(201);
    expect(response.body.success).toBe(true);
    expect(response.body.data.reward.value).toBeGreaterThan(0);
    expect(response.body.data.emotionComparison).toBeDefined();
  });
});
```

## Error Handling

```typescript
// Frontend error handling example
async function handleFeedbackSubmit(data) {
  try {
    const result = await submitFeedback(data);
    return result;
  } catch (error) {
    if (error.response) {
      // API error
      const { code, message } = error.response.data.error;

      if (code === 'E003') {
        toast.error('Invalid feedback data. Please check your inputs.');
      } else if (code === 'E004') {
        toast.error('Watch session not found. Please restart watching.');
      } else {
        toast.error('Something went wrong. Please try again.');
      }
    } else {
      // Network error
      toast.error('Network error. Please check your connection.');
    }
  }
}
```

## Performance Optimization

```typescript
// Use SWR for caching and revalidation
import useSWR from 'swr';

function useProgress(userId: string) {
  const { data, error, mutate } = useSWR(
    `/api/v1/progress/${userId}`,
    fetcher,
    {
      revalidateOnFocus: false,
      revalidateOnReconnect: true,
      dedupingInterval: 60000, // 1 minute
    }
  );

  return {
    progress: data,
    isLoading: !error && !data,
    isError: error,
    refresh: mutate
  };
}
```

## Deployment Checklist

- [ ] Environment variables configured
- [ ] Database initialized
- [ ] API endpoints tested
- [ ] Frontend components tested
- [ ] Error handling implemented
- [ ] Performance optimized
- [ ] Accessibility verified
- [ ] Documentation complete
- [ ] Analytics integrated
- [ ] Monitoring setup
