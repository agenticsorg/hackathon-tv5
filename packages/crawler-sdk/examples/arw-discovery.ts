/**
 * ARW Discovery Example
 *
 * Demonstrates Agent Ready Web discovery and machine view features
 */

import { CrawlerClient } from '../src';
import { ARWHelper } from '../src/arw/discovery';

async function main() {
  const client = new CrawlerClient({
    apiKey: process.env.CRAWLER_API_KEY || 'your-api-key',
    baseUrl: process.env.CRAWLER_BASE_URL || 'http://localhost:3000/api',
    debug: true,
  });

  const docsUrl = 'https://docs.example.com';

  console.log('=== ARW Discovery ===\n');

  try {
    // 1. Check if site supports ARW
    console.log('Checking ARW support...');
    const hasSupport = await client.hasARWSupport(docsUrl);

    if (hasSupport) {
      console.log('✓ Site supports Agent Ready Web!');
    } else {
      console.log('✗ Site does not support ARW');
      // Generate potential URLs anyway
      const potentialUrls = ARWHelper.generateLlmsTxtUrls(docsUrl);
      console.log('\nPotential llms.txt URLs to check:');
      potentialUrls.forEach((url, i) => {
        console.log(`  ${i + 1}. ${url}`);
      });
    }
    console.log();

    // 2. Discover ARW metadata
    console.log('Discovering ARW metadata...');
    const arwMetadata = await client.discoverARW(docsUrl);

    if (arwMetadata.discovered) {
      console.log('✓ ARW metadata discovered!');
      console.log('  llms.txt URL:', arwMetadata.llmsTxtUrl);
      console.log('  Discovery method:', arwMetadata.discoveryMethod);

      // 3. Get llms.txt content
      console.log('\nFetching llms.txt content...');
      const llmsTxt = await client.getLlmsTxt(docsUrl);

      if (llmsTxt) {
        console.log('✓ llms.txt content retrieved!');
        console.log('\n--- llms.txt Content ---');
        console.log(llmsTxt);
        console.log('--- End Content ---\n');

        // 4. Parse llms.txt sections
        const sections = client.arw.parseLlmsTxt(llmsTxt);
        console.log(`Parsed ${sections.length} sections:`);
        sections.forEach((section, i) => {
          console.log(`  ${i + 1}. ${section.title}`);
          console.log(`     URL: ${section.url}`);
          if (section.description) {
            console.log(`     Description: ${section.description}`);
          }
        });
      }
    } else {
      console.log('✗ No ARW support detected');
    }
    console.log();

    console.log('=== Machine View Generation ===\n');

    // 5. Generate machine-optimized view
    console.log('Generating machine view...');
    const machineView = await client.generateMachineView(docsUrl, {
      maxTokens: 4000,
      prioritySections: ['introduction', 'api', 'guide'],
    });

    console.log('✓ Machine view generated!');
    console.log(`  Sections: ${machineView.sections.length}`);
    console.log(`  Tokens: ${machineView.tokens || 'N/A'}`);
    console.log(`  Optimized: ${machineView.optimized}`);

    console.log('\n  Section breakdown:');
    machineView.sections.forEach((section, i) => {
      console.log(`    ${i + 1}. ${section.type}`);
      console.log(`       Priority: ${section.priority || 'default'}`);
      console.log(`       Length: ${section.content.length} chars`);
    });
    console.log();

    // 6. Scrape with ARW discovery
    console.log('=== Scraping with ARW ===\n');
    console.log('Scraping with ARW discovery enabled...');

    const scrapeResult = await client.scrape(docsUrl, {
      formats: ['markdown'],
      arwDiscovery: true,
      onlyMainContent: true,
    });

    if (scrapeResult.arw?.discovered) {
      console.log('✓ ARW data included in scrape result!');
      console.log('  llms.txt URL:', scrapeResult.arw.llmsTxtUrl);

      if (scrapeResult.arw.machineView) {
        console.log('  Machine view sections:', scrapeResult.arw.machineView.sections.length);
      }
    }

    // 7. Crawl with machine view
    console.log('\nStarting crawl with machine view...');
    const crawl = await client.crawl(docsUrl, {
      maxDepth: 2,
      limit: 20,
      arwDiscovery: true,
      machineView: true,
    });

    console.log('✓ Crawl started with ARW features!');
    console.log('  Crawl ID:', crawl.id);

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
