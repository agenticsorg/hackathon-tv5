# Recommendation UI Components

Netflix-style recommendation display for EmotiStream with RL-powered predictions.

## Components

### RecommendationCard
Individual content card with thumbnail, score, and predicted outcome.

**Features:**
- Gradient thumbnail based on category
- Score badge (0-100%)
- Exploration indicator
- Hover: scale animation, show outcome predictor
- Confidence indicator bar
- Watch Now and Details buttons

**Props:**
```typescript
interface RecommendationCardProps {
  contentId: string;
  title: string;
  category: string;
  duration: number; // in minutes
  combinedScore: number; // 0-1
  predictedOutcome: PredictedOutcome;
  reasoning: string;
  isExploration: boolean;
  onWatch: () => void;
  onDetails?: () => void;
  currentState?: EmotionalState;
}
```

### RecommendationGrid
Horizontal scrolling container for recommendation cards.

**Features:**
- Netflix-style horizontal scroll
- Desktop: 3-5 cards visible, scroll buttons
- Mobile: 1-2 cards, swipe-friendly
- Skeleton loaders during fetch
- Empty state with call-to-action
- Staggered entrance animations (50ms per card)

**Props:**
```typescript
interface RecommendationGridProps {
  recommendations?: Recommendation[];
  isLoading?: boolean;
  error?: string | null;
  currentState?: EmotionalState;
  onWatch: (contentId: string) => void;
  onSave?: (contentId: string) => void;
}
```

### RecommendationDetail
Full-screen modal with detailed recommendation information.

**Features:**
- Full reasoning explanation
- Q-value learning history
- Predicted emotional transition visualization
- Before → After mood comparison
- Confidence meter with explanation
- Watch Now / Save for Later actions

**Props:**
```typescript
interface RecommendationDetailProps {
  isOpen: boolean;
  onClose: () => void;
  recommendation: Recommendation;
  currentState?: EmotionalState;
  onWatch: () => void;
  onSave?: () => void;
}
```

### OutcomePredictor
Visual component showing expected emotional changes.

**Features:**
- Current State → Content → Predicted State flow
- Animated progress bars (valence, arousal, stress)
- Color-coded changes (green = positive, red = negative)
- Trend icons (up/down/stable)
- Compact mode for card display

**Props:**
```typescript
interface OutcomePredictorProps {
  currentState: EmotionalState;
  predictedOutcome: PredictedOutcome;
  compact?: boolean; // Compact view for cards
}
```

### RecommendationSkeleton
Loading placeholder matching card layout.

**Features:**
- Shimmer animation
- Exact card dimensions
- 3-5 skeletons in grid

## Animations

### Card Entrance
```typescript
initial={{ opacity: 0, y: 20 }}
animate={{ opacity: 1, y: 0 }}
transition={{ delay: index * 0.05 }}
```

### Card Hover
```typescript
whileHover={{ scale: 1.05 }}
transition={{ duration: 0.2 }}
```

### Confidence Bar
```typescript
initial={{ width: 0 }}
animate={{ width: `${confidence * 100}%` }}
transition={{ duration: 0.8, delay: 0.2 }}
```

### Outcome Predictor
```typescript
// Bars animate from current to predicted state
initial={{ width: `${currentValue * 100}%` }}
animate={{ width: `${predictedValue * 100}%` }}
transition={{ duration: 0.8, ease: "easeInOut" }}
```

## Integration Example

```typescript
import { RecommendationGrid } from '@/components/recommendations';
import { useRecommendations } from '@/hooks/use-recommendations';

function Dashboard() {
  const currentState = {
    valence: 0.3,
    arousal: 0.7,
    stress: 0.8
  };

  const desiredState = {
    valence: 0.8,
    arousal: 0.5,
    stress: 0.2
  };

  const { recommendations, isLoading, error, fetchRecommendations } = useRecommendations({
    currentState,
    desiredState,
    userId: 'user123',
    autoFetch: true
  });

  const handleWatch = (contentId: string) => {
    console.log('Watch:', contentId);
    // Navigate to player or start playback
  };

  const handleSave = (contentId: string) => {
    console.log('Save:', contentId);
    // Add to watchlist
  };

  return (
    <div className="space-y-8">
      {/* Emotion input components */}

      <RecommendationGrid
        recommendations={recommendations}
        isLoading={isLoading}
        error={error}
        currentState={currentState}
        onWatch={handleWatch}
        onSave={handleSave}
      />
    </div>
  );
}
```

## Category Styling

Categories have unique gradients and icons:

| Category | Gradient | Icon |
|----------|----------|------|
| meditation | purple → indigo | 🧘 |
| movie | red → pink | 🎬 |
| music | green → teal | 🎵 |
| series | blue → cyan | 📺 |
| documentary | amber → orange | 📚 |
| short | rose → fuchsia | 🎥 |
| exercise | emerald → lime | 💪 |
| podcast | violet → purple | 🎙️ |

## Score Color Coding

- **Green (≥80%)**: High match, confident recommendation
- **Yellow (60-79%)**: Moderate match, likely suitable
- **Red (<60%)**: Lower match, exploratory suggestion

## Responsive Design

### Desktop (md+)
- 3-5 cards visible
- Scroll buttons (left/right)
- Hover effects enabled
- Full detail view

### Mobile (<md)
- 1-2 cards visible
- Swipe gestures
- Tap for details
- Simplified outcome view

## Accessibility

- Proper ARIA labels
- Keyboard navigation
- Focus indicators
- Screen reader friendly
- Color contrast compliance

## Performance

- Virtual scrolling for large lists
- Image lazy loading
- Memoized card renders
- Optimized animations (GPU-accelerated)
- Debounced scroll events
