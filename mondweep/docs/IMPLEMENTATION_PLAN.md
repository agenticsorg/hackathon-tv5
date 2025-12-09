# **Nexus-UMMID Implementation Plan**

## **Hackathon Sprint: 4-Week Implementation**

**Project:** Nexus-UMMID - Cognitive Hypergraph Media Metadata Platform  
**Timeline:** 4 weeks (Hackathon focused)  
**Team:** Solo developer (@mondweep)  
**Platform:** Google Cloud Platform (GCP) Exclusive  
**GCP Project:** `agentics-foundation25lon-1899`

---

## **Week 1: Foundation & Infrastructure**

### **Day 1-2: GCP Setup & Core Services**

#### **Task 1.1: GCP Project Configuration**
```bash
# Set active project
gcloud config set project agentics-foundation25lon-1899
gcloud config set compute/region us-central1
gcloud config set compute/zone us-central1-a

# Enable required APIs
gcloud services enable run.googleapis.com
gcloud services enable firestore.googleapis.com
gcloud services enable aiplatform.googleapis.com
gcloud services enable sqladmin.googleapis.com
gcloud services enable pubsub.googleapis.com
gcloud services enable storage.googleapis.com
gcloud services enable cloudfunctions.googleapis.com
gcloud services enable workflows.googleapis.com
gcloud services enable cloudscheduler.googleapis.com
gcloud services enable secretmanager.googleapis.com

# Create service account
gcloud iam service-accounts create ummid-service \
  --display-name="UMMID Service Account"

# Grant necessary roles
gcloud projects add-iam-policy-binding agentics-foundation25lon-1899 \
  --member="serviceAccount:ummid-service@agentics-foundation25lon-1899.iam.gserviceaccount.com" \
  --role="roles/run.admin"

gcloud projects add-iam-policy-binding agentics-foundation25lon-1899 \
  --member="serviceAccount:ummid-service@agentics-foundation25lon-1899.iam.gserviceaccount.com" \
  --role="roles/datastore.user"

gcloud projects add-iam-policy-binding agentics-foundation25lon-1899 \
  --member="serviceAccount:ummid-service@agentics-foundation25lon-1899.iam.gserviceaccount.com" \
  --role="roles/aiplatform.user"
```

**Deliverable:** ✅ GCP project fully configured with all services enabled

---

#### **Task 1.2: Firestore Hypergraph Schema**

Create Firestore database and define hypergraph collections:

```bash
# Create Firestore database
gcloud firestore databases create \
  --location=us-central1 \
  --type=firestore-native
```

**Firestore Schema Design:**

```typescript
// mondweep/apps/metadata-api/src/schema/firestore-schema.ts

export interface Asset {
  id: string;                    // EIDR or internal UUID
  type: 'movie' | 'series' | 'episode';
  title: string;
  original_title?: string;
  year: number;
  synopsis: string;
  genres: string[];
  cast: string[];
  directors: string[];
  duration: number;              // minutes
  rating: string;                // MPAA rating
  
  // Technical metadata
  technical: {
    resolution: string;          // UHD, HD, SD
    aspect_ratio: string;        // 16:9, 2.35:1
    audio_config: string;        // 5.1, Atmos
    color_space: string;         // SDR, HDR10, Dolby Vision
  };
  
  // Enrichment metadata
  enrichment: {
    keywords: string[];
    mood_tags: string[];
    themes: string[];
    target_audience: string;
    similar_titles: string[];
    ai_generated: boolean;
    enrichment_source: string;
    enrichment_timestamp: string;
  };
  
  // Vector embedding
  embedding?: number[];
  
  // Audit trail
  created_at: FirebaseFirestore.Timestamp;
  updated_at: FirebaseFirestore.Timestamp;
  created_by: string;
  updated_by: string;
}

export interface RightsHyperedge {
  id: string;
  asset_id: string;              // Reference to Asset
  territory: string;             // ISO 3166-1 alpha-2
  platform: string;              // netflix, amazon, hulu, etc.
  license_type: 'exclusive' | 'non-exclusive';
  distribution_type: 'SVOD' | 'TVOD' | 'AVOD' | 'FAST';
  
  // Temporal validity (bitemporal)
  valid_from: FirebaseFirestore.Timestamp;
  valid_to: FirebaseFirestore.Timestamp;
  transaction_time: FirebaseFirestore.Timestamp;
  
  // Quality constraints
  max_resolution: string;
  audio_requirements: string[];
  
  // Business terms
  price_tier?: string;
  revenue_share?: number;
  
  // Status
  status: 'active' | 'expired' | 'pending' | 'conflict';
  conflict_with?: string[];      // IDs of conflicting rights
  
  created_at: FirebaseFirestore.Timestamp;
  updated_at: FirebaseFirestore.Timestamp;
}

export interface DeliveryLog {
  id: string;
  asset_id: string;
  platform: string;
  delivery_type: 'initial' | 'update' | 'takedown';
  
  // Timing
  initiated_at: FirebaseFirestore.Timestamp;
  completed_at?: FirebaseFirestore.Timestamp;
  duration_ms?: number;
  
  // Status
  status: 'pending' | 'processing' | 'success' | 'failed';
  error_type?: string;
  error_message?: string;
  
  // Package details
  package_location: string;      // GCS path
  package_size_bytes: number;
  
  // SLA tracking
  sla_target_hours: number;
  sla_met: boolean;
}
```

