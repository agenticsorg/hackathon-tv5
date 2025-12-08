# EmotiStream Frontend Implementation Plan

**Version:** 1.0
**Date:** 2025-12-06
**Planning Methodology:** GOAP (Goal-Oriented Action Planning)
**Target:** MVP Release

---

## Executive Summary

EmotiStream is an AI-powered emotional content recommendation system that analyzes user emotions and provides personalized content suggestions using reinforcement learning. This plan outlines the frontend implementation strategy to create an engaging, emotion-aware user experience that showcases the unique value proposition: **content recommendations that understand and respond to how you feel**.

### Core Value Proposition
- **Emotion Detection**: Real-time AI-powered emotion analysis via Gemini API
- **Smart Recommendations**: Q-learning algorithm that learns user preferences
- **Adaptive Learning**: System improves with every interaction
- **Personalized Journey**: Content tailored to emotional state transitions

---

## GOAP Analysis: Current State → Goal State

### Current State (World State)
```javascript
{
  backend: {
    emotionAnalysis: "implemented", // /api/v1/emotion/analyze
    recommendations: "implemented",  // /api/v1/recommend
    feedback: "implemented",         // /api/v1/feedback
    authentication: "implemented",   // /api/v1/auth/*
    geminiIntegration: "active",
    rlEngine: "active"
  },
  frontend: {
    exists: false,
    uiComponents: "missing",
    stateManagement: "missing",
    apiClient: "missing",
    authentication: "missing"
  },
  user: {
    canAuthenticate: false,
    canInputEmotion: false,
    canReceiveRecommendations: false,
    canProvideFeedback: false,
    canSeeProgress: false
  }
}
```

### Goal State
```javascript
{
  frontend: {
    exists: true,
    uiComponents: "implemented",
    stateManagement: "implemented",
    apiClient: "implemented",
    authentication: "implemented",
    animations: "polished",
    responsive: true
  },
  user: {
    canAuthenticate: true,
    canInputEmotion: true,
    canReceiveRecommendations: true,
    canProvideFeedback: true,
    canSeeProgress: true,
    hasDelightfulExperience: true
  },
  mvp: {
    deployed: true,
    demoReady: true,
    documentationComplete: true
  }
}
```

### Gap Analysis
The system needs a complete frontend implementation that:
1. Provides authentication and user management
2. Captures and visualizes emotional states
3. Displays personalized recommendations with reasoning
4. Collects feedback and shows learning progress
5. Creates an engaging, emotion-aware UX

---

## UX Design Principles

