/**
 * Platform-Specific Validator
 * Checks platform-specific requirements (Vercel, Netlify, etc.)
 */

export const PLATFORMS = {
  vercel: {
    name: 'Vercel',
    checks: [
      {
        name: 'vercel.json configuration',
        check: async (baseUrl, fetchFn) => {
          // Check if headers are properly configured
          try {
            const { res } = await fetchFn(`${baseUrl}/llms.txt`);
            const headers = {};
            res.headers.forEach((v, k) => headers[k] = v);

            if (!headers['ai-attribution'] || !headers['ai-inference']) {
              return {
                passed: false,
                message: 'Missing AI-* headers. Ensure vercel.json includes header configuration'
              };
            }
            return { passed: true, message: 'AI-* headers configured correctly' };
          } catch (e) {
            return { passed: false, message: `Header check failed: ${e.message}` };
          }
        }
      },
      {
        name: 'Static file serving',
        check: async (baseUrl, fetchFn) => {
          // Check if JSON is served with correct mime type
          try {
            const { res } = await fetchFn(`${baseUrl}/llms.json`);
            const mimeType = res.headers.get('content-type') || '';
            if (!mimeType.includes('application/json')) {
              return {
                passed: false,
                message: 'llms.json not served with application/json mime type. Check vercel.json rewrites'
              };
            }
            return { passed: true, message: 'JSON files served correctly' };
          } catch (e) {
            return { passed: false, message: `JSON serving check failed: ${e.message}` };
          }
        }
      }
    ]
  },
  netlify: {
    name: 'Netlify',
    checks: [
      {
        name: '_headers configuration',
        check: async (baseUrl, fetchFn) => {
          try {
            const { res } = await fetchFn(`${baseUrl}/llms.txt`);
            const headers = {};
            res.headers.forEach((v, k) => headers[k] = v);

            if (!headers['ai-attribution'] || !headers['ai-inference']) {
              return {
                passed: false,
                message: 'Missing AI-* headers. Ensure _headers file includes ARW headers'
              };
            }
            return { passed: true, message: 'AI-* headers configured correctly' };
          } catch (e) {
            return { passed: false, message: `Header check failed: ${e.message}` };
          }
        }
      }
    ]
  },
  cloudflare: {
    name: 'Cloudflare Pages',
    checks: [
      {
        name: '_headers configuration',
        check: async (baseUrl, fetchFn) => {
          try {
            const { res } = await fetchFn(`${baseUrl}/llms.txt`);
            const headers = {};
            res.headers.forEach((v, k) => headers[k] = v);

            if (!headers['ai-attribution'] || !headers['ai-inference']) {
              return {
                passed: false,
                message: 'Missing AI-* headers. Ensure _headers file includes ARW headers'
              };
            }
            return { passed: true, message: 'AI-* headers configured correctly' };
          } catch (e) {
            return { passed: false, message: `Header check failed: ${e.message}` };
          }
        }
      }
    ]
  }
};

export async function validatePlatform(platform, baseUrl, fetchFn, { timeout = 8000 } = {}) {
  const results = {
    passed: [],
    errors: [],
    warnings: [],
    platform: platform,
  };

  const platformConfig = PLATFORMS[platform.toLowerCase()];

  if (!platformConfig) {
    results.errors.push(
      `Unknown platform: ${platform}. Supported platforms: ${Object.keys(PLATFORMS).join(', ')}`
    );
    return results;
  }

  results.passed.push(`Running ${platformConfig.name}-specific checks`);

  for (const check of platformConfig.checks) {
    try {
      const result = await check.check(baseUrl, fetchFn);
      if (result.passed) {
        results.passed.push(`${check.name}: ${result.message}`);
      } else {
        results.errors.push(`${check.name}: ${result.message}`);
      }
    } catch (e) {
      results.errors.push(`${check.name} failed: ${e.message}`);
    }
  }

  return results;
}