**Firestore Indexes:**

```bash
# Create composite indexes
gcloud firestore indexes composite create \
  --collection-group=rights_hyperedges \
  --field-config field-path=asset_id,order=ascending \
  --field-config field-path=territory,order=ascending \
  --field-config field-path=platform,order=ascending \
  --field-config field-path=valid_from,order=ascending

gcloud firestore indexes composite create \
  --collection-group=delivery_logs \
  --field-config field-path=platform,order=ascending \
  --field-config field-path=status,order=ascending \
  --field-config field-path=initiated_at,order=descending
```

**Deliverable:** ✅ Firestore schema defined and indexes created

---

#### **Task 1.3: Cloud SQL with pgvector**

Setup PostgreSQL database with pgvector extension for embeddings:

```bash
# Create Cloud SQL instance
gcloud sql instances create ummid-vectors \
  --database-version=POSTGRES_15 \
  --tier=db-n1-standard-2 \
  --region=us-central1 \
  --storage-size=100GB \
  --storage-type=SSD \
  --storage-auto-increase \
  --backup-start-time=03:00 \
  --maintenance-window-day=SUN \
  --maintenance-window-hour=04

# Set root password
gcloud sql users set-password postgres \
  --instance=ummid-vectors \
  --password=$(openssl rand -base64 32)

# Create database
gcloud sql databases create ummid \
  --instance=ummid-vectors

# Enable pgvector extension (via Cloud SQL proxy)
gcloud sql connect ummid-vectors --user=postgres --database=ummid
```

```sql
-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Create embeddings table
CREATE TABLE asset_embeddings (
  id VARCHAR(255) PRIMARY KEY,
  asset_id VARCHAR(255) NOT NULL,
  embedding vector(768),  -- Gemini embedding dimensions
  embedding_model VARCHAR(100) NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(asset_id)
);

-- Create index for similarity search
CREATE INDEX ON asset_embeddings USING ivfflat (embedding vector_cosine_ops)
WITH (lists = 100);

-- Create metadata table for caching
CREATE TABLE metadata_cache (
  id VARCHAR(255) PRIMARY KEY,
  asset_id VARCHAR(255) NOT NULL,
  field_name VARCHAR(100) NOT NULL,
  field_value TEXT,
  source VARCHAR(100),
  confidence FLOAT,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(asset_id, field_name)
);

-- Create EIDR mapping table
CREATE TABLE eidr_mappings (
  internal_id VARCHAR(255) PRIMARY KEY,
  eidr_id VARCHAR(255) UNIQUE,
  tms_id VARCHAR(255),
  imdb_id VARCHAR(255),
  netflix_id VARCHAR(255),
  amazon_id VARCHAR(255),
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

**Deliverable:** ✅ Cloud SQL instance with pgvector configured

---

### **Day 3-4: RuVector Service Deployment**

#### **Task 1.4: Deploy RuVector to Cloud Run**

```bash
# Navigate to RuVector directory
cd mondweep/ruvector-engine

