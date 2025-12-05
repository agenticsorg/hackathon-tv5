import { URL } from 'url';

export function normalizeUrl(url: string): string {
  try {
    const parsed = new URL(url);
    // Remove trailing slash
    parsed.pathname = parsed.pathname.replace(/\/$/, '') || '/';
    // Sort query parameters
    parsed.searchParams.sort();
    // Remove fragment
    parsed.hash = '';
    return parsed.toString();
  } catch (error) {
    throw new Error(`Invalid URL: ${url}`);
  }
}

export function isValidUrl(url: string): boolean {
  try {
    new URL(url);
    return true;
  } catch {
    return false;
  }
}

export function isSameDomain(url1: string, url2: string): boolean {
  try {
    const parsed1 = new URL(url1);
    const parsed2 = new URL(url2);
    return parsed1.hostname === parsed2.hostname;
  } catch {
    return false;
  }
}

export function resolveUrl(base: string, relative: string): string {
  try {
    return new URL(relative, base).toString();
  } catch {
    return relative;
  }
}

export function extractDomain(url: string): string {
  try {
    const parsed = new URL(url);
    return parsed.hostname;
  } catch {
    return '';
  }
}

export function getBaseUrl(url: string): string {
  try {
    const parsed = new URL(url);
    return `${parsed.protocol}//${parsed.hostname}`;
  } catch {
    return '';
  }
}

export function isAbsoluteUrl(url: string): boolean {
  return /^https?:\/\//i.test(url);
}

export function sanitizeUrl(url: string): string {
  // Remove potentially dangerous protocols
  const dangerous = ['javascript:', 'data:', 'vbscript:', 'file:'];
  const lower = url.toLowerCase().trim();

  for (const protocol of dangerous) {
    if (lower.startsWith(protocol)) {
      return '';
    }
  }

  return url;
}
