import { test, expect } from '@playwright/test';

test.describe('EmotiStream User Journey', () => {
  test.describe('Landing Page', () => {
    test('should load landing page with correct content', async ({ page }) => {
      await page.goto('/');

      // Check title
      await expect(page).toHaveTitle(/EmotiStream/);

      // Check hero section
      await expect(page.locator('text=Understand Your Emotions')).toBeVisible();
      await expect(page.locator('text=AI-Powered Emotional Wellness')).toBeVisible();

      // Check navigation links (there are multiple login/register links)
      await expect(page.locator('a[href="/login"]').first()).toBeVisible();
      await expect(page.locator('a[href="/register"]').first()).toBeVisible();

      // Check feature cards
      await expect(page.locator('text=AI Emotion Detection')).toBeVisible();
      await expect(page.locator('h3:has-text("Personalized Recommendations")')).toBeVisible();
      await expect(page.locator('text=Track Your Progress')).toBeVisible();
    });

    test('should navigate to login page', async ({ page }) => {
      await page.goto('/');
      // Wait for page to be fully hydrated
      await page.waitForLoadState('networkidle');
      await page.locator('a[href="/login"]').first().click();
      await expect(page).toHaveURL('/login', { timeout: 10000 });
      await expect(page.locator('text=Welcome Back')).toBeVisible();
    });

    test('should navigate to register page', async ({ page }) => {
      await page.goto('/');
      // Wait for page to be fully hydrated
      await page.waitForLoadState('networkidle');
      await page.locator('a[href="/register"]').first().click();
      await expect(page).toHaveURL('/register', { timeout: 10000 });
      await expect(page.locator('h1:has-text("Create Account")')).toBeVisible();
    });

    test('should navigate to register via CTA button', async ({ page }) => {
      await page.goto('/');
      // Wait for page to be fully hydrated
      await page.waitForLoadState('networkidle');
      await page.locator('text=Start Your Journey').click();
      await expect(page).toHaveURL('/register', { timeout: 10000 });
    });
  });

  test.describe('Authentication Pages', () => {
    test('should show login form with all fields', async ({ page }) => {
      await page.goto('/login');

      await expect(page.locator('input[type="email"]')).toBeVisible();
      await expect(page.locator('input[type="password"]')).toBeVisible();
      await expect(page.locator('button:has-text("Sign In")')).toBeVisible();
      await expect(page.locator('a[href="/register"]')).toBeVisible();
    });

    test('should show register form with all fields', async ({ page }) => {
      await page.goto('/register');

      await expect(page.locator('input[placeholder="John Doe"]')).toBeVisible();
      await expect(page.locator('input[type="email"]')).toBeVisible();
      await expect(page.locator('input[type="password"]').first()).toBeVisible();
      await expect(page.locator('button:has-text("Create Account")')).toBeVisible();
    });

    test('should navigate between login and register', async ({ page }) => {
      await page.goto('/login');
      await page.click('a[href="/register"]');
      await expect(page).toHaveURL('/register');

      await page.click('a[href="/login"]');
      await expect(page).toHaveURL('/login');
    });
  });

  test.describe('Dashboard Page', () => {
    test('should load dashboard with emotion input', async ({ page }) => {
      await page.goto('/dashboard');

      // Check main components
      await expect(page.locator('text=Welcome back')).toBeVisible();
      await expect(page.locator('text=How are you feeling')).toBeVisible();
      await expect(page.locator('textarea')).toBeVisible();
      await expect(page.locator('button:has-text("Analyze")')).toBeVisible();
    });

    test('should show desired state selector', async ({ page }) => {
      await page.goto('/dashboard');

      // Check mood goal buttons
      await expect(page.locator('text=How do you want to feel')).toBeVisible();
      await expect(page.locator('button:has-text("Relax")')).toBeVisible();
      await expect(page.locator('button:has-text("Energize")')).toBeVisible();
    });

    test('should enable analyze button when text is entered', async ({ page }) => {
      await page.goto('/dashboard');

      const textarea = page.locator('textarea');
      const analyzeButton = page.locator('button:has-text("Analyze")');

      // Initially disabled
      await expect(analyzeButton).toBeDisabled();

      // Type enough text
      await textarea.fill('I am feeling stressed about my presentation tomorrow and a bit anxious about the outcome');

      // Should be enabled now
      await expect(analyzeButton).toBeEnabled();
    });

    test('should show character count', async ({ page }) => {
      await page.goto('/dashboard');

      const textarea = page.locator('textarea');
      await textarea.fill('Test message');

      // Should show character count
      await expect(page.locator('text=/\\d+.*characters/')).toBeVisible();
    });
  });

  test.describe('Progress Page', () => {
    test('should load progress page', async ({ page }) => {
      await page.goto('/progress');

      // Check navigation is highlighted
      await expect(page.locator('a[href="/progress"]')).toBeVisible();

      // Check for progress content (using the actual heading text)
      await expect(page.locator('text=Learning Progress')).toBeVisible();
    });
  });

  test.describe('Navigation', () => {
    test('should have working navigation between pages', async ({ page }) => {
      // Start at dashboard
      await page.goto('/dashboard');
      await expect(page.locator('a[href="/dashboard"]').first()).toBeVisible();

      // Go to progress
      await page.click('a[href="/progress"]');
      await expect(page).toHaveURL('/progress');

      // Go back to dashboard via link
      await page.click('a[href="/dashboard"]');
      await expect(page).toHaveURL('/dashboard');
    });

    test('should have EmotiStream logo that links to dashboard', async ({ page }) => {
      await page.goto('/dashboard');

      // Logo should be visible
      await expect(page.locator('text=EmotiStream').first()).toBeVisible();
    });
  });
});

test.describe('API Integration', () => {
  test('should have backend health check working', async ({ request }) => {
    const response = await request.get('http://localhost:4000/health');
    expect(response.ok()).toBeTruthy();
    const data = await response.json();
    expect(data.status).toBe('ok');
  });

  test('should analyze emotion via API', async ({ request }) => {
    const response = await request.post('http://localhost:4000/api/v1/emotion/analyze', {
      data: {
        text: 'I feel happy and excited today!',
        userId: 'test-user-e2e'
      }
    });
    expect(response.ok()).toBeTruthy();
    const data = await response.json();
    expect(data.success).toBe(true);
    expect(data.data.state).toBeDefined();
    expect(data.data.state.valence).toBeGreaterThan(0); // Happy = positive valence
  });

  test('should get recommendations via API', async ({ request }) => {
    const response = await request.post('http://localhost:4000/api/v1/recommend', {
      data: {
        userId: 'test-user-e2e',
        currentState: {
          valence: 0.5,
          arousal: 0.3,
          stressLevel: 0.2,
          primaryEmotion: 'neutral'
        },
        desiredState: {
          targetValence: 0.8,
          targetArousal: 0.5,
          targetStress: 0.1,
          intensity: 'moderate'
        }
      }
    });
    expect(response.ok()).toBeTruthy();
    const data = await response.json();
    expect(data.success).toBe(true);
    expect(data.data.recommendations).toBeDefined();
    expect(Array.isArray(data.data.recommendations)).toBe(true);
  });
});