Based on research into [emotion-based UI design patterns](https://www.interaction-design.org/literature/topics/emotional-response), [Netflix/Spotify UX patterns](https://www.shaped.ai/blog/key-insights-from-the-netflix-personalization-search-recommendation-workshop-2025), and [AI-powered interfaces](https://flexxited.com/v0-dev-guide-2025-ai-powered-ui-generation-for-react-and-tailwind-css), EmotiStream will follow these principles:

### 1. Emotional Journey Mapping
Design based on **how users feel**, not just what they do. Create a "mood map" rather than a traditional sitemap.

**Implementation:**
- Visual emotion state representation (color-coded mood rings)
- Progress visualization showing emotional transitions
- Journey timeline displaying before/after states

### 2. Don Norman's Three Levels
- **Visceral Design**: Immediate emotional impact through color, animation, and visual appeal
- **Behavioral Design**: Intuitive interactions that feel natural
- **Reflective Design**: Meaningful feedback that builds trust and understanding

### 3. Color Psychology for Emotion
- **Blues/Greens**: Calm, relaxed states (low arousal, positive valence)
- **Warm Orange/Yellow**: Energized, happy states (high arousal, positive valence)
- **Cool Purples**: Contemplative, introspective states
- **Muted Tones**: Stressed, negative states (to avoid amplifying negativity)
- **Gradients**: Emotional transitions and fluidity

### 4. Biometric UX Patterns (2025 Trend)
- Real-time adaptive interfaces that respond to emotional state
- Progressive disclosure based on stress levels
- Context-aware assistance when frustration detected

### 5. Netflix/Spotify-Inspired Patterns
- **Horizontal scrolling** for content categories
- **Card-based layouts** for recommendations
- **Personalization transparency**: Show WHY content was recommended
- **Confidence indicators**: Display AI certainty levels
- **Preview on hover**: Content details without navigation

### 6. Ethical Emotional Design
- **Transparency**: Show how the AI makes decisions
- **User control**: Easy opt-out and preference management
- **Avoid dark patterns**: No manipulative emotion triggers
- **Trust building**: Explain learning process clearly

---

## Technical Architecture

### Frontend Stack (2025 Best Practices)

Based on [React AI Stack research](https://www.builder.io/blog/react-ai-stack) and [modern UI patterns](https://www.shadcn.io), the recommended stack is:

```typescript
{
  framework: "Next.js 15 (App Router)",
  language: "TypeScript",
  styling: "Tailwind CSS 4.x",
  components: "shadcn/ui + Aceternity UI",
  animations: "Framer Motion",
  state: "Zustand + React Query",
  api: "Axios + React Query",
  auth: "JWT + HTTP-only cookies",
  forms: "React Hook Form + Zod",
  charts: "Recharts",
  icons: "Lucide React"
}
```

**Rationale:**
- **Next.js App Router**: Server components, built-in optimizations, easy deployment
- **Tailwind + shadcn/ui**: Utility-first styling with accessible components
- **Framer Motion**: 2025 standard for React animations
- **Zustand**: Lightweight state management (better than Redux for MVP)
- **React Query**: Server state caching, automatic refetching
- **Aceternity UI**: Stunning animations for emotional UX

### File Structure

```
apps/emotistream-web/
├── app/
│   ├── (auth)/
│   │   ├── login/
│   │   │   └── page.tsx
│   │   └── register/
│   │       └── page.tsx
│   ├── (app)/
│   │   ├── dashboard/
│   │   │   └── page.tsx          # Main emotion input + recommendations
│   │   ├── history/
│   │   │   └── page.tsx          # Emotional journey history
│   │   ├── progress/
│   │   │   └── page.tsx          # Learning progress analytics
│   │   └── layout.tsx            # Authenticated app layout
│   ├── layout.tsx                # Root layout
│   ├── page.tsx                  # Landing page
│   └── globals.css
├── components/
│   ├── ui/                       # shadcn/ui components
│   │   ├── button.tsx
│   │   ├── card.tsx
│   │   ├── input.tsx
│   │   └── ...
│   ├── emotion/
│   │   ├── emotion-input.tsx     # Text input for emotion detection
│   │   ├── emotion-visualizer.tsx # Mood ring/color visualization
│   │   ├── emotion-history-chart.tsx
│   │   └── desired-state-selector.tsx
│   ├── recommendations/
│   │   ├── recommendation-card.tsx
│   │   ├── recommendation-grid.tsx
│   │   ├── recommendation-reasoning.tsx
│   │   └── content-preview.tsx
│   ├── feedback/
│   │   ├── feedback-modal.tsx
│   │   ├── rating-input.tsx
│   │   └── emotion-after-input.tsx
│   ├── progress/
│   │   ├── learning-progress-chart.tsx
│   │   ├── reward-timeline.tsx
│   │   └── convergence-indicator.tsx
│   └── shared/
│       ├── navbar.tsx
│       ├── loading-states.tsx
│       └── error-boundaries.tsx
├── lib/
│   ├── api/
│   │   ├── client.ts             # Axios instance
│   │   ├── auth.ts               # Auth endpoints
│   │   ├── emotion.ts            # Emotion endpoints
│   │   ├── recommend.ts          # Recommendation endpoints
│   │   └── feedback.ts           # Feedback endpoints
│   ├── stores/
│   │   ├── auth-store.ts         # Zustand auth state
│   │   ├── emotion-store.ts      # Current emotion state
│   │   └── recommendation-store.ts
│   ├── hooks/
│   │   ├── use-auth.ts
│   │   ├── use-emotion-analysis.ts
│   │   ├── use-recommendations.ts
│   │   └── use-feedback.ts
│   ├── utils/
│   │   ├── emotion-colors.ts     # Color mapping logic
│   │   ├── emotion-labels.ts     # Human-readable labels
│   │   └── validators.ts
│   └── types/
│       └── api.ts                # API type definitions
├── public/
│   ├── images/
│   └── animations/
├── package.json
├── tsconfig.json
├── tailwind.config.ts
└── next.config.js
```

---

## API Integration Mapping

### Frontend Features → Backend Endpoints

| Frontend Feature | API Endpoint | Method | Purpose |
|-----------------|--------------|--------|---------|
| **Authentication** |
| Login | `/api/v1/auth/login` | POST | User authentication |
| Register | `/api/v1/auth/register` | POST | User registration |
| Token Refresh | `/api/v1/auth/refresh` | POST | Renew access token |
| **Emotion Detection** |
| Analyze Emotion | `/api/v1/emotion/analyze` | POST | Detect emotional state from text |
| Emotion History | `/api/v1/emotion/history/:userId` | GET | Past emotional states |
| **Recommendations** |
| Get Recommendations | `/api/v1/recommend` | POST | Personalized content suggestions |
| Recommendation History | `/api/v1/recommend/history/:userId` | GET | Past recommendations |
| **Feedback & Learning** |
| Submit Feedback | `/api/v1/feedback` | POST | Post-viewing feedback + RL update |
| Learning Progress | `/api/v1/feedback/progress/:userId` | GET | User learning metrics |
| Feedback History | `/api/v1/feedback/experiences/:userId` | GET | Past feedback experiences |

---

## Milestone Breakdown (GOAP Action Sequence)

### Milestone 1: Foundation & Authentication
**Goal:** Users can register, login, and maintain sessions
**Complexity:** Low
**Duration:** 2-3 days

#### Features to Implement
1. **Next.js Project Setup**
   - Initialize Next.js 15 with App Router
   - Configure Tailwind CSS 4.x
   - Install shadcn/ui, Framer Motion, Zustand, React Query
   - Setup ESLint, Prettier, TypeScript strict mode

2. **Authentication UI**
   - Login page with email/password form
   - Registration page with validation
   - JWT storage in HTTP-only cookies
   - Protected route middleware
   - Auth store (Zustand) with persist

3. **API Client Setup**
   - Axios instance with interceptors
   - Request/response logging
   - Error handling and retries
   - Auth token refresh logic

4. **Landing Page**
   - Hero section explaining EmotiStream
   - Value proposition visualization
   - Call-to-action (Sign Up / Login)

#### Success Criteria
- [ ] User can register with email/password (8+ chars, validation)
- [ ] User can login and receive JWT tokens
- [ ] Token refresh happens automatically before expiry
- [ ] Protected routes redirect to login when unauthenticated
- [ ] Landing page loads in <1s with smooth animations

#### UX Considerations
- **Password strength indicator** with real-time feedback
- **Error messages** that are helpful, not technical
- **Loading states** during API calls
- **Success animations** after registration
- **Smooth page transitions** using Framer Motion

---

### Milestone 2: Emotion Input & Visualization
**Goal:** Users can express emotions and see visual representations
**Complexity:** Medium
**Duration:** 3-4 days

#### Features to Implement
1. **Dashboard Layout**
   - Navbar with user menu
   - Main content area
   - Sidebar for quick actions
   - Mobile-responsive navigation

2. **Emotion Input Component**
   - Large textarea for natural language input
   - Character count (10-1000 chars)
   - Voice input option (future enhancement)
   - Submit button with loading state

3. **Emotion Visualizer**
   - **Mood Ring**: Circular gradient visualization
     - Color based on valence/arousal
     - Size based on stress level
     - Animated transitions
   - **Emotional State Cards**
     - Valence: -1 to +1 (sad to happy)
     - Arousal: -1 to +1 (calm to energized)
     - Stress: 0 to 1 (relaxed to stressed)
   - **Plutchik Emotion Wheel** (interactive)
   - **Primary Emotion Label** (large, clear)

4. **Desired State Selector**
   - Quick presets: "Relax", "Energize", "Focus", "Sleep"
   - Custom target valence/arousal sliders
   - Visual preview of target state

5. **API Integration**
   - POST `/api/v1/emotion/analyze`
   - React Query mutation with optimistic updates
   - Error handling for rate limits
   - Gemini vs. local detector indicator

#### Success Criteria
- [ ] User can input emotional text (10+ characters)
- [ ] Emotion analysis completes in <2s (with loading state)
- [ ] Visual representation updates with smooth animation
- [ ] User can see valence, arousal, stress levels clearly
- [ ] Desired state can be set via presets or custom inputs
- [ ] Mobile experience is touch-optimized

#### UX Considerations
- **Placeholder text** with examples: "I'm feeling stressed about work..."
- **Real-time character count** with color coding (red <10, green ≥10)
- **Smooth color transitions** on mood ring (300ms ease-in-out)
- **Haptic feedback** on mobile after analysis complete
- **Confidence indicator**: Show AI certainty (0-100%)
- **Explanation tooltip**: "Gemini AI detected high stress based on word patterns"

#### Color Mapping Logic
```typescript
// lib/utils/emotion-colors.ts
export function getEmotionColor(valence: number, arousal: number): string {
  // Positive, high energy → warm oranges/yellows
  if (valence > 0 && arousal > 0) return "from-orange-400 to-yellow-300";
  // Positive, low energy → calm blues/greens
  if (valence > 0 && arousal < 0) return "from-blue-400 to-green-300";
  // Negative, high energy → intense reds/purples
  if (valence < 0 && arousal > 0) return "from-red-400 to-purple-500";
  // Negative, low energy → muted grays/blues
  return "from-gray-400 to-blue-600";
}
```

---

### Milestone 3: Content Recommendations
**Goal:** Users receive personalized recommendations with explanations
**Complexity:** High
**Duration:** 4-5 days

#### Features to Implement
1. **Recommendation Grid**
   - Netflix-style horizontal scrolling cards
   - 3-5 recommendations per request
   - Lazy loading for performance
   - Skeleton loaders during fetch

2. **Recommendation Card**
   - **Content thumbnail** (placeholder images for MVP)
   - **Title + category** (movie, series, meditation, etc.)
   - **Duration** (runtime in minutes)
   - **Combined score** (0-1 scale as percentage)
   - **Predicted outcome**
     - Expected valence change
     - Expected arousal change
     - Expected stress reduction
   - **Reasoning explanation**
     - "High Q-value for stress reduction"
     - "Recommended based on past preferences"
   - **Exploration badge** (if isExploration === true)
   - **Call-to-action**: "Watch Now" button

3. **Recommendation Reasoning Panel**
   - Expandable detail view
   - **Why this content?**
     - Q-value history chart
     - Similarity score breakdown
     - Expected emotional transition
   - **How confident is the AI?**
     - Confidence percentage
     - Exploration rate context
   - **What happens next?**
     - "If you watch this and provide feedback, I'll learn your preferences better"

4. **API Integration**
   - POST `/api/v1/recommend`
   - Send current + desired emotional states
   - React Query with stale-while-revalidate
   - Automatic refetch on emotion state change

5. **Empty States**
   - "No recommendations yet" for new users
   - "Analyzing your emotional journey..." loading state
   - "Try describing your mood to get started" prompt

#### Success Criteria
- [ ] Recommendations load within 1.5s of emotion analysis
- [ ] User can see 3-5 personalized content suggestions
- [ ] Each card displays score, reasoning, and predicted outcome
- [ ] Recommendations update when desired state changes
- [ ] Exploration badge clearly indicates "trying something new"
- [ ] Mobile cards are swipeable (horizontal scroll)
- [ ] Empty states guide new users

#### UX Considerations
- **Progressive disclosure**: Basic info on card, details on expand
- **Visual hierarchy**: Score + title most prominent
- **Trust indicators**: Show confidence, explain reasoning
- **Smooth animations**: Card entrance (stagger 50ms), hover effects
- **Accessibility**: Keyboard navigation, screen reader support
- **Loading skeletons**: Match card layout for perceived performance

#### Recommendation Card Design
```typescript
// components/recommendations/recommendation-card.tsx
interface RecommendationCardProps {
  contentId: string;
  title: string;
  category: string;
  duration: number;
  combinedScore: number;
  predictedOutcome: {
    expectedValence: number;
    expectedArousal: number;
    expectedStress: number;
    confidence: number;
  };
  reasoning: string;
  isExploration: boolean;
  onWatch: () => void;
}

// Visual states:
// - Default: Subtle shadow, border
// - Hover: Lift animation (translateY -4px), stronger shadow
// - Active: Glow effect if selected
// - Exploration: Badge in top-right corner
```

---

### Milestone 4: Feedback Collection
**Goal:** Users can provide post-viewing feedback to train the system
**Complexity:** Medium
**Duration:** 3-4 days

#### Features to Implement
1. **Feedback Modal**
   - Triggered after "Watch Now" click
   - Timer tracking (watch duration)
   - Post-viewing emotion re-analysis
   - Rating collection (1-5 stars)

2. **Post-Viewing Emotion Input**
   - "How do you feel now?" text input
   - Emotion analysis using same visualizer
   - Side-by-side comparison: before vs. after

3. **Rating Input**
   - 5-star rating with hover states
   - Optional text feedback
   - "Did you complete the content?" checkbox

4. **Emotional Transition Visualization**
   - **Before/After Cards** side-by-side
   - **Arrow animation** showing transition
   - **Color gradient path** from start to end state
   - **Reward calculation** display

5. **API Integration**
   - POST `/api/v1/feedback`
   - Include stateBeforeViewing, actualPostState, watchDuration
   - Show reward calculation result
   - Update learning progress in real-time

6. **Success Feedback**
   - Confetti animation on positive reward
   - "Great choice! You felt 40% more relaxed" message
   - Learning progress update notification

#### Success Criteria
- [ ] User can describe post-viewing emotions
- [ ] Before/after states are clearly visualized
- [ ] Watch duration is automatically tracked
- [ ] Rating (1-5 stars) is intuitive to provide
- [ ] Feedback submission updates learning progress
- [ ] User sees immediate positive reinforcement
- [ ] Modal can be dismissed without submitting (tracked as incomplete)

#### UX Considerations
- **Non-intrusive timing**: Show feedback after natural breakpoint
- **Quick action**: Default to "completed" for ease
- **Visual reward**: Animate reward score with celebration
- **Educational**: Explain how feedback improves recommendations
- **Skippable**: Allow users to close without guilt
- **Positive framing**: "Help me learn!" vs. "Required feedback"

---

### Milestone 5: Learning Progress & Analytics
**Goal:** Users can see how the system is learning their preferences
**Complexity:** Medium
**Duration:** 3-4 days

#### Features to Implement
1. **Progress Dashboard**
   - Hero metrics cards:
     - Total experiences
     - Average reward
     - Current exploration rate
     - Convergence score
   - Time-series chart: reward over time
   - Emotional journey map
   - Content preference breakdown

2. **Learning Progress Chart**
   - **Reward Timeline** (line chart)
     - X-axis: Experience count
     - Y-axis: Reward (0-1)
     - Trend line showing improvement
   - **Recent Rewards** (last 10 experiences)
   - **Average by content type** (bar chart)

3. **Convergence Indicator**
   - Progress bar (0-100%)
   - Explanation: "How well I understand your preferences"
   - Visual states:
     - 0-30%: "Still exploring" (yellow)
     - 30-70%: "Learning patterns" (blue)
     - 70-100%: "Confident" (green)

4. **Emotional Journey Map**
   - Scatter plot: valence vs. arousal over time
   - Color-coded by stress level
   - Hover to see content watched
   - Cluster analysis visualization

5. **API Integration**
   - GET `/api/v1/feedback/progress/:userId`
   - GET `/api/v1/feedback/experiences/:userId`
   - GET `/api/v1/emotion/history/:userId`
   - React Query with 30s cache

6. **Insights Panel**
   - "You prefer calming content in the evening"
   - "Meditation has consistently reduced your stress by 45%"
   - "Your emotional range has expanded 30% this month"

#### Success Criteria
- [ ] User can see total experiences and average reward
- [ ] Reward timeline shows learning progress visually
- [ ] Convergence indicator updates with each feedback
- [ ] Emotional journey map displays state transitions
- [ ] Insights are actionable and personalized
- [ ] Charts are responsive and interactive
- [ ] Data loads within 1s with loading states

#### UX Considerations
- **Gamification**: Progress bars, achievement badges
- **Transparency**: Explain what metrics mean
- **Motivation**: Highlight improvements, celebrate milestones
- **Context**: Compare to baseline, show trends
- **Privacy**: Clarify data is local/personal only

---

### Milestone 6: Real-Time Features & Polish
**Goal:** Enhance UX with real-time updates and delightful interactions
**Complexity:** High
**Duration:** 3-4 days

#### Features to Implement
1. **Real-Time Recommendation Updates**
   - WebSocket connection (future) or polling
   - Live exploration rate updates
   - Background Q-value recalculations

2. **Advanced Animations**
   - **Page transitions**: Smooth routing animations
   - **Micro-interactions**:
     - Button hover effects
     - Card flip on recommendation reasoning
     - Confetti on high rewards
     - Ripple effects on clicks
   - **Emotion visualizer**:
     - Pulsing effect for high arousal
     - Glow intensity for stress
     - Smooth color morphing

3. **Accessibility Enhancements**
   - ARIA labels for screen readers
   - Keyboard navigation shortcuts
   - Focus indicators
   - Color contrast compliance (WCAG AA)
   - Reduced motion preferences

4. **Performance Optimizations**
   - Image lazy loading
   - Code splitting by route
   - React Query cache tuning
   - Framer Motion performance mode
   - Bundle size analysis

5. **Error Handling**
   - Retry logic for failed API calls
   - Graceful degradation (local detector fallback)
   - User-friendly error messages
   - Offline detection

6. **Mobile Optimizations**
   - Touch gestures (swipe, pinch)
   - Bottom sheet modals
   - Native-like animations
   - Haptic feedback

#### Success Criteria
- [ ] All animations run at 60fps
- [ ] Lighthouse score >90 (performance, accessibility)
- [ ] Offline state is handled gracefully
- [ ] Mobile experience feels native
- [ ] Keyboard navigation works for all features
- [ ] Screen readers can navigate the app
- [ ] No console errors in production

#### UX Considerations
- **Respect user preferences**: Honor `prefers-reduced-motion`
- **Progressive enhancement**: Core features work without JS
- **Perceived performance**: Optimistic updates, instant feedback
- **Delightful surprises**: Easter eggs, celebration animations
- **Consistency**: Animation timing, easing curves standardized

---

### Milestone 7: Documentation & Deployment
**Goal:** MVP is production-ready with complete documentation
**Complexity:** Low
**Duration:** 2-3 days

#### Features to Implement
1. **User Documentation**
   - Onboarding tour (first-time user experience)
   - Tooltips for complex features
   - Help center page
   - FAQ section

2. **Developer Documentation**
   - README with setup instructions
   - Component library (Storybook optional)
   - API integration guide
   - State management docs

3. **Testing**
   - Unit tests for utilities (emotion-colors, validators)
   - Integration tests for API client
   - E2E tests for critical flows (Playwright)
   - Manual QA checklist

4. **Deployment Setup**
   - Vercel deployment configuration
   - Environment variables setup
   - CI/CD pipeline (GitHub Actions)
   - Domain setup (optional)

5. **Demo Content**
   - Seed users for testing
   - Sample content recommendations
   - Demo video/screenshots

#### Success Criteria
- [ ] New developers can run the app locally in <10 minutes
- [ ] All critical user flows are E2E tested
- [ ] App is deployed to Vercel with custom domain
- [ ] Onboarding tour guides new users effectively
- [ ] Documentation covers all features
- [ ] Demo video showcases core value proposition

---

## Design Mockups (Text-Based Wireframes)

### Dashboard Layout
```
┌─────────────────────────────────────────────────────────────┐
│ [Logo] EmotiStream                    [User] [Settings] [▼]│
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ How are you feeling right now?                        │ │
│  │ ┌───────────────────────────────────────────────────┐ │ │
│  │ │ I'm stressed about work and feeling anxious...    │ │ │
│  │ │                                                   │ │ │
│  │ └───────────────────────────────────────────────────┘ │ │
│  │                               [Analyze Emotion] ──────┤ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌─────────────────┐  ┌─────────────────────────────────┐ │
│  │   MOOD RING     │  │ Your Emotional State            │ │
│  │   ╭─────────╮   │  │ Valence: -0.4 (Negative) 😟     │ │
│  │  ╱    🟠     ╲  │  │ Arousal: +0.3 (Moderate) ⚡      │ │
│  │ │   Stress   │ │  │ Stress: 0.7 (High) 🔥           │ │
│  │  ╲          ╱   │  │ Primary: Anxiety 😰              │ │
│  │   ╰─────────╯   │  │ Confidence: 85%                 │ │
│  └─────────────────┘  └─────────────────────────────────┘ │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ How do you want to feel?                              │ │
│  │ [Relax 🧘] [Energize ⚡] [Focus 🎯] [Sleep 😴]       │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ Personalized Recommendations                          │ │
│  │ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐        │ │
│  │ │ 🎬  │ │ 🎵  │ │ 🧘  │ │ 📺  │ │ 🎥  │        │ │
│  │ │ Med  │ │ Jazz │ │ Calm │ │ Doc  │ │ Short│        │ │
│  │ │ 92%  │ │ 88%  │ │ 85%  │ │ 78%  │ │ 75%  │        │ │
│  │ │[Watch│ │[Play]│ │[Start│ │[Watch│ │[Watch│        │ │
│  │ └──────┘ └──────┘ └──────┘ └──────┘ └──────┘        │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Recommendation Card (Expanded)
```
┌───────────────────────────────────────────────────────┐
│ 🧘 Deep Relaxation Meditation                        │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│ Category: Meditation | Duration: 15 min               │
│                                                        │
│ Combined Score: 92%                                    │
│ ████████████████████░░ Confidence: 87%                │
│                                                        │
│ Expected Outcome:                                      │
│ • Valence: -0.4 → +0.6 (From negative to positive)    │
│ • Arousal: +0.3 → -0.2 (From tense to calm)           │
│ • Stress: 0.7 → 0.2 (65% reduction)                   │
│                                                        │
│ Why this recommendation?                               │
│ "High Q-value (0.89) for stress reduction based on    │
│ past users with similar emotional profiles. Nature    │
│ sounds and guided breathing have shown 78% success    │
│ rate for anxiety relief."                             │
│                                                        │
│ [🎧 Watch Now] [ℹ️ More Details] [🔖 Save for Later] │
└───────────────────────────────────────────────────────┘
```

### Feedback Modal
```
┌───────────────────────────────────────────────────────┐
│ How was "Deep Relaxation Meditation"?                 │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│                                                        │
│ Before Watching          After Watching               │
│ ┌─────────┐              ┌─────────┐                 │
│ │  😰     │  ───────────→ │  😌     │                 │
│ │ Anxious │     Path      │ Relaxed │                 │
│ │ Stress  │              │ Calm    │                 │
│ └─────────┘              └─────────┘                 │
│                                                        │
│ How do you feel now?                                   │
│ ┌──────────────────────────────────────────────────┐  │
│ │ Much more relaxed, ready for bed                 │  │
│ └──────────────────────────────────────────────────┘  │
│                                  [Analyze Emotion]    │
│                                                        │
│ Did you complete the content?                          │
│ [✓] Yes  [ ] No (stopped at _____ minutes)            │
│                                                        │
│ Rate your experience:                                  │
│ [★★★★★] 5 stars                                      │
│                                                        │
│ Reward: +0.85 🎉                                       │
│ "Great choice! You felt 65% more relaxed"             │
│                                                        │
│ [Submit Feedback] [Skip]                              │
└───────────────────────────────────────────────────────┘
```

---

## Implementation Priority

### Phase 1: Core MVP (Weeks 1-2)
1. Milestone 1: Authentication ✓
2. Milestone 2: Emotion Input ✓
3. Milestone 3: Recommendations ✓

**Goal:** Functional emotional recommendation flow

### Phase 2: Learning Loop (Week 3)
4. Milestone 4: Feedback Collection ✓
5. Milestone 5: Progress Analytics ✓

**Goal:** Complete reinforcement learning cycle

### Phase 3: Polish & Ship (Week 4)
6. Milestone 6: Real-Time Features ✓
7. Milestone 7: Documentation & Deploy ✓

**Goal:** Production-ready MVP

---

## GOAP Action Plan Summary

### Preconditions → Actions → Effects

```
STATE: No frontend
ACTION: Setup Next.js + Tailwind + shadcn/ui
EFFECT: Development environment ready

STATE: No authentication
ACTION: Implement login/register UI + JWT handling
EFFECT: Users can authenticate

STATE: No emotion input
ACTION: Build emotion input form + API integration
EFFECT: Users can express emotions

STATE: No emotion visualization
ACTION: Build mood ring + emotion state cards
EFFECT: Users see emotional state visually

STATE: No recommendations
ACTION: Build recommendation grid + cards
EFFECT: Users receive personalized content

STATE: No feedback collection
ACTION: Build feedback modal + before/after comparison
EFFECT: Users can train the system

STATE: No progress tracking
ACTION: Build analytics dashboard + charts
EFFECT: Users see learning progress

STATE: No polish
ACTION: Add animations + accessibility + optimization
EFFECT: Delightful user experience

STATE: Not deployed
ACTION: Setup Vercel + write docs + create demo
EFFECT: Production-ready MVP
```

---

## Success Metrics

### Technical Metrics
- **Performance**: Lighthouse score >90
- **Accessibility**: WCAG AA compliance
- **Bundle Size**: <500KB initial load
- **API Response**: <2s average
- **Error Rate**: <1%

### User Experience Metrics
- **Time to First Recommendation**: <30s
- **Feedback Submission Rate**: >60%
- **Return User Rate**: >40%
- **Average Session Duration**: >5 minutes
- **User Satisfaction**: 4+ stars average

### Business Metrics
- **User Registration**: Track signups
- **Daily Active Users**: Track engagement
- **Recommendation Acceptance**: Track "Watch Now" clicks
- **Learning Progress**: Average convergence score
- **Content Coverage**: Variety of content consumed

---

## Risk Mitigation

### Technical Risks
1. **Gemini API Rate Limits**
   - Mitigation: Fallback to local emotion detector
   - Cache emotion analysis results (30s)

2. **Large Bundle Size**
   - Mitigation: Code splitting, lazy loading
   - Use lightweight alternatives (Zustand vs. Redux)

3. **Animation Performance**
   - Mitigation: CSS-based animations where possible
   - Respect `prefers-reduced-motion`
   - Use Framer Motion performance mode

### UX Risks
1. **Complex Emotion Input**
   - Mitigation: Examples, tooltips, onboarding tour
   - Quick presets for common moods

2. **Overwhelming Recommendations**
   - Mitigation: Limit to 5 recommendations initially
   - Progressive disclosure for details

3. **Feedback Fatigue**
   - Mitigation: Make feedback optional, quick, fun
   - Gamify with rewards, celebrations

---

## Future Enhancements (Post-MVP)

### Short-Term (1-2 months)
- **Voice input** for emotion detection
- **WebSocket** for real-time updates
- **Social features**: Share recommendations
- **Content library**: Expand beyond mock data
- **Advanced filters**: Genre, duration, mood

### Medium-Term (3-6 months)
- **Mobile apps** (React Native)
- **Offline mode** with service workers
- **Multi-modal input**: Image, video for emotion detection
- **Collaborative filtering**: Learn from similar users
- **Content partnerships**: Real streaming integrations

### Long-Term (6-12 months)
- **Wearable integration**: Heart rate, biometrics
- **AI therapist mode**: Emotional wellness coaching
- **Community features**: Groups, challenges
- **AR/VR experiences**: Immersive emotional content
- **White-label platform**: B2B for therapists, coaches

---

## Resources & References

### UX Research
- [Emotional Design in UX](https://www.interaction-design.org/literature/topics/emotional-response) - Don Norman's framework
- [Biometric UX Patterns](https://medium.com/@marketingtd64/biometric-ux-emotion-behavior-in-adaptive-ui-8523fc69cb2e) - Adaptive interfaces
- [Netflix Personalization Workshop 2025](https://www.shaped.ai/blog/key-insights-from-the-netflix-personalization-search-recommendation-workshop-2025) - Industry insights
- [Spotify Recommendation UX](https://www.music-tomorrow.com/blog/how-spotify-recommendation-system-works-complete-guide) - Domain adaptation patterns

### Technical Stack
- [React AI Stack 2025](https://www.builder.io/blog/react-ai-stack) - Modern framework selection
- [v0.dev AI UI Generation](https://flexxited.com/v0-dev-guide-2025-ai-powered-ui-generation-for-react-and-tailwind-css) - Component generation
- [shadcn/ui](https://www.shadcn.io) - Accessible component library
- [Aceternity UI](https://ui.aceternity.com/) - Advanced animations

### Backend Integration
- [EmotiStream API Documentation](/workspaces/hackathon-tv5/apps/emotistream/docs/API.md)
- [User Guide](/workspaces/hackathon-tv5/apps/emotistream/docs/USER_GUIDE.md)
- [Authentication Implementation](/workspaces/hackathon-tv5/apps/emotistream/docs/AUTH_IMPLEMENTATION.md)

---

## Appendix: API Type Definitions

```typescript
// lib/types/api.ts

export interface EmotionalState {
  valence: number;        // -1 to 1
  arousal: number;        // -1 to 1
  stressLevel: number;    // 0 to 1
  primaryEmotion: string;
  emotionVector: number[];
  confidence: number;     // 0 to 1
  timestamp: number;
}

export interface DesiredState {
  targetValence: number;
  targetArousal: number;
  targetStress: number;
  intensity: 'low' | 'moderate' | 'high';
  reasoning: string;
}

export interface Recommendation {
  contentId: string;
  title: string;
  category: string;
  duration: number;
  qValue: number;
  similarityScore: number;
  combinedScore: number;
  predictedOutcome: {
    expectedValence: number;
    expectedArousal: number;
    expectedStress: number;
    confidence: number;
  };
  reasoning: string;
  isExploration: boolean;
}

export interface FeedbackRequest {
  userId: string;
  contentId: string;
  actualPostState: EmotionalState;
  watchDuration: number;  // minutes
  completed: boolean;
  explicitRating?: number; // 1-5
}

export interface LearningProgress {
  totalExperiences: number;
  avgReward: number;
  explorationRate: number;
  convergenceScore: number;
  recentRewards: number[];
}
```

---

**End of Implementation Plan**

This plan provides a complete roadmap for building the EmotiStream frontend MVP using GOAP principles, modern UX patterns, and 2025 best practices. Each milestone builds on the previous one, ensuring a logical progression from basic authentication to a fully-featured emotional recommendation system.

**Next Steps:**
1. Review and approve this plan
2. Set up development environment (Milestone 1)
3. Begin implementation following milestone sequence
4. Weekly demos to stakeholders
5. Iterate based on user feedback

Generated by: Claude Code (GOAP Specialist)
Date: 2025-12-06
