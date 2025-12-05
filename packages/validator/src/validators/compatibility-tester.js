/**
 * Compatibility Tester - Tests actual agent accessibility
 * Simulates how different AI agents would access the ARW implementation
 */

export async function testCompatibility(baseUrl, fetchFn, { timeout = 8000 } = {}) {
  const results = {
    passed: [],
    errors: [],
    warnings: [],
    agents: {},
  };

  // Test 1: Simple HTTP client (most agents)
  results.agents.simpleHttp = await testSimpleHttpAccess(baseUrl, fetchFn, { timeout });

  // Test 2: YAML parser compatibility
  results.agents.yamlParser = await testYamlParsing(baseUrl, fetchFn, { timeout });

  // Test 3: JSON parser compatibility
  results.agents.jsonParser = await testJsonParsing(baseUrl, fetchFn, { timeout });

  // Test 4: Well-known discovery
  results.agents.wellKnownDiscovery = await testWellKnownDiscovery(baseUrl, fetchFn, { timeout });

  // Test 5: Header-based detection
  results.agents.headerDetection = await testHeaderDetection(baseUrl, fetchFn, { timeout });

  // Summarize results
  const totalTests = Object.keys(results.agents).length;
  const passedTests = Object.values(results.agents).filter(a => a.accessible).length;

  if (passedTests === totalTests) {
    results.passed.push(`All ${totalTests} agent compatibility tests passed`);
  } else {
    results.warnings.push(
      `${passedTests}/${totalTests} agent compatibility tests passed`
    );
  }

  return results;
}

async function testSimpleHttpAccess(baseUrl, fetchFn, { timeout }) {
  try {
    const { res } = await fetchFn(`${baseUrl}/llms.txt`, { timeout });
    if (res.ok) {
      return {
        accessible: true,
        method: 'HTTP GET /llms.txt',
        message: 'Successfully fetched via simple HTTP'
      };
    }
    return {
      accessible: false,
      method: 'HTTP GET /llms.txt',
      message: `HTTP ${res.status}`
    };
  } catch (e) {
    return {
      accessible: false,
      method: 'HTTP GET /llms.txt',
      message: e.message
    };
  }
}

async function testYamlParsing(baseUrl, fetchFn, { timeout }) {
  try {
    const { text } = await fetchFn(`${baseUrl}/llms.txt`, { timeout });
    const yaml = await import('js-yaml');
    const data = yaml.load(text);

    if (data && data.version && data.site) {
      return {
        accessible: true,
        method: 'YAML parsing',
        message: 'Successfully parsed YAML manifest'
      };
    }
    return {
      accessible: false,
      method: 'YAML parsing',
      message: 'Invalid manifest structure'
    };
  } catch (e) {
    return {
      accessible: false,
      method: 'YAML parsing',
      message: `Parse error: ${e.message}`
    };
  }
}

async function testJsonParsing(baseUrl, fetchFn, { timeout }) {
  try {
    const { text } = await fetchFn(`${baseUrl}/llms.json`, { timeout });
    const data = JSON.parse(text);

    if (data && data.version && data.site) {
      return {
        accessible: true,
        method: 'JSON parsing',
        message: 'Successfully parsed JSON manifest'
      };
    }
    return {
      accessible: false,
      method: 'JSON parsing',
      message: 'Invalid manifest structure'
    };
  } catch (e) {
    return {
      accessible: false,
      method: 'JSON parsing',
      message: `Parse error: ${e.message}`
    };
  }
}

async function testWellKnownDiscovery(baseUrl, fetchFn, { timeout }) {
  try {
    const { text } = await fetchFn(`${baseUrl}/.well-known/arw-manifest.json`, { timeout });
    const data = JSON.parse(text);

    // ARW v0.2+: well-known can contain full manifest or pointer
    if (data && (data.manifestUrl || (data.version && data.site))) {
      return {
        accessible: true,
        method: 'Well-known discovery',
        message: 'Successfully discovered via .well-known'
      };
    }
    return {
      accessible: false,
      method: 'Well-known discovery',
      message: 'Invalid well-known manifest structure'
    };
  } catch (e) {
    return {
      accessible: false,
      method: 'Well-known discovery',
      message: `Discovery failed: ${e.message}`
    };
  }
}

async function testHeaderDetection(baseUrl, fetchFn, { timeout }) {
  try {
    const { res } = await fetchFn(`${baseUrl}/`, { timeout });
    const headers = {};
    res.headers.forEach((v, k) => headers[k] = v);

    const hasAiHeaders = headers['ai-attribution'] || headers['ai-inference'];
    const hasLinkHeader = headers['link'] && headers['link'].includes('llms.txt');

    if (hasAiHeaders || hasLinkHeader) {
      return {
        accessible: true,
        method: 'Header detection',
        message: 'ARW detectable via HTTP headers'
      };
    }
    return {
      accessible: false,
      method: 'Header detection',
      message: 'No ARW headers found'
    };
  } catch (e) {
    return {
      accessible: false,
      method: 'Header detection',
      message: `Header check failed: ${e.message}`
    };
  }
}
