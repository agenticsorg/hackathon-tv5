/**
 * ARW Metadata Utilities
 * Server-side utilities for generating ARW metadata in Next.js
 */

import type { ARWHeadProps } from './types';

/**
 * Generate ARW metadata for Next.js metadata API
 *
 * @example
 * ```tsx
 * // In app/page.tsx
 * import { generateARWMetadata } from '@agent-ready-web/nextjs-plugin';
 *
 * export const metadata = {
 *   title: 'Home',
 *   ...generateARWMetadata({ priority: 'high' })
 * };
 * ```
 */
export function generateARWMetadata(props: ARWHeadProps = {}) {
  const { priority = 'normal', machineViewPath = '/index.llm.md' } = props;

  return {
    other: {
      'arw-priority': priority,
      'arw-enabled': 'true'
    },
    alternates: {
      types: {
        'text/x-llm+markdown': machineViewPath,
        'application/json': '/.well-known/arw-manifest.json'
      }
    }
  };
}
