/**
 * Feedback Persistence Store
 *
 * In-memory storage for feedback records with file-based persistence.
 * Will be replaced with AgentDB in future iterations.
 */

import fs from 'fs/promises';
import path from 'path';
import { v4 as uuidv4 } from 'uuid';
import { FeedbackRecord, FeedbackSubmission } from '../types/feedback.js';
import { DatabaseError } from '../utils/errors.js';
import { logger } from '../utils/logger.js';

export class FeedbackStore {
  private feedbackRecords: Map<string, FeedbackRecord>;
  private userFeedbackIndex: Map<string, Set<string>>; // userId -> feedbackIds
  private contentFeedbackIndex: Map<string, Set<string>>; // contentId -> feedbackIds
  private persistencePath: string;

  constructor(dataDir: string = './data') {
    this.feedbackRecords = new Map();
    this.userFeedbackIndex = new Map();
    this.contentFeedbackIndex = new Map();
    this.persistencePath = path.join(dataDir, 'feedback.json');
  }

  /**
   * Initialize store and load persisted data
   */
  async initialize(): Promise<void> {
    try {
      const data = await fs.readFile(this.persistencePath, 'utf-8');
      const records: FeedbackRecord[] = JSON.parse(data);

      for (const record of records) {
        // Convert date strings back to Date objects
        record.timestamp = new Date(record.timestamp);
        record.emotionBefore.timestamp = typeof record.emotionBefore.timestamp === 'number'
          ? record.emotionBefore.timestamp
          : new Date(record.emotionBefore.timestamp as unknown as string).getTime();
        record.emotionAfter.timestamp = typeof record.emotionAfter.timestamp === 'number'
          ? record.emotionAfter.timestamp
          : new Date(record.emotionAfter.timestamp as unknown as string).getTime();

        this.feedbackRecords.set(record.feedbackId, record);
        this.indexFeedback(record);
      }

      logger.info(`Loaded ${records.length} feedback records from disk`);
    } catch (error: any) {
      if (error.code === 'ENOENT') {
        logger.info('No existing feedback data found, starting fresh');
      } else {
        logger.error('Failed to load feedback data', { error });
        throw new DatabaseError('Failed to initialize feedback store');
      }
    }
  }

  /**
   * Save feedback submission
   */
  async saveFeedback(
    submission: FeedbackSubmission,
    reward: number,
    qValueBefore: number,
    qValueAfter: number
  ): Promise<FeedbackRecord> {
    const feedbackId = `fbk_${uuidv4()}`;

    const record: FeedbackRecord = {
      ...submission,
      feedbackId,
      reward,
      qValueBefore,
      qValueAfter,
      processed: true,
    };

    this.feedbackRecords.set(feedbackId, record);
    this.indexFeedback(record);

    // Persist to disk
    await this.persist();

    return record;
  }

  /**
   * Get feedback by ID
   */
  getFeedback(feedbackId: string): FeedbackRecord | undefined {
    return this.feedbackRecords.get(feedbackId);
  }

  /**
   * Get all feedback for a user
   */
  getUserFeedback(userId: string): FeedbackRecord[] {
    const feedbackIds = this.userFeedbackIndex.get(userId) || new Set();
    return Array.from(feedbackIds)
      .map(id => this.feedbackRecords.get(id)!)
      .filter(Boolean)
      .sort((a, b) => a.timestamp.getTime() - b.timestamp.getTime());
  }

  /**
   * Get all feedback for content
   */
  getContentFeedback(contentId: string): FeedbackRecord[] {
    const feedbackIds = this.contentFeedbackIndex.get(contentId) || new Set();
    return Array.from(feedbackIds)
      .map(id => this.feedbackRecords.get(id)!)
      .filter(Boolean)
      .sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime());
  }

  /**
   * Get recent feedback (last N records)
   */
  getRecentFeedback(limit: number = 10): FeedbackRecord[] {
    return Array.from(this.feedbackRecords.values())
      .sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime())
      .slice(0, limit);
  }

  /**
   * Get user's recent feedback
   */
  getUserRecentFeedback(userId: string, limit: number = 10): FeedbackRecord[] {
    return this.getUserFeedback(userId).slice(-limit);
  }

  /**
   * Count feedback records
   */
  count(): number {
    return this.feedbackRecords.size;
  }

  /**
   * Count user's feedback
   */
  countUserFeedback(userId: string): number {
    return (this.userFeedbackIndex.get(userId) || new Set()).size;
  }

  /**
   * Get all user IDs with feedback
   */
  getAllUserIds(): string[] {
    return Array.from(this.userFeedbackIndex.keys());
  }

  /**
   * Index feedback for fast lookups
   */
  private indexFeedback(record: FeedbackRecord): void {
    // User index
    if (!this.userFeedbackIndex.has(record.userId)) {
      this.userFeedbackIndex.set(record.userId, new Set());
    }
    this.userFeedbackIndex.get(record.userId)!.add(record.feedbackId);

    // Content index
    if (!this.contentFeedbackIndex.has(record.contentId)) {
      this.contentFeedbackIndex.set(record.contentId, new Set());
    }
    this.contentFeedbackIndex.get(record.contentId)!.add(record.feedbackId);
  }

  /**
   * Persist to disk
   */
  private async persist(): Promise<void> {
    try {
      // Ensure directory exists
      const dir = path.dirname(this.persistencePath);
      await fs.mkdir(dir, { recursive: true });

      // Write data
      const records = Array.from(this.feedbackRecords.values());
      await fs.writeFile(
        this.persistencePath,
        JSON.stringify(records, null, 2),
        'utf-8'
      );
    } catch (error) {
      logger.error('Failed to persist feedback data', { error });
      // Don't throw - we can continue without persistence
    }
  }

  /**
   * Clear all data (for testing)
   */
  async clear(): Promise<void> {
    this.feedbackRecords.clear();
    this.userFeedbackIndex.clear();
    this.contentFeedbackIndex.clear();
    await this.persist();
  }
}
