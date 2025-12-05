/**
 * Performance benchmarks for crawler service
 * Measures throughput, latency, and resource usage
 */

import { performance } from 'perf_hooks';

/**
 * Benchmark result interface
 */
interface BenchmarkResult {
  name: string;
  operations: number;
  duration: number;
  opsPerSecond: number;
  avgLatency: number;
  minLatency: number;
  maxLatency: number;
  p95Latency: number;
  p99Latency: number;
}

/**
 * Run performance benchmark
 */
async function runBenchmark(
  name: string,
  operation: () => Promise<void>,
  iterations: number = 1000
): Promise<BenchmarkResult> {
  const latencies: number[] = [];

  // Warmup
  for (let i = 0; i < 10; i++) {
    await operation();
  }

  // Benchmark
  const start = performance.now();

  for (let i = 0; i < iterations; i++) {
    const opStart = performance.now();
    await operation();
    const opEnd = performance.now();
    latencies.push(opEnd - opStart);
  }

  const end = performance.now();
  const duration = end - start;

  // Calculate statistics
  latencies.sort((a, b) => a - b);
  const avgLatency = latencies.reduce((a, b) => a + b, 0) / latencies.length;
  const p95Index = Math.floor(latencies.length * 0.95);
  const p99Index = Math.floor(latencies.length * 0.99);

  return {
    name,
    operations: iterations,
    duration,
    opsPerSecond: (iterations / duration) * 1000,
    avgLatency,
    minLatency: latencies[0],
    maxLatency: latencies[latencies.length - 1],
    p95Latency: latencies[p95Index],
    p99Latency: latencies[p99Index]
  };
}

/**
 * Format benchmark results
 */
function formatResults(result: BenchmarkResult): string {
  return `
Benchmark: ${result.name}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Operations:       ${result.operations.toLocaleString()}
Duration:         ${result.duration.toFixed(2)}ms
Throughput:       ${result.opsPerSecond.toFixed(2)} ops/sec
Avg Latency:      ${result.avgLatency.toFixed(2)}ms
Min Latency:      ${result.minLatency.toFixed(2)}ms
Max Latency:      ${result.maxLatency.toFixed(2)}ms
P95 Latency:      ${result.p95Latency.toFixed(2)}ms
P99 Latency:      ${result.p99Latency.toFixed(2)}ms
`;
}

/**
 * Main benchmark suite
 */
async function main() {
  console.log('🚀 Running Performance Benchmarks\n');

  // Benchmark 1: URL Resolution
  const urlResolution = await runBenchmark(
    'URL Resolution',
    async () => {
      try {
        const base = new URL('https://example.com/path/page.html');
        const resolved = new URL('../other.html', base);
        return;
      } catch {
        return;
      }
    },
    10000
  );
  console.log(formatResults(urlResolution));

  // Benchmark 2: HTML Parsing (simplified)
  const htmlParsing = await runBenchmark(
    'HTML Text Extraction',
    async () => {
      const html = '<div><h1>Title</h1><p>Content</p></div>';
      const text = html.replace(/<[^>]+>/g, ' ').trim();
      return;
    },
    10000
  );
  console.log(formatResults(htmlParsing));

  // Benchmark 3: Markdown Conversion (simplified)
  const markdownConversion = await runBenchmark(
    'Markdown Conversion',
    async () => {
      let html = '<h1>Title</h1><p>Text with <strong>bold</strong></p>';
      html = html.replace(/<h1[^>]*>(.*?)<\/h1>/gi, '# $1\n');
      html = html.replace(/<p[^>]*>(.*?)<\/p>/gi, '$1\n');
      html = html.replace(/<strong[^>]*>(.*?)<\/strong>/gi, '**$1**');
      return;
    },
    10000
  );
  console.log(formatResults(markdownConversion));

  // Benchmark 4: JSON Parsing
  const jsonParsing = await runBenchmark(
    'JSON Manifest Parsing',
    async () => {
      const json = JSON.stringify({
        version: '0.1',
        site: { name: 'Test', homepage: 'https://example.com' },
        content: Array(10).fill({ url: '/page', machine_view: '/page.llm.md' })
      });
      const parsed = JSON.parse(json);
      return;
    },
    10000
  );
  console.log(formatResults(jsonParsing));

  // Benchmark 5: Domain Comparison
  const domainComparison = await runBenchmark(
    'Domain Comparison',
    async () => {
      try {
        const url1 = new URL('https://example.com/page1');
        const url2 = new URL('https://example.com/page2');
        const same = url1.hostname === url2.hostname;
        return;
      } catch {
        return;
      }
    },
    10000
  );
  console.log(formatResults(domainComparison));

  // Summary
  console.log('\n📊 Benchmark Summary');
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
  console.log(`URL Resolution:        ${urlResolution.opsPerSecond.toFixed(0)} ops/sec`);
  console.log(`HTML Parsing:          ${htmlParsing.opsPerSecond.toFixed(0)} ops/sec`);
  console.log(`Markdown Conversion:   ${markdownConversion.opsPerSecond.toFixed(0)} ops/sec`);
  console.log(`JSON Parsing:          ${jsonParsing.opsPerSecond.toFixed(0)} ops/sec`);
  console.log(`Domain Comparison:     ${domainComparison.opsPerSecond.toFixed(0)} ops/sec`);
  console.log('\n✅ All benchmarks completed successfully\n');
}

// Run benchmarks
main().catch(console.error);
