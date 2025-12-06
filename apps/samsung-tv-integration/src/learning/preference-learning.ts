/**
 * Preference Learning System
 *
 * On-device WASM-accelerated Q-Learning for TV content recommendations
 * Uses ReasoningBank-style pattern storage for successful viewing patterns
 */

import {
  ContentMetadata,
  ViewingSession,
  UserPreference,
  LearningAction,
  LearningState,
  LearningConfig,
  LearningConfigSchema,
  ViewingPattern,
  LearningFeedback,
  LearningStats,
  Genre,
  ContentType,
  Recommendation,
} from './types.js';
import {
  generateContentEmbedding,
  generateStateEmbedding,
  generatePreferenceEmbedding,
  cosineSimilarity,
  batchSimilarity,
  ContentEmbeddingCache,
} from './embeddings.js';

// Q-table for state-action values
interface QEntry {
  state: string; // serialized state
  action: LearningAction;
  qValue: number;
  visits: number;
  lastUpdate: number;
}

/**
 * On-device Preference Learning System
 * Implements Q-Learning with experience replay for content recommendations
 */
export class PreferenceLearningSystem {
  private config: LearningConfig;
  private qTable: Map<string, Map<LearningAction, QEntry>> = new Map();
  private experienceBuffer: Array<{
    state: LearningState;
    action: LearningAction;
    reward: number;
    nextState: LearningState;
  }> = [];
  private patterns: Map<string, ViewingPattern> = new Map();
  private sessions: ViewingSession[] = [];
  private preferences: UserPreference;
  private embeddingCache: ContentEmbeddingCache;
  private contentLibrary: Map<string, ContentMetadata> = new Map();
  private currentExplorationRate: number;
  private totalReward: number = 0;
  private episodeCount: number = 0;

  // All available actions
  private readonly ACTIONS: LearningAction[] = [
    'recommend_similar',
    'recommend_popular',
    'recommend_trending',
    'recommend_genre',
    'recommend_new_release',
    'recommend_continue_watching',
    'recommend_based_on_time',
    'explore_new_genre',
    'explore_new_type',
  ];

  constructor(config: Partial<LearningConfig> = {}) {
    this.config = LearningConfigSchema.parse(config);
    this.currentExplorationRate = this.config.explorationRate;
    this.preferences = {
      userId: 'default',
      favoriteGenres: [],
      favoriteTypes: [],
      preferredDuration: { min: 0, max: 180 },
      preferredTimeSlots: {},
      dislikedGenres: [],
      watchedContentIds: [],
    };
    this.embeddingCache = new ContentEmbeddingCache(this.config.memorySize);
  }

  /**
   * Get current learning state from context
   */
  getCurrentState(): LearningState {
    const now = new Date();
    const hour = now.getHours();
    const day = now.getDay();

    // Determine time of day
    let timeOfDay: 'morning' | 'afternoon' | 'evening' | 'night';
    if (hour >= 5 && hour < 12) timeOfDay = 'morning';
    else if (hour >= 12 && hour < 17) timeOfDay = 'afternoon';
    else if (hour >= 17 && hour < 21) timeOfDay = 'evening';
    else timeOfDay = 'night';

    // Get recent viewing history
    const recentSessions = this.sessions.slice(-10);
    const recentGenres = new Set<Genre>();
    const recentTypes = new Set<ContentType>();

    for (const session of recentSessions) {
      session.contentMetadata.genres.forEach(g => recentGenres.add(g));
      recentTypes.add(session.contentMetadata.type);
    }

    // Calculate average completion rate
    const avgCompletion = recentSessions.length > 0
      ? recentSessions.reduce((sum, s) => sum + s.completionRate, 0) / recentSessions.length
      : 0.5;

    return {
      timeOfDay,
      dayOfWeek: day === 0 || day === 6 ? 'weekend' : 'weekday',
      recentGenres: Array.from(recentGenres).slice(0, 5) as Genre[],
      recentTypes: Array.from(recentTypes).slice(0, 3) as ContentType[],
      sessionCount: this.sessions.length,
      avgCompletionRate: avgCompletion,
      lastContentId: recentSessions[recentSessions.length - 1]?.contentId,
    };
  }

