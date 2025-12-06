# Recommendation Component Tree

Visual hierarchy and data flow of the recommendation system.

```
EmotionDashboard
│
├─ EmotionInput
│  └─ onAnalyzed(currentState) ──────┐
│                                     │
├─ DesiredStateSelector               │
│  └─ onSelect(desiredState) ─────┐  │
│                                  │  │
└─ RecommendationGrid ◄──────────[useRecommendations(current, desired)]
   │                                  │
   ├─ Props:                          │
   │  ├─ recommendations[]            │
   │  ├─ isLoading                    │
   │  ├─ error                         │
   │  ├─ currentState                  │
   │  ├─ onWatch(contentId)            │
   │  └─ onSave(contentId)             │
   │                                   │
   ├─ Header                           │
   │  ├─ Title: "Personalized For You"│
   │  ├─ Count Badge                   │
   │  └─ Scroll Controls (Desktop)    │
   │     ├─ ChevronLeft Button         │
   │     └─ ChevronRight Button        │
   │                                   │
   ├─ Horizontal Scroll Container     │
   │  │                                │
   │  ├─ RecommendationCard (1) ──────┴──[onClick Details]──┐
   │  │  │                                                    │
   │  │  ├─ Thumbnail (Gradient)                             │
   │  │  │  ├─ Category Icon (Emoji)                         │
   │  │  │  ├─ Score Badge (Top Left)                        │
   │  │  │  ├─ Exploration Badge (Top Right)                 │
   │  │  │  └─ Hover Overlay                                 │
   │  │  │     ├─ Watch Button ──[onClick]─► onWatch()       │
   │  │  │     └─ Info Button ──[onClick]──► setSelected()   │
   │  │  │                                                    │
   │  │  ├─ Content Info                                     │
   │  │  │  ├─ Title                                          │
   │  │  │  ├─ Category Badge                                │
   │  │  │  └─ Duration                                       │
   │  │  │                                                    │
   │  │  ├─ OutcomePredictor (Compact) [Hover Only]         │
   │  │  │  ├─ Mood Change Indicator                         │
   │  │  │  ├─ Energy Change Indicator                       │
   │  │  │  └─ Stress Change Indicator                       │
   │  │  │                                                    │
   │  │  ├─ Reasoning Text [Hover Only]                      │
   │  │  │                                                    │
   │  │  ├─ Watch Button (Always Visible)                    │
   │  │  │                                                    │
   │  │  └─ Confidence Bar (Bottom)                          │
   │  │     └─ Gradient Fill (0-100%)                        │
   │  │                                                       │
   │  ├─ RecommendationCard (2)                              │
   │  ├─ RecommendationCard (3)                              │
   │  ├─ RecommendationCard (4)                              │
   │  └─ RecommendationCard (5)                              │
   │                                                          │
   ├─ Loading State                                          │
   │  └─ RecommendationSkeletonGrid                          │
   │     ├─ RecommendationSkeleton (1)                       │
   │     ├─ RecommendationSkeleton (2)                       │
   │     ├─ RecommendationSkeleton (3)                       │
   │     ├─ RecommendationSkeleton (4)                       │
   │     └─ RecommendationSkeleton (5)                       │
   │                                                          │
   ├─ Empty State                                            │
   │  ├─ Sparkles Icon                                       │
   │  ├─ Heading: "Ready to Discover Content"               │
   │  └─ Description: "Describe your mood..."                │
   │                                                          │
   └─ Error State                                            │
      ├─ Error Icon                                          │
      └─ Error Message                                       │
                                                              │
                                                              │
RecommendationDetail Modal ◄──────────────────────────────────┘
│
├─ Props:
│  ├─ isOpen (boolean)
│  ├─ onClose()
│  ├─ recommendation (full object)
│  ├─ currentState
│  ├─ onWatch()
│  └─ onSave()
│
├─ Backdrop (Click to Close)
│
└─ Modal Container (Spring Animation)
   │
   ├─ Header
   │  ├─ Thumbnail (Large, Gradient)
   │  ├─ Close Button (Top Right)
   │  └─ Score Badge (Bottom Left)
   │
   ├─ Scrollable Content
   │  │
   │  ├─ Title Section
   │  │  ├─ Title (h2)
   │  │  ├─ Category Badge
   │  │  ├─ Duration
   │  │  └─ Exploration Badge
   │  │
   │  ├─ Why This Content Section
   │  │  ├─ Brain Icon
   │  │  ├─ Section Title
   │  │  └─ Reasoning Text (Full)
   │  │
   │  ├─ Expected Emotional Impact
   │  │  ├─ Target Icon
   │  │  ├─ Section Title
   │  │  └─ OutcomePredictor (Detailed)
   │  │     ├─ Valence (Mood)
   │  │     │  ├─ Current Value
   │  │     │  ├─ Progress Bar (Animated)
   │  │     │  ├─ Predicted Value
   │  │     │  └─ Change Indicator
   │  │     │
   │  │     ├─ Arousal (Energy)
   │  │     │  ├─ Current Value
   │  │     │  ├─ Progress Bar (Animated)
   │  │     │  ├─ Predicted Value
   │  │     │  └─ Change Indicator
   │  │     │
   │  │     └─ Stress
   │  │        ├─ Current Value
   │  │        ├─ Progress Bar (Animated)
   │  │        ├─ Predicted Value
   │  │        └─ Change Indicator
   │  │
   │  ├─ Learning History [Optional]
   │  │  ├─ TrendingUp Icon
   │  │  ├─ Section Title
   │  │  ├─ Interaction Count
   │  │  └─ Q-Value Timeline
   │  │     ├─ Entry (Most Recent)
   │  │     ├─ Entry
   │  │     ├─ Entry
   │  │     ├─ Entry
   │  │     └─ Entry (Oldest Shown)
   │  │
   │  └─ Confidence Explanation
   │     ├─ Progress Bar (Animated)
   │     ├─ Percentage
   │     └─ Explanation Text
   │
   └─ Action Footer
      ├─ Watch Now Button (Primary)
      │  ├─ Play Icon
      │  └─ "Watch Now" Text
      │
      └─ Save Button [Optional]
         ├─ Bookmark Icon
         └─ "Save" Text
```

