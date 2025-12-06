import { Router, Request, Response } from 'express';

const router = Router();

// In-memory storage for user data (would be Firestore in production)
const userStorage: Map<string, {
  preferences: any;
  learning: any;
  history: any[];
  updatedAt: string;
}> = new Map();

// Initialize default user for demo
userStorage.set('demo-user', {
  preferences: {
    name: '',
    favoriteGenres: [],
    preferredMoods: []
  },
  learning: {
    moodSelections: {},
    genrePreferences: {},
    enrichmentCount: 0,
    validationCount: 0,
    searchQueries: [],
    lastUpdated: null
  },
  history: [],
  updatedAt: new Date().toISOString()
});

/**
 * GET /api/v1/user/:userId
 * Get all user data (preferences, learning, history)
 */
router.get('/:userId', (req: Request, res: Response) => {
  const { userId } = req.params;
  const userData = userStorage.get(userId);

  if (!userData) {
    // Create new user
    const newUser = {
      preferences: { name: '', favoriteGenres: [], preferredMoods: [] },
      learning: { moodSelections: {}, genrePreferences: {}, enrichmentCount: 0, validationCount: 0, searchQueries: [], lastUpdated: null },
      history: [],
      updatedAt: new Date().toISOString()
    };
    userStorage.set(userId, newUser);
    return res.json({ success: true, data: newUser });
  }

  res.json({ success: true, data: userData });
});

/**
 * PUT /api/v1/user/:userId/preferences
 * Update user preferences
 */
router.put('/:userId/preferences', (req: Request, res: Response) => {
  const { userId } = req.params;
  const preferences = req.body;

  let userData = userStorage.get(userId);
  if (!userData) {
    userData = {
      preferences: {},
      learning: { moodSelections: {}, genrePreferences: {}, enrichmentCount: 0, validationCount: 0, searchQueries: [], lastUpdated: null },
      history: [],
      updatedAt: new Date().toISOString()
    };
  }

  userData.preferences = { ...userData.preferences, ...preferences };
  userData.updatedAt = new Date().toISOString();
  userStorage.set(userId, userData);

  res.json({ success: true, data: userData.preferences, message: 'Preferences updated' });
});

/**
 * PUT /api/v1/user/:userId/learning
 * Update learning insights
 */
router.put('/:userId/learning', (req: Request, res: Response) => {
  const { userId } = req.params;
  const learning = req.body;

  let userData = userStorage.get(userId);
  if (!userData) {
    userData = {
      preferences: { name: '', favoriteGenres: [], preferredMoods: [] },
      learning: {},
      history: [],
      updatedAt: new Date().toISOString()
    };
  }

  userData.learning = { ...userData.learning, ...learning, lastUpdated: new Date().toISOString() };
  userData.updatedAt = new Date().toISOString();
  userStorage.set(userId, userData);

  res.json({ success: true, data: userData.learning, message: 'Learning insights updated' });
});

/**
 * GET /api/v1/user/:userId/history
 * Get watch history
 */
router.get('/:userId/history', (req: Request, res: Response) => {
  const { userId } = req.params;
  const userData = userStorage.get(userId);

  if (!userData) {
    return res.json({ success: true, data: [] });
  }

  res.json({ success: true, data: userData.history });
});

/**
 * POST /api/v1/user/:userId/history
 * Add item to watch history
 */
router.post('/:userId/history', (req: Request, res: Response) => {
  const { userId } = req.params;
  const historyItem = req.body;

  let userData = userStorage.get(userId);
  if (!userData) {
    userData = {
      preferences: { name: '', favoriteGenres: [], preferredMoods: [] },
      learning: { moodSelections: {}, genrePreferences: {}, enrichmentCount: 0, validationCount: 0, searchQueries: [], lastUpdated: null },
      history: [],
      updatedAt: new Date().toISOString()
    };
  }

  // Remove if already exists (to update position)
  userData.history = userData.history.filter((h: any) => h.id !== historyItem.id);

  // Add to front
  userData.history.unshift({
    ...historyItem,
    viewedAt: historyItem.viewedAt || new Date().toISOString()
  });

  // Keep only last 100 items
  if (userData.history.length > 100) {
    userData.history = userData.history.slice(0, 100);
  }

  userData.updatedAt = new Date().toISOString();
  userStorage.set(userId, userData);

  res.json({ success: true, data: userData.history, message: 'History updated' });
});

/**
 * DELETE /api/v1/user/:userId/history
 * Clear watch history
 */
router.delete('/:userId/history', (req: Request, res: Response) => {
  const { userId } = req.params;

  let userData = userStorage.get(userId);
  if (userData) {
    userData.history = [];
    userData.updatedAt = new Date().toISOString();
    userStorage.set(userId, userData);
  }

  res.json({ success: true, message: 'History cleared' });
});

/**
 * GET /api/v1/user/:userId/learning/summary
 * Get learning summary with insights
 */
router.get('/:userId/learning/summary', (req: Request, res: Response) => {
  const { userId } = req.params;
  const userData = userStorage.get(userId);

  if (!userData) {
    return res.json({
      success: true,
      data: {
        topMoods: [],
        topGenres: [],
        totalInteractions: 0,
        insights: []
      }
    });
  }

  const { learning } = userData;

  // Calculate top moods
  const topMoods = Object.entries(learning.moodSelections || {})
    .sort((a: any, b: any) => b[1] - a[1])
    .slice(0, 3)
    .map(([mood, count]) => ({ mood, count }));

  // Calculate top genres
  const topGenres = Object.entries(learning.genrePreferences || {})
    .sort((a: any, b: any) => b[1] - a[1])
    .slice(0, 5)
    .map(([genre, count]) => ({ genre, count }));

  // Calculate total interactions
  const moodSelectionCount = Object.values(learning.moodSelections || {}).reduce((a: number, b: any) => a + b, 0) as number;
  const totalInteractions = moodSelectionCount + (learning.enrichmentCount || 0) + (learning.validationCount || 0);

  // Generate insights
  const insights: string[] = [];
  if (topMoods.length > 0) {
    insights.push(`You tend to search for ${topMoods[0].mood} content most often`);
  }
  if (topGenres.length > 0) {
    insights.push(`${topGenres[0].genre} is your most explored genre`);
  }
  if (learning.enrichmentCount > 5) {
    insights.push(`You've used AI enrichment ${learning.enrichmentCount} times - power user!`);
  }
  if (userData.history.length > 10) {
    insights.push(`You've explored ${userData.history.length} titles so far`);
  }

  res.json({
    success: true,
    data: {
      topMoods,
      topGenres,
      totalInteractions,
      enrichmentCount: learning.enrichmentCount || 0,
      validationCount: learning.validationCount || 0,
      historyCount: userData.history.length,
      insights,
      lastUpdated: learning.lastUpdated
    }
  });
});

export default router;
