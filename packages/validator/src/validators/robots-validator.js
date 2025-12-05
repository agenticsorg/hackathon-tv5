/**
 * Robots.txt Validator - Checks for ARW hints
 * Validates that robots.txt includes ARW discovery hints
 */

export async function validateRobotsTxt(baseUrl, fetchFn, { timeout = 8000 } = {}) {
  const results = {
    passed: [],
    errors: [],
    warnings: [],
  };

  const robotsUrl = `${baseUrl}/robots.txt`;

  try {
    const { res, text } = await fetchFn(robotsUrl, { timeout });

    // Check for ARW hints
    const hasLlmsTxt = text.includes('/llms.txt') || text.includes('llms.txt');
    const hasWellKnown = text.includes('/.well-known/arw-manifest.json') ||
                         text.includes('.well-known/arw-manifest');
    const hasArwComment = text.toLowerCase().includes('arw') ||
                          text.toLowerCase().includes('agent-ready');

    if (hasLlmsTxt) {
      results.passed.push('robots.txt references /llms.txt');
    } else {
      results.warnings.push(
        'robots.txt should include a comment or reference to /llms.txt for better discovery'
      );
    }

    if (hasWellKnown) {
      results.passed.push('robots.txt references /.well-known/arw-manifest.json');
    }

    if (hasArwComment) {
      results.passed.push('robots.txt includes ARW discovery hints');
    } else {
      results.warnings.push(
        'Consider adding ARW discovery hints in robots.txt (e.g., # Agent-Ready Web: /llms.txt)'
      );
    }

    // Check for blocking of ARW endpoints
    const blockingPatterns = [
      /Disallow:\s*\/llms\.txt/i,
      /Disallow:\s*\/llms\.json/i,
      /Disallow:\s*\/\.well-known\/arw/i,
    ];

    for (const pattern of blockingPatterns) {
      if (pattern.test(text)) {
        results.errors.push(
          'robots.txt is blocking ARW endpoints - this prevents agent discovery'
        );
        break;
      }
    }

  } catch (e) {
    results.warnings.push(`Could not fetch robots.txt: ${e.message}`);
  }

  return results;
}