# Create .env for Cloud Run
cat > .env.production << EOF
PORT=8080
GCP_PROJECT_ID=agentics-foundation25lon-1899
GCP_REGION=us-central1
VECTOR_DIMENSIONS=768
MAX_INSTANCES=100
NODE_ENV=production
EOF

# Build and deploy to Cloud Run
gcloud run deploy ruvector-engine \
  --source . \
  --region us-central1 \
  --platform managed \
  --allow-unauthenticated \
  --memory 2Gi \
  --cpu 2 \
  --min-instances 1 \
  --max-instances 10 \
  --timeout 300 \
  --service-account ummid-service@agentics-foundation25lon-1899.iam.gserviceaccount.com \
  --set-env-vars "$(cat .env.production | tr '\n' ',')"

# Get service URL
export RUVECTOR_URL=$(gcloud run services describe ruvector-engine \
  --region us-central1 \
  --format 'value(status.url)')

echo "RuVector deployed at: $RUVECTOR_URL"

# Test deployment
curl $RUVECTOR_URL/api/v1/health
```

**Deliverable:** ✅ RuVector service deployed and accessible

---

#### **Task 1.5: Create Metadata API Service**

```bash
# Create new service directory
mkdir -p mondweep/apps/metadata-api
cd mondweep/apps/metadata-api

# Initialize Node.js project
npm init -y

# Install dependencies
npm install express @google-cloud/firestore @google-cloud/vertexai \
  @google-cloud/pubsub @google-cloud/storage pg dotenv cors helmet \
  express-rate-limit zod

npm install -D typescript @types/node @types/express @types/pg \
  ts-node nodemon
```

**Create basic API structure:**

```typescript
// mondweep/apps/metadata-api/src/index.ts
import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import rateLimit from 'express-rate-limit';
import { Firestore } from '@google-cloud/firestore';
import { VertexAI } from '@google-cloud/vertexai';

const app = express();
const port = process.env.PORT || 8080;

// Middleware
app.use(helmet());
app.use(cors());
app.use(express.json());

// Rate limiting
const limiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100 // limit each IP to 100 requests per windowMs
});
app.use('/api/', limiter);

// Initialize GCP clients
const firestore = new Firestore({
  projectId: 'agentics-foundation25lon-1899'
});

const vertexAI = new VertexAI({
  project: 'agentics-foundation25lon-1899',
  location: 'us-central1'
});

// Health check
app.get('/health', (req, res) => {
  res.json({ status: 'healthy', timestamp: new Date().toISOString() });
});

// API routes
import { assetRoutes } from './routes/assets';
import { rightsRoutes } from './routes/rights';
import { deliveryRoutes } from './routes/delivery';
import { searchRoutes } from './routes/search';

app.use('/api/v1/assets', assetRoutes);
app.use('/api/v1/rights', rightsRoutes);
app.use('/api/v1/delivery', deliveryRoutes);
app.use('/api/v1/search', searchRoutes);

// Error handling
app.use((err: any, req: express.Request, res: express.Response, next: express.NextFunction) => {
  console.error(err.stack);
  res.status(500).json({ error: 'Internal server error' });
});

app.listen(port, () => {
  console.log(`Metadata API listening on port ${port}`);
});
```

**Deploy to Cloud Run:**

```bash
# Create Dockerfile
cat > Dockerfile << 'EOF'
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY . .
RUN npm run build
EXPOSE 8080
CMD ["node", "dist/index.js"]
EOF

# Deploy
gcloud run deploy metadata-api \
  --source . \
  --region us-central1 \
  --platform managed \
  --allow-unauthenticated \
  --memory 1Gi \
  --cpu 1 \
  --min-instances 1 \
  --max-instances 20 \
  --service-account ummid-service@agentics-foundation25lon-1899.iam.gserviceaccount.com
```

**Deliverable:** ✅ Metadata API service deployed

---

### **Day 5-7: Agentic-Synth Integration**

#### **Task 1.6: Setup Agentic-Synth for Test Data**

```bash
cd mondweep/apps/metadata-api

# Install agentic-synth
npm install @ruvector/agentic-synth
```

**Create synthetic data generator:**

```typescript
// mondweep/apps/metadata-api/src/utils/synthetic-data.ts
import { AgenticSynth } from '@ruvector/agentic-synth';

export class SyntheticDataGenerator {
  private synth: AgenticSynth;
  
