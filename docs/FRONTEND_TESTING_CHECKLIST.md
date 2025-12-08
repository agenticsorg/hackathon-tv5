# EmotiStream Frontend Testing Checklist

This document provides a complete testing checklist for the EmotiStream frontend when it is implemented.

## Manual Testing Checklist

### Authentication
- [ ] Register with valid credentials
- [ ] Register with invalid email format
- [ ] Register with password < 8 characters
- [ ] Register with existing email (should show error)
- [ ] Login with valid credentials
- [ ] Login with invalid credentials
- [ ] JWT token stored in localStorage after login
- [ ] Protected routes redirect to login when not authenticated
- [ ] Dashboard accessible after successful login
- [ ] Logout clears token and redirects to login
- [ ] Refresh page maintains authentication state
- [ ] Token expiration handling

### Emotion Analysis
- [ ] Input less than 10 characters (analyze button disabled)
- [ ] Input exactly 10 characters (analyze button enabled)
- [ ] Character counter updates in real-time
- [ ] Click analyze shows loading spinner
- [ ] API request sent with correct payload
- [ ] Success response updates mood ring visualization
- [ ] Mood ring indicator positioned correctly
- [ ] Quadrant labels show correctly (Excited/Calm/Sad/Stressed)
- [ ] Emotional state metrics displayed (valence, arousal, stress)
- [ ] Error handling shows user-friendly message
- [ ] Can analyze again with different text

### Recommendations
- [ ] Grid loads after emotion analysis
- [ ] Shows loading skeleton while fetching
- [ ] Cards display all required information
- [ ] Exploration badge visible on exploration recommendations
- [ ] Q-values displayed with 2 decimal precision
- [ ] Similarity scores shown as percentages
- [ ] Predicted outcomes show valence/arousal/stress
- [ ] Hover animation works on cards
- [ ] Grid is responsive (1 column mobile, 2-3 desktop)
- [ ] Empty state shown when no recommendations
- [ ] "Watch Now" button triggers feedback modal

### Feedback Modal
- [ ] Modal opens when "Watch Now" clicked
- [ ] Before emotion state displayed correctly
- [ ] After emotion state input fields present
- [ ] Star rating component (1-5 stars) interactive
- [ ] Completion toggle works
- [ ] Watch duration auto-calculated from session
- [ ] Submit button posts to /api/v1/feedback
- [ ] Reward score calculated and displayed
- [ ] Reward message changes based on score
- [ ] Confetti animation for rewards > 0.7
- [ ] Modal closes after successful submission
- [ ] Error shown if submission fails
- [ ] Can submit feedback for multiple items

### Progress Dashboard
- [ ] Metrics load for authenticated user
- [ ] Total experiences count accurate
- [ ] Average reward calculated correctly
- [ ] Exploration rate displayed as percentage
- [ ] Convergence score shown (0-100)
- [ ] Convergence stage label correct
- [ ] Progress bar color matches stage
- [ ] Emotional journey chart renders
- [ ] Journey chart shows valence/arousal trajectory
- [ ] Reward timeline chart displays
- [ ] Trend line calculated with moving average
- [ ] Charts are interactive (hover to see values)
- [ ] Responsive layout on mobile

### Performance
- [ ] Initial page load < 3 seconds
- [ ] Emotion analysis response < 2 seconds
- [ ] Recommendation fetch < 1 second
- [ ] No UI blocking during API calls
- [ ] Smooth animations (60fps)
- [ ] Images lazy-loaded
- [ ] API requests debounced where appropriate
- [ ] No memory leaks on navigation

### Security
- [ ] JWT token in Authorization header (not URL)
- [ ] No sensitive data in browser console
- [ ] API errors don't expose stack traces
- [ ] XSS protection (input sanitized)
- [ ] CSRF token validation
- [ ] Rate limiting on authentication endpoints
- [ ] HTTPS enforced in production
- [ ] No localStorage data leakage

### Accessibility
- [ ] All interactive elements keyboard accessible
- [ ] Focus indicators visible
- [ ] Screen reader announces dynamic content
- [ ] ARIA labels on icon buttons
- [ ] Color contrast WCAG AA compliant
- [ ] Form inputs have associated labels
- [ ] Error messages announced to screen readers
- [ ] Modal traps focus when open
- [ ] Skip to main content link present
- [ ] Semantic HTML structure

### Cross-Browser
- [ ] Chrome (latest)
- [ ] Firefox (latest)
- [ ] Safari (latest)
- [ ] Edge (latest)
- [ ] Mobile Safari (iOS)
- [ ] Mobile Chrome (Android)

### Responsive Design
- [ ] Mobile (375px width)
- [ ] Tablet (768px width)
- [ ] Desktop (1024px width)
- [ ] Wide (1440px+ width)
- [ ] Landscape orientation
- [ ] Touch interactions work

## Automated Test Coverage Goals

### Unit Tests (Components)
- **Target**: 80%+ coverage
- [ ] EmotionInput component
- [ ] MoodRing component
- [ ] RecommendationCard component
- [ ] RecommendationGrid component
- [ ] FeedbackModal component
- [ ] ProgressMetrics component
- [ ] ConvergenceChart component
- [ ] LoginForm component
- [ ] RegisterForm component

### Integration Tests
- **Target**: 70%+ coverage
- [ ] API client configuration
- [ ] Authentication flow
- [ ] Emotion analysis flow
- [ ] Recommendation flow
- [ ] Feedback submission flow
- [ ] Progress data fetching
- [ ] Error handling

