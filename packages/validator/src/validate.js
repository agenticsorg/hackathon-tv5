import fetch from 'node-fetch';
import yaml from 'js-yaml';
import { validateFormats } from './validators/format-validator.js';
import { validateWellKnown } from './validators/well-known-validator.js';
import { validateRobotsTxt } from './validators/robots-validator.js';
import { validatePlatform } from './validators/platform-validator.js';
import { testCompatibility } from './validators/compatibility-tester.js';

const REQUIRED_HEADERS = {
  'ai-attribution': true,
  'ai-inference': true,
};

function joinUrl(base, path) {
  if (!path) return base;
  if (path.startsWith('http')) return path;
  return base.replace(/\/$/, '') + (path.startsWith('/') ? '' : '/') + path;
}

export async function fetchText(url, { timeout = 8000 } = {}) {
  const ctrl = new AbortController();
  const id = setTimeout(() => ctrl.abort(), timeout);
  try {
    const res = await fetch(url, { signal: ctrl.signal, redirect: 'follow' });
    if (!res.ok) throw new Error(`HTTP ${res.status} for ${url}`);
    const text = await res.text();
    return { res, text };
  } finally {
    clearTimeout(id);
  }
}

export async function validateDomain(baseUrl, { timeout = 8000, platform = null, full = false } = {}) {
  const passed = [];
  const errors = [];
  const warnings = [];
  let profile = null;
  let level = null;

  // 1) Fetch llms.txt
  const manifestUrl = joinUrl(baseUrl, '/llms.txt');
  let manifestText, manifestRes, manifest;
  try {
    const r = await fetchText(manifestUrl, { timeout });
    manifestText = r.text;
    manifestRes = r.res;
  } catch (e) {
    errors.push(`Failed to fetch /llms.txt: ${e.message}`);
    return { passed, errors, profile, level };
  }

  // 2) Parse YAML
  try {
    manifest = yaml.load(manifestText);
    passed.push('Parsed llms.txt YAML');
  } catch (e) {
    errors.push(`Invalid YAML in llms.txt: ${e.message}`);
    return { passed, errors, profile, level };
  }

  // 3) Basic fields
  const version = manifest?.version;
  profile = manifest?.profile || null;
  if (!version) errors.push('Missing `version` in llms.txt');
  if (!manifest?.site?.name || !manifest?.site?.homepage)
    errors.push('Missing `site.name` or `site.homepage`');
  if (!manifest?.policies) errors.push('Missing `policies` block');
  if (manifest?.content && manifest.content.length) {
    const hasMV = manifest.content.some((c) => !!c.machine_view);
    if (!hasMV) errors.push('No `content[*].machine_view` entries found');
    else passed.push('Found machine_view entries');
  } else {
    errors.push('No `content` entries found in llms.txt');
  }

  // 4) Attempt to fetch one machine view
  let mvUrl = null;
  try {
    const first = manifest?.content?.find((c) => c.machine_view);
    if (first) {
      mvUrl = joinUrl(baseUrl, first.machine_view);
      const { res } = await fetchText(mvUrl, { timeout });
      const ctype = (res.headers.get('content-type') || '').toLowerCase();
      if (!ctype.includes('text/markdown')) {
        errors.push(
          `Machine view Content-Type should be text/markdown, got: ${ctype || 'missing'}`
        );
      } else {
        passed.push('Machine view served as text/markdown');
      }
      // Check required headers
      const headers = {};
      res.headers.forEach((v, k) => {
        headers[k] = v;
      });
      for (const key of Object.keys(REQUIRED_HEADERS)) {
        if (!headers[key]) errors.push(`Missing required header on machine view: ${key}`);
        else passed.push(`Header present: ${key}`);
      }
    }
  } catch (e) {
    errors.push(`Failed to fetch machine view (${mvUrl || 'n/a'}): ${e.message}`);
  }

  // 5) Actions sanity
  if (manifest?.actions?.length) {
    passed.push(`Found ${manifest.actions.length} actions`);
    const bad = manifest.actions.filter((a) => !(a.id && a.endpoint && a.method && a.auth));
    if (bad.length)
      errors.push(
        `Actions missing required fields: ${bad.map((a) => a?.id || '(no id)').join(', ')}`
      );
  }

  // 6) Determine level
  const hasDiscovery =
    !errors.find((e) => e.includes('/llms.txt')) && manifest?.content?.length > 0;
  const hasSemantics = hasDiscovery; // heuristic; a stricter check would fetch and scan for chunk markers
  const hasActions = (manifest?.actions?.length || 0) > 0;
  const hasProtocols = (manifest?.protocols?.length || 0) > 0;

  if (hasDiscovery) level = 'ARW-1';
  if (hasSemantics) level = 'ARW-2';
  if (hasActions) level = 'ARW-3';
  if (hasProtocols) level = 'ARW-4';

  // Run extended validators if full check requested
  if (full) {
    // Format validation (YAML + JSON)
    const formatResults = await validateFormats(baseUrl, fetchText, { timeout });
    passed.push(...formatResults.passed);
    errors.push(...formatResults.errors);
    warnings.push(...formatResults.warnings);

    // Well-known manifest
    const wellKnownResults = await validateWellKnown(baseUrl, fetchText, { timeout });
    passed.push(...wellKnownResults.passed);
    errors.push(...wellKnownResults.errors);
    warnings.push(...wellKnownResults.warnings);

    // robots.txt
    const robotsResults = await validateRobotsTxt(baseUrl, fetchText, { timeout });
    passed.push(...robotsResults.passed);
    errors.push(...robotsResults.errors);
    warnings.push(...robotsResults.warnings);
  }

  // Platform-specific validation
  if (platform) {
    const platformResults = await validatePlatform(platform, baseUrl, fetchText, { timeout });
    passed.push(...platformResults.passed);
    errors.push(...platformResults.errors);
    warnings.push(...platformResults.warnings);
  }

  return { passed, errors, warnings, profile, level };
}