  constructor() {
    this.synth = new AgenticSynth({
      provider: 'gemini',
      apiKey: process.env.GEMINI_API_KEY,
      model: 'gemini-2.0-flash-exp',
      cache: { enabled: true, maxSize: 1000 }
    });
  }
  
  async generateTestMetadata(count: number = 100) {
    return await this.synth.generateStructured({
      count,
      schema: {
        title: { type: 'string', format: 'movieTitle' },
        original_title: { type: 'string', format: 'movieTitle' },
        year: { type: 'number', min: 1920, max: 2025 },
        synopsis: { 
          type: 'string', 
          length: { min: 150, max: 500 },
          description: 'Compelling movie synopsis'
        },
        genres: { 
          type: 'array', 
          items: { 
            type: 'string',
            enum: ['Action', 'Comedy', 'Drama', 'Horror', 'Sci-Fi', 'Thriller', 'Romance', 'Documentary']
          },
          count: { min: 1, max: 3 }
        },
        cast: { 
          type: 'array', 
          items: { type: 'string', format: 'fullName' },
          count: { min: 3, max: 10 }
        },
        directors: { 
          type: 'array', 
          items: { type: 'string', format: 'fullName' },
          count: { min: 1, max: 2 }
        },
        duration: { type: 'number', min: 60, max: 240 },
        rating: { 
          type: 'string', 
          enum: ['G', 'PG', 'PG-13', 'R', 'NC-17', 'TV-MA', 'TV-14']
        },
        technical: {
          resolution: { type: 'string', enum: ['SD', 'HD', 'UHD'] },
          aspect_ratio: { type: 'string', enum: ['16:9', '2.35:1', '1.85:1'] },
          audio_config: { type: 'string', enum: ['Stereo', '5.1', 'Atmos'] },
          color_space: { type: 'string', enum: ['SDR', 'HDR10', 'Dolby Vision'] }
        }
      }
    });
  }
  
  async generateRightsScenarios(assetIds: string[], count: number = 50) {
    return await this.synth.generateStructured({
      count,
      schema: {
        asset_id: { type: 'string', enum: assetIds },
        territory: { 
          type: 'string', 
          enum: ['US', 'GB', 'FR', 'DE', 'JP', 'CA', 'AU', 'MX', 'BR', 'IN']
        },
        platform: { 
          type: 'string', 
          enum: ['netflix', 'amazon', 'hulu', 'disney', 'apple', 'roku', 'pluto']
        },
        license_type: { type: 'string', enum: ['exclusive', 'non-exclusive'] },
        distribution_type: { type: 'string', enum: ['SVOD', 'TVOD', 'AVOD', 'FAST'] },
        valid_from: { type: 'date', min: '2024-01-01', max: '2025-12-31' },
        valid_to: { type: 'date', min: '2025-01-01', max: '2027-12-31' }
      }
    });
  }
  
  async generatePlatformTestCases(platform: string) {
    // Generate edge cases for platform validation
    return await this.synth.generateStructured({
      count: 20,
      schema: {
        test_case_name: { type: 'string' },
        scenario: { type: 'string', enum: ['missing_field', 'invalid_format', 'character_limit', 'encoding_issue'] },
        asset_data: { type: 'object' },
        expected_error: { type: 'string' },
        severity: { type: 'string', enum: ['critical', 'warning', 'info'] }
      }
    });
  }
}
```

**Create test data seeding script:**

```typescript
// mondweep/apps/metadata-api/scripts/seed-test-data.ts
import { SyntheticDataGenerator } from '../src/utils/synthetic-data';
import { Firestore } from '@google-cloud/firestore';