  /**
   * Serialize state for Q-table lookup
   */
  private serializeState(state: LearningState): string {
    return JSON.stringify({
      t: state.timeOfDay,
      d: state.dayOfWeek,
      g: state.recentGenres.slice(0, 3).sort(),
      y: state.recentTypes.slice(0, 2).sort(),
      c: Math.floor(state.avgCompletionRate * 10) / 10,
    });
  }

  /**
   * Get Q-value for state-action pair
   */
  private getQValue(stateKey: string, action: LearningAction): number {
    const stateEntry = this.qTable.get(stateKey);
    if (!stateEntry) return 0;
    const entry = stateEntry.get(action);
    return entry?.qValue || 0;
  }

  /**
   * Set Q-value for state-action pair
   */
  private setQValue(stateKey: string, action: LearningAction, value: number): void {
    let stateEntry = this.qTable.get(stateKey);
    if (!stateEntry) {
      stateEntry = new Map();
      this.qTable.set(stateKey, stateEntry);
    }

    const existing = stateEntry.get(action);
    stateEntry.set(action, {
      state: stateKey,
      action,
      qValue: value,
      visits: (existing?.visits || 0) + 1,
      lastUpdate: Date.now(),
    });
  }

  /**
   * Select action using epsilon-greedy policy
   */
  selectAction(state: LearningState): LearningAction {
    const stateKey = this.serializeState(state);

    // Exploration: random action
    if (Math.random() < this.currentExplorationRate) {
      return this.ACTIONS[Math.floor(Math.random() * this.ACTIONS.length)];
    }

    // Exploitation: best action based on Q-values
    let bestAction = this.ACTIONS[0];
    let bestValue = -Infinity;

    for (const action of this.ACTIONS) {
      const qValue = this.getQValue(stateKey, action);
      if (qValue > bestValue) {
        bestValue = qValue;
        bestAction = action;
      }
    }

    return bestAction;
  }

  /**
   * Calculate reward from viewing session
   */
  calculateReward(session: ViewingSession): number {
    let reward = 0;

    // Completion rate is the primary signal (0 to 0.5)
    reward += session.completionRate * 0.5;

    // User rating if provided (0 to 0.3)
    if (session.userRating) {
      reward += (session.userRating / 5) * 0.3;
    } else {
      // Implicit rating based on completion
      reward += session.completionRate * 0.15;
    }

    // Watch duration relative to expected (0 to 0.1)
    const expectedDuration = session.contentMetadata.duration || 90;
    const durationRatio = Math.min(1, session.watchDuration / expectedDuration);
    reward += durationRatio * 0.1;

    // Engagement signals from implicit feedback (0 to 0.1)
    const { paused, rewound, fastForwarded } = session.implicit;
    // Rewinding suggests engagement, fast-forwarding suggests boredom
    reward += (rewound * 0.02 - fastForwarded * 0.02);
    reward = Math.max(0, Math.min(1, reward));

    return reward;
  }

  /**
   * Update Q-value using temporal difference learning
   */
  updateQValue(
    state: LearningState,
    action: LearningAction,
    reward: number,
    nextState: LearningState
  ): void {
    const stateKey = this.serializeState(state);
    const nextStateKey = this.serializeState(nextState);

    // Get current Q-value
    const currentQ = this.getQValue(stateKey, action);

    // Get max Q-value for next state
    let maxNextQ = 0;
    for (const a of this.ACTIONS) {
      maxNextQ = Math.max(maxNextQ, this.getQValue(nextStateKey, a));
    }

    // TD update: Q(s,a) = Q(s,a) + α * (r + γ * max Q(s',a') - Q(s,a))
    const newQ = currentQ + this.config.learningRate * (
      reward + this.config.discountFactor * maxNextQ - currentQ
    );

    this.setQValue(stateKey, action, newQ);
  }

