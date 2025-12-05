/**
 * Well-Known Validator - Checks /.well-known/arw-manifest.json
 * Validates the presence and structure of the well-known manifest
 */

export async function validateWellKnown(baseUrl, fetchFn, { timeout = 8000 } = {}) {
  const results = {
    passed: [],
    errors: [],
    warnings: [],
  };

  const wellKnownUrl = `${baseUrl}/.well-known/arw-manifest.json`;

  try {
    const { res, text } = await fetchFn(wellKnownUrl, { timeout });

    // Check mime type
    const mimeType = res.headers.get('content-type') || '';
    if (!mimeType.includes('application/json')) {
      results.errors.push(
        `/.well-known/arw-manifest.json must use application/json mime type (got: ${mimeType})`
      );
    } else {
      results.passed.push('Well-known manifest has correct mime type');
    }

    // Parse and validate structure
    const manifest = JSON.parse(text);

    // ARW v0.2+: well-known contains full manifest, not just a pointer
    // Check for either old format (manifestUrl) or new format (full manifest)
    if (manifest.manifestUrl) {
      // Old format - pointer to llms.txt
      results.warnings.push('Well-known uses legacy pointer format. Consider embedding full manifest.');
      results.passed.push('Well-known manifest has manifestUrl');
    } else if (manifest.version && manifest.site) {
      // New format - full manifest embedded
      results.passed.push('Well-known manifest contains full ARW manifest (recommended)');

      if (!manifest.version) {
        results.errors.push('Well-known manifest missing version field');
      } else {
        results.passed.push(`Well-known manifest version: ${manifest.version}`);
      }

      if (!manifest.profile) {
        results.warnings.push('Well-known manifest missing profile field');
      } else {
        results.passed.push(`Well-known manifest profile: ${manifest.profile}`);
      }

      if (manifest.arw) {
        results.passed.push('Well-known includes ARW metadata with discovery endpoints');
      }
    } else {
      results.errors.push('Well-known manifest has invalid structure (needs either manifestUrl or full manifest)');
    }

    // Check for AI-* headers
    const headers = {};
    res.headers.forEach((v, k) => headers[k.toLowerCase()] = v);

    if (headers['ai-manifest'] === 'true') {
      results.passed.push('AI-Manifest header present');
    } else {
      results.warnings.push('AI-Manifest header not present (recommended)');
    }

  } catch (e) {
    if (e.message.includes('404') || e.message.includes('HTTP 404')) {
      results.warnings.push(
        'Missing /.well-known/arw-manifest.json - recommended for better discovery'
      );
    } else {
      results.errors.push(`Failed to fetch or parse well-known manifest: ${e.message}`);
    }
  }

  return results;
}
