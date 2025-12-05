/**
 * React hooks for ARW integration
 */

'use client';

import { useState, useEffect } from 'react';
import type { ARWManifest, UseARWReturn } from '../types';

/**
 * Hook to access ARW manifest data
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const { manifest, loading, error } = useARW();
 *
 *   if (loading) return <div>Loading ARW manifest...</div>;
 *   if (error) return <div>Error: {error.message}</div>;
 *
 *   return (
 *     <div>
 *       <h1>{manifest?.site.name}</h1>
 *       <p>{manifest?.site.description}</p>
 *     </div>
 *   );
 * }
 * ```
 */
export function useARW(): UseARWReturn {
  const [manifest, setManifest] = useState<ARWManifest | undefined>();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    async function fetchManifest() {
      try {
        const response = await fetch('/.well-known/arw-manifest.json');

        if (!response.ok) {
          throw new Error(`Failed to fetch ARW manifest: ${response.status}`);
        }

        const data = await response.json();
        setManifest(data);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err : new Error('Unknown error'));
        setManifest(undefined);
      } finally {
        setLoading(false);
      }
    }

    fetchManifest();
  }, []);

  return { manifest, loading, error };
}

/**
 * Hook to check if ARW is enabled and properly configured
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const isARWEnabled = useARWEnabled();
 *
 *   return (
 *     <div>
 *       ARW Status: {isARWEnabled ? 'Enabled' : 'Disabled'}
 *     </div>
 *   );
 * }
 * ```
 */
export function useARWEnabled(): boolean {
  const { manifest, loading } = useARW();
  return !loading && manifest !== undefined;
}

/**
 * Hook to get current page's machine view URL
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const machineViewUrl = useMachineViewUrl();
 *
 *   return (
 *     <a href={machineViewUrl}>View Machine-Readable Version</a>
 *   );
 * }
 * ```
 */
export function useMachineViewUrl(customPath?: string): string | null {
  const [machineViewUrl, setMachineViewUrl] = useState<string | null>(null);

  useEffect(() => {
    if (typeof window === 'undefined') {
      return;
    }

    const path = customPath || window.location.pathname;
    const url = `${path}.llm.md`;
    setMachineViewUrl(url);
  }, [customPath]);

  return machineViewUrl;
}