  /**
   * Record a viewing session and learn from it
   */
  recordSession(session: ViewingSession, selectedAction: LearningAction): void {
    // Store session
    this.sessions.push(session);
    if (this.sessions.length > this.config.memorySize) {
      this.sessions.shift();
    }

    // Update watched content
    this.preferences.watchedContentIds.push(session.contentId);

    // Calculate reward
    const reward = this.calculateReward(session);
    this.totalReward += reward;
    this.episodeCount++;

    // Get states
    const currentState = this.getCurrentState();

    // Store experience for replay
    const experience = {
      state: { ...currentState, lastContentId: session.contentId },
      action: selectedAction,
      reward,
      nextState: currentState,
    };
    this.experienceBuffer.push(experience);
    if (this.experienceBuffer.length > this.config.memorySize) {
      this.experienceBuffer.shift();
    }

    // Update Q-value
    this.updateQValue(experience.state, selectedAction, reward, experience.nextState);

    // Update preferences based on positive feedback
    if (reward > 0.6) {
      this.updatePreferences(session);
    }

    // Store as successful pattern if high reward
    if (reward > 0.7) {
      this.storePattern(experience.state, selectedAction, reward);
    }

    // Decay exploration rate
    this.currentExplorationRate = Math.max(
      this.config.minExploration,
      this.currentExplorationRate * this.config.explorationDecay
    );
  }

  /**
   * Update user preferences based on positive session
   */
  private updatePreferences(session: ViewingSession): void {
    const { contentMetadata } = session;

    // Update favorite genres
    for (const genre of contentMetadata.genres) {
      if (!this.preferences.favoriteGenres.includes(genre)) {
        this.preferences.favoriteGenres.push(genre);
        if (this.preferences.favoriteGenres.length > 10) {
          this.preferences.favoriteGenres.shift();
        }
      }
    }

    // Update favorite types
    if (!this.preferences.favoriteTypes.includes(contentMetadata.type)) {
      this.preferences.favoriteTypes.push(contentMetadata.type);
      if (this.preferences.favoriteTypes.length > 5) {
        this.preferences.favoriteTypes.shift();
      }
    }

    // Update preferred time slots
    if (session.contextual) {
      const timeSlot = session.contextual.timeOfDay;
      if (!this.preferences.preferredTimeSlots[timeSlot]) {
        this.preferences.preferredTimeSlots[timeSlot] = [];
      }
      if (!this.preferences.preferredTimeSlots[timeSlot].includes(contentMetadata.type)) {
        this.preferences.preferredTimeSlots[timeSlot].push(contentMetadata.type);
      }
    }

    this.preferences.lastUpdated = new Date().toISOString();
  }

  /**
   * Store successful viewing pattern
   */
  private storePattern(state: LearningState, action: LearningAction, reward: number): void {
    const patternId = `${this.serializeState(state)}-${action}`;
    const existing = this.patterns.get(patternId);

    if (existing) {
      // Update existing pattern
      existing.occurrences++;
      existing.reward = (existing.reward * (existing.occurrences - 1) + reward) / existing.occurrences;
      existing.successRate = (existing.successRate * (existing.occurrences - 1) + (reward > 0.7 ? 1 : 0)) / existing.occurrences;
      existing.updatedAt = new Date().toISOString();
    } else {
      // Create new pattern
      const pattern: ViewingPattern = {
        patternId,
        state,
        action,
        reward,
        successRate: reward > 0.7 ? 1 : 0,
        occurrences: 1,
        embedding: Array.from(generateStateEmbedding(
          state.timeOfDay,
          state.dayOfWeek,
          state.recentGenres,
          state.recentTypes,
          state.sessionCount,
          state.avgCompletionRate
        )),
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      };
      this.patterns.set(patternId, pattern);
    }
  }

