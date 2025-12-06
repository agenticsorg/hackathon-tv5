# EmotiStream Frontend Test Templates

Complete test templates ready to use when the frontend is implemented.

## Integration Test Templates

### 1. API Client Tests

File: `apps/emotistream-web/src/__tests__/integration/api-client.test.ts`

```typescript
import { apiClient } from '@/lib/api/client'
import MockAdapter from 'axios-mock-adapter'

describe('API Client', () => {
  let mock: MockAdapter

  beforeEach(() => {
    mock = new MockAdapter(apiClient)
  })

  afterEach(() => {
    mock.restore()
  })

  test('configures base URL correctly', () => {
    expect(apiClient.defaults.baseURL).toBe(process.env.NEXT_PUBLIC_API_URL)
  })

  test('includes auth token in headers when available', () => {
    const token = 'test-jwt-token'
    localStorage.setItem('auth_token', token)

    // Re-create client to pick up token
    const { apiClient: newClient } = require('@/lib/api/client')

    expect(newClient.defaults.headers.common['Authorization']).toBe(`Bearer ${token}`)
  })

  test('handles 401 unauthorized by clearing token', async () => {
    localStorage.setItem('auth_token', 'invalid-token')
    mock.onGet('/test').reply(401, { error: 'Unauthorized' })

    await expect(apiClient.get('/test')).rejects.toThrow()
    expect(localStorage.getItem('auth_token')).toBeNull()
  })

  test('retries failed requests up to 3 times', async () => {
    let attempts = 0
    mock.onGet('/test').reply(() => {
      attempts++
      if (attempts < 3) {
        return [500, { error: 'Server error' }]
      }
      return [200, { success: true }]
    })

    const response = await apiClient.get('/test')
    expect(attempts).toBe(3)
    expect(response.data).toEqual({ success: true })
  })

  test('transforms error responses correctly', async () => {
    mock.onPost('/test').reply(400, {
      error: 'VALIDATION_ERROR',
      message: 'Invalid input',
      details: { field: 'email' }
    })

    try {
      await apiClient.post('/test', {})
    } catch (error: any) {
      expect(error.code).toBe('VALIDATION_ERROR')
      expect(error.message).toBe('Invalid input')
      expect(error.details).toEqual({ field: 'email' })
    }
  })
})
```

### 2. Authentication Flow Tests

File: `apps/emotistream-web/src/__tests__/integration/auth-flow.test.ts`

```typescript
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { AuthProvider, useAuth } from '@/lib/hooks/useAuth'
import { apiClient } from '@/lib/api/client'
import MockAdapter from 'axios-mock-adapter'

describe('Authentication Flow', () => {
  let mock: MockAdapter

  beforeEach(() => {
    mock = new MockAdapter(apiClient)
    localStorage.clear()
  })

  afterEach(() => {
    mock.restore()
  })

  test('registers new user successfully', async () => {
    mock.onPost('/auth/register').reply(201, {
      success: true,
      data: {
        token: 'new-jwt-token',
        user: { id: '1', email: 'newuser@example.com', name: 'New User' }
      }
    })

    const TestComponent = () => {
      const { register } = useAuth()
      return (
        <button onClick={() => register('newuser@example.com', 'password123', 'New User')}>
          Register
        </button>
      )
    }

    render(
      <AuthProvider>
        <TestComponent />
      </AuthProvider>
    )

    fireEvent.click(screen.getByText('Register'))

    await waitFor(() => {
      expect(localStorage.getItem('auth_token')).toBe('new-jwt-token')
    })
  })

  test('logs in existing user', async () => {
    mock.onPost('/auth/login').reply(200, {
      success: true,
      data: {
        token: 'login-jwt-token',
        user: { id: '1', email: 'test@example.com', name: 'Test User' }
      }
    })

    const TestComponent = () => {
      const { login } = useAuth()
      return (
        <button onClick={() => login('test@example.com', 'password123')}>
          Login
        </button>
      )
    }

    render(
      <AuthProvider>
        <TestComponent />
      </AuthProvider>
    )

    fireEvent.click(screen.getByText('Login'))

    await waitFor(() => {
      expect(localStorage.getItem('auth_token')).toBe('login-jwt-token')
    })
  })

  test('includes token in subsequent requests', async () => {
    localStorage.setItem('auth_token', 'test-token')

    mock.onGet('/protected').reply((config) => {
      const authHeader = config.headers?.Authorization
      if (authHeader === 'Bearer test-token') {
        return [200, { success: true }]
      }
      return [401, { error: 'Unauthorized' }]
    })

    const response = await apiClient.get('/protected')
    expect(response.data.success).toBe(true)
  })

  test('redirects to dashboard after login', async () => {
    mock.onPost('/auth/login').reply(200, {
      success: true,
      data: { token: 'token', user: { id: '1' } }
    })

    // Test with router mock
    const mockPush = jest.fn()
    jest.mock('next/navigation', () => ({
      useRouter: () => ({ push: mockPush })
    }))

    const TestComponent = () => {
      const { login } = useAuth()
      const router = useRouter()

      const handleLogin = async () => {
        await login('test@example.com', 'password123')
        router.push('/dashboard')
      }

      return <button onClick={handleLogin}>Login</button>
    }

    render(
      <AuthProvider>
        <TestComponent />
      </AuthProvider>
    )

    fireEvent.click(screen.getByText('Login'))

    await waitFor(() => {
      expect(mockPush).toHaveBeenCalledWith('/dashboard')
    })
  })

  test('logs out and clears token', async () => {
    localStorage.setItem('auth_token', 'test-token')

    mock.onPost('/auth/logout').reply(200, { success: true })

    const TestComponent = () => {
      const { logout } = useAuth()
      return <button onClick={logout}>Logout</button>
    }

    render(
      <AuthProvider>
        <TestComponent />
      </AuthProvider>
    )

    fireEvent.click(screen.getByText('Logout'))

    await waitFor(() => {
      expect(localStorage.getItem('auth_token')).toBeNull()
    })
  })
})
```

