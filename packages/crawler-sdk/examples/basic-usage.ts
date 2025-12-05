/**
 * Basic Usage Example
 *
 * Demonstrates basic scraping and crawling functionality
 */

import { CrawlerClient } from '../src';

async function main() {
  // Initialize client
  const client = new CrawlerClient({
    apiKey: process.env.CRAWLER_API_KEY || 'your-api-key',
    baseUrl: process.env.CRAWLER_BASE_URL || 'http://localhost:3000/api',
    debug: true,
  });

  console.log('=== Basic Scraping ===\n');

  try {
    // 1. Scrape a single page
    console.log('Scraping single page...');
    const scrapeResult = await client.scrape('https://example.com', {
      formats: ['markdown', 'html'],
      onlyMainContent: true,
    });

    console.log('✓ Scrape successful!');
    console.log('  URL:', scrapeResult.url);
    console.log('  Title:', scrapeResult.metadata?.title);
    console.log('  Content length:', scrapeResult.markdown?.length || 0);
    console.log();

    // 2. Scrape multiple pages
    console.log('Scraping multiple pages...');
    const batchResults = await client.scrapeMany([
      'https://example.com',
      'https://example.com/about',
    ]);

    console.log(`✓ Scraped ${batchResults.length} pages`);
    batchResults.forEach((result, i) => {
      console.log(`  ${i + 1}. ${result.url} - ${result.metadata?.title}`);
    });
    console.log();

    console.log('=== Site Mapping ===\n');

    // 3. Generate site map
    console.log('Generating site map...');
    const mapResult = await client.map('https://example.com', {
      limit: 100,
      includeSubdomains: false,
    });

    console.log('✓ Map generated!');
    console.log(`  Found ${mapResult.links.length} links`);
    console.log('  Sample links:');
    mapResult.links.slice(0, 5).forEach((link, i) => {
      console.log(`    ${i + 1}. ${link}`);
    });
    console.log();

    console.log('=== Multi-Page Crawling ===\n');

    // 4. Start a crawl
    console.log('Starting crawl...');
    const crawl = await client.crawl('https://example.com', {
      maxDepth: 2,
      limit: 50,
      arwDiscovery: true,
    });

    console.log('✓ Crawl started!');
    console.log('  Crawl ID:', crawl.id);
    console.log('  Status:', crawl.status);
    console.log();

    // 5. Poll for crawl completion
    console.log('Waiting for crawl to complete...');
    const finalStatus = await client.crawlAndWait(crawl.id, {
      pollInterval: 2000,
    });

    console.log('✓ Crawl completed!');
    console.log(`  Total pages: ${finalStatus.total}`);
    console.log(`  Completed: ${finalStatus.completed}`);
    console.log(`  Credits used: ${finalStatus.creditsUsed}`);

    if (finalStatus.data) {
      console.log('\n  Sample results:');
      finalStatus.data.slice(0, 3).forEach((result, i) => {
        console.log(`    ${i + 1}. ${result.url}`);
        console.log(`       Title: ${result.metadata?.title}`);
      });
    }

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
