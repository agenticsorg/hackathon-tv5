# EmotiStream API Client & State Management

## Overview
Complete API client layer and state management implementation for EmotiStream frontend.

## Files Created

### 1. Type Definitions
**File**: `/workspaces/hackathon-tv5/apps/emotistream-web/src/types/index.ts`

**Exports**:
- `User` - User profile type
- `EmotionAnalysis` - Emotion detection result
- `ContentItem` - Recommended content
- `Recommendation` - Content recommendation with Q-values
- `Feedback` - User feedback on recommendations
- `LearningProgress` - Q-learning progress metrics
- `Experience` - Q-learning experience tuple

### 2. API Client
**File**: `/workspaces/hackathon-tv5/apps/emotistream-web/src/lib/api/client.ts`

**Features**:
- Axios instance with base configuration
- Request interceptor for auth token injection
- Response interceptor for token refresh on 401
- Automatic redirect to login on auth failure
- 10-second timeout
- CORS credentials support

### 3. Auth API Module
**File**: `/workspaces/hackathon-tv5/apps/emotistream-web/src/lib/api/auth.ts`

**Exports**:
- `login(email, password)` → POST `/auth/login`
- `register(email, password, name)` → POST `/auth/register`
- `refreshToken(refreshToken)` → POST `/auth/refresh`
- `logout()` → POST `/auth/logout`
- `getCurrentUser()` → GET `/auth/me`

### 4. Emotion API Module
**File**: `/workspaces/hackathon-tv5/apps/emotistream-web/src/lib/api/emotion.ts`

**Exports**:
- `analyzeEmotion(userId, text)` → POST `/emotion/analyze`
- `getEmotionHistory(userId, limit, offset)` → GET `/emotion/history/:userId`
- `getLatestEmotion(userId)` → GET latest emotion
- `deleteEmotion(emotionId)` → DELETE `/emotion/:emotionId`

### 5. Recommendation API Module
**File**: `/workspaces/hackathon-tv5/apps/emotistream-web/src/lib/api/recommend.ts`

**Exports**:
- `getRecommendations(userId, currentState, desiredState, limit)` → POST `/recommend`
- `getRecommendationHistory(userId, limit, offset)` → GET `/recommend/history/:userId`
- `getContent(contentId)` → GET `/content/:contentId`
- `searchContent(query, filters)` → GET `/content/search`

### 6. Feedback API Module
**File**: `/workspaces/hackathon-tv5/apps/emotistream-web/src/lib/api/feedback.ts`

**Exports**:
- `submitFeedback(data)` → POST `/feedback`
- `getLearningProgress(userId)` → GET `/feedback/progress/:userId`
- `getExperiences(userId, limit, offset)` → GET `/feedback/experiences/:userId`
- `getFeedbackHistory(userId, limit, offset)` → GET `/feedback/history/:userId`
- `getQTable(userId)` → GET `/feedback/qtable/:userId`

### 7. Auth Store (Zustand)
**File**: `/workspaces/hackathon-tv5/apps/emotistream-web/src/lib/stores/auth-store.ts`

**State**:
- `user: User | null`
- `isAuthenticated: boolean`
- `accessToken: string | null`
- `refreshToken: string | null`

**Actions**:
- `login(user, accessToken, refreshToken)` - Store auth state
- `logout()` - Clear auth state
- `updateUser(updates)` - Update user profile

**Persistence**: LocalStorage

### 8. Emotion Store (Zustand)
**File**: `/workspaces/hackathon-tv5/apps/emotistream-web/src/lib/stores/emotion-store.ts`

**State**:
- `currentEmotion: EmotionAnalysis | null`
- `desiredState: string | null`
- `emotionHistory: EmotionAnalysis[]` (last 10)

**Actions**:
- `setCurrentEmotion(emotion)` - Set current emotion
- `setDesiredState(state)` - Set desired emotional state
- `addToHistory(emotion)` - Add to history
- `clearHistory()` - Clear history

**Persistence**: LocalStorage

### 9. Recommendation Store (Zustand)
**File**: `/workspaces/hackathon-tv5/apps/emotistream-web/src/lib/stores/recommendation-store.ts`

**State**:
- `recommendations: Recommendation[]`
- `selectedContent: ContentItem | null`
- `currentRecommendation: Recommendation | null`
- `explorationMode: boolean`

**Actions**:
- `setRecommendations(recommendations)` - Set recommendations
- `selectContent(content, recommendation)` - Select content
- `clearSelected()` - Clear selection
- `toggleExplorationMode()` - Toggle exploration

**Persistence**: SessionStorage

### 10. Auth Hooks (React Query)
**File**: `/workspaces/hackathon-tv5/apps/emotistream-web/src/lib/hooks/use-auth.ts`

**Hooks**:
- `useLogin()` - Login mutation
- `useRegister()` - Register mutation
- `useLogout()` - Logout mutation
- `useCurrentUser()` - Get current user query
- `useRefreshToken()` - Refresh token mutation

### 11. Emotion Hooks (React Query)
**File**: `/workspaces/hackathon-tv5/apps/emotistream-web/src/lib/hooks/use-emotion.ts`