### 3. Emotion Analysis Flow Tests

File: `apps/emotistream-web/src/__tests__/integration/emotion-flow.test.ts`

```typescript
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { EmotionAnalyzer } from '@/components/emotion/emotion-analyzer'
import { apiClient } from '@/lib/api/client'
import MockAdapter from 'axios-mock-adapter'

describe('Emotion Analysis Flow', () => {
  let mock: MockAdapter

  beforeEach(() => {
    mock = new MockAdapter(apiClient)
  })

  afterEach(() => {
    mock.restore()
  })

  test('submits text for analysis and displays results', async () => {
    const mockResponse = {
      success: true,
      data: {
        emotionalState: {
          valence: 0.7,
          arousal: 0.6,
          stressLevel: 0.3,
          primaryEmotion: 'joy',
          confidence: 0.9,
          timestamp: Date.now()
        },
        desiredState: {
          targetValence: 0.8,
          targetArousal: 0.5,
          targetStress: 0.2,
          intensity: 'moderate',
          reasoning: 'User seeks to enhance positive emotions'
        }
      }
    }

    mock.onPost('/emotion/analyze').reply(200, mockResponse)

    render(<EmotionAnalyzer />)

    const input = screen.getByPlaceholderText(/how are you feeling/i)
    fireEvent.change(input, {
      target: { value: 'I am feeling really great and excited about today!' }
    })

    const button = screen.getByRole('button', { name: /analyze/i })
    fireEvent.click(button)

    // Should show loading state
    await waitFor(() => {
      expect(screen.getByText(/analyzing/i)).toBeInTheDocument()
    })

    // Should display results
    await waitFor(() => {
      expect(screen.getByText(/valence.*0.7/i)).toBeInTheDocument()
      expect(screen.getByText(/arousal.*0.6/i)).toBeInTheDocument()
      expect(screen.getByText(/joy/i)).toBeInTheDocument()
    })
  })

  test('displays loading state during analysis', async () => {
    mock.onPost('/emotion/analyze').reply(() => {
      return new Promise(resolve => {
        setTimeout(() => resolve([200, { success: true }]), 1000)
      })
    })

    render(<EmotionAnalyzer />)

    const input = screen.getByPlaceholderText(/how are you feeling/i)
    fireEvent.change(input, { target: { value: 'I feel amazing!' } })

    fireEvent.click(screen.getByRole('button', { name: /analyze/i }))

    expect(screen.getByRole('button', { name: /analyzing/i })).toBeDisabled()
    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument()
  })

  test('updates mood ring visualization with results', async () => {
    const mockResponse = {
      success: true,
      data: {
        emotionalState: {
          valence: 0.8,
          arousal: 0.7,
          stressLevel: 0.2,
          primaryEmotion: 'joy',
          confidence: 0.95
        }
      }
    }

    mock.onPost('/emotion/analyze').reply(200, mockResponse)

    render(<EmotionAnalyzer />)

    const input = screen.getByPlaceholderText(/how are you feeling/i)
    fireEvent.change(input, { target: { value: 'Super excited!' } })
    fireEvent.click(screen.getByRole('button', { name: /analyze/i }))

    await waitFor(() => {
      const moodIndicator = screen.getByTestId('mood-indicator')
      expect(moodIndicator).toHaveAttribute('data-valence', '0.8')
      expect(moodIndicator).toHaveAttribute('data-arousal', '0.7')
    })
  })

  test('handles API errors gracefully', async () => {
    mock.onPost('/emotion/analyze').reply(500, {
      error: 'GEMINI_API_ERROR',
      message: 'Gemini API temporarily unavailable'
    })

    render(<EmotionAnalyzer />)

    const input = screen.getByPlaceholderText(/how are you feeling/i)
    fireEvent.change(input, { target: { value: 'I feel amazing!' } })
    fireEvent.click(screen.getByRole('button', { name: /analyze/i }))

    await waitFor(() => {
      expect(screen.getByText(/temporarily unavailable/i)).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /try again/i })).toBeInTheDocument()
    })
  })

  test('validates minimum text length', () => {
    render(<EmotionAnalyzer />)

    const input = screen.getByPlaceholderText(/how are you feeling/i)
    const button = screen.getByRole('button', { name: /analyze/i })

    fireEvent.change(input, { target: { value: 'Short' } })

    expect(button).toBeDisabled()
    expect(screen.getByText(/at least 10 characters/i)).toBeInTheDocument()
  })
})
```

