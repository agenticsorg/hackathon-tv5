/**
 * Format Validator - Checks both YAML and JSON formats
 * Validates mime types, content structure, and format consistency
 */

import yaml from 'js-yaml';

/**
 * Validate that both llms.txt (YAML) and llms.json exist and are consistent
 */
export async function validateFormats(baseUrl, fetchFn, { timeout = 8000 } = {}) {
  const results = {
    passed: [],
    errors: [],
    warnings: [],
  };

  // Check llms.txt (YAML)
  const yamlUrl = `${baseUrl}/llms.txt`;
  let yamlData = null;
  let yamlMimeType = null;

  try {
    const { res, text } = await fetchFn(yamlUrl, { timeout });
    yamlMimeType = res.headers.get('content-type') || '';

    // Validate mime type for YAML (must be text/plain for universal compatibility)
    if (!yamlMimeType.includes('text/plain')) {
      results.errors.push(
        `llms.txt MUST use text/plain mime type for Claude compatibility (got: ${yamlMimeType}). ` +
        `application/yaml causes binary data corruption in Claude WebFetch.`
      );
    } else {
      results.passed.push('llms.txt has correct mime type (text/plain)');
    }

    // Check for charset
    if (!yamlMimeType.includes('charset=utf-8')) {
      results.warnings.push('llms.txt should specify charset=utf-8');
    } else {
      results.passed.push('llms.txt specifies charset=utf-8');
    }

    // Parse YAML
    yamlData = yaml.load(text);
    results.passed.push('llms.txt is valid YAML');
  } catch (e) {
    results.errors.push(`Failed to fetch or parse llms.txt: ${e.message}`);
  }

  // Check llms.json
  const jsonUrl = `${baseUrl}/llms.json`;
  let jsonData = null;
  let jsonMimeType = null;

  try {
    const { res, text } = await fetchFn(jsonUrl, { timeout });
    jsonMimeType = res.headers.get('content-type') || '';

    // Validate mime type for JSON
    if (!jsonMimeType.includes('application/json')) {
      results.errors.push(
        `llms.json must use application/json mime type (got: ${jsonMimeType})`
      );
    } else {
      results.passed.push('llms.json has correct mime type (application/json)');
    }

    // Check for charset
    if (!jsonMimeType.includes('charset=utf-8')) {
      results.warnings.push('llms.json should specify charset=utf-8');
    } else {
      results.passed.push('llms.json specifies charset=utf-8');
    }

    // Parse JSON
    jsonData = JSON.parse(text);
    results.passed.push('llms.json is valid JSON');
  } catch (e) {
    results.warnings.push(
      `Missing llms.json - agents may fail to read YAML. ${e.message}`
    );
  }

  // Compare content if both exist
  if (yamlData && jsonData) {
    // Check version field type consistency (must be string in both)
    if (yamlData.version && jsonData.version) {
      const yamlVersion = String(yamlData.version);
      const jsonVersion = String(jsonData.version);
      if (yamlVersion !== jsonVersion) {
        results.errors.push(
          `Version mismatch: YAML="${yamlVersion}" vs JSON="${jsonVersion}"`
        );
      } else {
        results.passed.push('Version field matches between YAML and JSON');
      }
    }

    const isEqual = deepCompare(yamlData, jsonData);
    if (!isEqual) {
      results.errors.push(
        'llms.txt and llms.json have different content - they must be identical'
      );
    } else {
      results.passed.push('llms.txt and llms.json have identical content');
    }
  }

  return results;
}

/**
 * Deep comparison of two objects/arrays
 */
function deepCompare(obj1, obj2) {
  // Normalize and compare
  const normalize = (obj) => JSON.parse(JSON.stringify(obj));
  const str1 = JSON.stringify(normalize(obj1), Object.keys(normalize(obj1)).sort());
  const str2 = JSON.stringify(normalize(obj2), Object.keys(normalize(obj2)).sort());
  return str1 === str2;
}

/**
 * Convert YAML manifest to JSON
 */
export function convertYamlToJson(yamlText) {
  try {
    const data = yaml.load(yamlText);
    return JSON.stringify(data, null, 2);
  } catch (e) {
    throw new Error(`Failed to convert YAML to JSON: ${e.message}`);
  }
}