async function seedTestData() {
  const generator = new SyntheticDataGenerator();
  const firestore = new Firestore();
  
  console.log('Generating 1000 test assets...');
  const assets = await generator.generateTestMetadata(1000);
  
  console.log(`Generated ${assets.data.length} assets`);
  console.log(`Quality: ${(assets.metadata.quality * 100).toFixed(1)}%`);
  
  // Insert into Firestore
  const batch = firestore.batch();
  let count = 0;
  
  for (const asset of assets.data) {
    const docRef = firestore.collection('assets').doc();
    batch.set(docRef, {
      ...asset,
      id: docRef.id,
      created_at: Firestore.Timestamp.now(),
      updated_at: Firestore.Timestamp.now(),
      created_by: 'synthetic-generator',
      updated_by: 'synthetic-generator'
    });
    
    count++;
    
    // Firestore batch limit is 500
    if (count % 500 === 0) {
      await batch.commit();
      console.log(`Inserted ${count} assets...`);
    }
  }
  
  await batch.commit();
  console.log(`✅ Inserted ${count} total assets`);
  
  // Generate rights scenarios
  const assetIds = assets.data.map((_, i) => `asset-${i}`);
  console.log('Generating rights scenarios...');
  const rights = await generator.generateRightsScenarios(assetIds.slice(0, 100), 200);
  
  // Insert rights
  const rightsBatch = firestore.batch();
  for (const right of rights.data) {
    const docRef = firestore.collection('rights_hyperedges').doc();
    rightsBatch.set(docRef, {
      ...right,
      id: docRef.id,
      status: 'active',
      created_at: Firestore.Timestamp.now(),
      updated_at: Firestore.Timestamp.now()
    });
  }
  
  await rightsBatch.commit();
  console.log(`✅ Inserted ${rights.data.length} rights hyperedges`);
}

seedTestData().catch(console.error);
```

**Run seeding:**

```bash
npm run seed-test-data
```

**Deliverable:** ✅ 1000+ synthetic test records in Firestore

---

## **Week 2: Intelligence & Enrichment**

### **Day 8-10: Gemini Integration for Embeddings**

#### **Task 2.1: Embedding Generation Service**

```typescript
// mondweep/apps/metadata-api/src/services/embedding-service.ts
import { VertexAI } from '@google-cloud/vertexai';
import { Pool } from 'pg';

export class EmbeddingService {
  private vertexAI: VertexAI;
  private pgPool: Pool;
  
  constructor() {
    this.vertexAI = new VertexAI({
      project: 'agentics-foundation25lon-1899',
      location: 'us-central1'
    });
    
    this.pgPool = new Pool({
      host: process.env.CLOUD_SQL_HOST,
      database: 'ummid',
      user: 'postgres',
      password: process.env.CLOUD_SQL_PASSWORD,
      max: 20
    });
  }
  
  async generateEmbedding(text: string): Promise<number[]> {
    const model = this.vertexAI.preview.getGenerativeModel({
      model: 'gemini-2.0-flash-exp'
    });
    
    const result = await model.embedContent(text);
    return result.embedding.values;
  }
  
  async generateAssetEmbedding(asset: any): Promise<number[]> {
    // Combine metadata fields for comprehensive embedding
    const text = `
      Title: ${asset.title}
      Synopsis: ${asset.synopsis}
      Genres: ${asset.genres.join(', ')}
      Cast: ${asset.cast.join(', ')}
      Directors: ${asset.directors.join(', ')}
      Year: ${asset.year}
      Rating: ${asset.rating}
      Mood: ${asset.enrichment?.mood_tags?.join(', ') || ''}
      Keywords: ${asset.enrichment?.keywords?.join(', ') || ''}
    `.trim();
    
    return await this.generateEmbedding(text);
  }
  
  async storeEmbedding(assetId: string, embedding: number[]): Promise<void> {
    const query = `
      INSERT INTO asset_embeddings (id, asset_id, embedding, embedding_model)
      VALUES ($1, $2, $3, $4)
      ON CONFLICT (asset_id) 
      DO UPDATE SET embedding = $3, updated_at = CURRENT_TIMESTAMP
    `;
    
    await this.pgPool.query(query, [
      `emb-${assetId}`,
      assetId,
      `[${embedding.join(',')}]`,
      'gemini-2.0-flash-exp'
    ]);
  }
  
  async semanticSearch(queryText: string, limit: number = 10): Promise<any[]> {
    const queryEmbedding = await this.generateEmbedding(queryText);
    
    const query = `
      SELECT 
        asset_id,
        1 - (embedding <=> $1::vector) as similarity
      FROM asset_embeddings
      ORDER BY embedding <=> $1::vector
      LIMIT $2
    `;
    
    const result = await this.pgPool.query(query, [
      `[${queryEmbedding.join(',')}]`,
      limit
    ]);
    
    return result.rows;
  }
}
```

**Deliverable:** ✅ Embedding generation and semantic search working

---

#### **Task 2.2: Metadata Enrichment with Gemini**

```typescript
// mondweep/apps/metadata-api/src/services/enrichment-service.ts
import { VertexAI } from '@google-cloud/vertexai';

