# ARW Crawler Service

High-performance web crawler API service with ARW (Agent-Ready Web) support.

## Features

- 🚀 Fast HTML parsing and content extraction
- 🤖 ARW manifest discovery and machine view generation
- 📊 Batch processing with queue management
- 🔄 Asynchronous crawling with depth control
- 🧪 90%+ test coverage with comprehensive test suites

## Installation

```bash
npm install
```

## Testing

```bash
# Run all tests
npm test

# Run specific test suites
npm run test:unit          # Unit tests only
npm run test:integration   # Integration tests
npm run test:e2e          # E2E tests

# Watch mode
npm run test:watch

# Coverage report
npm run test:coverage

# Performance benchmarks
npm run test:benchmark
```

## Test Structure

```
tests/
├── unit/                 # Unit tests (isolated components)
│   ├── crawler-engine.test.ts
│   ├── arw-discovery.test.ts
│   ├── content-extractor.test.ts
│   └── ...
├── integration/          # Integration tests (API endpoints)
│   ├── api-endpoints.test.ts
│   └── ...
├── e2e/                 # End-to-end tests (real scenarios)
│   ├── real-website.test.ts
│   └── ...
├── fixtures/            # Test data and samples
│   ├── html-samples.ts
│   ├── arw-manifests.ts
│   └── ...
├── mocks/               # Mock implementations
│   └── ...
└── benchmarks/          # Performance tests
    └── performance.bench.ts
```

## Coverage Goals

- ✅ 90%+ code coverage
- ✅ All critical paths tested
- ✅ Edge cases covered
- ✅ Error scenarios validated
- ✅ Performance benchmarks

## API Documentation

See [API.md](./docs/API.md) for complete API documentation.

## License

MIT