// New: Doctor command - comprehensive health check
export async function runDoctorCheck(baseUrl, { timeout = 8000, platform = null } = {}) {
  const report = {
    timestamp: new Date().toISOString(),
    url: baseUrl,
    sections: {},
    summary: {
      totalChecks: 0,
      passed: 0,
      warnings: 0,
      errors: 0,
    }
  };

  // 1. Core validation
  const coreResults = await validateDomain(baseUrl, { timeout, full: false });
  report.sections.core = coreResults;

  // 2. Format validation
  const formatResults = await validateFormats(baseUrl, fetchText, { timeout });
  report.sections.formats = formatResults;

  // 3. Well-known
  const wellKnownResults = await validateWellKnown(baseUrl, fetchText, { timeout });
  report.sections.wellKnown = wellKnownResults;

  // 4. Robots.txt
  const robotsResults = await validateRobotsTxt(baseUrl, fetchText, { timeout });
  report.sections.robots = robotsResults;

  // 5. Compatibility testing
  const compatResults = await testCompatibility(baseUrl, fetchText, { timeout });
  report.sections.compatibility = compatResults;

  // 6. Platform-specific (if provided)
  if (platform) {
    const platformResults = await validatePlatform(platform, baseUrl, fetchText, { timeout });
    report.sections.platform = platformResults;
  }

  // Calculate summary
  for (const section of Object.values(report.sections)) {
    report.summary.passed += section.passed?.length || 0;
    report.summary.warnings += section.warnings?.length || 0;
    report.summary.errors += section.errors?.length || 0;
  }
  report.summary.totalChecks = report.summary.passed + report.summary.warnings + report.summary.errors;

  return report;
}

// Export validator functions for CLI use
export { validateFormats, validateWellKnown, validateRobotsTxt, validatePlatform, testCompatibility };
