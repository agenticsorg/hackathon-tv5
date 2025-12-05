/**
 * Streaming Example
 *
 * Demonstrates real-time crawl streaming via WebSocket
 */

import { CrawlerClient } from '../src';

async function main() {
  const client = new CrawlerClient({
    apiKey: process.env.CRAWLER_API_KEY || 'your-api-key',
    baseUrl: process.env.CRAWLER_BASE_URL || 'http://localhost:3000/api',
    debug: true,
  });

  console.log('=== Real-Time Crawl Streaming ===\n');

  try {
    // Start a crawl
    console.log('Starting crawl...');
    const crawl = await client.crawl('https://example.com', {
      maxDepth: 3,
      limit: 50,
      arwDiscovery: true,
    });

    console.log('✓ Crawl started!');
    console.log('  Crawl ID:', crawl.id);
    console.log('\nStreaming results in real-time...\n');

    let resultsReceived = 0;
    const startTime = Date.now();

    // Stream results via WebSocket
    await client.streamCrawl(crawl.id, {
      onStatus: (status) => {
        const elapsed = ((Date.now() - startTime) / 1000).toFixed(1);
        console.log(
          `[${elapsed}s] Progress: ${status.completed}/${status.total} pages (${status.creditsUsed} credits)`
        );
      },

      onResult: (result) => {
        resultsReceived++;
        console.log(`\n[Result #${resultsReceived}]`);
        console.log(`  URL: ${result.url}`);
        console.log(`  Title: ${result.metadata?.title || 'N/A'}`);
        console.log(`  Status: ${result.metadata?.statusCode || 'N/A'}`);

        if (result.arw?.discovered) {
          console.log(`  ✓ ARW Support: ${result.arw.llmsTxtUrl}`);
        }

        if (result.error) {
          console.log(`  ✗ Error: ${result.error}`);
        }
      },

      onError: (error) => {
        console.error('\n✗ Stream error:', error.message);
      },

      onComplete: () => {
        const totalTime = ((Date.now() - startTime) / 1000).toFixed(1);
        console.log(`\n✓ Crawl completed in ${totalTime}s!`);
        console.log(`  Total results received: ${resultsReceived}`);
      },
    });

    // Keep the script running until stream completes
    await new Promise((resolve) => {
      setTimeout(resolve, 60000); // Max 60 seconds
    });

  } catch (error) {
    console.error('Error:', error);
    process.exit(1);
  } finally {
    // Clean up
    client.closeStream();
  }
}

// Run the example
if (require.main === module) {
  main().catch(console.error);
}

export default main;
