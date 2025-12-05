# Agent Ready Web Crawler SDK

A comprehensive TypeScript SDK for interacting with the Agent Ready Web crawler service. This SDK provides a clean, type-safe interface for web scraping, crawling, and ARW (Agent Ready Web) discovery features.

## Features

- 🚀 **Full TypeScript support** with complete type definitions
- 🔄 **Automatic retry** with exponential backoff
- 🌊 **Real-time streaming** via WebSocket
- 📦 **Batch operations** for efficient API usage
- 🤖 **ARW discovery** for llms.txt and machine-view
- ⚡ **Promise-based** async/await API
- 🛡️ **Comprehensive error handling**
- 📝 **Extensive documentation** and examples

## Installation

```bash
npm install @agent-ready-web/crawler-sdk
```

## Quick Start

```typescript
import { CrawlerClient } from '@agent-ready-web/crawler-sdk';

// Initialize client
const client = new CrawlerClient({
  apiKey: 'your-api-key',
  baseUrl: 'https://api.crawler.example.com',
});

// Scrape a single page
const result = await client.scrape('https://example.com');
console.log(result.markdown);

// Start a multi-page crawl
const crawl = await client.crawl('https://example.com', {
  maxDepth: 3,
  limit: 100,
});
console.log('Crawl ID:', crawl.id);
```

## Configuration

```typescript
const client = new CrawlerClient({
  apiKey: 'your-api-key',           // Optional: API key for authentication
  baseUrl: 'http://localhost:3000', // Optional: Base URL (default: http://localhost:3000/api)
  timeout: 30000,                   // Optional: Request timeout in ms (default: 30000)
  retryAttempts: 3,                 // Optional: Number of retry attempts (default: 3)
  retryDelay: 1000,                 // Optional: Initial retry delay in ms (default: 1000)
  debug: false,                     // Optional: Enable debug logging (default: false)
});
```

## Core Features

### Scraping

```typescript
// Single page
const result = await client.scrape('https://example.com', {
  formats: ['markdown', 'html', 'links', 'screenshot'],
  onlyMainContent: true,
});

// Multiple pages
const results = await client.scrapeMany([
  'https://example.com/page1',
  'https://example.com/page2',
]);
```

### Crawling

```typescript
// Start crawl
const crawl = await client.crawl('https://example.com', {
  maxDepth: 3,
  limit: 100,
  arwDiscovery: true,
  machineView: true,
});

// Check status
const status = await client.getCrawlStatus(crawl.id);

// Wait for completion
const results = await client.crawlAndWait('https://example.com');
```

### Real-Time Streaming

```typescript
await client.streamCrawl(crawl.id, {
  onStatus: (status) => console.log('Progress:', status.completed),
  onResult: (result) => console.log('Page:', result.url),
  onComplete: () => console.log('Done!'),
});
```

### ARW Discovery

```typescript
// Check support
const hasSupport = await client.hasARWSupport('https://docs.example.com');

// Discover llms.txt
const arw = await client.discoverARW('https://docs.example.com');
console.log('llms.txt:', arw.llmsTxtUrl);

// Generate machine view
const view = await client.generateMachineView('https://docs.example.com', {
  maxTokens: 4000,
});
```

## Error Handling

```typescript
import {
  CrawlerError,
  AuthenticationError,
  RateLimitError,
  ValidationError,
} from '@agent-ready-web/crawler-sdk';

try {
  const result = await client.scrape('https://example.com');
} catch (error) {
  if (error instanceof AuthenticationError) {
    console.error('Authentication failed');
  } else if (error instanceof RateLimitError) {
    console.error('Rate limit exceeded');
  } else if (error instanceof ValidationError) {
    console.error('Invalid input:', error.details);
  }
}
```

## Examples

Check out the [examples](./examples) directory for complete working examples:

- [basic-usage.ts](./examples/basic-usage.ts) - Basic scraping and crawling
- [arw-discovery.ts](./examples/arw-discovery.ts) - ARW discovery and machine view
- [streaming.ts](./examples/streaming.ts) - Real-time streaming
- [batch-operations.ts](./examples/batch-operations.ts) - Batch operations

Run examples:

```bash
npm run example:basic
npm run example:arw
npm run example:stream
npm run example:batch
```

## API Reference

### CrawlerClient

Main client class for interacting with the crawler service.

#### Methods

- `scrape(url, options?)` - Scrape a single page
- `scrapeMany(urls, options?)` - Scrape multiple pages
- `crawl(url, options?)` - Start a multi-page crawl
- `getCrawlStatus(crawlId)` - Get crawl status
- `cancelCrawl(crawlId)` - Cancel a crawl
- `crawlAndWait(url, options?)` - Start crawl and wait for completion
- `streamCrawl(crawlId, options)` - Stream crawl results in real-time
- `closeStream()` - Close WebSocket stream
- `map(url, options?)` - Generate site map
- `batch(operations)` - Execute batch operations
- `discoverARW(url)` - Discover ARW metadata
- `hasARWSupport(url)` - Check ARW support
- `getLlmsTxt(url)` - Get llms.txt content
- `generateMachineView(url, options?)` - Generate machine view
- `setApiKey(apiKey)` - Update API key

## Development

```bash
# Install dependencies
npm install

# Build the SDK
npm run build

# Run tests
npm test

# Type checking
npm run typecheck

# Lint
npm run lint
```

## License

MIT

## Contributing

Contributions are welcome! Please read our contributing guidelines and submit pull requests.

## Support

For issues and questions:
- GitHub Issues: https://github.com/agent-ready-web/agent-ready-web/issues

## Acknowledgments

Inspired by [Firecrawl](https://www.firecrawl.dev/) and built for the Agent Ready Web ecosystem.
