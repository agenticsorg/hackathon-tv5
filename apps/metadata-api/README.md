# Nexus-UMMID Metadata API

Enterprise-grade metadata API for the Entertainment Discovery platform. Built for 400M+ users on GCP Cloud Run.

## 🚀 Quick Start

```bash
# Install dependencies
npm install

# Copy environment configuration
cp .env.example .env

# Run development server
npm run dev

# Build for production
npm run build

# Start production server
npm start
```

## 📡 API Endpoints

### Health Check
```bash
GET /health
```

### Metadata Operations

**Get all metadata (paginated)**
```bash
GET /api/v1/metadata?page=1&limit=10
```

**Get metadata by ID**
```bash
GET /api/v1/metadata/:id
```

**Create metadata**
```bash
POST /api/v1/metadata
Content-Type: application/json

{
  "title": "Sample Movie",
  "type": "movie",
  "genres": ["action", "sci-fi"],
  "language": "en"
}
```

**Update metadata**
```bash
PUT /api/v1/metadata/:id
Content-Type: application/json

{
  "synopsis": "Updated synopsis"
}
```

**Delete metadata**
```bash
DELETE /api/v1/metadata/:id
```

**Search metadata**
```bash
GET /api/v1/metadata/search/query?q=thriller&limit=10
```

**Enrich metadata with AI**
```bash
POST /api/v1/metadata/:id/enrich
Content-Type: application/json

{
  "fields": ["synopsis", "keywords", "moodTags"],
  "model": "gemini-2.0"
}
```

**Validate metadata**
```bash
POST /api/v1/metadata/:id/validate
Content-Type: application/json

{
  "platform": "netflix"
}
```

## 🏗️ Architecture

```
apps/metadata-api/
├── src/
│   ├── index.ts              # Express server entry point
│   ├── types/
│   │   └── index.ts          # TypeScript interfaces
│   ├── routes/
│   │   └── metadata.ts       # API route handlers
│   ├── services/
│   │   └── MetadataService.ts # Business logic
│   └── middleware/
│       ├── errorHandler.ts   # Error handling
│       └── logger.ts         # Request logging
├── dist/                     # Compiled JavaScript
├── package.json
├── tsconfig.json
└── README.md
```

## 🔧 Technology Stack

- **Runtime**: Node.js 18+
- **Framework**: Express.js
- **Language**: TypeScript (strict mode)
- **Validation**: express-validator
- **Security**: helmet, cors
- **Logging**: winston
- **Compression**: compression

## 📦 Core Types

### MediaMetadata
Complete metadata structure for media assets including:
- Basic info (title, type, release date)
- Descriptive metadata (synopsis, genres, keywords)
- Credits (director, cast, producers)
- Technical specs (resolution, aspect ratio, audio)
- Platform availability & rights
- AI-powered enrichment data

### ContentRecommendation
Semantic recommendations with similarity scores

### EnrichmentRequest/Result
AI-powered metadata enrichment tracking

### ValidationResult
Platform-specific validation results

## 🔐 Security Features

- Helmet.js security headers
- CORS configuration
- Request size limits
- Input validation
- Error sanitization
- Rate limiting (planned)

## 📊 Monitoring

- Request/response logging
- Performance metrics
- Error tracking
- Health check endpoint

## 🚢 Deployment

Built for GCP Cloud Run with:
- Automatic scaling
- Zero-downtime deployments
- Health check probes
- Graceful shutdown
- Environment-based configuration

## 📝 Development

```bash
# Type checking
npm run typecheck

# Linting
npm run lint

# Run tests
npm run test

# Watch mode
npm run dev
```

## 🔮 Future Enhancements

- [ ] Vertex AI integration for semantic search
- [ ] Firestore/Cloud SQL persistence
- [ ] AgentDB learning integration
- [ ] Claude Flow workflow orchestration
- [ ] MCP server for AI assistants
- [ ] ARW manifest for agent discovery
- [ ] Platform connector implementations
- [ ] Rights collision detection
- [ ] Real-time updates with Pub/Sub

## 📄 License

MIT

## 👥 Author

mondweep - Nexus-UMMID Hackathon