### 4. Recommendation Flow Tests

File: `apps/emotistream-web/src/__tests__/integration/recommendation-flow.test.ts`

```typescript
import { render, screen, waitFor } from '@testing-library/react'
import { RecommendationList } from '@/components/recommendations/recommendation-list'
import { apiClient } from '@/lib/api/client'
import MockAdapter from 'axios-mock-adapter'

describe('Recommendation Flow', () => {
  let mock: MockAdapter

  beforeEach(() => {
    mock = new MockAdapter(apiClient)
  })

  afterEach(() => {
    mock.restore()
  })

  test('fetches and displays recommendations', async () => {
    const mockRecommendations = {
      success: true,
      data: {
        recommendations: [
          {
            contentId: 'movie-1',
            title: 'The Matrix',
            qValue: 0.85,
            similarityScore: 0.92,
            combinedScore: 0.88,
            isExploration: false,
            predictedOutcome: {
              expectedValence: 0.7,
              expectedArousal: 0.8,
              expectedStress: 0.3,
              confidence: 0.9
            }
          },
          {
            contentId: 'movie-2',
            title: 'Inception',
            qValue: 0.45,
            similarityScore: 0.88,
            combinedScore: 0.66,
            isExploration: true,
            predictedOutcome: {
              expectedValence: 0.6,
              expectedArousal: 0.9,
              expectedStress: 0.4,
              confidence: 0.7
            }
          }
        ]
      }
    }

    mock.onPost('/recommend').reply(200, mockRecommendations)

    render(
      <RecommendationList
        emotionalState={{
          valence: 0.5,
          arousal: 0.6,
          stressLevel: 0.4,
          primaryEmotion: 'joy'
        }}
        desiredState={{
          targetValence: 0.7,
          targetArousal: 0.5,
          targetStress: 0.3
        }}
      />
    )

    // Should show loading state
    expect(screen.getByTestId('loading-skeleton')).toBeInTheDocument()

    // Should display recommendations
    await waitFor(() => {
      expect(screen.getByText('The Matrix')).toBeInTheDocument()
      expect(screen.getByText('Inception')).toBeInTheDocument()
    })

    // Check exploration badge
    const inceptionCard = screen.getByText('Inception').closest('[data-testid="recommendation-card"]')
    expect(inceptionCard).toContainElement(screen.getByText(/exploration/i))
  })

  test('shows exploration badges correctly', async () => {
    const mockRecommendations = {
      success: true,
      data: {
        recommendations: [
          { contentId: '1', title: 'Movie 1', isExploration: true, combinedScore: 0.7 },
          { contentId: '2', title: 'Movie 2', isExploration: false, combinedScore: 0.9 }
        ]
      }
    }

    mock.onPost('/recommend').reply(200, mockRecommendations)

    render(<RecommendationList emotionalState={{}} desiredState={{}} />)

    await waitFor(() => {
      const explorationBadges = screen.getAllByText(/exploration/i)
      expect(explorationBadges).toHaveLength(1)
    })
  })

  test('handles empty results gracefully', async () => {
    mock.onPost('/recommend').reply(200, {
      success: true,
      data: { recommendations: [] }
    })

    render(<RecommendationList emotionalState={{}} desiredState={{}} />)

    await waitFor(() => {
      expect(screen.getByText(/no recommendations/i)).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /try again/i })).toBeInTheDocument()
    })
  })

  test('retries on error', async () => {
    let attempts = 0
    mock.onPost('/recommend').reply(() => {
      attempts++
      if (attempts < 2) {
        return [500, { error: 'Server error' }]
      }
      return [200, { success: true, data: { recommendations: [] } }]
    })

    render(<RecommendationList emotionalState={{}} desiredState={{}} />)

    await waitFor(() => {
      expect(attempts).toBeGreaterThan(1)
    })
  })
})
```