export class EnrichmentService {
  private vertexAI: VertexAI;
  
  constructor() {
    this.vertexAI = new VertexAI({
      project: 'agentics-foundation25lon-1899',
      location: 'us-central1'
    });
  }
  
  async enrichMetadata(partialMetadata: any): Promise<any> {
    const model = this.vertexAI.preview.getGenerativeModel({
      model: 'gemini-2.0-flash-exp'
    });
    
    const prompt = `
      You are a professional media metadata specialist. Given the following partial metadata,
      generate missing fields with high-quality, professional content suitable for streaming platforms.
      
      Existing Metadata:
      ${JSON.stringify(partialMetadata, null, 2)}
      
      Generate the following if missing:
      1. synopsis: A compelling 150-200 word synopsis
      2. keywords: 10-15 relevant SEO keywords
      3. mood_tags: 3-5 descriptive mood tags (e.g., "dark", "uplifting", "intense")
      4. themes: 3-5 thematic elements
      5. target_audience: Detailed target audience description
      6. similar_titles: 5 similar titles with brief explanations
      
      Return ONLY valid JSON with these fields. Do not include any markdown formatting or code blocks.
    `;
    
    const result = await model.generateContent(prompt);
    const responseText = result.response.text();
    
    // Clean up response (remove markdown code blocks if present)
    const cleanedText = responseText
      .replace(/```json\n?/g, '')
      .replace(/```\n?/g, '')
      .trim();
    
    try {
      const enriched = JSON.parse(cleanedText);
      
      return {
        ...partialMetadata,
        enrichment: {
          ...enriched,
          ai_generated: true,
          enrichment_source: 'gemini-2.0-flash-exp',
          enrichment_timestamp: new Date().toISOString()
        }
      };
    } catch (error) {
      console.error('Failed to parse Gemini response:', cleanedText);
      throw new Error('Invalid JSON response from Gemini');
    }
  }
  
  async generateSynopsis(title: string, genres: string[], cast: string[]): Promise<string> {
    const model = this.vertexAI.preview.getGenerativeModel({
      model: 'gemini-2.0-flash-exp'
    });
    
    const prompt = `
      Generate a compelling 150-200 word synopsis for a ${genres.join('/')} film titled "${title}" 
      starring ${cast.slice(0, 3).join(', ')}.
      
      The synopsis should:
      - Be engaging and professional
      - Avoid spoilers
      - Include genre-appropriate tone
      - Be suitable for streaming platform descriptions
      
      Return ONLY the synopsis text, no additional formatting.
    `;
    
    const result = await model.generateContent(prompt);
    return result.response.text().trim();
  }
}
```

**Create enrichment endpoint:**

```typescript
// mondweep/apps/metadata-api/src/routes/enrichment.ts
import express from 'express';
import { EnrichmentService } from '../services/enrichment-service';
import { Firestore } from '@google-cloud/firestore';

const router = express.Router();
const enrichmentService = new EnrichmentService();
const firestore = new Firestore();

// Enrich single asset
router.post('/assets/:id/enrich', async (req, res) => {
  try {
    const { id } = req.params;
    
    // Fetch asset from Firestore
    const assetDoc = await firestore.collection('assets').doc(id).get();
    
    if (!assetDoc.exists) {
      return res.status(404).json({ error: 'Asset not found' });
    }
    
    const asset = assetDoc.data();
    
    // Enrich metadata
    const enriched = await enrichmentService.enrichMetadata(asset);
    
    // Update Firestore
    await firestore.collection('assets').doc(id).update({
      enrichment: enriched.enrichment,
      updated_at: Firestore.Timestamp.now()
    });
    
    res.json({ success: true, enrichment: enriched.enrichment });
  } catch (error) {
    console.error('Enrichment error:', error);
    res.status(500).json({ error: 'Enrichment failed' });
  }
});

