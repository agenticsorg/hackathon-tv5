/**
 * ARW Context Provider for React applications
 */

'use client';

import { createContext, useContext } from 'react';
import type { ARWProviderProps, ARWManifest } from '../types';
import { useARW } from '../hooks/useARW';

interface ARWContextValue {
  manifest?: ARWManifest;
  loading: boolean;
  error: Error | null;
  config?: ARWProviderProps['config'];
}

const ARWContext = createContext<ARWContextValue | null>(null);

/**
 * Provider component for ARW context
 *
 * @example
 * ```tsx
 * // In your root layout or _app.tsx
 * export default function RootLayout({ children }) {
 *   return (
 *     <ARWProvider config={{ priority: 'high' }}>
 *       {children}
 *     </ARWProvider>
 *   );
 * }
 * ```
 */
export function ARWProvider({ children, config }: ARWProviderProps) {
  const { manifest, loading, error } = useARW();

  const value: ARWContextValue = {
    manifest,
    loading,
    error,
    config
  };

  return (
    <ARWContext.Provider value={value}>
      {children}
    </ARWContext.Provider>
  );
}

/**
 * Hook to access ARW context
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const { manifest, config } = useARWContext();
 *
 *   return (
 *     <div>
 *       <h1>{manifest?.site.name}</h1>
 *       <p>Priority: {config?.priority}</p>
 *     </div>
 *   );
 * }
 * ```
 */
export function useARWContext(): ARWContextValue {
  const context = useContext(ARWContext);

  if (!context) {
    throw new Error('useARWContext must be used within ARWProvider');
  }

  return context;
}
