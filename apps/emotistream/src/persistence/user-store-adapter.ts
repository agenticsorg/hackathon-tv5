/**
 * User Store Adapter
 *
 * Provides a unified interface for user storage that works with both
 * file-based storage (development) and PostgreSQL (production).
 */

import { UserStore, User } from './user-store.js';
import { getPostgresUserStore, PostgresUserStore } from './postgres-store.js';
import { v4 as uuidv4 } from 'uuid';

export interface IUserStore {
  generateUserId(): string;
  create(user: User): Promise<void>;
  getById(userId: string): Promise<User | undefined>;
  getByEmail(email: string): Promise<User | undefined>;
  updateLastActive(userId: string): Promise<void>;
}

/**
 * Adapter for file-based UserStore (makes it async-compatible)
 */
export class FileUserStoreAdapter implements IUserStore {
  private store: UserStore;

  constructor() {
    this.store = new UserStore();
  }

  generateUserId(): string {
    return this.store.generateUserId();
  }

  async create(user: User): Promise<void> {
    this.store.create(user);
  }

  async getById(userId: string): Promise<User | undefined> {
    return this.store.getById(userId);
  }

  async getByEmail(email: string): Promise<User | undefined> {
    return this.store.getByEmail(email);
  }

  async updateLastActive(userId: string): Promise<void> {
    this.store.updateLastActive(userId);
  }
}

/**
 * Adapter for PostgresUserStore (converts to User interface)
 */
export class PostgresUserStoreAdapter implements IUserStore {
  private store: PostgresUserStore;

  constructor() {
    this.store = getPostgresUserStore();
  }

  generateUserId(): string {
    return uuidv4();
  }

  async create(user: User): Promise<void> {
    await this.store.createUser(user.email, user.password, user.displayName);
  }

  async getById(userId: string): Promise<User | undefined> {
    const result = await this.store.findById(userId);
    if (!result) return undefined;

    return {
      id: result.id,
      email: result.email,
      password: '', // Password not returned for security
      displayName: result.displayName || '',
      dateOfBirth: '',
      createdAt: Date.now(),
      lastActive: Date.now(),
    };
  }

  async getByEmail(email: string): Promise<User | undefined> {
    const result = await this.store.findByEmail(email);
    if (!result) return undefined;

    return {
      id: result.id,
      email: result.email,
      password: result.passwordHash,
      displayName: result.displayName || '',
      dateOfBirth: '',
      createdAt: Date.now(),
      lastActive: Date.now(),
    };
  }

  async updateLastActive(userId: string): Promise<void> {
    // PostgreSQL doesn't have this yet, but we could add it
    // For now, this is a no-op
  }
}

/**
 * Get the appropriate user store based on environment
 */
export function getUserStoreAdapter(): IUserStore {
  const usePostgres = process.env.USE_POSTGRES === 'true';

  if (usePostgres) {
    console.log('Using PostgreSQL user store');
    return new PostgresUserStoreAdapter();
  }

  console.log('Using file-based user store');
  return new FileUserStoreAdapter();
}