  /**
   * Experience replay - sample and learn from past experiences
   */
  experienceReplay(batchSize?: number): void {
    const size = batchSize || this.config.batchSize;
    if (this.experienceBuffer.length < size) return;

    // Random sampling
    for (let i = 0; i < size; i++) {
      const idx = Math.floor(Math.random() * this.experienceBuffer.length);
      const exp = this.experienceBuffer[idx];
      this.updateQValue(exp.state, exp.action, exp.reward, exp.nextState);
    }
  }

  /**
   * Add content to the library
   */
  addContent(content: ContentMetadata): void {
    this.contentLibrary.set(content.id, content);
    // Pre-compute embedding
    this.embeddingCache.getOrCompute(content);
  }

  /**
   * Add multiple contents to the library
   */
  addContents(contents: ContentMetadata[]): void {
    for (const content of contents) {
      this.addContent(content);
    }
  }

  /**
   * Get recommendations based on current state and learned policy
   */
  getRecommendations(count: number = 5): Recommendation[] {
    const state = this.getCurrentState();
    const action = this.selectAction(state);

    // Get candidate content based on action
    const candidates = this.getCandidates(action, state);

    // Score and rank candidates
    const scored = candidates.map(content => ({
      content,
      score: this.scoreContent(content, state, action),
    }));

    scored.sort((a, b) => b.score - a.score);

    // Return top recommendations
    return scored.slice(0, count).map(({ content, score }) => ({
      contentId: content.id,
      title: content.title,
      type: content.type,
      genres: content.genres,
      score,
      reason: this.getRecommendationReason(action, content),
      action,
      confidence: this.getActionConfidence(state, action),
      appId: content.appId,
    }));
  }

  /**
   * Get candidate content based on action
   */
  private getCandidates(action: LearningAction, state: LearningState): ContentMetadata[] {
    const allContent = Array.from(this.contentLibrary.values());
    const unwatched = allContent.filter(c => !this.preferences.watchedContentIds.includes(c.id));

    switch (action) {
      case 'recommend_similar':
        if (state.lastContentId) {
          const lastContent = this.contentLibrary.get(state.lastContentId);
          if (lastContent) {
            return this.findSimilarContent(lastContent, unwatched, 20);
          }
        }
        return unwatched.slice(0, 20);

      case 'recommend_popular':
        return [...unwatched].sort((a, b) => (b.popularity || 0) - (a.popularity || 0)).slice(0, 20);

      case 'recommend_trending':
        // Trending = popular + recent
        return [...unwatched]
          .filter(c => c.releaseYear && c.releaseYear >= new Date().getFullYear() - 1)
          .sort((a, b) => (b.popularity || 0) - (a.popularity || 0))
          .slice(0, 20);

      case 'recommend_genre':
        const favoriteGenre = this.preferences.favoriteGenres[0];
        if (favoriteGenre) {
          return unwatched.filter(c => c.genres.includes(favoriteGenre)).slice(0, 20);
        }
        return unwatched.slice(0, 20);

      case 'recommend_new_release':
        const currentYear = new Date().getFullYear();
        return [...unwatched]
          .filter(c => c.releaseYear === currentYear)
          .sort((a, b) => (b.rating || 0) - (a.rating || 0))
          .slice(0, 20);

      case 'recommend_continue_watching':
        // Content user started but didn't finish
        const partialWatched = this.sessions
          .filter(s => s.completionRate < 0.9 && s.completionRate > 0.1)
          .map(s => s.contentId);
        return allContent.filter(c => partialWatched.includes(c.id)).slice(0, 20);

      case 'recommend_based_on_time':
        const preferredTypes = this.preferences.preferredTimeSlots[state.timeOfDay] || [];
        if (preferredTypes.length > 0) {
          return unwatched.filter(c => preferredTypes.includes(c.type)).slice(0, 20);
        }
        return unwatched.slice(0, 20);

      case 'explore_new_genre':
        const watchedGenres = new Set(this.preferences.favoriteGenres);
        return unwatched.filter(c => !c.genres.some(g => watchedGenres.has(g))).slice(0, 20);

      case 'explore_new_type':
        const watchedTypes = new Set(this.preferences.favoriteTypes);
        return unwatched.filter(c => !watchedTypes.has(c.type)).slice(0, 20);

      default:
        return unwatched.slice(0, 20);
    }
  }

