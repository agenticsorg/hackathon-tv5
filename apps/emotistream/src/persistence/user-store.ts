import { FileStore } from './file-store.js';
import { v4 as uuidv4 } from 'uuid';

export interface User {
  id: string;
  email: string;
  password: string;
  displayName: string;
  dateOfBirth: string;
  createdAt: number;
  lastActive: number;
}

/**
 * User storage with file-based persistence
 */
export class UserStore {
  private store: FileStore<User>;
  private emailIndex: Map<string, string> = new Map(); // email -> userId

  constructor() {
    this.store = new FileStore<User>('users.json');
    this.buildEmailIndex();
  }

  /**
   * Build email index from stored users
   */
  private buildEmailIndex(): void {
    for (const [userId, user] of this.store.entries()) {
      this.emailIndex.set(user.email.toLowerCase(), userId);
    }
  }

  /**
   * Generate a new user ID
   */
  generateUserId(): string {
    return uuidv4();
  }

  /**
   * Create a new user
   */
  create(user: User): void {
    this.store.set(user.id, user);
    this.emailIndex.set(user.email.toLowerCase(), user.id);
  }

  /**
   * Get user by ID
   */
  getById(userId: string): User | undefined {
    return this.store.get(userId);
  }

  /**
   * Get user by email
   */
  getByEmail(email: string): User | undefined {
    const userId = this.emailIndex.get(email.toLowerCase());
    if (!userId) return undefined;
    return this.store.get(userId);
  }

  /**
   * Update user's last active timestamp
   */
  updateLastActive(userId: string): void {
    const user = this.store.get(userId);
    if (user) {
      user.lastActive = Date.now();
      this.store.set(userId, user);
    }
  }

  /**
   * Delete user
   */
  delete(userId: string): boolean {
    const user = this.store.get(userId);
    if (user) {
      this.emailIndex.delete(user.email.toLowerCase());
      return this.store.delete(userId);
    }
    return false;
  }

  /**
   * Force save to disk
   */
  flush(): void {
    this.store.flush();
  }
}