---

## Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│ User Interaction Layer                                      │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ Emotion Input                                               │
│ ┌─────────────┐                                             │
│ │ User types  │─► Gemini API ─► Emotion Analysis           │
│ │ description │                                             │
│ └─────────────┘                                             │
│                          │                                  │
│                          ▼                                  │
│                  currentState: {                            │
│                    valence: 0.3,                            │
│                    arousal: 0.7,                            │
│                    stress: 0.8                              │
│                  }                                          │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ Desired State Selector                                      │
│ ┌─────────────┐                                             │
│ │ User selects│─► Predefined or Custom                      │
│ │ goal        │                                             │
│ └─────────────┘                                             │
│                          │                                  │
│                          ▼                                  │
│                  desiredState: {                            │
│                    valence: 0.8,                            │
│                    arousal: 0.5,                            │
│                    stress: 0.2                              │
│                  }                                          │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ useRecommendations Hook                                     │
│                                                             │
│  useEffect(() => {                                          │
│    if (currentState && desiredState) {                      │
│      fetchRecommendations();                                │
│    }                                                        │
│  }, [currentState, desiredState]);                          │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ API Request                                                 │
│                                                             │
│  POST /api/recommend                                        │
│  {                                                          │
│    userId: "user123",                                       │
│    currentState: { valence, arousal, stress },              │
│    desiredState: { valence, arousal, stress },              │
│    limit: 10                                                │
│  }                                                          │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ Backend: Recommendation Engine (RL)                         │
│                                                             │
│  1. Calculate state distance                                │
│  2. Query Q-table for best actions                          │
│  3. Apply ε-greedy exploration                              │
│  4. Score content by Q-values                               │
│  5. Predict emotional outcomes                              │
│  6. Generate reasoning                                      │
│  7. Sort by combined score                                  │
│  8. Return top N recommendations                            │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ API Response                                                │
│                                                             │
│  {                                                          │
│    recommendations: [                                       │
│      {                                                      │
│        contentId: "med-123",                                │
│        title: "Calm Morning",                               │
│        category: "meditation",                              │
│        duration: 15,                                        │
│        combinedScore: 0.92,                                 │
│        predictedOutcome: {                                  │
│          expectedValence: 0.8,                              │
│          expectedArousal: 0.4,                              │
│          expectedStress: 0.2,                               │
│          confidence: 0.88                                   │
│        },                                                   │
│        reasoning: "...",                                    │
│        isExploration: false,                                │
│        qValueHistory: [...]                                 │
│      },                                                     │
│      // ... more recommendations                            │
│    ]                                                        │
│  }                                                          │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ React State Update                                          │
│                                                             │
│  setRecommendations(data.recommendations);                  │
│  setIsLoading(false);                                       │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ RecommendationGrid Render                                   │
│                                                             │
│  {recommendations.map((rec, i) => (                         │
│    <motion.div                                              │
│      initial={{ opacity: 0, y: 20 }}                       │
│      animate={{ opacity: 1, y: 0 }}                        │
│      transition={{ delay: i * 0.05 }}                      │
│    >                                                        │
│      <RecommendationCard                                    │
│        {...rec}                                             │
│        currentState={currentState}                          │
│        onWatch={() => handleWatch(rec.contentId)}           │
│      />                                                     │
│    </motion.div>                                            │
│  ))}                                                        │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ User Interaction                                            │
│                                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                  │
│  │ Hover    │  │ Click    │  │ Click    │                  │
│  │ Card     │  │ Watch    │  │ Info     │                  │
│  └──────────┘  └──────────┘  └──────────┘                  │
│       │             │              │                        │
│       ▼             ▼              ▼                        │
│  Show Details  Navigate to   Open Detail                   │
│  + Outcome     Player         Modal                         │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ Feedback Loop (Watch/Rate)                                  │
│                                                             │
│  POST /api/feedback                                         │
│  {                                                          │
│    userId: "user123",                                       │
│    contentId: "med-123",                                    │
│    action: "watch",                                         │
│    duration: 900, // 15 min                                 │
│    rating: 5,     // 1-5 stars                              │
│    emotionalChange: {                                       │
│      before: currentState,                                  │
│      after: measuredState                                   │
│    }                                                        │
│  }                                                          │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ RL Model Update                                             │
│                                                             │
│  1. Calculate reward from rating + emotional change         │
│  2. Update Q-value for (state, action) pair                │
│  3. Improve future recommendations                          │
└─────────────────────────────────────────────────────────────┘
```

---

## Component Dependencies

```
RecommendationGrid
├── framer-motion (animations)
├── lucide-react (icons)
├── tailwindcss (styling)
├── react (core)
│
├── RecommendationCard
│   ├── framer-motion
│   ├── lucide-react
│   ├── OutcomePredictor
│   └── category-thumbnails (utils)
│
├── RecommendationSkeletonGrid
│   └── RecommendationSkeleton
│       └── framer-motion
│
└── RecommendationDetail
    ├── framer-motion
    ├── lucide-react
    ├── OutcomePredictor
    └── category-thumbnails (utils)