  /**
   * Find content similar to a reference using embeddings
   */
  private findSimilarContent(
    reference: ContentMetadata,
    candidates: ContentMetadata[],
    limit: number
  ): ContentMetadata[] {
    const refEmbedding = this.embeddingCache.getOrCompute(reference);
    const candidateEmbeddings = candidates.map(c => this.embeddingCache.getOrCompute(c));

    const similarities = batchSimilarity(refEmbedding, candidateEmbeddings, limit);
    return similarities.map(s => candidates[s.index]);
  }

  /**
   * Score content for ranking
   */
  private scoreContent(content: ContentMetadata, state: LearningState, action: LearningAction): number {
    let score = 0;

    // Base score from preference similarity
    const prefEmbedding = generatePreferenceEmbedding(
      this.preferences.favoriteGenres,
      this.preferences.favoriteTypes
    );
    const contentEmbedding = this.embeddingCache.getOrCompute(content);
    score += cosineSimilarity(prefEmbedding, contentEmbedding) * 0.4;

    // Genre match bonus
    const genreMatch = content.genres.filter(g =>
      this.preferences.favoriteGenres.includes(g)
    ).length / Math.max(1, content.genres.length);
    score += genreMatch * 0.2;

    // Rating boost
    score += ((content.rating || 5) / 10) * 0.15;

    // Popularity boost
    score += ((content.popularity || 50) / 100) * 0.1;

    // Contextual boost (time-appropriate content)
    if (state.timeOfDay === 'night' && content.genres.includes('thriller')) {
      score += 0.05;
    }
    if (state.dayOfWeek === 'weekend' && content.type === 'movie') {
      score += 0.05;
    }

    // Exploration bonus for diverse content
    if (action.startsWith('explore_')) {
      score += 0.1;
    }

    return Math.min(1, Math.max(0, score));
  }

  /**
   * Get human-readable recommendation reason
   */
  private getRecommendationReason(action: LearningAction, content: ContentMetadata): string {
    const reasons: Record<LearningAction, string> = {
      recommend_similar: `Similar to content you enjoyed`,
      recommend_popular: `Popular with viewers like you`,
      recommend_trending: `Trending right now`,
      recommend_genre: `Matches your favorite ${content.genres[0] || 'genre'}`,
      recommend_new_release: `New release you might like`,
      recommend_continue_watching: `Continue where you left off`,
      recommend_based_on_time: `Perfect for ${this.getCurrentState().timeOfDay} viewing`,
      explore_new_genre: `Discover something new in ${content.genres[0] || 'a new genre'}`,
      explore_new_type: `Try a different type of content`,
    };
    return reasons[action] || 'Recommended for you';
  }

  /**
   * Get confidence in selected action
   */
  private getActionConfidence(state: LearningState, action: LearningAction): number {
    const stateKey = this.serializeState(state);
    const stateEntry = this.qTable.get(stateKey);

    if (!stateEntry) return 0.5; // No data

    const entry = stateEntry.get(action);
    if (!entry) return 0.5;

    // Confidence based on visits and Q-value
    const visitConfidence = Math.min(1, entry.visits / 10);
    const valueConfidence = (entry.qValue + 1) / 2; // Normalize to 0-1

    return (visitConfidence + valueConfidence) / 2;
  }