### E2E Tests
- **Target**: Critical user journeys
- [ ] Complete user registration → analysis → recommendation → feedback flow
- [ ] Login → dashboard → view progress flow
- [ ] Error recovery flows

## Test Data Requirements

### User Accounts
- Valid test user: `test@example.com` / `password123`
- Admin test user: `admin@example.com` / `admin123`
- Invalid user: `invalid@example.com` / `wrongpass`

### Test Emotions
- Positive excited: "I'm feeling absolutely amazing today! Everything is going great!"
- Negative stressed: "I'm so overwhelmed and anxious about everything going wrong."
- Neutral calm: "I'm feeling quite relaxed and peaceful right now."
- Mixed: "I'm excited about the new project but worried about the deadline."

### Test Content IDs
- Movie: `movie-123`
- Series: `series-456`
- Documentary: `doc-789`

## Bug Reporting Template

```markdown
### Bug Title
[Clear, descriptive title]

### Environment
- Browser: [Chrome 120]
- OS: [Windows 11]
- Screen size: [1920x1080]
- Frontend version: [v1.0.0]
- Backend version: [v1.0.0]

### Steps to Reproduce
1. [First step]
2. [Second step]
3. [Third step]

### Expected Behavior
[What should happen]

### Actual Behavior
[What actually happened]

### Screenshots
[Attach screenshots if applicable]

### Console Errors
[Paste any console errors]

### Network Tab
[Paste any failed network requests]

### Severity
- [ ] Critical (blocks core functionality)
- [ ] High (major feature broken)
- [ ] Medium (minor feature broken)
- [ ] Low (cosmetic issue)
```

## Performance Benchmarks

### Page Load Times
- **Homepage**: < 1.5s
- **Dashboard**: < 2.0s
- **Analysis page**: < 2.0s
- **Progress page**: < 2.5s

### API Response Times
- **POST /auth/login**: < 500ms
- **POST /emotion/analyze**: < 2000ms (Gemini API)
- **POST /recommend**: < 1000ms
- **POST /feedback**: < 500ms
- **GET /progress/:userId**: < 800ms

### Interaction Times
- **Button click feedback**: < 100ms
- **Form validation**: < 50ms
- **Modal open/close**: < 300ms
- **Chart render**: < 500ms

## Accessibility Standards

### WCAG 2.1 Level AA Compliance
- [ ] 1.1.1 Non-text Content (A)
- [ ] 1.3.1 Info and Relationships (A)
- [ ] 1.4.3 Contrast (Minimum) (AA)
- [ ] 2.1.1 Keyboard (A)
- [ ] 2.4.3 Focus Order (A)
- [ ] 2.4.7 Focus Visible (AA)
- [ ] 3.1.1 Language of Page (A)
- [ ] 3.2.1 On Focus (A)
- [ ] 3.3.1 Error Identification (A)
- [ ] 3.3.2 Labels or Instructions (A)
- [ ] 4.1.2 Name, Role, Value (A)

## Security Testing

### Authentication Security
- [ ] No password in URL or localStorage
- [ ] Token has expiration
- [ ] Token refresh mechanism
- [ ] Logout invalidates token
- [ ] No auth bypass via client-side routing

### Input Validation
- [ ] XSS protection on all text inputs
- [ ] SQL injection prevention
- [ ] CSRF token on state-changing operations
- [ ] Rate limiting on API calls
- [ ] Input length limits enforced

### Data Protection
- [ ] No sensitive data in console logs
- [ ] No API keys in frontend code
- [ ] HTTPS enforced
- [ ] Secure cookie flags set
- [ ] No localStorage data leakage

## Testing Tools

### Recommended Testing Stack
- **Unit Tests**: Jest + React Testing Library
- **Integration Tests**: Jest + MSW (Mock Service Worker)
- **E2E Tests**: Playwright or Cypress
- **Visual Regression**: Percy or Chromatic
- **Accessibility**: axe-core + jest-axe
- **Performance**: Lighthouse CI

### CI/CD Integration
```yaml
# .github/workflows/test.yml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
      - run: npm ci
      - run: npm run test
      - run: npm run test:integration
      - run: npm run test:e2e
      - run: npm run lint
      - run: npm run typecheck
```

## Regression Testing

### Before Each Release
- [ ] Run full test suite
- [ ] Manual smoke test on staging
- [ ] Cross-browser testing
- [ ] Mobile testing
- [ ] Accessibility audit
- [ ] Performance audit
- [ ] Security scan

### Critical User Journeys
1. **New User Journey**
   - Register → Verify email → Login → First analysis → View recommendations → Submit feedback

2. **Returning User Journey**
   - Login → View dashboard → Check progress → New analysis → View recommendations

3. **Power User Journey**
   - Login → Multiple analyses → Compare recommendations → Submit multiple feedbacks → View progress trends

## Test Environment Setup

### Local Development
```bash
# Frontend
cd apps/emotistream-web
npm install
cp .env.example .env.local
# Update .env.local with backend URL
npm run dev
```

### Environment Variables
```env
NEXT_PUBLIC_API_URL=http://localhost:3000/api/v1
NEXT_PUBLIC_WS_URL=ws://localhost:3000
NEXT_PUBLIC_ENABLE_ANALYTICS=false
```

### Mock Data
Use MSW to mock API responses for consistent testing:
```typescript
// mocks/handlers.ts
export const handlers = [
  rest.post('/api/v1/auth/login', (req, res, ctx) => {
    return res(ctx.json({ token: 'mock-jwt-token', user: { id: '1', email: 'test@example.com' } }))
  }),
  // More handlers...
]
```

---

**Document Version**: 1.0
**Last Updated**: 2025-12-06
**Status**: Ready for implementation