**Hooks**:
- `useAnalyzeEmotion()` - Analyze emotion mutation
- `useEmotionHistory(userId, limit, offset)` - Emotion history query
- `useLatestEmotion(userId)` - Latest emotion query
- `useDeleteEmotion()` - Delete emotion mutation

### 12. Recommendation Hooks (React Query)
**File**: `/workspaces/hackathon-tv5/apps/emotistream-web/src/lib/hooks/use-recommendations.ts`

**Hooks**:
- `useRecommendations()` - Get recommendations mutation
- `useRecommendationHistory(userId, limit, offset)` - History query
- `useContent(contentId)` - Get content by ID query
- `useSearchContent(query, filters)` - Search content query

### 13. Feedback Hooks (React Query)
**File**: `/workspaces/hackathon-tv5/apps/emotistream-web/src/lib/hooks/use-feedback.ts`

**Hooks**:
- `useSubmitFeedback()` - Submit feedback mutation
- `useLearningProgress(userId)` - Learning progress query
- `useExperiences(userId, limit, offset)` - Experiences query
- `useFeedbackHistory(userId, limit, offset)` - Feedback history query
- `useQTable(userId)` - Q-table query

### 14. Query Provider
**File**: `/workspaces/hackathon-tv5/apps/emotistream-web/src/lib/providers/query-provider.tsx`

**Features**:
- React Query client provider
- Global query defaults (1 min stale time, retry once)
- Global mutation defaults (no retry)
- React Query Devtools integration

### 15. Index Exports
**Files**:
- `/workspaces/hackathon-tv5/apps/emotistream-web/src/lib/api/index.ts` - API exports
- `/workspaces/hackathon-tv5/apps/emotistream-web/src/lib/stores/index.ts` - Store exports
- `/workspaces/hackathon-tv5/apps/emotistream-web/src/lib/hooks/index.ts` - Hook exports

## Usage Examples

### Authentication
```typescript
import { useLogin, useAuthStore } from '@/lib';

function LoginForm() {
  const { mutate: login, isPending } = useLogin();
  const { isAuthenticated } = useAuthStore();

  const handleSubmit = (email: string, password: string) => {
    login({ email, password });
  };
}
```

### Emotion Analysis
```typescript
import { useAnalyzeEmotion, useEmotionStore } from '@/lib';

function EmotionInput() {
  const { mutate: analyze } = useAnalyzeEmotion();
  const { currentEmotion } = useEmotionStore();

  const handleAnalyze = (text: string) => {
    analyze({ userId: 'user-id', text });
  };
}
```

### Get Recommendations
```typescript
import { useRecommendations } from '@/lib/hooks';

function RecommendationList() {
  const { mutate: getRecommendations, data } = useRecommendations();

  const handleGetRecommendations = () => {
    getRecommendations({
      userId: 'user-id',
      currentState: 'anxious',
      desiredState: 'calm',
      limit: 5
    });
  };
}
```

### Submit Feedback
```typescript
import { useSubmitFeedback } from '@/lib/hooks';

function FeedbackForm() {
  const { mutate: submitFeedback } = useSubmitFeedback();

  const handleSubmit = (rating: number) => {
    submitFeedback({
      userId: 'user-id',
      recommendationId: 'rec-id',
      rating,
      wasHelpful: rating >= 3,
      resultingEmotion: 'calm'
    });
  };
}
```

## Features

### Token Management
- Automatic token injection in requests
- Automatic token refresh on 401 errors
- Token persistence in localStorage
- Automatic logout on refresh failure

### State Persistence
- Auth state in localStorage
- Emotion state in localStorage
- Recommendation state in sessionStorage

### Cache Management
- Query invalidation on mutations
- Stale time configuration per query
- Automatic refetch on success

### Error Handling
- Axios interceptors for global error handling
- React Query error states
- Automatic retry logic

## API Endpoint Mapping

| Hook/Function | Method | Endpoint | Description |
|--------------|--------|----------|-------------|
| `login()` | POST | `/auth/login` | User login |
| `register()` | POST | `/auth/register` | User registration |
| `refreshToken()` | POST | `/auth/refresh` | Refresh access token |
| `logout()` | POST | `/auth/logout` | User logout |
| `getCurrentUser()` | GET | `/auth/me` | Get current user |
| `analyzeEmotion()` | POST | `/emotion/analyze` | Analyze emotion from text |
| `getEmotionHistory()` | GET | `/emotion/history/:userId` | Get emotion history |
| `getRecommendations()` | POST | `/recommend` | Get content recommendations |
| `getRecommendationHistory()` | GET | `/recommend/history/:userId` | Get recommendation history |
| `submitFeedback()` | POST | `/feedback` | Submit feedback |
| `getLearningProgress()` | GET | `/feedback/progress/:userId` | Get learning progress |
| `getExperiences()` | GET | `/feedback/experiences/:userId` | Get Q-learning experiences |
| `getQTable()` | GET | `/feedback/qtable/:userId` | Get Q-table values |

## Environment Variables

```env
NEXT_PUBLIC_API_URL=http://localhost:3000/api/v1
```

## Next Steps

1. Wrap root component with `QueryProvider`
2. Create UI components using these hooks
3. Add error boundaries for API failures
4. Implement loading states
5. Add toast notifications for success/error
6. Create dashboard to display learning progress
7. Add Q-table visualization