// Batch enrichment
router.post('/batch-enrich', async (req, res) => {
  try {
    const { asset_ids } = req.body;
    
    const results = [];
    
    for (const id of asset_ids) {
      const assetDoc = await firestore.collection('assets').doc(id).get();
      
      if (assetDoc.exists) {
        const asset = assetDoc.data();
        const enriched = await enrichmentService.enrichMetadata(asset);
        
        await firestore.collection('assets').doc(id).update({
          enrichment: enriched.enrichment,
          updated_at: Firestore.Timestamp.now()
        });
        
        results.push({ id, status: 'enriched' });
      } else {
        results.push({ id, status: 'not_found' });
      }
    }
    
    res.json({ results });
  } catch (error) {
    console.error('Batch enrichment error:', error);
    res.status(500).json({ error: 'Batch enrichment failed' });
  }
});

export { router as enrichmentRoutes };
```

**Deliverable:** ✅ AI-powered metadata enrichment working

---

### **Day 11-14: Platform Validation & Connectors**

#### **Task 2.3: Netflix IMF Validator**

```typescript
// mondweep/apps/metadata-api/src/validators/netflix-validator.ts

export interface NetflixValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
}

export class NetflixValidator {
  validate(asset: any): NetflixValidationResult {
    const errors: string[] = [];
    const warnings: string[] = [];
    
    // Check mandatory fields
    if (!asset.title || asset.title.length === 0) {
      errors.push('Title is required');
    }
    
    if (!asset.synopsis || asset.synopsis.length < 50) {
      errors.push('Synopsis must be at least 50 characters');
    }
    
    if (asset.synopsis && asset.synopsis.length > 500) {
      errors.push('Synopsis must not exceed 500 characters');
    }
    
    // Technical requirements
    if (!asset.technical) {
      errors.push('Technical metadata is required');
    } else {
      // Resolution check
      if (asset.technical.resolution !== 'UHD') {
        warnings.push('Netflix prefers UHD resolution');
      }
      
      // Audio check
      if (!['5.1', 'Atmos'].includes(asset.technical.audio_config)) {
        errors.push('Audio must be 5.1 or Atmos for Netflix');
      }
      
      // Color space check
      if (!['HDR10', 'Dolby Vision'].includes(asset.technical.color_space)) {
        warnings.push('Netflix prefers HDR10 or Dolby Vision');
      }
    }
    
    // Dolby Vision check
    if (asset.technical?.color_space === 'Dolby Vision' && !asset.dolby_vision_xml) {
      errors.push('Dolby Vision requires XML sidecar metadata');
    }
    
    // Rating check
    if (!asset.rating) {
      errors.push('Content rating is required');
    }
    
    return {
      valid: errors.length === 0,
      errors,
      warnings
    };
  }
}
```

**Deliverable:** ✅ Platform validators implemented

---

## **Week 3: Distribution & Workflows**

### **Day 15-17: Cloud Workflows for Delivery**

#### **Task 3.1: Create Delivery Workflow**

```yaml
# mondweep/workflows/netflix-delivery.yaml
main:
  params: [asset_id]
  steps:
    - init:
        assign:
          - project_id: "agentics-foundation25lon-1899"
          - asset_id: ${asset_id}
    
    - fetch_metadata:
        call: http.get
        args:
          url: ${"https://metadata-api-xxx.run.app/api/v1/assets/" + asset_id}
        result: asset
    
    - validate_netflix:
        call: http.post
        args:
          url: "https://metadata-api-xxx.run.app/api/v1/validate/netflix"
          body:
            asset: ${asset}
        result: validation
    
    - check_validation:
        switch:
          - condition: ${validation.body.valid == false}
            next: validation_failed
        next: generate_imf
    
    - validation_failed:
        raise:
          message: ${"Validation failed: " + validation.body.errors}
    
    - generate_imf:
        call: http.post
        args:
          url: "https://metadata-api-xxx.run.app/api/v1/delivery/netflix/generate-imf"
          body:
            asset_id: ${asset_id}
        result: imf_package
    
    - upload_to_netflix:
        call: http.post
        args:
          url: "https://metadata-api-xxx.run.app/api/v1/delivery/netflix/upload"
          body:
            package_location: ${imf_package.body.location}
        result: upload_result
    
    - log_delivery:
        call: http.post
        args:
          url: "https://metadata-api-xxx.run.app/api/v1/delivery/log"
          body:
            asset_id: ${asset_id}
            platform: "netflix"
            status: "success"
            package_location: ${imf_package.body.location}
    
    - return_result:
        return: ${upload_result}