```

---

## State Management

```typescript
// Component-level state
const [selectedRecommendation, setSelectedRecommendation] = 
  useState<Recommendation | null>(null);

// Hook-level state
const {
  recommendations,  // Recommendation[]
  isLoading,        // boolean
  error,            // string | null
  fetchRecommendations,  // () => Promise<void>
  refresh           // () => Promise<void>
} = useRecommendations({
  currentState,
  desiredState,
  userId,
  autoFetch: true
});

// User interaction state
const handleWatch = (contentId: string) => {
  // Track analytics
  // Navigate to player
  // Update watched history
};

const handleSave = (contentId: string) => {
  // Add to watchlist
  // Update UI
  // Sync to backend
};
```

---

## Animation States

```typescript
// Card lifecycle states
enum CardState {
  ENTERING = "entering",      // Initial entrance
  IDLE = "idle",              // Resting state
  HOVERED = "hovered",        // Mouse over
  PRESSED = "pressed",        // Mouse down
  SELECTED = "selected"       // Detail modal open
}

// Grid states
enum GridState {
  LOADING = "loading",        // Fetching data
  LOADED = "loaded",          // Data displayed
  EMPTY = "empty",            // No recommendations
  ERROR = "error"             // Failed to load
}

// Modal states
enum ModalState {
  CLOSED = "closed",          // Not visible
  OPENING = "opening",        // Animating in
  OPEN = "open",              // Fully visible
  CLOSING = "closing"         // Animating out
}
```

---

**Status**: Component tree complete and documented
**File Location**: `/workspaces/hackathon-tv5/apps/emotistream-web/docs/COMPONENT_TREE.md`