  /**
   * Process feedback on a recommendation
   */
  processFeedback(feedback: LearningFeedback): void {
    // Calculate implicit reward from feedback
    let reward = 0;

    if (feedback.selected) {
      reward += 0.3; // User selected this recommendation

      if (feedback.completionRate !== undefined) {
        reward += feedback.completionRate * 0.4;
      }

      if (feedback.userRating !== undefined) {
        reward += (feedback.userRating / 5) * 0.3;
      }
    }

    // Update Q-value for this state-action
    const state = this.getCurrentState();
    const nextState = this.getCurrentState();
    this.updateQValue(state, feedback.action, reward, nextState);
  }

  /**
   * Get learning statistics
   */
  getStats(): LearningStats {
    const actionCounts = new Map<LearningAction, { count: number; totalReward: number }>();

    for (const exp of this.experienceBuffer) {
      const stats = actionCounts.get(exp.action) || { count: 0, totalReward: 0 };
      stats.count++;
      stats.totalReward += exp.reward;
      actionCounts.set(exp.action, stats);
    }

    const topActions = Array.from(actionCounts.entries())
      .map(([action, stats]) => ({
        action,
        count: stats.count,
        avgReward: stats.totalReward / stats.count,
      }))
      .sort((a, b) => b.avgReward - a.avgReward);

    return {
      totalSessions: this.sessions.length,
      totalPatterns: this.patterns.size,
      avgReward: this.episodeCount > 0 ? this.totalReward / this.episodeCount : 0,
      explorationRate: this.currentExplorationRate,
      topActions,
      learningProgress: Math.min(1, this.sessions.length / 100),
      lastTrainingTime: this.patterns.size > 0
        ? Array.from(this.patterns.values()).slice(-1)[0]?.updatedAt
        : undefined,
    };
  }

  /**
   * Get user preferences
   */
  getPreferences(): UserPreference {
    return { ...this.preferences };
  }

  /**
   * Export learned model for persistence
   */
  exportModel(): {
    qTable: Array<{ state: string; actions: Array<{ action: LearningAction; qValue: number; visits: number }> }>;
    patterns: ViewingPattern[];
    preferences: UserPreference;
    config: LearningConfig;
    stats: { totalReward: number; episodeCount: number; explorationRate: number };
  } {
    const qTableExport: Array<{
      state: string;
      actions: Array<{ action: LearningAction; qValue: number; visits: number }>
    }> = [];

    for (const [state, actions] of this.qTable.entries()) {
      const actionExport: Array<{ action: LearningAction; qValue: number; visits: number }> = [];
      for (const [action, entry] of actions.entries()) {
        actionExport.push({ action, qValue: entry.qValue, visits: entry.visits });
      }
      qTableExport.push({ state, actions: actionExport });
    }

    return {
      qTable: qTableExport,
      patterns: Array.from(this.patterns.values()),
      preferences: this.preferences,
      config: this.config,
      stats: {
        totalReward: this.totalReward,
        episodeCount: this.episodeCount,
        explorationRate: this.currentExplorationRate,
      },
    };
  }

  /**
   * Import learned model from persistence
   */
  importModel(model: ReturnType<typeof this.exportModel>): void {
    // Restore Q-table
    this.qTable.clear();
    for (const { state, actions } of model.qTable) {
      const actionMap = new Map<LearningAction, QEntry>();
      for (const { action, qValue, visits } of actions) {
        actionMap.set(action, { state, action, qValue, visits, lastUpdate: Date.now() });
      }
      this.qTable.set(state, actionMap);
    }

    // Restore patterns
    this.patterns.clear();
    for (const pattern of model.patterns) {
      this.patterns.set(pattern.patternId, pattern);
    }

    // Restore preferences
    this.preferences = model.preferences;

    // Restore stats
    this.totalReward = model.stats.totalReward;
    this.episodeCount = model.stats.episodeCount;
    this.currentExplorationRate = model.stats.explorationRate;
  }
}
