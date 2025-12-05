/**
 * Batch Operations Example
 *
 * Demonstrates executing multiple operations in a single request
 */

import { CrawlerClient } from '../src';

async function main() {
  const client = new CrawlerClient({
    apiKey: process.env.CRAWLER_API_KEY || 'your-api-key',
    baseUrl: process.env.CRAWLER_BASE_URL || 'http://localhost:3000/api',
    debug: true,
  });

  console.log('=== Batch Operations ===\n');

  try {
    // Define multiple operations
    const operations = [
      {
        type: 'scrape' as const,
        url: 'https://example.com',
        options: {
          formats: ['markdown'],
          onlyMainContent: true,
        },
      },
      {
        type: 'scrape' as const,
        url: 'https://example.com/about',
        options: {
          formats: ['markdown', 'html'],
        },
      },
      {
        type: 'map' as const,
        url: 'https://example.com',
        options: {
          limit: 100,
        },
      },
      {
        type: 'crawl' as const,
        url: 'https://docs.example.com',
        options: {
          maxDepth: 2,
          limit: 20,
          arwDiscovery: true,
        },
      },
    ];

    console.log(`Executing ${operations.length} operations in batch...\n`);

    // Execute all operations
    const startTime = Date.now();
    const result = await client.batch(operations);
    const duration = Date.now() - startTime;

    console.log(`✓ Batch completed in ${duration}ms!\n`);

    // Process results
    console.log('=== Results ===\n');

    result.results.forEach((opResult, i) => {
      const op = operations[i];
      console.log(`${i + 1}. ${op.type.toUpperCase()} ${op.url}`);

      if ('url' in opResult && 'markdown' in opResult) {
        // Scrape result
        console.log(`   ✓ Success - ${opResult.metadata?.title || 'No title'}`);
        console.log(`   Content length: ${opResult.markdown?.length || 0} chars`);
      } else if ('links' in opResult) {
        // Map result
        console.log(`   ✓ Success - Found ${opResult.links.length} links`);
      } else if ('id' in opResult) {
        // Crawl result
        console.log(`   ✓ Started - Crawl ID: ${opResult.id}`);
      }

      console.log();
    });

    // Check for errors
    if (result.errors.length > 0) {
      console.log('=== Errors ===\n');
      result.errors.forEach((err) => {
        const op = operations[err.index];
        console.log(`${err.index + 1}. ${op.type.toUpperCase()} ${op.url}`);
        console.log(`   ✗ Error: ${err.error}\n`);
      });
    }

    // Summary
    console.log('=== Summary ===\n');
    console.log(`Total operations: ${operations.length}`);
    console.log(`Successful: ${result.results.length - result.errors.length}`);
    console.log(`Failed: ${result.errors.length}`);
    console.log(`Total time: ${duration}ms`);
    console.log(`Average time per operation: ${(duration / operations.length).toFixed(2)}ms`);

  } catch (error) {
    console.error('Error:', error);
    process.exit(1);
  }
}

// Run the example
if (require.main === module) {
  main();
}

export default main;
