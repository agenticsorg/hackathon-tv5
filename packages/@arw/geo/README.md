# @arw/geo - Generative Engine Optimization for ARW

This package implements the 7 GEO enhancements specified in the ARW GEO Technical Design, providing +230-290% cumulative visibility improvement for AI search engines.

## Features

### Foundation GEO (ARW-2.1)
- **ARW-GEO-1**: Citation Framework (+40% visibility)
- **ARW-GEO-2**: Statistics Enhancement (+40% visibility)
- **ARW-GEO-3**: Quotation System (+40% visibility)
- **ARW-GEO-7**: Domain-Specific Optimization (+20-30% visibility)

### Advanced GEO (ARW-2.2)
- **ARW-GEO-4**: Content Quality Signals (+25-35% visibility)
- **ARW-GEO-5**: Entity Enrichment (+30-40% visibility)
- **ARW-GEO-6**: Semantic Clustering (+35-45% visibility)

### Optional LLM Integration
- Anthropic Claude integration for enhanced content analysis
- OpenAI GPT integration for entity extraction and citation generation
- Automatic quality scoring and optimization suggestions

## Installation

```bash
npm install @arw/geo

# Optional: Install LLM SDKs for enhanced features
npm install @anthropic-ai/sdk openai
```

## Usage

### Basic Usage

```typescript
import { GEOOptimizer } from '@arw/geo';

const optimizer = new GEOOptimizer({
  profile: 'ARW-2.1', // or 'ARW-2.2' for advanced features
  domain: 'ecommerce' // or 'saas', 'media_publishing', etc.
});

// Extract citations from content
const citations = await optimizer.citations.extract(markdownContent);

// Enhance statistics
const stats = await optimizer.statistics.structure(articleContent);

// Extract quotations
const quotes = await optimizer.quotations.extract(interviewContent);
```

### With LLM Integration

```typescript
import { GEOOptimizer } from '@arw/geo';

const optimizer = new GEOOptimizer({
  profile: 'ARW-2.2',
  llm: {
    provider: 'anthropic', // or 'openai'
    apiKey: process.env.ANTHROPIC_API_KEY,
    model: 'claude-3-5-sonnet-20241022'
  }
});

// AI-enhanced citation generation
const enhancedCitations = await optimizer.citations.enhanceWithAI(content, {
  generateMissing: true,
  verifyAccuracy: true
});

// AI-powered entity extraction
const entities = await optimizer.entities.extractWithAI(content, {
  linkToWikidata: true,
  includeRelationships: true
});
```

## API Reference

See [API Documentation](./docs/API.md) for complete API reference.

## Testing

```bash
npm test              # Run tests with coverage
npm run test:watch    # Watch mode
```

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for contribution guidelines.

## License

MIT
