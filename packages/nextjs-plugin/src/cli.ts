#!/usr/bin/env node

/**
 * ARW CLI Tool
 * Standalone command-line tool for generating .llm.md files
 */

import { BuildProcessor } from './generator/build-processor';
import { DevWatcher } from './generator/dev-watcher';

interface CLIOptions {
  buildDir?: string;
  sourceDir?: string;
  outputDir?: string;
  watch?: boolean;
}

async function parseArgs(): Promise<CLIOptions> {
  const args = process.argv.slice(2);
  const options: CLIOptions = {};

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];

    switch (arg) {
      case '--watch':
      case '-w':
        options.watch = true;
        break;

      case '--build-dir':
      case '-b':
        options.buildDir = args[++i];
        break;

      case '--source-dir':
      case '-s':
        options.sourceDir = args[++i];
        break;

      case '--output-dir':
      case '-o':
        options.outputDir = args[++i];
        break;

      case '--help':
      case '-h':
        printHelp();
        process.exit(0);
        break;

      default:
        if (arg.startsWith('-')) {
          console.error(`Unknown option: ${arg}`);
          printHelp();
          process.exit(1);
        }
    }
  }

  return options;
}

function printHelp(): void {
  console.log(`
ARW Generate - Generate .llm.md files from Next.js pages

Usage:
  arw-generate [options]

Options:
  -w, --watch              Watch mode for development
  -b, --build-dir <dir>    Next.js build directory (default: .next)
  -s, --source-dir <dir>   Source directory (default: app)
  -o, --output-dir <dir>   Output directory (default: public)
  -h, --help               Show this help message

Examples:
  arw-generate                    # Generate once
  arw-generate --watch            # Watch mode
  arw-generate --source-dir pages # Use pages router

Post-build usage:
  Add to package.json:
  {
    "scripts": {
      "postbuild": "arw-generate"
    }
  }
`);
}

async function main(): Promise<void> {
  try {
    const options = await parseArgs();

    console.log('[ARW] ARW Generate v0.1.0');

    // Create build processor
    const processor = new BuildProcessor({
      buildDir: options.buildDir,
      sourceDir: options.sourceDir,
      outputDir: options.outputDir
    });

    if (options.watch) {
      // Watch mode
      console.log('[ARW] Running in watch mode...');

      const watcher = new DevWatcher(processor);
      await watcher.start();

      // Keep process alive
      process.on('SIGINT', async () => {
        console.log('\n[ARW] Shutting down...');
        await watcher.stop();
        process.exit(0);
      });

      process.on('SIGTERM', async () => {
        await watcher.stop();
        process.exit(0);
      });

      console.log('[ARW] Press Ctrl+C to stop');
    } else {
      // One-time generation
      await processor.processAll();
      process.exit(0);
    }
  } catch (error) {
    console.error('[ARW] Fatal error:', error);
    process.exit(1);
  }
}

// Run CLI
main();
