/**
 * Build Processor
 * Post-build processing to generate .llm.md files
 * Build-system agnostic - works with both Webpack and Turbopack
 */

import { glob } from 'glob';
import { promises as fs } from 'fs';
import path from 'path';
import { ASTParser } from './ast-parser';
import { HTMLProcessor } from './html-processor';
import { MarkdownGenerator } from './markdown-generator';
import { FileManager } from '../utils/file-manager';

export interface BuildProcessorOptions {
  buildDir?: string;
  sourceDir?: string;
  outputDir?: string;
  includePatterns?: string[];
  excludePatterns?: string[];
  cleanStale?: boolean;
}

export class BuildProcessor {
  private astParser: ASTParser;
  private htmlProcessor: HTMLProcessor;
  private markdownGen: MarkdownGenerator;
  private fileManager: FileManager;
  private buildDir: string;
  private sourceDir: string;
  private outputDir: string;
  private includePatterns: string[];
  private excludePatterns: string[];
  private cleanStale: boolean;

  constructor(options: BuildProcessorOptions = {}) {
    this.astParser = new ASTParser();
    this.htmlProcessor = new HTMLProcessor();
    this.markdownGen = new MarkdownGenerator();
    this.fileManager = new FileManager();

    this.buildDir = options.buildDir || '.next';
    this.sourceDir = options.sourceDir || 'app';
    this.outputDir = options.outputDir || 'public';
    this.includePatterns = options.includePatterns || ['**/*.{tsx,jsx}'];
    this.excludePatterns = options.excludePatterns || [
      'node_modules/**',
      '.next/**',
      '**/*.test.*',
      '**/*.spec.*',
      '**/layout.{tsx,jsx}',
      '**/_*.{tsx,jsx}'
    ];
    this.cleanStale = options.cleanStale ?? true;
  }

  /**
   * Process all files with arwConfig
   */
  async processAll(): Promise<void> {
    console.log('[ARW] Starting .llm.md generation...');

    try {
      // Find all source files with arwConfig
      const sourceFiles = await this.findFilesWithARWConfig();

      if (sourceFiles.length === 0) {
        console.log('[ARW] No files with arwConfig found.');
        return;
      }

      console.log(`[ARW] Found ${sourceFiles.length} files with arwConfig`);

      const generatedPaths: string[] = [];

      // Process each file
      for (const sourceFile of sourceFiles) {
        const outputPath = await this.processFile(sourceFile);
        if (outputPath) {
          generatedPaths.push(outputPath);
        }
      }

      // Clean stale files if enabled
      if (this.cleanStale) {
        await this.fileManager.cleanStaleFiles(this.outputDir, generatedPaths);
      }

      console.log(`[ARW] Generation complete! Generated ${generatedPaths.length} files.`);
    } catch (error) {
      console.error('[ARW] Error during processing:', error);
      throw error;
    }
  }

  /**
   * Process a single file
   */
  async processFile(sourceFile: string): Promise<string | null> {
    try {
      // 1. Extract arwConfig from source file
      const config = await this.astParser.extractARWConfig(sourceFile);
      if (!config) {
        return null;
      }

      // 2. Find corresponding HTML file in .next/server
      const htmlPath = await this.findRenderedHTML(sourceFile);

      let markdown: string;

      if (htmlPath) {
        // 3. Extract chunks from HTML
        const html = await this.fileManager.readFile(htmlPath);
        const chunks = await this.htmlProcessor.extractChunks(html);

        // 4. Generate markdown with chunks
        markdown = this.markdownGen.generate(config, chunks);
      } else {
        // No HTML found, generate from config only
        console.warn(`[ARW] No HTML found for ${sourceFile}, generating from config only`);
        markdown = this.markdownGen.generateFromConfigOnly(config);
      }

      // 5. Write to output directory
      const outputPath = this.getOutputPath(sourceFile);
      await this.fileManager.writeMarkdown(outputPath, markdown);

      console.log(`[ARW] ✓ Generated ${outputPath}`);
      return outputPath;
    } catch (error) {
      console.error(`[ARW] Error processing ${sourceFile}:`, error);
      return null;
    }
  }

  /**
   * Process a single file with provided HTML content (for dev mode)
   */
  async processFileWithHTML(sourceFile: string, htmlContent: string): Promise<string | null> {
    try {
      // 1. Extract arwConfig from source file
      const config = await this.astParser.extractARWConfig(sourceFile);
      if (!config) {
        return null;
      }

      // 2. Extract chunks from provided HTML
      const chunks = await this.htmlProcessor.extractChunks(htmlContent);

      // 3. Generate markdown with chunks
      const markdown = this.markdownGen.generate(config, chunks);

      // 4. Write to output directory
      const outputPath = this.getOutputPath(sourceFile);
      await this.fileManager.writeMarkdown(outputPath, markdown);

      console.log(`[ARW] ✓ Generated ${outputPath}`);
      return outputPath;
    } catch (error) {
      console.error(`[ARW] Error processing ${sourceFile}:`, error);
      return null;
    }
  }

