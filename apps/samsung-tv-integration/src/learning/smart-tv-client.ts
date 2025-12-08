/**
 * Smart TV Client
 *
 * Integrates Samsung TV control with self-learning recommendations
 * Tracks viewing sessions and learns user preferences automatically
 */

import { SamsungTVClient, createTVClient } from '../lib/tv-client.js';
import { SamsungTVDevice, TVApp, STREAMING_APPS } from '../lib/types.js';
import { PreferenceLearningSystem } from './preference-learning.js';
import { LearningPersistence } from './persistence.js';
import {
  ContentMetadata,
  ViewingSession,
  Recommendation,
  LearningStats,
  LearningAction,
  LearningFeedback,
  Genre,
  ContentType,
} from './types.js';

interface SmartTVConfig {
  autoLearn: boolean;
  autoSave: boolean;
  saveInterval: number; // minutes
  minSessionDuration: number; // minutes to count as session
}

const DEFAULT_CONFIG: SmartTVConfig = {
  autoLearn: true,
  autoSave: true,
  saveInterval: 5,
  minSessionDuration: 5,
};

/**
 * Smart TV Client with Self-Learning Capabilities
 */
export class SmartTVClient {
  private tvClient: SamsungTVClient;
  private learner: PreferenceLearningSystem;
  private persistence: LearningPersistence;
  private config: SmartTVConfig;

  private currentSession: Partial<ViewingSession> | null = null;
  private sessionStartTime: Date | null = null;
  private lastAction: LearningAction | null = null;
  private autoSaveTimer: ReturnType<typeof setInterval> | null = null;

  constructor(
    device: SamsungTVDevice,
    config: Partial<SmartTVConfig> = {}
  ) {
    this.tvClient = createTVClient(device);
    this.learner = new PreferenceLearningSystem();
    this.persistence = new LearningPersistence();
    this.config = { ...DEFAULT_CONFIG, ...config };

    // Load existing model
    this.loadState();

    // Start auto-save if enabled
    if (this.config.autoSave) {
      this.startAutoSave();
    }
  }

  /**
   * Connect to the TV
   */
  async connect(): Promise<{ success: boolean; token?: string; error?: string }> {
    return this.tvClient.connect();
  }

  /**
   * Disconnect from the TV
   */
  disconnect(): void {
    this.endCurrentSession();
    this.saveState();
    this.tvClient.disconnect();
    this.stopAutoSave();
  }

  /**
   * Get personalized recommendations
   */
  getRecommendations(count: number = 5): Recommendation[] {
    return this.learner.getRecommendations(count);
  }

  /**
   * Launch content and start tracking session
   */
  async launchContent(
    content: ContentMetadata,
    recommendation?: Recommendation
  ): Promise<{ success: boolean; error?: string }> {
    // End any existing session first
    this.endCurrentSession();

    // Determine app to launch
    const appId = content.appId || this.getAppForContent(content);
    if (!appId) {
      return { success: false, error: 'No app available for this content' };
    }

    // Launch the app
    const result = await this.tvClient.launchApp(appId);
    if (!result.success) {
      return result;
    }

    // Start tracking session
    this.startSession(content, recommendation?.action || 'recommend_similar');

    return { success: true };
  }

  /**
   * Launch a streaming app
   */
  async launchApp(appName: keyof typeof STREAMING_APPS): Promise<{ success: boolean; error?: string }> {
    return this.tvClient.launchStreamingApp(appName);
  }

  /**
   * Start a viewing session
   */
  private startSession(content: ContentMetadata, action: LearningAction): void {
    const now = new Date();
    const hour = now.getHours();
    const day = now.getDay();

    this.sessionStartTime = now;
    this.lastAction = action;

    this.currentSession = {
      id: `session-${Date.now()}`,
      contentId: content.id,
      contentMetadata: content,
      startTime: now.toISOString(),
      watchDuration: 0,
      completionRate: 0,
      implicit: {
        paused: 0,
        rewound: 0,
        fastForwarded: 0,
        volumeChanges: 0,
      },
      contextual: {
        timeOfDay: hour >= 5 && hour < 12 ? 'morning' :
                   hour >= 12 && hour < 17 ? 'afternoon' :
                   hour >= 17 && hour < 21 ? 'evening' : 'night',
        dayOfWeek: day === 0 || day === 6 ? 'weekend' : 'weekday',
      },
    };
  }

  /**
   * End the current viewing session
   */
  endCurrentSession(userRating?: number): ViewingSession | null {
    if (!this.currentSession || !this.sessionStartTime) {
      return null;
    }

    const now = new Date();
    const durationMinutes = (now.getTime() - this.sessionStartTime.getTime()) / (1000 * 60);

    // Don't record very short sessions
    if (durationMinutes < this.config.minSessionDuration) {
      this.currentSession = null;
      this.sessionStartTime = null;
      return null;
    }

    // Complete the session
    const session: ViewingSession = {
      ...this.currentSession as ViewingSession,
      endTime: now.toISOString(),
      watchDuration: durationMinutes,
      completionRate: Math.min(1, durationMinutes / (this.currentSession.contentMetadata?.duration || 90)),
      userRating,
    };

    // Record and learn from session
    if (this.config.autoLearn && this.lastAction) {
      this.learner.recordSession(session, this.lastAction);
    }

    // Reset
    this.currentSession = null;
    this.sessionStartTime = null;
    this.lastAction = null;

    return session;
  }