### 5. Feedback Flow Tests

File: `apps/emotistream-web/src/__tests__/integration/feedback-flow.test.ts`

```typescript
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { FeedbackModal } from '@/components/feedback/feedback-modal'
import { apiClient } from '@/lib/api/client'
import MockAdapter from 'axios-mock-adapter'

describe('Feedback Flow', () => {
  let mock: MockAdapter

  beforeEach(() => {
    mock = new MockAdapter(apiClient)
  })

  afterEach(() => {
    mock.restore()
  })

  const mockData = {
    contentId: 'movie-1',
    contentTitle: 'The Matrix',
    beforeState: {
      valence: 0.5,
      arousal: 0.5,
      stressLevel: 0.5,
      primaryEmotion: 'neutral'
    },
    userId: 'user-1'
  }

  test('opens modal and displays before/after comparison', () => {
    render(<FeedbackModal isOpen={true} onClose={jest.fn()} {...mockData} />)

    expect(screen.getByText('The Matrix')).toBeInTheDocument()
    expect(screen.getByText(/before watching/i)).toBeInTheDocument()
    expect(screen.getByText(/after watching/i)).toBeInTheDocument()
  })

  test('submits star rating and completion status', async () => {
    const mockResponse = {
      success: true,
      data: {
        reward: 0.8,
        message: '🎉 Excellent choice!',
        confetti: true
      }
    }

    mock.onPost('/feedback').reply(200, mockResponse)

    render(<FeedbackModal isOpen={true} onClose={jest.fn()} {...mockData} />)

    // Set after emotion state
    fireEvent.change(screen.getByLabelText(/valence/i), { target: { value: '0.8' } })
    fireEvent.change(screen.getByLabelText(/arousal/i), { target: { value: '0.7' } })

    // Click star rating (4 stars)
    const stars = screen.getAllByTestId('star')
    fireEvent.click(stars[3])

    // Toggle completion
    fireEvent.click(screen.getByLabelText(/completed/i))

    // Submit
    fireEvent.click(screen.getByRole('button', { name: /submit/i }))

    await waitFor(() => {
      expect(screen.getByText(/excellent choice/i)).toBeInTheDocument()
      expect(screen.getByText(/0.8/i)).toBeInTheDocument()
    })
  })

  test('calculates and displays reward', async () => {
    const mockResponse = {
      success: true,
      data: {
        reward: 0.75,
        components: {
          directionAlignment: 0.8,
          magnitude: 0.7,
          proximityBonus: 0.1,
          completionPenalty: 0
        }
      }
    }

    mock.onPost('/feedback').reply(200, mockResponse)

    render(<FeedbackModal isOpen={true} onClose={jest.fn()} {...mockData} />)

    // Fill form and submit
    fireEvent.change(screen.getByLabelText(/valence/i), { target: { value: '0.7' } })
    fireEvent.click(screen.getByRole('button', { name: /submit/i }))

    await waitFor(() => {
      expect(screen.getByText(/reward.*0.75/i)).toBeInTheDocument()
    })
  })

  test('shows reward animation for high scores', async () => {
    const mockResponse = {
      success: true,
      data: { reward: 0.9, confetti: true }
    }

    mock.onPost('/feedback').reply(200, mockResponse)

    render(<FeedbackModal isOpen={true} onClose={jest.fn()} {...mockData} />)

    fireEvent.change(screen.getByLabelText(/valence/i), { target: { value: '0.9' } })
    fireEvent.click(screen.getByRole('button', { name: /submit/i }))

    await waitFor(() => {
      expect(screen.getByTestId('confetti-animation')).toBeInTheDocument()
    })
  })

  test('closes modal after successful submission', async () => {
    const onClose = jest.fn()
    mock.onPost('/feedback').reply(200, { success: true, data: { reward: 0.5 } })

    render(<FeedbackModal isOpen={true} onClose={onClose} {...mockData} />)

    fireEvent.change(screen.getByLabelText(/valence/i), { target: { value: '0.6' } })
    fireEvent.click(screen.getByRole('button', { name: /submit/i }))

    await waitFor(() => {
      expect(onClose).toHaveBeenCalled()
    })
  })

  test('shows error if submission fails', async () => {
    mock.onPost('/feedback').reply(500, {
      error: 'INTERNAL_ERROR',
      message: 'Failed to process feedback'
    })

    render(<FeedbackModal isOpen={true} onClose={jest.fn()} {...mockData} />)

    fireEvent.change(screen.getByLabelText(/valence/i), { target: { value: '0.7' } })
    fireEvent.click(screen.getByRole('button', { name: /submit/i }))

    await waitFor(() => {
      expect(screen.getByText(/failed to process/i)).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /try again/i })).toBeInTheDocument()
    })
  })
})
```

