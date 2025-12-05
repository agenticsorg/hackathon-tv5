/**
 * ARW Head component for Next.js metadata
 */

'use client';

import { usePathname } from 'next/navigation';
import type { ARWHeadProps } from '../types';

/**
 * Component that adds ARW discovery metadata to page head
 *
 * @example
 * ```tsx
 * // In your page component
 * export default function Page() {
 *   return (
 *     <>
 *       <ARWHead priority="high" />
 *       <main>Your content</main>
 *     </>
 *   );
 * }
 * ```
 */
export function ARWHead({ priority = 'normal', machineViewPath }: ARWHeadProps) {
  const pathname = usePathname();
  const machineView = machineViewPath || `${pathname}.llm.md`;

  return (
    <>
      <link
        rel="alternate"
        type="text/x-llm+markdown"
        href={machineView}
      />
      <link
        rel="alternate"
        type="application/json"
        href="/.well-known/arw-manifest.json"
      />
      <meta name="arw-priority" content={priority} />
      <meta name="arw-enabled" content="true" />
    </>
  );
}

/**
 * Metadata generator for App Router
 *
 * @example
 * ```tsx
 * // In app/page.tsx
 * import { generateARWMetadata } from '@agent-ready-web/nextjs-plugin/components';
 *
 * export const metadata = generateARWMetadata({
 *   priority: 'high',
 *   machineViewPath: '/index.llm.md'
 * });
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