```

**Deploy workflow:**

```bash
gcloud workflows deploy netflix-delivery \
  --source=mondweep/workflows/netflix-delivery.yaml \
  --location=us-central1
```

**Deliverable:** ✅ Automated delivery workflows

---

### **Day 18-21: Analytics & Monitoring**

#### **Task 3.2: BigQuery Analytics**

```bash
# Create BigQuery dataset
bq mk --dataset \
  --location=us-central1 \
  agentics-foundation25lon-1899:ummid_analytics

# Create tables
bq mk --table \
  agentics-foundation25lon-1899:ummid_analytics.delivery_metrics \
  asset_id:STRING,platform:STRING,delivery_time_ms:INTEGER,status:STRING,timestamp:TIMESTAMP
```

**Create analytics queries:**

```sql
-- Time to market by platform
CREATE OR REPLACE VIEW `agentics-foundation25lon-1899.ummid_analytics.time_to_market` AS
SELECT
  platform,
  AVG(delivery_time_ms) / 1000 / 60 / 60 as avg_hours,
  PERCENTILE_CONT(delivery_time_ms, 0.95) OVER (PARTITION BY platform) / 1000 / 60 / 60 as p95_hours,
  COUNT(*) as total_deliveries
FROM `agentics-foundation25lon-1899.ummid_analytics.delivery_metrics`
WHERE status = 'success'
  AND timestamp >= TIMESTAMP_SUB(CURRENT_TIMESTAMP(), INTERVAL 30 DAY)
GROUP BY platform;

-- Error rate by platform
CREATE OR REPLACE VIEW `agentics-foundation25lon-1899.ummid_analytics.error_rates` AS
SELECT
  platform,
  COUNTIF(status = 'failed') as failures,
  COUNT(*) as total,
  ROUND(COUNTIF(status = 'failed') * 100.0 / COUNT(*), 2) as error_rate_pct
FROM `agentics-foundation25lon-1899.ummid_analytics.delivery_metrics`
WHERE timestamp >= TIMESTAMP_SUB(CURRENT_TIMESTAMP(), INTERVAL 30 DAY)
GROUP BY platform
ORDER BY error_rate_pct DESC;
```

**Deliverable:** ✅ Analytics dashboard queries

---

## **Week 4: Demo & Polish**

### **Day 22-24: Web UI Development**

#### **Task 4.1: Create React Dashboard**

```bash
# Create React app
cd mondweep/apps
npx create-react-app ummid-dashboard --template typescript
cd ummid-dashboard

# Install dependencies
npm install @mui/material @emotion/react @emotion/styled \
  recharts axios react-router-dom
```

**Key components:**
- Asset browser with semantic search
- Rights management with collision detection
- Delivery status dashboard
- Analytics charts

**Deliverable:** ✅ Functional web UI

---

### **Day 25-28: Demo Preparation**

#### **Task 4.2: Create Demo Script**

1. **Ingest demo content** (100 assets)
2. **Semantic search demo** ("dark thriller with twist ending")
3. **Rights collision detection** (show overlapping rights)
4. **Platform validation** (Netflix, Amazon)
5. **Automated delivery** (trigger workflow)
6. **Analytics dashboard** (show SLA metrics)

**Deliverable:** ✅ Polished demo ready for presentation

---

## **Success Metrics**

| Metric | Target | Status |
|--------|--------|--------|
| Assets ingested | 1000+ | ⏳ |
| Semantic search latency | <100ms | ⏳ |
| Entity resolution accuracy | >95% | ⏳ |
| Platform validators | 3+ | ⏳ |
| Delivery workflows | 2+ | ⏳ |
| API uptime | >99% | ⏳ |
| Demo completeness | 100% | ⏳ |

---

## **Risk Mitigation**

| Risk | Mitigation |
|------|------------|
| Gemini API rate limits | Implement caching + backoff |
| Cloud SQL connection limits | Use connection pooling |
| Firestore write limits | Batch operations |
| Time constraints | Focus on core features first |
| Budget overruns | Monitor costs daily |

---

**Next Step:** Start with Day 1 tasks! 🚀

**Status:** ✅ Ready to implement  
**Last Updated:** 2025-12-05