  /**
   * Find all source files with arwConfig export
   */
  private async findFilesWithARWConfig(): Promise<string[]> {
    const filesWithConfig: string[] = [];

    // Search in all potential source directories
    const searchDirs = [this.sourceDir, 'pages'].filter(async (dir) => {
      try {
        await fs.access(dir);
        return true;
      } catch {
        return false;
      }
    });

    for (const dir of searchDirs) {
      try {
        await fs.access(dir);
      } catch {
        continue; // Directory doesn't exist, skip
      }

      // Find all matching files
      const files = await glob(this.includePatterns, {
        cwd: dir,
        ignore: this.excludePatterns,
        absolute: false
      });

      // Check each file for arwConfig export
      for (const file of files) {
        const fullPath = path.join(dir, file);
        const config = await this.astParser.extractARWConfig(fullPath);

        if (config) {
          filesWithConfig.push(fullPath);
        }
      }
    }

    return filesWithConfig;
  }

  /**
   * Wait for HTML file to exist (for build-time generation)
   */
  private async waitForHTMLFile(htmlPath: string, maxWaitMs: number = 10000): Promise<boolean> {
    const startTime = Date.now();
    const pollInterval = 200; // Check every 200ms

    while (Date.now() - startTime < maxWaitMs) {
      if (await this.fileManager.fileExists(htmlPath)) {
        return true;
      }
      await new Promise(resolve => setTimeout(resolve, pollInterval));
    }

    return false;
  }

  /**
   * Find rendered HTML file for a source file
   * Handles both webpack and Turbopack build outputs
   */
  private async findRenderedHTML(sourceFile: string): Promise<string | null> {
    // Determine which directory the source is in (app or pages)
    const isAppRouter = sourceFile.startsWith('app');
    const baseDir = isAppRouter ? 'app' : 'pages';

    // Get relative path from base directory
    const relativePath = path.relative(baseDir, sourceFile);
    const withoutExt = relativePath.replace(/\.(tsx|jsx)$/, '');

    // For App Router: page.tsx files map to index.html
    // app/page.tsx -> .next/server/app/index.html
    // app/about/page.tsx -> .next/server/app/about/index.html
    let htmlBasePath = withoutExt;

    if (isAppRouter) {
      // Convert page.tsx to index.html pattern
      if (withoutExt === 'page') {
        htmlBasePath = 'index';
      } else if (withoutExt.endsWith('/page')) {
        htmlBasePath = withoutExt.replace(/\/page$/, '/index');
      }
    }

    // Try multiple potential HTML paths
    const potentialPaths = [
      // App Router patterns
      path.join(this.buildDir, 'server', 'app', htmlBasePath + '.html'),
      path.join(this.buildDir, 'server', 'app', htmlBasePath),
      path.join(this.buildDir, 'server', 'app', withoutExt + '.html'),
      path.join(this.buildDir, 'server', 'app', withoutExt, 'index.html'),

      // Pages Router patterns
      path.join(this.buildDir, 'server', 'pages', withoutExt + '.html'),
      path.join(this.buildDir, 'server', 'pages', withoutExt, 'index.html'),

      // Static export patterns
      path.join('out', htmlBasePath + '.html'),
      path.join('out', withoutExt + '.html'),
      path.join('out', withoutExt, 'index.html')
    ];

    // First, try immediate check for all paths
    for (const htmlPath of potentialPaths) {
      if (await this.fileManager.fileExists(htmlPath)) {
        console.log(`[ARW] Found HTML at: ${htmlPath}`);
        return htmlPath;
      }
    }

    // If not found immediately, poll for the most likely path
    // (for build-time generation where files may not exist yet)
    const mostLikelyPath = potentialPaths[0];
    console.log(`[ARW] Waiting for HTML file: ${mostLikelyPath}`);

    if (await this.waitForHTMLFile(mostLikelyPath, 10000)) {
      console.log(`[ARW] Found HTML at: ${mostLikelyPath}`);
      return mostLikelyPath;
    }

    console.warn(`[ARW] HTML not found for ${sourceFile}. Tried paths:`, potentialPaths.slice(0, 4));
    return null;
  }

  /**
   * Get output path for a source file
   */
  private getOutputPath(sourceFile: string): string {
    // Determine base directory
    const isAppRouter = sourceFile.startsWith('app');
    const baseDir = isAppRouter ? 'app' : 'pages';

    // Get relative path
    const relativePath = path.relative(baseDir, sourceFile);
    const withoutExt = relativePath.replace(/\.(tsx|jsx)$/, '');

    // Convert to output filename
    // app/page.tsx -> index.llm.md
    // app/about/page.tsx -> about.llm.md
    // pages/index.tsx -> index.llm.md
    // pages/about.tsx -> about.llm.md

    let fileName: string;

    if (withoutExt === 'page' || withoutExt === 'index') {
      fileName = 'index';
    } else if (withoutExt.endsWith('/page')) {
      fileName = withoutExt.replace(/\/page$/, '');
    } else {
      fileName = withoutExt;
    }

    return path.join(this.outputDir, fileName + '.llm.md');
  }
}
