/**
 * Development Watcher
 * File system watcher for development mode
 * Build-system agnostic - uses chokidar instead of webpack HMR
 */

import type { BuildProcessor } from './build-processor';

export class DevWatcher {
  private watcher?: any; // chokidar.FSWatcher
  private processor: BuildProcessor;
  private watchPaths: string[];
  private devServerUrl: string;

  constructor(processor: BuildProcessor, watchPaths?: string[], devServerPort: number = 3000) {
    this.processor = processor;
    this.watchPaths = watchPaths || [
      'app/**/*.{tsx,jsx}',
      'pages/**/*.{tsx,jsx}'
    ];
    this.devServerUrl = `http://localhost:${devServerPort}`;
  }

  /**
   * Start watching for file changes
   */
  async start(): Promise<void> {
    try {
      // Dynamically import chokidar (optional dependency)
      const chokidar = await import('chokidar');

      console.log('[ARW] Starting development watcher...');

      this.watcher = chokidar.watch(this.watchPaths, {
        ignored: [
          '**/node_modules/**',
          '**/.next/**',
          '**/dist/**',
          '**/*.test.*',
          '**/*.spec.*'
        ],
        persistent: true,
        ignoreInitial: true,
        awaitWriteFinish: {
          stabilityThreshold: 500,
          pollInterval: 100
        }
      });

      // Handle file changes
      this.watcher.on('change', async (filePath: string) => {
        console.log(`[ARW] File changed: ${filePath}`);
        await this.handleFileChange(filePath);
      });

      // Handle new files
      this.watcher.on('add', async (filePath: string) => {
        console.log(`[ARW] File added: ${filePath}`);
        await this.handleFileChange(filePath);
      });

      // Handle deleted files
      this.watcher.on('unlink', (filePath: string) => {
        console.log(`[ARW] File deleted: ${filePath}`);
        // Could clean up corresponding .llm.md file here
      });

      // Handle errors
      this.watcher.on('error', (error: Error) => {
        console.error('[ARW] Watcher error:', error);
      });

      console.log('[ARW] Watcher started successfully');
    } catch (error) {
      if ((error as any).code === 'MODULE_NOT_FOUND') {
        console.warn(
          '[ARW] chokidar not found. Install it for development watching: npm install chokidar'
        );
      } else {
        console.error('[ARW] Failed to start watcher:', error);
        throw error;
      }
    }
  }

  /**
   * Stop watching
   */
  async stop(): Promise<void> {
    if (this.watcher) {
      await this.watcher.close();
      console.log('[ARW] Watcher stopped');
    }
  }

  /**
   * Handle file change event
   */
  private async handleFileChange(filePath: string): Promise<void> {
    try {
      // Wait a bit for file to be fully written
      await new Promise(resolve => setTimeout(resolve, 100));

      console.log(`[ARW] Processing ${filePath}`);

      // Try to fetch HTML from dev server for full content
      const htmlContent = await this.fetchDevServerHTML(filePath);

      if (htmlContent) {
        console.log(`[ARW] Fetched HTML from dev server (${htmlContent.length} bytes)`);
        await this.processor.processFileWithHTML(filePath, htmlContent);
      } else {
        // Fallback to config-only generation
        console.log(`[ARW] Dev server not ready, using config-only generation`);
        await this.processor.processFile(filePath);
      }
    } catch (error) {
      console.error(`[ARW] Error processing changed file ${filePath}:`, error);
    }
  }

  /**
   * Fetch rendered HTML from development server
   */
  private async fetchDevServerHTML(filePath: string): Promise<string | null> {
    try {
      // Convert file path to URL path
      const urlPath = this.filePathToURL(filePath);
      const url = `${this.devServerUrl}${urlPath}`;

      console.log(`[ARW] Fetching HTML from: ${url}`);

      // Make HTTP request to dev server
      const response = await fetch(url, {
        headers: {
          'User-Agent': 'ARW-Dev-Watcher'
        }
      });

      if (!response.ok) {
        console.warn(`[ARW] Failed to fetch ${url}: ${response.status}`);
        return null;
      }

      const html = await response.text();
      return html;
    } catch (error) {
      console.warn(`[ARW] Could not fetch HTML from dev server:`, (error as Error).message);
      return null;
    }
  }

  /**
   * Convert file path to URL path
   */
  private filePathToURL(filePath: string): string {
    // Normalize path separators
    const normalized = filePath.replace(/\\/g, '/');

    // Extract route from file path
    let route = '';

    if (normalized.includes('app/')) {
      // App Router: app/page.tsx -> /
      // App Router: app/about/page.tsx -> /about
      const afterApp = normalized.split('app/')[1];
      route = afterApp
        .replace(/page\.(tsx|jsx|ts|js)$/, '')
        .replace(/\/$/, '');
      route = route ? `/${route}` : '/';
    } else if (normalized.includes('pages/')) {
      // Pages Router: pages/index.tsx -> /
      // Pages Router: pages/about.tsx -> /about
      const afterPages = normalized.split('pages/')[1];
      route = afterPages
        .replace(/index\.(tsx|jsx|ts|js)$/, '')
        .replace(/\.(tsx|jsx|ts|js)$/, '');
      route = route ? `/${route}` : '/';
    }

    return route || '/';
  }

  /**
   * Check if watcher is running
   */
  isRunning(): boolean {
    return !!this.watcher;
  }
}
