/**
 * File System Manager
 * Handles file operations for .llm.md generation
 */

import { promises as fs } from 'fs';
import path from 'path';
import crypto from 'crypto';

interface CacheEntry {
  hash: string;
  timestamp: number;
}

export class FileManager {
  private cache: Map<string, CacheEntry> = new Map();

  /**
   * Write markdown content to file
   */
  async writeMarkdown(outputPath: string, content: string): Promise<void> {
    try {
      // Ensure output directory exists
      const dir = path.dirname(outputPath);
      await fs.mkdir(dir, { recursive: true });

      // Check if content has changed (for caching)
      const contentHash = this.hashContent(content);
      const cached = this.cache.get(outputPath);

      if (cached && cached.hash === contentHash) {
        // Content unchanged, skip write
        console.log(`[ARW] Skipping write for ${outputPath} (content unchanged)`);
        return;
      }

      // Write file
      await fs.writeFile(outputPath, content, 'utf-8');
      console.log(`[ARW] Wrote file: ${outputPath} (${content.length} bytes)`);

      // Update cache
      this.cache.set(outputPath, {
        hash: contentHash,
        timestamp: Date.now()
      });
    } catch (error) {
      console.error(`[ARW] Error writing ${outputPath}:`, error);
      throw error;
    }
  }

  /**
   * Check if a file exists
   */
  async fileExists(filePath: string): Promise<boolean> {
    try {
      await fs.access(filePath);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Read file content
   */
  async readFile(filePath: string): Promise<string> {
    try {
      return await fs.readFile(filePath, 'utf-8');
    } catch (error) {
      console.error(`[ARW] Error reading ${filePath}:`, error);
      throw error;
    }
  }

  /**
   * Delete a file
   */
  async deleteFile(filePath: string): Promise<void> {
    try {
      await fs.unlink(filePath);
      this.cache.delete(filePath);
    } catch (error) {
      // Ignore errors if file doesn't exist
      if ((error as any).code !== 'ENOENT') {
        console.error(`[ARW] Error deleting ${filePath}:`, error);
      }
    }
  }

  /**
   * Clean stale .llm.md files that no longer have corresponding source files
   */
  async cleanStaleFiles(
    outputDir: string,
    validPaths: string[]
  ): Promise<void> {
    try {
      const validSet = new Set(validPaths);
      const files = await this.listMarkdownFiles(outputDir);

      for (const file of files) {
        if (!validSet.has(file)) {
          console.log(`[ARW] Removing stale file: ${file}`);
          await this.deleteFile(file);
        }
      }
    } catch (error) {
      console.error('[ARW] Error cleaning stale files:', error);
    }
  }

  /**
   * List all .llm.md files in a directory
   */
  private async listMarkdownFiles(dir: string): Promise<string[]> {
    const files: string[] = [];

    try {
      const entries = await fs.readdir(dir, { withFileTypes: true });

      for (const entry of entries) {
        const fullPath = path.join(dir, entry.name);

        if (entry.isDirectory()) {
          // Recursively search subdirectories
          const subFiles = await this.listMarkdownFiles(fullPath);
          files.push(...subFiles);
        } else if (entry.name.endsWith('.llm.md')) {
          files.push(fullPath);
        }
      }
    } catch (error) {
      // Ignore errors if directory doesn't exist
      if ((error as any).code !== 'ENOENT') {
        console.error(`[ARW] Error listing files in ${dir}:`, error);
      }
    }

    return files;
  }

  /**
   * Hash content for cache comparison
   */
  private hashContent(content: string): string {
    return crypto.createHash('sha256').update(content).digest('hex');
  }

  /**
   * Clear cache
   */
  clearCache(): void {
    this.cache.clear();
  }

  /**
   * Get cache statistics
   */
  getCacheStats(): { size: number; entries: number } {
    const entries = Array.from(this.cache.values());
    return {
      size: entries.length,
      entries: entries.length
    };
  }
}