---

## Component Test Templates

See individual component test examples in QA_TEST_REPORT.md sections:
- EmotionInput Component
- MoodRing Component
- RecommendationCard Component

---

## E2E Test Template (Playwright)

File: `apps/emotistream-web/e2e/user-journey.spec.ts`

```typescript
import { test, expect } from '@playwright/test'

test.describe('Complete User Journey', () => {
  test('new user completes full flow', async ({ page }) => {
    // 1. Register
    await page.goto('/register')
    await page.fill('[name="email"]', 'newuser@example.com')
    await page.fill('[name="password"]', 'SecurePass123!')
    await page.fill('[name="name"]', 'New User')
    await page.click('button[type="submit"]')

    // 2. Redirected to dashboard
    await expect(page).toHaveURL('/dashboard')
    await expect(page.locator('h1')).toContainText('Welcome')

    // 3. Perform emotion analysis
    await page.goto('/analyze')
    await page.fill('[placeholder*="feeling"]', 'I am feeling absolutely amazing today! Everything is going perfectly!')
    await page.click('button:has-text("Analyze")')

    // 4. Wait for analysis
    await expect(page.locator('[data-testid="mood-ring"]')).toBeVisible()

    // 5. View recommendations
    await expect(page.locator('[data-testid="recommendation-card"]').first()).toBeVisible()

    // 6. Click "Watch Now"
    await page.locator('[data-testid="recommendation-card"]').first().locator('button:has-text("Watch Now")').click()

    // 7. Submit feedback
    await expect(page.locator('[data-testid="feedback-modal"]')).toBeVisible()
    await page.locator('[data-testid="star"]').nth(3).click() // 4 stars
    await page.locator('[name="completed"]').check()
    await page.click('button:has-text("Submit Feedback")')

    // 8. View progress
    await page.goto('/progress')
    await expect(page.locator('text=Total Experiences')).toBeVisible()
    await expect(page.locator('text=1')).toBeVisible() // First experience
  })
})
```

---

**Document Version**: 1.0
**Last Updated**: 2025-12-06
**Usage**: Copy templates into frontend project when created
