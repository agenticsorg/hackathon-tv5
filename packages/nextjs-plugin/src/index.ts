/**
 * ARW Next.js Plugin
 *
 * Seamless integration of Agent-Ready Web (ARW) into Next.js applications.
 * Automatically generates machine-readable content alongside React components.
 * Build-system agnostic - works with both Webpack and Turbopack.
 */

import type { NextConfig } from 'next';
import type { ARWNextConfig } from './types';
import { mergeConfig } from './config/defaults';
import { BuildProcessor } from './generator/build-processor';
import { DevWatcher } from './generator/dev-watcher';
import { GEOBuildProcessor } from './geo/build-processor';
import { mergeGEOConfig } from './geo/utils';

/**
 * Next.js plugin for ARW integration
 *
 * @example
 * ```javascript
 * // next.config.js
 * const { withARW } = require('@agent-ready-web/nextjs-plugin');
 *
 * module.exports = withARW({
 *   arw: {
 *     autoGenerate: true,
 *     manifest: {
 *       siteName: 'My Site',
 *       homepage: 'https://example.com',
 *       contact: 'ai@example.com'
 *     }
 *   }
 * });
 * ```
 */
export function withARW(userNextConfig: NextConfig & { arw?: ARWNextConfig } = {}): NextConfig {
  const { arw: arwConfig, ...nextConfig } = userNextConfig;

  if (!arwConfig) {
    console.warn('[@agent-ready-web/nextjs-plugin] No ARW configuration provided. Plugin will be inactive.');
    return nextConfig;
  }

  const config = mergeConfig(arwConfig);

  // Validate required manifest fields
  if (!config.manifest.siteName || !config.manifest.homepage || !config.manifest.contact) {
    throw new Error(
      '[@agent-ready-web/nextjs-plugin] Missing required manifest fields: siteName, homepage, and contact are required.'
    );
  }

  // Initialize build processor and watcher if autoGenerate is enabled
  let processor: BuildProcessor | null = null;
  let geoProcessor: GEOBuildProcessor | null = null;

  if (config.autoGenerate) {
    processor = new BuildProcessor({
      buildDir: config.generation?.buildDir,
      sourceDir: config.generation?.sourceDir,
      outputDir: config.outputDir,
      includePatterns: config.generation?.includePatterns,
      excludePatterns: config.generation?.excludePatterns,
      cleanStale: config.generation?.cleanStale
    });

    // Setup development watcher if watch is enabled
    if (config.watch && process.env.NODE_ENV === 'development') {
      const watcher = new DevWatcher(processor);
      watcher.start().catch((error) => {
        console.error('[@agent-ready-web/nextjs-plugin] Failed to start watcher:', error);
      });

      // Cleanup on process exit
      process.on('SIGINT', async () => {
        await watcher.stop();
      });
    }
  }

  // Initialize GEO processor if enabled
  if (config.geo?.enabled) {
    const geoConfig = mergeGEOConfig(config.geo);
    geoProcessor = new GEOBuildProcessor(geoConfig);
  }

  return {
    ...nextConfig,

    // ✅ Build-system agnostic approach
    // Add webpack hook for post-build generation
    webpack: (webpackConfig: any, context: any) => {
      // Apply user's webpack config if it exists
      const userWebpack = nextConfig.webpack;
      if (typeof userWebpack === 'function') {
        webpackConfig = userWebpack(webpackConfig, context);
      }

      // Add ARW post-build plugin for production builds
      if (processor && !context.isServer && context.nextRuntime !== 'edge') {
        webpackConfig.plugins = webpackConfig.plugins || [];
        webpackConfig.plugins.push({
          apply: (compiler: any) => {
            compiler.hooks.done.tapPromise('ARWBuildProcessor', async () => {
              // Run on build completion (not in development watch mode)
              if (process.env.NODE_ENV !== 'development' || process.env.NEXT_PHASE === 'phase-production-build') {
                try {
                  await processor!.processAll();

                  // Run GEO processor after main build if enabled
                  if (geoProcessor) {
                    await geoProcessor.processAll(config.outputDir || './public');
                  }
                } catch (error) {
                  console.error('[@agent-ready-web/nextjs-plugin] Generation failed:', error);
                }
              }
            });
          }
        });
      }

      return webpackConfig;
    },

    // Add rewrites for ARW endpoints
    async rewrites() {
      const userRewrites = typeof nextConfig.rewrites === 'function'
        ? await nextConfig.rewrites()
        : [];

      const arwRewrites = [
        {
          source: '/llms.txt',
          destination: '/.well-known/arw-manifest.json'
        }
      ];

      // Merge rewrites
      if (Array.isArray(userRewrites)) {
        return [...arwRewrites, ...userRewrites];
      }

      return {
        beforeFiles: arwRewrites,
        afterFiles: userRewrites.afterFiles || [],
        fallback: userRewrites.fallback || []
      };
    },

    // Add custom headers for ARW
    async headers() {
      const userHeaders = typeof nextConfig.headers === 'function'
        ? await nextConfig.headers()
        : [];

      const arwHeaders = [
        {
          source: '/:path*.llm.md',
          headers: [
            {
              key: 'Content-Type',
              value: 'text/x-llm+markdown'
            },
            {
              key: 'X-ARW-Version',
              value: '0.1'
            }
          ]
        },
        {
          source: '/.well-known/arw-manifest.json',
          headers: [
            {
              key: 'Content-Type',
              value: 'application/json'
            },
            {
              key: 'X-ARW-Profile',
              value: 'ARW-1'
            }
          ]
        }
      ];

      return [...arwHeaders, ...userHeaders];
    }
  };
}

// Re-export types for convenience
export type {
  ARWNextConfig,
  ARWManifestConfig,
  ARWPolicies,
  ARWGenerationOptions,
  ARWRuntimeOptions,
  ARWChunk,
  PageMetadata,
  ARWManifest,
  ARWContentItem,
  MachineView,
  ARWGEOConfig
} from './types';

export { defaultConfig, defaultPolicies, mergeConfig } from './config/defaults';
export { generateARWMetadata } from './metadata';

// Export generator modules for advanced usage
export { BuildProcessor } from './generator/build-processor';
export { DevWatcher } from './generator/dev-watcher';
export { ASTParser } from './generator/ast-parser';
export { HTMLProcessor } from './generator/html-processor';
export { MarkdownGenerator } from './generator/markdown-generator';
export { FileManager } from './utils/file-manager';

// Export GEO modules
export { GEOBuildProcessor } from './geo/build-processor';
export { mergeGEOConfig, defaultGEOConfig } from './geo/utils';
export type * from './geo/types';
