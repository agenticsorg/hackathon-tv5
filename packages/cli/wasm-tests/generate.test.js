// Node.js tests for WASM generation
import { generate_manifest_wasm, validate_manifest_wasm } from '../wasm-pkg/nodejs/arw_cli.js';

console.log('🧪 Testing WASM generation...\n');

// Test 1: Generate minimal manifest
console.log('Test 1: Generate minimal manifest');
try {
    const config = {
        site_name: 'Test Site',
        homepage: 'https://example.com',
        contact: 'ai@example.com',
        profile: 'ARW-1'
    };

    const manifest = generate_manifest_wasm(config);
    console.assert(manifest.includes('version: 1.0'), 'Should have version');
    console.assert(manifest.includes('profile: ARW-1'), 'Should have profile');
    console.assert(manifest.includes("name: 'Test Site'"), 'Should have name');
    console.assert(manifest.includes("homepage: 'https://example.com'"), 'Should have homepage');
    console.assert(manifest.includes("contact: 'ai@example.com'"), 'Should have contact');
    console.log('✓ Passed\n');
} catch (error) {
    console.error('✗ Failed:', error.message);
    process.exit(1);
}

// Test 2: Generate with description
console.log('Test 2: Generate with description');
try {
    const config = {
        site_name: 'Test Site',
        homepage: 'https://example.com',
        contact: 'ai@example.com',
        profile: 'ARW-2',
        description: 'A test site for WASM generation'
    };

    const manifest = generate_manifest_wasm(config);
    console.assert(manifest.includes('profile: ARW-2'), 'Should have ARW-2 profile');
    console.assert(
        manifest.includes("description: 'A test site for WASM generation'"),
        'Should have description'
    );
    console.log('✓ Passed\n');
} catch (error) {
    console.error('✗ Failed:', error.message);
    process.exit(1);
}

// Test 3: Generate and validate round-trip
console.log('Test 3: Generate and validate round-trip');
try {
    const config = {
        site_name: 'Round Trip Test',
        homepage: 'https://roundtrip.com',
        contact: 'test@roundtrip.com',
        profile: 'ARW-1'
    };

    const manifest = generate_manifest_wasm(config);
    const result = await validate_manifest_wasm(manifest);

    console.assert(result.valid === true, 'Generated manifest should be valid');
    console.assert(result.errors.length === 0, 'Should have no errors');
    console.log('✓ Passed\n');
} catch (error) {
    console.error('✗ Failed:', error.message);
    process.exit(1);
}

// Test 4: Generate with missing required fields
console.log('Test 4: Generate with missing required fields');
try {
    const config = {
        site_name: 'Incomplete Site',
        homepage: 'https://example.com'
        // Missing contact and profile
    };

    generate_manifest_wasm(config);
    console.error('✗ Failed: Should have thrown error');
    process.exit(1);
} catch (error) {
    console.assert(error.includes('Invalid config'), 'Should have config error');
    console.log('✓ Passed\n');
}

// Test 5: Generate for all profiles
console.log('Test 5: Generate for all profiles');
try {
    const profiles = ['ARW-1', 'ARW-2', 'ARW-3'];

    for (const profile of profiles) {
        const config = {
            site_name: `Test Site ${profile}`,
            homepage: 'https://example.com',
            contact: 'ai@example.com',
            profile: profile
        };

        const manifest = generate_manifest_wasm(config);
        console.assert(
            manifest.includes(`profile: ${profile}`),
            `Should have ${profile} profile`
        );
    }
    console.log('✓ Passed\n');
} catch (error) {
    console.error('✗ Failed:', error.message);
    process.exit(1);
}

// Test 6: Generate with special characters
console.log('Test 6: Generate with special characters');
try {
    const config = {
        site_name: 'Test & Site "with" special \'chars\'',
        homepage: 'https://example.com/path?query=value',
        contact: 'ai+test@example.com',
        profile: 'ARW-1',
        description: 'Testing special: & < > \' "'
    };

    const manifest = generate_manifest_wasm(config);
    console.assert(manifest.includes('name: \'Test & Site'), 'Should handle special chars');

    // Validate that it's valid YAML
    const result = await validate_manifest_wasm(manifest);
    console.assert(result.valid === true, 'Should produce valid YAML');
    console.log('✓ Passed\n');
} catch (error) {
    console.error('✗ Failed:', error.message);
    process.exit(1);
}

// Test 7: Generation consistency
console.log('Test 7: Generation consistency');
try {
    const config = {
        site_name: 'Consistency Test',
        homepage: 'https://example.com',
        contact: 'ai@example.com',
        profile: 'ARW-1'
    };

    const manifest1 = generate_manifest_wasm(config);
    const manifest2 = generate_manifest_wasm(config);

    console.assert(manifest1 === manifest2, 'Should generate consistently');
    console.log('✓ Passed\n');
} catch (error) {
    console.error('✗ Failed:', error.message);
    process.exit(1);
}

// Test 8: Generate and validate complex manifest
console.log('Test 8: Generate and validate complex manifest');
try {
    const config = {
        site_name: 'Complex Test Site',
        homepage: 'https://complex.example.com',
        contact: 'ai@complex.example.com',
        profile: 'ARW-2',
        description: 'A complex site with all features enabled'
    };

    const manifest = generate_manifest_wasm(config);

    // Should be valid
    const result = await validate_manifest_wasm(manifest);
    console.assert(result.valid === true, 'Should be valid');

    // Should contain expected sections
    console.assert(manifest.includes('content: []'), 'Should have content section');
    console.assert(manifest.includes('policies:'), 'Should have policies section');
    console.assert(manifest.includes('training:'), 'Should have training policy');
    console.assert(manifest.includes('inference:'), 'Should have inference policy');
    console.assert(manifest.includes('attribution:'), 'Should have attribution policy');

    console.log('✓ Passed\n');
} catch (error) {
    console.error('✗ Failed:', error.message);
    process.exit(1);
}

console.log('✓ All generation tests passed!');
