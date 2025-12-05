/**
 * Utility functions for ARW manifest generation
 */

import type { ARWManifest, ARWContentItem, PageMetadata } from '../types';

/**
 * Create an ARW content item from page metadata
 */
export function createContentItem(
  url: string,
  machineViewPath: string,
  metadata: PageMetadata
): ARWContentItem {
  return {
    url,
    machine_view: machineViewPath,
    purpose: metadata.purpose || 'general',
    priority: metadata.priority || 'normal',
    chunks: metadata.chunks
  };
}

/**
 * Validate ARW manifest structure
 */
export function validateManifest(manifest: ARWManifest): { valid: boolean; errors: string[] } {
  const errors: string[] = [];

  // Required fields
  if (!manifest.version) {
    errors.push('Missing required field: version');
  }

  if (!manifest.profile) {
    errors.push('Missing required field: profile');
  }

  if (!manifest.site) {
    errors.push('Missing required field: site');
  } else {
    if (!manifest.site.name) {
      errors.push('Missing required field: site.name');
    }
    if (!manifest.site.homepage) {
      errors.push('Missing required field: site.homepage');
    }
    if (!manifest.site.contact) {
      errors.push('Missing required field: site.contact');
    }
  }

  if (!manifest.content || !Array.isArray(manifest.content)) {
    errors.push('Missing or invalid field: content (must be an array)');
  }

  return {
    valid: errors.length === 0,
    errors
  };
}

/**
 * Sort content items by priority
 */
export function sortContentByPriority(content: ARWContentItem[]): ARWContentItem[] {
  const priorityOrder = { high: 0, normal: 1, low: 2 };

  return [...content].sort((a, b) => {
    const aPriority = priorityOrder[a.priority as keyof typeof priorityOrder] ?? 1;
    const bPriority = priorityOrder[b.priority as keyof typeof priorityOrder] ?? 1;
    return aPriority - bPriority;
  });
}