  /**
   * Record implicit feedback during viewing
   */
  recordImplicitFeedback(type: 'pause' | 'rewind' | 'fastForward' | 'volumeChange'): void {
    if (!this.currentSession?.implicit) return;

    switch (type) {
      case 'pause':
        this.currentSession.implicit.paused++;
        break;
      case 'rewind':
        this.currentSession.implicit.rewound++;
        break;
      case 'fastForward':
        this.currentSession.implicit.fastForwarded++;
        break;
      case 'volumeChange':
        this.currentSession.implicit.volumeChanges++;
        break;
    }
  }

  /**
   * Process explicit feedback on a recommendation
   */
  processFeedback(feedback: LearningFeedback): void {
    this.learner.processFeedback(feedback);
  }

  /**
   * Add content to the learning system
   */
  addContent(content: ContentMetadata): void {
    this.learner.addContent(content);
  }

  /**
   * Add multiple contents to the learning system
   */
  addContents(contents: ContentMetadata[]): void {
    this.learner.addContents(contents);
  }

  /**
   * Trigger experience replay for batch learning
   */
  trainModel(batchSize?: number): void {
    this.learner.experienceReplay(batchSize);
  }

  /**
   * Get learning statistics
   */
  getLearningStats(): LearningStats {
    return this.learner.getStats();
  }

  /**
   * Get user preferences
   */
  getPreferences() {
    return this.learner.getPreferences();
  }

  /**
   * Save current state to disk
   */
  saveState(): void {
    this.persistence.saveModel(this.learner);
  }

  /**
   * Load state from disk
   */
  loadState(): boolean {
    return this.persistence.loadModel(this.learner);
  }

  /**
   * Get storage statistics
   */
  getStorageStats() {
    return this.persistence.getStorageStats();
  }

  /**
   * Clear all learned data
   */
  clearLearning(): void {
    this.persistence.clearAll();
    this.learner = new PreferenceLearningSystem();
  }

  // TV Control passthrough methods

  async powerOn() {
    return this.tvClient.powerOn();
  }

  async powerOff() {
    this.endCurrentSession();
    return this.tvClient.powerOff();
  }

  async volumeUp(steps?: number) {
    this.recordImplicitFeedback('volumeChange');
    return this.tvClient.setVolume('up', steps);
  }

  async volumeDown(steps?: number) {
    this.recordImplicitFeedback('volumeChange');
    return this.tvClient.setVolume('down', steps);
  }

  async mute() {
    return this.tvClient.setVolume('mute');
  }

  async pause() {
    this.recordImplicitFeedback('pause');
    return this.tvClient.sendKey('KEY_PAUSE');
  }

  async play() {
    return this.tvClient.sendKey('KEY_PLAY');
  }

  async rewind() {
    this.recordImplicitFeedback('rewind');
    return this.tvClient.sendKey('KEY_REWIND');
  }

  async fastForward() {
    this.recordImplicitFeedback('fastForward');
    return this.tvClient.sendKey('KEY_FF');
  }

  async navigate(direction: 'up' | 'down' | 'left' | 'right' | 'enter' | 'back') {
    return this.tvClient.navigate(direction);
  }

  async goHome() {
    this.endCurrentSession();
    return this.tvClient.goHome();
  }

  async getApps() {
    return this.tvClient.getApps();
  }

  async getState() {
    return this.tvClient.getState();
  }

  // Private helpers

  private getAppForContent(content: ContentMetadata): string | null {
    if (content.appId) return content.appId;

    // Map content types to default apps
    const typeToApp: Partial<Record<ContentType, keyof typeof STREAMING_APPS>> = {
      movie: 'NETFLIX',
      tv_show: 'NETFLIX',
      music: 'SPOTIFY',
    };

    const appKey = typeToApp[content.type];
    return appKey ? STREAMING_APPS[appKey] : null;
  }

  private startAutoSave(): void {
    if (this.autoSaveTimer) return;

    this.autoSaveTimer = setInterval(() => {
      this.saveState();
    }, this.config.saveInterval * 60 * 1000);
  }

  private stopAutoSave(): void {
    if (this.autoSaveTimer) {
      clearInterval(this.autoSaveTimer);
      this.autoSaveTimer = null;
    }
  }
}

/**
 * Create a Smart TV Client
 */
export function createSmartTVClient(
  device: SamsungTVDevice,
  config?: Partial<SmartTVConfig>
): SmartTVClient {
  return new SmartTVClient(device, config);
}

/**
 * Create a Smart TV Client from IP
 */
export function createSmartTVClientFromIP(
  ip: string,
  options?: { port?: number; mac?: string; token?: string },
  config?: Partial<SmartTVConfig>
): SmartTVClient {
  const device: SamsungTVDevice = {
    id: `samsung-tv-${ip.replace(/\./g, '-')}`,
    name: `Samsung TV (${ip})`,
    ip,
    port: options?.port || 8002,
    mac: options?.mac,
    token: options?.token,
    isOnline: false,
  };
  return new SmartTVClient(device, config);
}
