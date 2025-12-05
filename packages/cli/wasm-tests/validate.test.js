// Node.js tests for WASM validation
import { validate_manifest_wasm, validate_manifest_json_wasm } from '../wasm-pkg/nodejs/arw_cli.js';

console.log('🧪 Testing WASM validation...\n');

// Test 1: Valid minimal manifest
console.log('Test 1: Valid minimal manifest');
try {
    const manifest = `
version: 1.0
profile: ARW-1
site:
  name: Test Site
  homepage: https://example.com
  contact: ai@example.com
policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
`;

    const result = await validate_manifest_wasm(manifest);
    console.assert(result.valid === true, 'Should be valid');
    console.assert(result.errors.length === 0, 'Should have no errors');
    console.log('✓ Passed\n');
} catch (error) {
    console.error('✗ Failed:', error.message);
    process.exit(1);
}

// Test 2: Invalid profile
console.log('Test 2: Invalid profile');
try {
    const manifest = `
version: 1.0
profile: INVALID_PROFILE
site:
  name: Test Site
  homepage: https://example.com
  contact: ai@example.com
policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
`;

    const result = await validate_manifest_wasm(manifest);
    console.assert(result.valid === false, 'Should be invalid');
    console.assert(result.errors.length > 0, 'Should have errors');
    console.assert(
        result.errors.some(e => e.path === 'profile'),
        'Should have error for profile'
    );
    console.log('✓ Passed\n');
} catch (error) {
    console.error('✗ Failed:', error.message);
    process.exit(1);
}

// Test 3: Missing required fields
console.log('Test 3: Missing required fields');
try {
    const manifest = `
version: 1.0
profile: ARW-1
`;

    const result = await validate_manifest_wasm(manifest);
    console.assert(result.valid === false, 'Should be invalid');
    console.assert(result.errors.length > 0, 'Should have errors');
    console.log('✓ Passed\n');
} catch (error) {
    console.error('✗ Failed:', error.message);
    process.exit(1);
}

// Test 4: Invalid YAML syntax
console.log('Test 4: Invalid YAML syntax');
try {
    const manifest = `this is: not: valid: yaml: [`;

    await validate_manifest_wasm(manifest);
    console.error('✗ Failed: Should have thrown error');
    process.exit(1);
} catch (error) {
    console.assert(error.includes('Failed to parse YAML'), 'Should have YAML parse error');
    console.log('✓ Passed\n');
}

// Test 5: JSON format validation
console.log('Test 5: JSON format validation');
try {
    const manifest = JSON.stringify({
        version: 1.0,
        profile: 'ARW-1',
        site: {
            name: 'Test Site',
            homepage: 'https://example.com',
            contact: 'ai@example.com'
        },
        policies: {
            training: { allowed: false },
            inference: { allowed: true },
            attribution: { required: true }
        }
    });

    const result = await validate_manifest_json_wasm(manifest);
    console.assert(result.valid === true, 'Should be valid');
    console.assert(result.errors.length === 0, 'Should have no errors');
    console.log('✓ Passed\n');
} catch (error) {
    console.error('✗ Failed:', error.message);
    process.exit(1);
}

// Test 6: Complete ARW-2 manifest
console.log('Test 6: Complete ARW-2 manifest');
try {
    const manifest = `
version: 1.0
profile: ARW-2
site:
  name: Complete Test Site
  description: A fully featured test site
  homepage: https://example.com
  contact: ai@example.com
  logo: https://example.com/logo.png

content:
  - url: /docs
    title: Documentation
    format: markdown
    frequency: weekly

policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
`;

    const result = await validate_manifest_wasm(manifest);
    console.assert(result.valid === true, 'Should be valid');
    console.log('✓ Passed\n');
} catch (error) {
    console.error('✗ Failed:', error.message);
    process.exit(1);
}

// Test 7: Invalid URL format
console.log('Test 7: Invalid URL format');
try {
    const manifest = `
version: 1.0
profile: ARW-1
site:
  name: Test Site
  homepage: not-a-valid-url
  contact: ai@example.com
policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
`;

    const result = await validate_manifest_wasm(manifest);
    console.assert(result.valid === false, 'Should be invalid');
    console.log('✓ Passed\n');
} catch (error) {
    console.error('✗ Failed:', error.message);
    process.exit(1);
}

// Test 8: Invalid email format
console.log('Test 8: Invalid email format');
try {
    const manifest = `
version: 1.0
profile: ARW-1
site:
  name: Test Site
  homepage: https://example.com
  contact: not-an-email
policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
`;

    const result = await validate_manifest_wasm(manifest);
    console.assert(result.valid === false, 'Should be invalid');
    console.log('✓ Passed\n');
} catch (error) {
    console.error('✗ Failed:', error.message);
    process.exit(1);
}

console.log('✓ All validation tests passed!');
