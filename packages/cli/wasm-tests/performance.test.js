// Performance tests for WASM bindings
import { validate_manifest_wasm, generate_manifest_wasm, get_version_info } from '../wasm-pkg/nodejs/arw_cli.js';

console.log('⚡ Testing WASM performance...\n');

const ITERATIONS = 1000;

// Helper function to measure performance
async function measurePerformance(name, fn, iterations = ITERATIONS) {
    const start = performance.now();

    for (let i = 0; i < iterations; i++) {
        await fn();
    }

    const end = performance.now();
    const total = end - start;
    const average = total / iterations;

    console.log(`${name}:`);
    console.log(`  Total: ${total.toFixed(2)}ms`);
    console.log(`  Average: ${average.toFixed(3)}ms per operation`);
    console.log(`  Throughput: ${(1000 / average).toFixed(0)} ops/sec\n`);

    return { total, average };
}

// Test 1: Validation performance
console.log('Test 1: Validation performance');
const manifest = `
version: 1.0
profile: ARW-2
site:
  name: Performance Test Site
  description: Testing validation performance
  homepage: https://example.com
  contact: ai@example.com
  logo: https://example.com/logo.png

content:
  - url: /docs
    title: Documentation
    format: markdown
    frequency: weekly
  - url: /api
    title: API Reference
    format: openapi
    frequency: monthly

policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
`;

const validationPerf = await measurePerformance(
    'Manifest validation',
    async () => await validate_manifest_wasm(manifest)
);

// Test 2: Generation performance
console.log('Test 2: Generation performance');
const config = {
    site_name: 'Performance Test',
    homepage: 'https://perf.example.com',
    contact: 'ai@perf.example.com',
    profile: 'ARW-2',
    description: 'Performance testing site'
};

const generationPerf = await measurePerformance(
    'Manifest generation',
    () => generate_manifest_wasm(config)
);

// Test 3: Version info performance
console.log('Test 3: Version info retrieval');
const versionPerf = await measurePerformance(
    'Version info retrieval',
    () => get_version_info(),
    10000  // More iterations for simple operations
);

// Test 4: Cold start performance
console.log('Test 4: Cold start performance');
console.log('(First call after module load)\n');

const coldStartTests = [
    {
        name: 'First validation',
        fn: async () => await validate_manifest_wasm(manifest)
    },
    {
        name: 'First generation',
        fn: () => generate_manifest_wasm(config)
    }
];

for (const test of coldStartTests) {
    const start = performance.now();
    await test.fn();
    const end = performance.now();
    console.log(`${test.name}: ${(end - start).toFixed(3)}ms\n`);
}

// Test 5: Large manifest performance
console.log('Test 5: Large manifest performance');
const largeManifest = `
version: 1.0
profile: ARW-3
site:
  name: Large Test Site
  description: Testing performance with large manifests
  homepage: https://example.com
  contact: ai@example.com

content:
${Array(100).fill(null).map((_, i) => `
  - url: /doc-${i}
    title: Document ${i}
    format: markdown
    frequency: weekly
`).join('')}

policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true

actions:
${Array(50).fill(null).map((_, i) => `
  - id: action-${i}
    name: Action ${i}
    description: Test action ${i}
    method: GET
    endpoint: https://api.example.com/action-${i}
`).join('')}
`;

const largePerf = await measurePerformance(
    'Large manifest validation',
    async () => await validate_manifest_wasm(largeManifest),
    100  // Fewer iterations for large manifests
);

// Test 6: Concurrent operations
console.log('Test 6: Concurrent operations');
const concurrentOps = 100;
const start = performance.now();

const promises = Array(concurrentOps).fill(null).map(() =>
    validate_manifest_wasm(manifest)
);

await Promise.all(promises);

const end = performance.now();
const total = end - start;
console.log(`Concurrent operations:`);
console.log(`  Operations: ${concurrentOps}`);
console.log(`  Total time: ${total.toFixed(2)}ms`);
console.log(`  Average: ${(total / concurrentOps).toFixed(3)}ms per operation`);
console.log(`  Effective throughput: ${(concurrentOps / (total / 1000)).toFixed(0)} ops/sec\n`);

// Performance summary
console.log('═══════════════════════════════════════');
console.log('Performance Summary');
console.log('═══════════════════════════════════════');
console.log(`Validation: ${validationPerf.average.toFixed(3)}ms avg`);
console.log(`Generation: ${generationPerf.average.toFixed(3)}ms avg`);
console.log(`Version info: ${versionPerf.average.toFixed(3)}ms avg`);
console.log(`Large manifest: ${largePerf.average.toFixed(3)}ms avg`);
console.log('═══════════════════════════════════════\n');

// Performance assertions
console.log('Performance assertions:');
try {
    console.assert(
        validationPerf.average < 10,
        `Validation should be < 10ms (got ${validationPerf.average.toFixed(3)}ms)`
    );
    console.log('✓ Validation performance acceptable');

    console.assert(
        generationPerf.average < 1,
        `Generation should be < 1ms (got ${generationPerf.average.toFixed(3)}ms)`
    );
    console.log('✓ Generation performance acceptable');

    console.assert(
        versionPerf.average < 0.1,
        `Version info should be < 0.1ms (got ${versionPerf.average.toFixed(3)}ms)`
    );
    console.log('✓ Version info performance acceptable');

    console.assert(
        largePerf.average < 50,
        `Large manifest should be < 50ms (got ${largePerf.average.toFixed(3)}ms)`
    );
    console.log('✓ Large manifest performance acceptable');

    console.log('\n✓ All performance tests passed!');
} catch (error) {
    console.error('\n✗ Performance assertion failed:', error.message);
    process.exit(1);
}
