# Media Gateway Implementation Guidance Summary
## Research Analysis from media-gateway-hackathon-research Repository

**Analysis Date:** 2025-12-06
**Source Repository:** https://github.com/globalbusinessadvisors/media-gateway-hackathon-research
**Research Agent:** SPARC Pseudocode Phase

---

## Executive Overview

Media Gateway is a unified TV content discovery system built entirely in **Rust** that aggregates 10+ streaming platforms (Netflix, Prime Video, Disney+, Hulu, Apple TV+, YouTube, Crave, HBO Max, etc.) into a single intelligent discovery experience. The system targets the **"45-minute decision problem"**—time users waste deciding what to watch due to content fragmentation across platforms.

### Core Architecture: 4-Layer Design

1. **Layer 1 (Ingestion):** 23 micro-repositories handling MCP connectors, normalizers, entity resolver, authentication, and MCP server
2. **Layer 2 (Intelligence):** 8 repositories for multi-agent orchestration, recommendation engine, semantic search (SONA neural architecture)
3. **Layer 3 (Consolidation):** 5 repositories unifying global metadata, availability index, rights engine, personalization
4. **Layer 4 (End-User Experience):** 6 applications - CLI, web (Next.js), mobile (iOS/Android), smart TV (Tizen/webOS)

---

## 1. RECOMMENDED ALGORITHMS FOR SEARCH/RECOMMENDATION

### 1.1 Hybrid Recommendation Engine

**Algorithm: Reciprocal Rank Fusion (RRF)**

```
RRF(document) = Σ [1/(k + rank_i) × weight_i]

Where:
- k = 60 (constant)
- rank_i = position in ranking i
- weight_i = strategy weight

Weights:
- Collaborative Filtering: 0.35
- Content-Based: 0.25
- Graph Neural Network (GNN): 0.30
- Context-Aware: 0.10
```

**Post-Processing Pipeline:**
1. **Diversity Injection:** Maximal Marginal Relevance (MMR) with λ=0.85
2. **Trust Filtering:** Threshold=0.6 (filter below)
3. **Availability Filtering:** By region/subscriptions
4. **Explanation Generation:** Provide reasoning for recommendations

### 1.2 Semantic Search

**Algorithm: HNSW (Hierarchical Navigable Small World)**

- **Embedding Dimensions:** 768-1536 for content, 512 for users, 256 for genres
- **Index Type:** Ruvector hypergraph with vector indexes
- **Search Strategy:** Multi-modal combining plot embeddings, metadata, and mood vectors

### 1.3 Graph Neural Network (GraphSAGE)

**Architecture:**
- **Layers:** 3-layer network (512→256→128 neurons)
- **Neighborhood Sampling:** 25→15→10 neighbors across layers
- **Attention Heads:** 8→4→2 per layer
- **Parameters:** ~183K total
- **Performance:** 45ms p50 latency, 95ms p99

**Training Strategy:**
```
Supervised learning on:
- User-content interactions (watch completion, ratings)
- Content-content similarity edges
- Collaborative filtering signals
```

### 1.4 Semantic Routing (Tiny Dancer)

**Algorithm: FastGRNN (Fast Gated Recurrent Neural Network)**

- **Parameter Count:** ~200K (90% reduction vs traditional LSTM)
- **Latency:** <5ms inference time
- **Gating Mechanism:** `z_t = σ(Wz @ h_{t-1} + Uz @ x_t + b_z)`
- **Classification:** 6 primary query strategy classes

**Query Routing:**
- Similarity queries → GraphRoPE + EdgeFeatured attention
- Hierarchical content → Hyperbolic + Poincaré operations
- Cross-platform → Cross + LocalGlobal combination

### 1.5 Runtime Personalization (Two-Tier LoRA)

**Algorithm: Low-Rank Adaptation (LoRA)**

```
W' = W + (α/r) * B @ A

Where:
- W = pre-trained weights
- A, B = low-rank matrices
- α = scaling factor
- r = rank (typically 4-8)
```

**Anti-Forgetting: Elastic Weight Consolidation++ (EWC++)**

```
Loss = Task_Loss + (λ/2) * Σ F_i * (θ_i - θ*_i)²

Where:
- F_i = Fisher information (importance weight)
- θ_i = current parameter
- θ*_i = anchor parameter (pre-trained)
- λ = regularization strength
```

**Trigger Conditions:**
- Interaction count ≥ 5
- Elapsed time ≥ 1 hour
- Signal strength > 0.7

**Signal Hierarchy (descending importance):**
1. watch_complete
2. rating
3. watch_start
4. click
5. skip

---

## 2. DATA STRUCTURE PATTERNS

### 2.1 Ruvector Hypergraph Schema

**Node Types and Embeddings:**

| Node Type | Embedding Dimensions | Purpose |
|-----------|---------------------|---------|
| Content | 1536 | Plot + metadata representation |
| TVShow | 1536 | Series-level information |
| Episode | 768 | Episode-specific data |
| Person | 768 | Filmography and career data |
| User | 512 | Preference representation |
| Genre/Mood | 256 | Category embeddings |

**Edge Types:**
- **Standard Edges:** ACTED_IN, DIRECTED, SIMILAR_TO, WATCHED, RATED
- **Hyperedges:** AVAILABILITY_WINDOW connecting (Content × Platform × Region × TimeWindow × LicenseTerms)

**Hyperedge Attributes:**
```rust
struct AvailabilityWindow {
    pricing_model: PricingModel,  // SVOD/TVOD/AVOD/FREE
    quality_tiers: Vec<Quality>,   // SD, HD, 4K, HDR, DolbyVision
    audio_formats: Vec<Audio>,     // Stereo, 5.1, Atmos
    rights: ContentRights,         // streaming/download permissions
    exclusivity: bool,
    deep_link: String,
    trust_score: f32,
    verification_source: Source,
    verified_at: DateTime<Utc>,
}
```

### 2.2 CRDT Data Structures for Synchronization

**Last-Writer-Wins (LWW) Registers:**
```rust
struct LWWRegister<T> {
    value: T,
    timestamp: HybridLogicalClock,  // HLC for deterministic ordering
    device_id: String,
}
```

**OR-Sets (Observed-Removed Sets):**
```rust
struct ORSet<T> {
    additions: HashMap<T, HashSet<(DeviceId, Timestamp)>>,
    removals: HashMap<T, HashSet<(DeviceId, Timestamp)>>,
}

// Removal always wins when timestamps identical
```

**Use Cases:**
- Watch progress: LWW Register (position in seconds + HLC timestamp)
- Watchlist: OR-Set (additions/removals tracked separately)

### 2.3 PubNub Channel Architecture

**Channel Hierarchy:**
```
user.{userId}.sync              → Watch progress, preferences
user.{userId}.devices           → Device-to-device commands
region.{regionCode}.trending    → Regional content updates
global.new_releases             → Platform-wide announcements
```

**Message Structure:**
```rust
struct SyncMessage {
    message_type: MessageType,  // WatchProgress, Watchlist, DeviceCommand
    payload: serde_json::Value,
    timestamp: HybridLogicalClock,
    device_id: String,
    checksum: String,           // Message integrity
}
```

### 2.4 LoRA Adapter Storage

**In-Memory (Memorystore/Valkey):**
```rust
struct LoRAAdapter {
    user_id: String,
    matrix_a: Vec<Vec<f32>>,    // Low-rank matrix A
    matrix_b: Vec<Vec<f32>>,    // Low-rank matrix B
    scaling_factor: f32,
    interaction_count: u32,
    last_updated: DateTime<Utc>,
    metadata: AdapterMetadata,
}
```

**Storage Metrics:**
- Size: 2-8 MB per user
- TTL: 30 days in cache
- Cold storage: Cloud SQL PostgreSQL

### 2.5 ReasoningBank Pattern Storage

**Pattern Schema:**
```rust
struct ReasoningPattern {
    pattern_id: Uuid,
    query_embedding: Vec<f32>,           // 384-dim
    successful_content_ids: Vec<String>,
    user_segment: UserSegment,
    contextual_metadata: HashMap<String, String>,  // time_of_day, device_type
    confidence_score: f32,
    usage_count: u32,
    created_at: DateTime<Utc>,
    last_used: DateTime<Utc>,
}
```

**Index Strategy:**
- HNSW vector index on query_embedding (10ms p50 retrieval)
- Secondary index on user_segment + contextual_metadata

---

## 3. API ENDPOINT DESIGNS

### 3.1 MCP Connector Interface (Rust Trait)

```rust
#[async_trait]
pub trait MCPConnector: Send + Sync {
    /// Platform identification
    fn platform_type(&self) -> PlatformType;

    /// Tool schema definitions for MCP server
    fn get_tool_schemas(&self) -> Vec<MCPToolSchema>;

    /// Initialize with platform credentials
    async fn initialize(
        &mut self,
        credentials: AuthCredentials
    ) -> Result<(), ConnectorError>;

    /// Fetch content catalog with pagination
    async fn fetch_catalog(
        &self,
        region: &str,
        cursor: Option<String>,
        limit: usize,
    ) -> Result<MCPResponse, ConnectorError>;

    /// Retrieve specific content metadata
    async fn fetch_content(
        &self,
        platform_id: &str,
    ) -> Result<ContentMetadata, ConnectorError>;

    /// Search platform catalog
    async fn search_content(
        &self,
        query: &str,
        region: &str,
        limit: usize,
    ) -> Result<MCPResponse, ConnectorError>;

    /// Health check endpoint
    async fn health_check(&self) -> Result<ConnectorHealth, ConnectorError>;

    /// Rate limiting configuration
    fn rate_limit_config(&self) -> RateLimitConfig;
}
```

### 3.2 SONA Integration gRPC Services

**sona-adapter Service:**
```protobuf
service SONAAdapter {
    rpc LoadAdapter(LoadAdapterRequest) returns (LoadAdapterResponse);
    rpc AdaptEmbeddings(AdaptEmbeddingsRequest) returns (AdaptEmbeddingsResponse);
    rpc UpdateFromInteraction(UpdateRequest) returns (UpdateResponse);
}

message LoadAdapterRequest {
    string user_id = 1;
    bool force_reload = 2;
}

message AdaptEmbeddingsRequest {
    string user_id = 1;
    repeated float base_embeddings = 2;
    map<string, string> context = 3;  // device_type, time_of_day
}
```

**semantic-router Service:**
```protobuf
service SemanticRouter {
    rpc Route(RouteRequest) returns (RouteResponse);
    rpc SelectModel(SelectModelRequest) returns (SelectModelResponse);
}

message RouteRequest {
    string query = 1;
    map<string, string> context = 2;
}

message RouteResponse {
    string handler_type = 1;  // similarity, hierarchical, cross-platform
    repeated AttentionMechanism mechanisms = 2;  // top-3 selected
    float confidence = 3;
}
```

**reasoning-bank Service:**
```protobuf
service ReasoningBank {
    rpc StorePattern(StorePatternRequest) returns (StorePatternResponse);
    rpc FindSimilarPatterns(FindPatternsRequest) returns (FindPatternsResponse);
    rpc ApplyPatterns(ApplyPatternsRequest) returns (ApplyPatternsResponse);
}

message FindPatternsRequest {
    repeated float query_embedding = 1;
    UserSegment segment = 2;
    map<string, string> context = 3;
    float similarity_threshold = 4;  // default 0.75
    int32 max_results = 5;           // default 10
}
```

### 3.3 RESTful API Gateway (Cloud Run)

**Search Endpoint:**
```http
POST /api/v1/search
Content-Type: application/json
Authorization: Bearer {access_token}

Request:
{
    "query": "thriller movies similar to inception",
    "user_id": "user_123",
    "region": "USA",
    "platforms": ["netflix", "prime", "disney"],
    "limit": 20,
    "context": {
        "device": "mobile",
        "time_of_day": "evening"
    }
}

Response:
{
    "results": [
        {
            "content_id": "tt1375666",
            "title": "Inception",
            "availability": [
                {
                    "platform": "netflix",
                    "deep_link": "https://netflix.com/watch/...",
                    "pricing": "SVOD",
                    "quality": ["HD", "4K"],
                    "trust_score": 0.95
                }
            ],
            "recommendation_score": 0.92,
            "explanation": "Highly rated psychological thriller with complex narrative"
        }
    ],
    "metadata": {
        "total_results": 157,
        "latency_ms": 245,
        "strategies_used": ["collaborative", "gnn", "content_based"]
    }
}
```

**Recommendation Endpoint:**
```http
GET /api/v1/recommendations?user_id={user_id}&limit={limit}&mood={mood}
Authorization: Bearer {access_token}

Response:
{
    "recommendations": [...],
    "personalization_applied": true,
    "adapter_version": "v2.3.1",
    "interaction_count": 47
}
```

### 3.4 ARW (Agent-Ready Web) Protocol

**Machine View Response:**
```json
{
    "@context": "https://agentics.org/arw/v1",
    "id": "arw:content:tt1375666",
    "title": "Inception",
    "contentType": "Movie",
    "description": "A thief who steals corporate secrets...",
    "availability": {
        "platforms": [
            {
                "name": "Netflix",
                "region": "US",
                "pricing": {"model": "SVOD", "price": null},
                "deepLink": "netflix://watch/70131314",
                "qualities": ["HD", "4K", "HDR"]
            }
        ]
    },
    "embeddings": {
        "plot": [0.123, -0.456, ...],  // 1536-dim
        "mood": [0.789, 0.234, ...]    // 256-dim
    },
    "taxonomy": {
        "genres": ["sci-fi", "thriller"],
        "themes": ["dreams", "heist", "reality"]
    }
}
```

**ARW Headers:**
```http
AI-Request-ID: req_abc123
AI-Agent-Name: MediaGateway/ContentSearcher
AI-Purpose: discovery
AI-Token-Budget: 50000
```

---

## 4. INTEGRATION PATTERNS WITH EXTERNAL SERVICES

### 4.1 Aggregator API Integration Strategy

**Problem:** Direct platform APIs largely unavailable (only YouTube public)

**Solution:** Third-party aggregators as primary data source

**Recommended Aggregators:**

| Service | Coverage | Key Features | Cost |
|---------|----------|--------------|------|
| **Streaming Availability API** | 10+ platforms, global | Deep links, expiry tracking, IMDb/TMDb mapping | Paid tiers |
| **Watchmode** | 200+ services, 50+ countries | Episode-level links, pricing data | Tiered access |
| **JustWatch** | 50+ countries | Reliable deep-linking, regional content | Partnership model |
| **International Showtimes** | 100+ markets | Regional pricing, licensing data | Enterprise |

**Integration Pattern:**
```rust
// Aggregator facade pattern
pub struct AggregatorClient {
    streaming_availability: StreamingAvailabilityAPI,
    watchmode: WatchmodeAPI,
    justwatch: JustWatchAPI,
    fallback_strategy: FallbackStrategy,
}

impl AggregatorClient {
    pub async fn fetch_availability(
        &self,
        content_id: &str,
        region: &str,
    ) -> Result<Vec<Availability>, AggregatorError> {
        // Try primary, fallback to secondary
        match self.streaming_availability.fetch(content_id, region).await {
            Ok(data) => Ok(data),
            Err(_) => {
                // Log fallback event
                self.watchmode.fetch(content_id, region).await
            }
        }
    }
}
```

### 4.2 OAuth 2.0 Integration (YouTube, User Authentication)

**Authorization Code + PKCE Flow:**

```rust
// Step 1: Generate PKCE verifier and challenge
let code_verifier = generate_random_string(128);  // 43-128 chars
let code_challenge = base64_url_encode(sha256(code_verifier));

// Step 2: Authorization request
let auth_url = format!(
    "https://accounts.google.com/o/oauth2/v2/auth?\
     client_id={}&\
     redirect_uri={}&\
     response_type=code&\
     scope={}&\
     code_challenge={}&\
     code_challenge_method=S256",
    client_id, redirect_uri, scope, code_challenge
);

// Step 3: Exchange authorization code for tokens
let token_response = http_client.post("https://oauth2.googleapis.com/token")
    .form(&[
        ("code", &authorization_code),
        ("client_id", &client_id),
        ("redirect_uri", &redirect_uri),
        ("code_verifier", &code_verifier),
        ("grant_type", "authorization_code"),
    ])
    .send()
    .await?;

// Step 4: Store tokens securely
keyring.set_entry("access_token", &token_response.access_token)?;
keyring.set_entry("refresh_token", &token_response.refresh_token)?;
```

**Token Refresh:**
```rust
async fn refresh_access_token(
    refresh_token: &str
) -> Result<TokenResponse, OAuthError> {
    let response = http_client.post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", &client_id),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await?;

    Ok(response.json::<TokenResponse>().await?)
}
```

### 4.3 Device Authorization Grant (RFC 8628)

**For TV/CLI devices without keyboard:**

```rust
// Step 1: Request device and user codes
let device_auth = http_client.post("https://oauth2.googleapis.com/device/code")
    .form(&[
        ("client_id", &client_id),
        ("scope", &scope),
    ])
    .send()
    .await?
    .json::<DeviceAuthResponse>()
    .await?;

// Step 2: Display user code
println!("Visit {} and enter code: {}",
    device_auth.verification_url,
    device_auth.user_code
);

// Step 3: Poll for authorization
let mut interval = Duration::from_secs(device_auth.interval);
loop {
    tokio::time::sleep(interval).await;

    match poll_for_token(&device_auth.device_code).await {
        Ok(tokens) => {
            store_tokens(tokens).await?;
            break;
        }
        Err(PollError::AuthorizationPending) => continue,
        Err(PollError::SlowDown) => {
            interval += Duration::from_secs(5);
            continue;
        }
        Err(e) => return Err(e),
    }
}
```

### 4.4 Deep Linking Integration

**iOS Universal Links:**
```swift
// apple-app-site-association file at https://yourdomain.com/.well-known/
{
    "applinks": {
        "apps": [],
        "details": [
            {
                "appID": "TEAM_ID.com.mediagateway.ios",
                "paths": ["/watch/*", "/show/*", "/movie/*"]
            }
        ]
    }
}

// Handle in AppDelegate
func application(
    _ application: UIApplication,
    continue userActivity: NSUserActivity,
    restorationHandler: @escaping ([UIUserActivityRestoring]?) -> Void
) -> Bool {
    guard let url = userActivity.webpageURL else { return false }
    // Parse content_id from URL
    // Open native streaming app with deep link
    return true
}
```

**Android App Links:**
```xml
<!-- AndroidManifest.xml -->
<activity android:name=".MainActivity">
    <intent-filter android:autoVerify="true">
        <action android:name="android.intent.action.VIEW" />
        <category android:name="android.intent.category.DEFAULT" />
        <category android:name="android.intent.category.BROWSABLE" />
        <data
            android:scheme="https"
            android:host="mediagateway.com"
            android:pathPrefix="/watch" />
    </intent-filter>
</activity>
```

```json
// assetlinks.json at https://mediagateway.com/.well-known/
[
  {
    "relation": ["delegate_permission/common.handle_all_urls"],
    "target": {
      "namespace": "android_app",
      "package_name": "com.mediagateway.android",
      "sha256_cert_fingerprints": ["..."]
    }
  }
]
```

### 4.5 E2B Sandbox Integration

**Secure Agent Code Execution:**

```rust
use e2b_sdk::{Sandbox, SandboxConfig};

pub async fn execute_agent_code(
    code: &str,
    user_preferences: &UserPreferences,
) -> Result<ExecutionResult, SandboxError> {
    // Create isolated sandbox
    let config = SandboxConfig::default()
        .with_timeout(60)           // 60 seconds
        .with_memory_limit(2048)    // 2GB
        .with_template("discovery-agent");

    let sandbox = Sandbox::create(config).await?;

    // Upload input data
    sandbox.upload_file(
        "preferences.json",
        serde_json::to_string(user_preferences)?
    ).await?;

    // Execute code
    let execution = sandbox.execute_code(code).await?;

    // Validate and download results
    if execution.exit_code == 0 {
        let results_json = sandbox.download_file("results.json").await?;
        let results: Vec<Recommendation> = serde_json::from_str(&results_json)?;

        // Cleanup
        sandbox.terminate().await?;

        Ok(ExecutionResult::Success(results))
    } else {
        sandbox.terminate().await?;
        Ok(ExecutionResult::Error(execution.stderr))
    }
}
```

**Sandbox Templates:**
- `discovery-agent`: General search/discovery
- `recommendation-agent`: scikit-learn, pandas for analytics
- `sona-updater`: LoRA weight updates with EWC++

### 4.6 PubNub Real-Time Synchronization

**Channel Subscription:**
```rust
use pubnub::{Pubnub, subscribe::Subscribe};

async fn setup_device_sync(user_id: &str, device_id: &str) {
    let pubnub = Pubnub::new(
        subscribe_key,
        publish_key,
        Some(user_id.to_string()),
    );

    // Subscribe to user's sync channel
    let subscription = pubnub
        .subscribe()
        .channels(vec![
            format!("user.{}.sync", user_id),
            format!("user.{}.devices", user_id),
        ])
        .execute()
        .await?;

    // Handle messages
    subscription.stream().for_each(|message| async {
        match message {
            SubscribeMessage::Message(msg) => {
                handle_sync_message(msg).await;
            }
            SubscribeMessage::Presence(presence) => {
                handle_device_presence(presence).await;
            }
        }
    }).await;
}
```

**Publishing Watch Progress:**
```rust
async fn publish_watch_progress(
    user_id: &str,
    content_id: &str,
    position_seconds: u64,
    hlc_timestamp: HybridLogicalClock,
) -> Result<(), PubNubError> {
    let message = SyncMessage {
        message_type: MessageType::WatchProgress,
        payload: json!({
            "content_id": content_id,
            "position": position_seconds,
            "timestamp": hlc_timestamp,
            "device_id": get_device_id(),
        }),
        timestamp: hlc_timestamp,
        device_id: get_device_id(),
        checksum: compute_checksum(&payload),
    };

    pubnub
        .publish()
        .channel(format!("user.{}.sync", user_id))
        .message(message)
        .execute()
        .await?;

    Ok(())
}
```

### 4.7 Google Cloud Platform Integration

**Service Communication Pattern:**
```rust
// gRPC service discovery via DNS
let channel = Channel::from_static("dns:///recommendation-service:50051")
    .connect()
    .await?;

let mut client = RecommendationServiceClient::new(channel);

// Call with Workload Identity authentication
let request = tonic::Request::new(RecommendationRequest {
    user_id: user_id.to_string(),
    limit: 20,
    context: context_map,
});

let response = client.get_recommendations(request).await?;
```

**Cloud Pub/Sub Event Publishing:**
```rust
use google_cloud_pubsub::{Client, Publisher};

async fn publish_interaction_event(event: InteractionEvent) -> Result<(), Error> {
    let client = Client::default().await?;
    let topic = client.topic("user-interactions");
    let publisher = topic.new_publisher(None);

    let message = serde_json::to_vec(&event)?;
    let message_id = publisher.publish(message).await?;

    Ok(())
}
```

---

## 5. PERFORMANCE OPTIMIZATION STRATEGIES

### 5.1 Caching Strategies

**Multi-Tier Caching:**

| Layer | Technology | TTL | Use Case |
|-------|-----------|-----|----------|
| L1 (Process) | In-memory HashMap | 5 min | Hot embeddings, active sessions |
| L2 (Distributed) | Memorystore (Valkey) | 1-24 hours | LoRA adapters, trending content |
| L3 (Persistent) | Cloud SQL | 7-30 days | User profiles, metadata |

**Cache-Aside Pattern:**
```rust
async fn get_user_adapter(user_id: &str) -> Result<LoRAAdapter, Error> {
    // Try L2 cache first
    if let Some(adapter) = memorystore.get(user_id).await? {
        return Ok(adapter);
    }

    // Cache miss - load from database
    let adapter = database.load_adapter(user_id).await?;

    // Populate cache for future requests
    memorystore.set(user_id, &adapter, Duration::hours(24)).await?;

    Ok(adapter)
}
```

**Embedding Cache:**
```rust
// Pre-compute and cache content embeddings
async fn warmup_content_cache(trending_ids: &[String]) {
    let embeddings = batch_embed_content(trending_ids).await?;

    for (id, embedding) in trending_ids.iter().zip(embeddings) {
        memorystore.set(
            &format!("embedding:{}", id),
            &embedding,
            Duration::hours(6),
        ).await?;
    }
}
```

### 5.2 Database Query Optimization

**Connection Pooling:**
```rust
use sqlx::postgres::{PgPool, PgPoolOptions};

let pool = PgPoolOptions::new()
    .max_connections(100)
    .min_connections(10)
    .idle_timeout(Duration::minutes(10))
    .acquire_timeout(Duration::seconds(5))
    .connect(&database_url)
    .await?;
```

**Prepared Statements:**
```rust
// Compile once, execute many times
let stmt = sqlx::query!(
    r#"
    SELECT content_id, title, embeddings
    FROM content
    WHERE platform = $1 AND region = $2
    ORDER BY popularity DESC
    LIMIT $3
    "#
)
.prepare(&pool)
.await?;

let results = stmt
    .fetch_all(&pool, &platform, &region, &limit)
    .await?;
```

**Indexing Strategy:**
```sql
-- Composite index for common queries
CREATE INDEX idx_content_platform_region_popularity
ON content (platform, region, popularity DESC);

-- Partial index for active content
CREATE INDEX idx_active_content
ON content (verified_at)
WHERE trust_score >= 0.6;

-- GIN index for array containment
CREATE INDEX idx_content_genres
ON content USING GIN (genres);
```

### 5.3 Batching and Parallelization

**Batch MCP Requests:**
```rust
use futures::future::join_all;

async fn fetch_multi_platform_availability(
    content_id: &str,
    platforms: &[Platform],
) -> Vec<Result<Availability, Error>> {
    let futures = platforms.iter().map(|platform| {
        async move {
            platform.fetch_availability(content_id).await
        }
    });

    join_all(futures).await
}
```

**Parallel Agent Execution:**
```rust
use tokio::task::JoinSet;

async fn parallel_agent_search(query: &str) -> CombinedResults {
    let mut join_set = JoinSet::new();

    // Spawn parallel agent tasks
    join_set.spawn(semantic_search_agent(query.to_string()));
    join_set.spawn(collaborative_filter_agent(query.to_string()));
    join_set.spawn(gnn_recommendation_agent(query.to_string()));

    // Collect results
    let mut results = Vec::new();
    while let Some(res) = join_set.join_next().await {
        if let Ok(agent_result) = res {
            results.push(agent_result);
        }
    }

    merge_results(results)
}
```

### 5.4 Async I/O Optimization

**Tokio Runtime Configuration:**
```rust
#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() {
    // Application entry point
}
```

**Stream Processing:**
```rust
use tokio_stream::StreamExt;

async fn process_catalog_stream(platform: &Platform) {
    let mut stream = platform.catalog_stream().await?;

    while let Some(batch) = stream.next().await {
        // Process batch without loading entire catalog into memory
        process_content_batch(batch).await?;
    }
}
```

### 5.5 Compression and Network Optimization

**gRPC Compression:**
```rust
use tonic::codec::CompressionEncoding;

let channel = Channel::from_static("http://localhost:50051")
    .connect()
    .await?;

let client = RecommendationServiceClient::new(channel)
    .send_compressed(CompressionEncoding::Gzip)
    .accept_compressed(CompressionEncoding::Gzip);
```

**PubNub Delta Encoding:**
```rust
// Only send changed fields
fn compute_delta(previous: &WatchProgress, current: &WatchProgress) -> WatchProgressDelta {
    WatchProgressDelta {
        content_id: current.content_id.clone(),
        position_delta: current.position - previous.position,
        timestamp: current.timestamp,
    }
}
```

### 5.6 Warm Instance Pools (E2B)

**Pre-Warmed Sandboxes:**
```rust
struct SandboxPool {
    discovery_pool: Vec<Sandbox>,
    recommendation_pool: Vec<Sandbox>,
    target_pool_size: usize,
}

impl SandboxPool {
    async fn maintain_pool(&mut self) {
        // Keep pool warm
        while self.discovery_pool.len() < self.target_pool_size {
            let sandbox = Sandbox::create(
                SandboxConfig::default()
                    .with_template("discovery-agent")
            ).await?;
            self.discovery_pool.push(sandbox);
        }
    }

    async fn acquire_sandbox(&mut self) -> Sandbox {
        self.discovery_pool.pop()
            .unwrap_or_else(|| {
                // Pool exhausted - create on demand
                Sandbox::create_sync(SandboxConfig::default())
            })
    }
}
```

**Pool Metrics:**
- Discovery pool: 10 instances (eliminate ~150ms cold start)
- Recommendation pool: 5 instances
- Total monthly hours: ~1,000 (based on 3,800 daily executions)

### 5.7 Rate Limiting and Circuit Breaking

**Token Bucket Rate Limiter:**
```rust
use governor::{Quota, RateLimiter};

struct PlatformClient {
    client: HttpClient,
    rate_limiter: RateLimiter<String, DefaultKeyedStateStore, DefaultClock>,
}

impl PlatformClient {
    async fn fetch_with_rate_limit(&self, url: &str) -> Result<Response, Error> {
        // Wait for token bucket availability
        self.rate_limiter
            .until_key_ready(&self.platform_name)
            .await;

        self.client.get(url).send().await
    }
}
```

**Circuit Breaker Pattern:**
```rust
use circuit_breaker::{CircuitBreaker, Config};

let breaker = CircuitBreaker::new(Config {
    failure_threshold: 5,      // Open after 5 failures
    timeout: Duration::secs(30),  // Try again after 30s
    success_threshold: 2,      // Close after 2 successes
});

let result = breaker.call(|| async {
    aggregator_api.fetch_availability(content_id).await
}).await;

match result {
    Ok(data) => process_data(data),
    Err(CircuitBreakerError::Open) => {
        // Fallback to cached data
        get_cached_availability(content_id).await
    }
    Err(e) => Err(e),
}
```

---

## 6. ERROR HANDLING PATTERNS

### 6.1 Error Type Hierarchy

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MediaGatewayError {
    #[error("Platform connector error: {0}")]
    Connector(#[from] ConnectorError),

    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),

    #[error("Agent execution error: {0}")]
    Agent(#[from] AgentError),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}

#[derive(Error, Debug)]
pub enum ConnectorError {
    #[error("Rate limit exceeded for {platform}")]
    RateLimitExceeded { platform: String },

    #[error("Platform API unavailable: {platform}")]
    PlatformUnavailable { platform: String },

    #[error("Invalid response from {platform}: {reason}")]
    InvalidResponse { platform: String, reason: String },

    #[error("Authentication failed for {platform}")]
    AuthenticationFailed { platform: String },
}
```

### 6.2 Retry Strategies

**Exponential Backoff with Jitter:**
```rust
use backoff::{ExponentialBackoff, retry};

async fn fetch_with_retry<F, T>(operation: F) -> Result<T, Error>
where
    F: Fn() -> Future<Output = Result<T, Error>>,
{
    let backoff = ExponentialBackoff {
        initial_interval: Duration::from_secs(2),
        max_interval: Duration::from_secs(60),
        max_elapsed_time: Some(Duration::from_secs(300)),
        multiplier: 2.0,
        randomization_factor: 0.3,  // Jitter
        ..Default::default()
    };

    retry(backoff, || async {
        match operation().await {
            Ok(result) => Ok(result),
            Err(e) if e.is_retryable() => Err(backoff::Error::transient(e)),
            Err(e) => Err(backoff::Error::permanent(e)),
        }
    }).await
}
```

**Retry Decision Logic:**
```rust
impl ConnectorError {
    fn is_retryable(&self) -> bool {
        match self {
            ConnectorError::RateLimitExceeded { .. } => true,
            ConnectorError::PlatformUnavailable { .. } => true,
            ConnectorError::InvalidResponse { .. } => false,  // Don't retry parse errors
            ConnectorError::AuthenticationFailed { .. } => false,  // Require user action
        }
    }
}
```

### 6.3 Graceful Degradation

**Fallback Strategies:**
```rust
async fn get_recommendations_with_fallback(
    user_id: &str
) -> Result<Vec<Recommendation>, Error> {
    // Try personalized recommendations
    match get_personalized_recommendations(user_id).await {
        Ok(recs) => Ok(recs),
        Err(e) => {
            warn!("Personalized recommendations failed: {}. Falling back to trending.", e);

            // Fallback 1: Trending content
            match get_trending_content().await {
                Ok(trending) => Ok(trending),
                Err(e2) => {
                    error!("Trending fallback failed: {}. Using cached popular.", e2);

                    // Fallback 2: Cached popular content
                    get_cached_popular_content().await
                        .map_err(|e3| {
                            error!("All fallbacks exhausted: {}", e3);
                            e3
                        })
                }
            }
        }
    }
}
```

**Availability Trust Warnings:**
```rust
struct AvailabilityResult {
    content: Content,
    availability: Vec<PlatformAvailability>,
    warnings: Vec<AvailabilityWarning>,
}

async fn fetch_availability_with_warnings(
    content_id: &str
) -> AvailabilityResult {
    let mut availability = fetch_availability(content_id).await?;
    let mut warnings = Vec::new();

    for platform_avail in &availability {
        if platform_avail.trust_score < 0.5 {
            warnings.push(AvailabilityWarning {
                platform: platform_avail.platform.clone(),
                message: format!(
                    "Availability data may be outdated (trust score: {:.2})",
                    platform_avail.trust_score
                ),
            });
        }
    }

    AvailabilityResult {
        content,
        availability,
        warnings,
    }
}
```

### 6.4 CRDT Conflict Resolution

**HLC Timestamp Ordering:**
```rust
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct HybridLogicalClock {
    physical_time: u64,      // Unix timestamp in milliseconds
    logical_counter: u64,    // Monotonic counter for same physical time
    node_id: String,         // Tie-breaker for identical HLC values
}

impl HybridLogicalClock {
    fn tick(&mut self, wall_time: u64) {
        if wall_time > self.physical_time {
            self.physical_time = wall_time;
            self.logical_counter = 0;
        } else {
            self.logical_counter += 1;
        }
    }
}

// Last-Writer-Wins resolution
fn resolve_lww_conflict(
    local: &LWWRegister<WatchProgress>,
    remote: &LWWRegister<WatchProgress>,
) -> LWWRegister<WatchProgress> {
    if remote.timestamp > local.timestamp {
        remote.clone()
    } else {
        local.clone()
    }
}
```

**OR-Set Merge:**
```rust
fn merge_or_sets<T: Eq + Hash>(
    local: &ORSet<T>,
    remote: &ORSet<T>,
) -> ORSet<T> {
    let mut merged = ORSet::new();

    // Merge additions
    for (item, timestamps) in local.additions.iter() {
        merged.additions.entry(item.clone())
            .or_insert_with(HashSet::new)
            .extend(timestamps);
    }
    for (item, timestamps) in remote.additions.iter() {
        merged.additions.entry(item.clone())
            .or_insert_with(HashSet::new)
            .extend(timestamps);
    }

    // Merge removals
    for (item, timestamps) in local.removals.iter() {
        merged.removals.entry(item.clone())
            .or_insert_with(HashSet::new)
            .extend(timestamps);
    }
    for (item, timestamps) in remote.removals.iter() {
        merged.removals.entry(item.clone())
            .or_insert_with(HashSet::new)
            .extend(timestamps);
    }

    merged
}

// Determine current membership (removal always wins)
fn or_set_members<T: Eq + Hash + Clone>(set: &ORSet<T>) -> HashSet<T> {
    let mut members = HashSet::new();

    for (item, add_timestamps) in &set.additions {
        let removed = set.removals.get(item)
            .map(|rem_timestamps| {
                // Check if any removal timestamp >= all addition timestamps
                rem_timestamps.iter().any(|rem_ts| {
                    add_timestamps.iter().all(|add_ts| rem_ts >= add_ts)
                })
            })
            .unwrap_or(false);

        if !removed {
            members.insert(item.clone());
        }
    }

    members
}
```

### 6.5 Agent Error Recovery

**E2B Sandbox Failure Handling:**
```rust
async fn execute_agent_with_recovery(
    code: &str,
    max_retries: u32,
) -> Result<ExecutionResult, AgentError> {
    let mut retries = 0;

    loop {
        match execute_in_sandbox(code).await {
            Ok(result) => return Ok(result),
            Err(SandboxError::Timeout) if retries < max_retries => {
                retries += 1;
                warn!("Sandbox timeout. Retry {}/{}", retries, max_retries);

                // Extend timeout on retry
                increase_sandbox_timeout();
                continue;
            }
            Err(SandboxError::OutOfMemory) => {
                error!("Sandbox OOM. Falling back to simpler algorithm.");
                return execute_fallback_algorithm().await;
            }
            Err(e) => return Err(AgentError::SandboxFailed(e)),
        }
    }
}
```

### 6.6 OAuth Error Handling

**Token Refresh Flow:**
```rust
async fn authenticated_request<T>(
    endpoint: &str,
    access_token: &str,
) -> Result<T, Error> {
    let response = http_client
        .get(endpoint)
        .bearer_auth(access_token)
        .send()
        .await?;

    match response.status() {
        StatusCode::OK => Ok(response.json().await?),
        StatusCode::UNAUTHORIZED => {
            // Access token expired - attempt refresh
            let new_token = refresh_access_token().await?;

            // Retry with new token
            let retry_response = http_client
                .get(endpoint)
                .bearer_auth(&new_token)
                .send()
                .await?;

            Ok(retry_response.json().await?)
        }
        status => Err(Error::HttpError(status)),
    }
}
```

---

## 7. TRUST SCORING SYSTEM

### 7.1 Trust Score Calculation

**Component Weights (Total 100%):**

| Component | Weight | Calculation |
|-----------|--------|-------------|
| Source Reliability | 25% | Platform API = 1.0, Aggregator = 0.70, Scraping = 0.40 |
| Metadata Accuracy | 25% | Cross-source validation, completeness, entity resolution |
| Availability Confidence | 20% | HTTP deep-link validation, recency decay |
| Recommendation Quality | 15% | CTR, watch completion, rating correlation |
| User Preference Confidence | 15% | Interaction count, consistency, recency |

**Composite Score:**
```rust
fn calculate_trust_score(components: &TrustComponents) -> f32 {
    const WEIGHTS: [f32; 5] = [0.25, 0.25, 0.20, 0.15, 0.15];

    let scores = [
        components.source_reliability,
        components.metadata_accuracy,
        components.availability_confidence,
        components.recommendation_quality,
        components.user_preference_confidence,
    ];

    scores.iter()
        .zip(WEIGHTS.iter())
        .map(|(score, weight)| score * weight)
        .sum()
}
```

### 7.2 Time Decay Function

**Exponential Decay:**
```rust
fn apply_time_decay(
    original_trust: f32,
    verified_at: DateTime<Utc>,
    current_time: DateTime<Utc>,
) -> f32 {
    let days_since_verification = (current_time - verified_at).num_days() as f32;

    // 1% decay per day
    let decay_rate = 0.01;

    original_trust * (1.0 - decay_rate * days_since_verification).max(0.0)
}
```

**Verification Strategy:**
```rust
async fn verify_availability(
    deep_link: &str,
    platform: Platform,
) -> VerificationResult {
    // HTTP HEAD request to validate link
    let response = http_client
        .head(deep_link)
        .timeout(Duration::from_secs(5))
        .send()
        .await?;

    let is_available = response.status().is_success();

    VerificationResult {
        is_available,
        verified_at: Utc::now(),
        platform,
        http_status: response.status(),
    }
}
```

### 7.3 Trust Filtering Thresholds

**Application:**
```rust
async fn filter_by_trust(
    results: Vec<SearchResult>,
    min_trust: f32,
) -> Vec<SearchResult> {
    results.into_iter()
        .filter(|result| {
            let current_trust = apply_time_decay(
                result.trust_score,
                result.verified_at,
                Utc::now(),
            );

            current_trust >= min_trust
        })
        .collect()
}
```

**Thresholds:**
- Recommendations: 0.6 minimum trust
- Availability display: 0.5 minimum (with warning if < 0.6)
- Deep-link activation: 0.5 minimum

---

## 8. PRIVACY AND SECURITY ARCHITECTURE

### 8.1 Three-Tier Data Handling

**Tier 1: On-Device (Never Transmitted):**
```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

struct OnDeviceStorage {
    cipher: Aes256Gcm,
    local_db_path: PathBuf,
}

impl OnDeviceStorage {
    fn store_watch_history(&self, history: &WatchHistory) -> Result<(), Error> {
        let serialized = serde_json::to_vec(history)?;

        // Encrypt before writing to disk
        let nonce = Nonce::from_slice(&generate_random_nonce());
        let ciphertext = self.cipher.encrypt(nonce, serialized.as_ref())?;

        // Store in local SQLite database
        self.local_db.execute(
            "INSERT INTO watch_history (encrypted_data) VALUES (?1)",
            params![ciphertext],
        )?;

        Ok(())
    }
}
```

**Tier 2: Aggregated with Differential Privacy:**
```rust
use differential_privacy::{DpEngine, LaplaceMechanism};

async fn send_federated_gradients(
    local_gradients: Vec<f32>,
    epsilon: f64,
    delta: f64,
) -> Result<(), Error> {
    // Clip gradients to bounded sensitivity
    let clipping_threshold = 1.0;
    let clipped_gradients: Vec<f32> = local_gradients.iter()
        .map(|&g| g.clamp(-clipping_threshold, clipping_threshold))
        .collect();

    // Add Laplace noise for differential privacy
    let dp_engine = DpEngine::new(epsilon, delta);
    let noisy_gradients: Vec<f32> = clipped_gradients.iter()
        .map(|&g| dp_engine.add_laplace_noise(g, clipping_threshold))
        .collect();

    // Send to aggregation server
    federated_learning_client.upload_gradients(noisy_gradients).await?;

    Ok(())
}
```

**Parameters:**
- ε (epsilon) = 1.0 (privacy budget)
- δ (delta) = 1e-5 (failure probability)
- Clipping threshold C = 1.0 (gradient norm bound)

**Tier 3: Public Metadata (Non-Identifying):**
```rust
struct PublicMetadata {
    content_id: String,
    title: String,
    genres: Vec<String>,
    aggregate_popularity: f32,  // Aggregated across all users
    average_rating: f32,         // Aggregated, no individual ratings
    release_year: u16,
}
```

### 8.2 Federated Learning Cycle

**Weekly Update Workflow:**
```rust
// 1. Server distributes global model
async fn download_global_model() -> Result<ModelWeights, Error> {
    federated_server.download_latest_model().await
}

// 2. Local training on device
async fn train_locally(
    model: &mut Model,
    local_data: &WatchHistory,
    epochs: u32,
) -> Vec<f32> {
    for _ in 0..epochs {
        let batch = local_data.sample_batch(32);
        let loss = model.forward(batch);
        model.backward(loss);
    }

    model.get_gradients()
}

// 3. Secure aggregation
async fn secure_aggregation(
    local_gradients: Vec<f32>,
    user_id: &str,
) -> Result<(), Error> {
    // Add differential privacy noise
    let noisy_gradients = add_dp_noise(local_gradients, EPSILON, DELTA);

    // Encrypt gradients with user's public key
    let encrypted = encrypt_gradients(noisy_gradients, user_id)?;

    // Upload to aggregation server
    federated_server.contribute_gradients(encrypted).await?;

    Ok(())
}

// 4. Server aggregates and updates global model
// (Server-side operation - not in client code)
```

### 8.3 OAuth 2.1 Security (RFC 9700)

**Mandatory Controls:**

**1. PKCE for All Clients:**
```rust
// Generate cryptographically secure verifier
fn generate_pkce_verifier() -> String {
    use rand::Rng;
    let random_bytes: Vec<u8> = (0..128)
        .map(|_| rand::thread_rng().gen())
        .collect();

    base64_url_encode(&random_bytes)
}

// Compute SHA256 challenge
fn compute_pkce_challenge(verifier: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let result = hasher.finalize();

    base64_url_encode(&result)
}
```

**2. Sender-Constrained Tokens (DPoP):**
```rust
use jsonwebtoken::{encode, Header, EncodingKey};

// Demonstration of Proof-of-Possession
fn create_dpop_proof(
    http_method: &str,
    http_uri: &str,
    access_token_hash: &str,
    private_key: &EncodingKey,
) -> Result<String, Error> {
    let claims = json!({
        "htm": http_method,
        "htu": http_uri,
        "ath": access_token_hash,
        "iat": Utc::now().timestamp(),
        "jti": generate_random_jti(),
    });

    let header = Header {
        typ: Some("dpop+jwt".to_string()),
        alg: Algorithm::ES256,
        ..Default::default()
    };

    encode(&header, &claims, private_key)
}

// Include DPoP header in requests
async fn authenticated_request_with_dpop(
    endpoint: &str,
    access_token: &str,
    dpop_private_key: &EncodingKey,
) -> Result<Response, Error> {
    let dpop_proof = create_dpop_proof(
        "GET",
        endpoint,
        &hash_access_token(access_token),
        dpop_private_key,
    )?;

    http_client
        .get(endpoint)
        .header("Authorization", format!("DPoP {}", access_token))
        .header("DPoP", dpop_proof)
        .send()
        .await
}
```

**3. Token Rotation:**
```rust
async fn refresh_with_rotation(
    refresh_token: &str,
) -> Result<TokenResponse, Error> {
    let response = http_client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", &client_id),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await?
        .json::<TokenResponse>()
        .await?;

    // New refresh token issued - old one invalidated
    keyring.set_entry("access_token", &response.access_token)?;
    keyring.set_entry("refresh_token", &response.refresh_token)?;

    Ok(response)
}
```

**4. Short Token Lifetimes:**
- Access tokens: 15 minutes
- Refresh tokens: 7 days (rotated on each use)

### 8.4 Data Deletion Cascade

**GDPR/CCPA Compliance:**
```rust
async fn delete_user_data(user_id: &str) -> Result<(), Error> {
    // 1. Delete from primary database
    database.execute(
        "DELETE FROM users WHERE user_id = ?1",
        params![user_id],
    ).await?;

    // 2. Delete LoRA adapters
    memorystore.delete(&format!("lora:{}", user_id)).await?;
    database.execute(
        "DELETE FROM lora_adapters WHERE user_id = ?1",
        params![user_id],
    ).await?;

    // 3. Delete reasoning patterns
    database.execute(
        "DELETE FROM reasoning_patterns WHERE user_id = ?1",
        params![user_id],
    ).await?;

    // 4. Purge interaction history
    database.execute(
        "DELETE FROM interactions WHERE user_id = ?1",
        params![user_id],
    ).await?;

    // 5. Remove PubNub subscriptions
    pubnub.unsubscribe_all(user_id).await?;

    // 6. Audit log
    audit_log.record_deletion(user_id, Utc::now()).await?;

    Ok(())
}
```

---

## 9. MULTI-AGENT COORDINATION (SPARC METHODOLOGY)

### 9.1 Agent Roles and Responsibilities

**Nine Specialized Agents:**

| Agent | Responsibility | Latency Budget |
|-------|---------------|----------------|
| **SwarmLead** | Task delegation, SPARC phase management | 10ms |
| **ContentSearcher** | Ruvector vector/graph traversal | 100ms |
| **RecommendationBuilder** | GNN + collaborative filtering fusion | 150ms |
| **AvailabilityChecker** | Rights validation, deep-link verification | 80ms |
| **DeviceCoordinator** | Cross-device command broadcasting | 50ms |
| **ContextKeeper** | AgentDB memory retrieval (Redis) | 20ms |
| **PatternLearner** | ReasoningBank persistence | 40ms |
| **ResultMerger** | Deduplication, RRF ranking | 30ms |
| **QualityAssurer** | Trust scoring, edge case handling | 50ms |

**Total E2E Latency Target:** ~430ms (including parallelization)

### 9.2 SPARC Phase Timing

**Sequential Phases:**
1. **Specification (50ms):** SwarmLead analyzes query, identifies intent
2. **Pseudocode (30ms):** Determine search strategy (semantic, collaborative, graph)
3. **Architecture (200ms parallel):** Spawn ContentSearcher, RecommendationBuilder, AvailabilityChecker
4. **Refinement (100ms):** ResultMerger applies RRF, deduplication
5. **Completion (50ms):** QualityAssurer filters by trust, generates explanations

### 9.3 Agent Coordination Protocol

**Pre-Task Hook:**
```bash
npx claude-flow@alpha hooks pre-task \
  --agent-id "content-searcher" \
  --description "Semantic search for 'thriller movies'" \
  --dependencies "context-keeper"
```

**Session Restore:**
```bash
npx claude-flow@alpha hooks session-restore \
  --session-id "swarm-20251206-001" \
  --agent-id "content-searcher"
```

**Post-Edit (Memory Update):**
```bash
npx claude-flow@alpha hooks post-edit \
  --file "/tmp/search-results.json" \
  --memory-key "swarm/content-searcher/results" \
  --session-id "swarm-20251206-001"
```

**Notification:**
```bash
npx claude-flow@alpha hooks notify \
  --message "Found 157 matching titles" \
  --agent-id "content-searcher" \
  --level "info"
```

**Post-Task:**
```bash
npx claude-flow@alpha hooks post-task \
  --task-id "search-thriller-movies" \
  --agent-id "content-searcher" \
  --status "completed" \
  --results-path "/tmp/search-results.json"
```

**Session End:**
```bash
npx claude-flow@alpha hooks session-end \
  --session-id "swarm-20251206-001" \
  --export-metrics true \
  --summary "Completed content discovery with 157 results"
```

### 9.4 Memory Coordination

**AgentDB (Redis-backed) Memory Keys:**
```
swarm/{agent_id}/status           → Agent state (idle, working, completed)
swarm/{agent_id}/results          → Intermediate results
swarm/shared/user-context         → User preferences, interaction history
swarm/shared/search-query         → Original query
swarm/shared/final-results        → Merged recommendations
```

**Memory Storage Example:**
```rust
use redis::AsyncCommands;

async fn store_agent_results(
    agent_id: &str,
    results: &SearchResults,
    redis_client: &mut redis::aio::Connection,
) -> Result<(), Error> {
    let key = format!("swarm/{}/results", agent_id);
    let serialized = serde_json::to_string(results)?;

    redis_client.set_ex(
        &key,
        serialized,
        3600,  // 1 hour TTL
    ).await?;

    Ok(())
}
```

---

## 10. CLI ARCHITECTURE (DEVELOPER/OPERATOR TOOL)

### 10.1 Core Commands

**Search Commands:**
```bash
# Semantic query
tv-discover search "thriller movies like inception"

# Similar content
tv-discover similar --content-id tt1375666 --limit 10
```

**Browse Commands:**
```bash
tv-discover browse trending --region USA
tv-discover browse new-releases --platform netflix
tv-discover browse expiring --days 7
tv-discover browse categories --genre sci-fi
```

**Recommendation Commands:**
```bash
tv-discover recommend for-me --limit 20
tv-discover recommend mood --mood "relaxing" --time 30min
tv-discover recommend social --friends-list alice,bob
```

**Playback Commands:**
```bash
tv-discover watch now --content-id tt1375666 --platform netflix
tv-discover watch queue --content-id tt0468569
```

**Device Management:**
```bash
tv-discover devices list
tv-discover devices connect --device living-room-tv
tv-discover devices cast --content-id tt1375666 --device living-room-tv
```

**Account Management:**
```bash
tv-discover accounts link --platform netflix
tv-discover accounts status
tv-discover accounts unlink --platform hulu
```

**Preferences:**
```bash
tv-discover preferences genres --set "sci-fi,thriller,documentary"
tv-discover preferences platforms --enable netflix,prime,disney
tv-discover preferences show
```

### 10.2 CLI Implementation (Rust TUI)

**Framework: ratatui + clap**

```rust
use clap::{Parser, Subcommand};
use ratatui::{Terminal, backend::CrosstermBackend};

#[derive(Parser)]
#[command(name = "tv-discover")]
#[command(about = "Unified TV content discovery CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for content
    Search {
        query: String,

        #[arg(long)]
        region: Option<String>,

        #[arg(long, default_value = "20")]
        limit: usize,
    },

    /// Browse content categories
    Browse {
        #[command(subcommand)]
        category: BrowseCategory,
    },

    /// Get recommendations
    Recommend {
        #[command(subcommand)]
        mode: RecommendMode,
    },

    // ... other commands
}

#[derive(Subcommand)]
enum BrowseCategory {
    Trending {
        #[arg(long)]
        region: Option<String>,
    },
    NewReleases {
        #[arg(long)]
        platform: Option<String>,
    },
    Expiring {
        #[arg(long, default_value = "7")]
        days: u32,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Search { query, region, limit } => {
            execute_search(&query, region.as_deref(), limit).await?;
        }
        Commands::Browse { category } => {
            execute_browse(category).await?;
        }
        // ... handle other commands
    }

    Ok(())
}
```

**TUI Interface:**
```rust
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

async fn display_search_results(results: &[SearchResult]) -> Result<(), Error> {
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;

    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),      // Header
                Constraint::Min(0),         // Results
                Constraint::Length(3),      // Footer
            ])
            .split(f.size());

        // Header
        let header = Paragraph::new("Search Results")
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(header, chunks[0]);

        // Results list
        let items: Vec<ListItem> = results.iter()
            .map(|result| {
                let text = format!(
                    "{} - {} ({})",
                    result.title,
                    result.platforms.join(", "),
                    result.recommendation_score
                );
                ListItem::new(text)
            })
            .collect();

        let results_list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Results"));
        f.render_widget(results_list, chunks[1]);

        // Footer
        let footer = Paragraph::new("↑/↓ Navigate | Enter Select | Q Quit")
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(footer, chunks[2]);
    })?;

    Ok(())
}
```

### 10.3 Configuration Management

**Config File Location:** `~/.config/tv-discover/config.toml`

```toml
[profile]
active = "personal"

[profiles.personal]
user_id = "user_123"
region = "USA"

[profiles.personal.subscriptions]
netflix = { tier = "Premium", ad_support = false }
prime = { tier = "Standard", ad_support = false }
disney = { tier = "Standard", ad_support = true }

[profiles.personal.preferences]
genres = ["sci-fi", "thriller", "documentary"]
platforms = ["netflix", "prime", "disney"]

[profiles.personal.devices]
living-room-tv = { type = "SmartTV", platform = "Tizen", id = "device_456" }
mobile = { type = "Mobile", platform = "iOS", id = "device_789" }

[api]
base_url = "https://api.mediagateway.com/v1"
timeout_seconds = 30

[cache]
enabled = true
ttl_seconds = 3600
```

**Loading Configuration:**
```rust
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Deserialize, Serialize)]
struct Config {
    profile: ProfileConfig,
    profiles: HashMap<String, UserProfile>,
    api: ApiConfig,
    cache: CacheConfig,
}

fn load_config() -> Result<Config, Error> {
    let config_path = dirs::config_dir()
        .ok_or(Error::ConfigDirNotFound)?
        .join("tv-discover/config.toml");

    let contents = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&contents)?;

    Ok(config)
}
```

### 10.4 Token Storage (OS Keyring)

```rust
use keyring::Entry;

fn store_oauth_tokens(
    platform: &str,
    access_token: &str,
    refresh_token: &str,
) -> Result<(), Error> {
    let service = "tv-discover";

    let access_entry = Entry::new(service, &format!("{}_access", platform))?;
    access_entry.set_password(access_token)?;

    let refresh_entry = Entry::new(service, &format!("{}_refresh", platform))?;
    refresh_entry.set_password(refresh_token)?;

    Ok(())
}

fn retrieve_oauth_tokens(platform: &str) -> Result<(String, String), Error> {
    let service = "tv-discover";

    let access_entry = Entry::new(service, &format!("{}_access", platform))?;
    let access_token = access_entry.get_password()?;

    let refresh_entry = Entry::new(service, &format!("{}_refresh", platform))?;
    let refresh_token = refresh_entry.get_password()?;

    Ok((access_token, refresh_token))
}
```

---

## 11. DEPLOYMENT PATTERNS

### 11.1 GKE Autopilot Service Organization

**Namespace Structure:**
```yaml
# namespaces.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: layer1-ingestion
---
apiVersion: v1
kind: Namespace
metadata:
  name: layer2-intelligence
---
apiVersion: v1
kind: Namespace
metadata:
  name: layer3-consolidation
```

**Layer 1 Services (Ingestion):**
```yaml
# mcp-connector-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: netflix-connector
  namespace: layer1-ingestion
spec:
  replicas: 3
  selector:
    matchLabels:
      app: netflix-connector
  template:
    metadata:
      labels:
        app: netflix-connector
    spec:
      serviceAccountName: mcp-connector-sa
      containers:
      - name: connector
        image: us-central1-docker.pkg.dev/PROJECT_ID/media-gateway/netflix-connector:latest
        env:
        - name: PLATFORM
          value: "netflix"
        - name: RATE_LIMIT_QPS
          value: "10"
        resources:
          requests:
            cpu: "500m"
            memory: "1Gi"
          limits:
            cpu: "1000m"
            memory: "2Gi"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
```

**Layer 2 Services (Intelligence):**
```yaml
# recommendation-engine-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: recommendation-engine
  namespace: layer2-intelligence
spec:
  replicas: 5
  selector:
    matchLabels:
      app: recommendation-engine
  template:
    metadata:
      labels:
        app: recommendation-engine
    spec:
      serviceAccountName: intelligence-sa
      containers:
      - name: engine
        image: us-central1-docker.pkg.dev/PROJECT_ID/media-gateway/recommendation-engine:latest
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: cloudsql-credentials
              key: connection-string
        - name: REDIS_URL
          value: "redis://memorystore-instance:6379"
        resources:
          requests:
            cpu: "2000m"
            memory: "4Gi"
            nvidia.com/gpu: "1"  # For GNN inference
          limits:
            cpu: "4000m"
            memory: "8Gi"
            nvidia.com/gpu: "1"
```

**Layer 3 Services (Consolidation):**
```yaml
# metadata-fabric-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: metadata-fabric
  namespace: layer3-consolidation
spec:
  replicas: 4
  selector:
    matchLabels:
      app: metadata-fabric
  template:
    metadata:
      labels:
        app: metadata-fabric
    spec:
      serviceAccountName: consolidation-sa
      containers:
      - name: fabric
        image: us-central1-docker.pkg.dev/PROJECT_ID/media-gateway/metadata-fabric:latest
        env:
        - name: RUVECTOR_URL
          value: "http://ruvector-service:8080"
        resources:
          requests:
            cpu: "1000m"
            memory: "2Gi"
          limits:
            cpu: "2000m"
            memory: "4Gi"
```

### 11.2 Cloud Run API Gateway

```yaml
# api-gateway-service.yaml
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: api-gateway
  namespace: default
spec:
  template:
    metadata:
      annotations:
        autoscaling.knative.dev/minScale: "3"
        autoscaling.knative.dev/maxScale: "100"
        autoscaling.knative.dev/target: "80"  # Target 80% CPU
    spec:
      serviceAccountName: api-gateway-sa
      containers:
      - image: us-central1-docker.pkg.dev/PROJECT_ID/media-gateway/api-gateway:latest
        env:
        - name: GKE_ENDPOINT
          value: "http://search-api.layer3-consolidation.svc.cluster.local"
        - name: CORS_ALLOWED_ORIGINS
          value: "https://app.mediagateway.com,https://mobile.mediagateway.com"
        resources:
          limits:
            cpu: "2000m"
            memory: "1Gi"
        ports:
        - containerPort: 8080
```

### 11.3 Terraform Infrastructure

**Main Configuration:**
```hcl
# terraform/environments/dev/main.tf
terraform {
  required_version = ">= 1.0"

  backend "gcs" {
    bucket = "media-gateway-terraform-state-dev"
    prefix = "terraform/state"
  }

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 5.0"
    }
  }
}

provider "google" {
  project = var.project_id
  region  = var.region
}

module "gke" {
  source = "../../modules/gke"

  project_id   = var.project_id
  region       = var.region
  cluster_name = "media-gateway-cluster"

  node_pools = [
    {
      name         = "layer1-pool"
      machine_type = "e2-standard-4"
      min_count    = 3
      max_count    = 10
    },
    {
      name         = "layer2-gpu-pool"
      machine_type = "n1-standard-4"
      accelerator_type = "nvidia-tesla-t4"
      accelerator_count = 1
      min_count    = 2
      max_count    = 8
    },
  ]
}

module "database" {
  source = "../../modules/database"

  project_id    = var.project_id
  region        = var.region
  instance_name = "media-gateway-db"
  tier          = "db-custom-4-16384"  # 4 vCPU, 16GB RAM

  high_availability = true
  backup_enabled    = true
}

module "cache" {
  source = "../../modules/cache"

  project_id    = var.project_id
  region        = var.region
  instance_name = "media-gateway-cache"
  memory_size_gb = 10

  high_availability = true
  redis_version     = "REDIS_7_0"
}

module "pubsub" {
  source = "../../modules/pubsub"

  project_id = var.project_id

  topics = [
    "user-interactions",
    "content-updates",
    "availability-changes",
  ]

  subscriptions = [
    {
      name  = "interaction-processor"
      topic = "user-interactions"
      push_endpoint = "https://api-gateway-SERVICE_ID-uc.a.run.app/webhooks/interactions"
    },
  ]
}
```

### 11.4 CI/CD Pipeline (Cloud Build)

```yaml
# cloudbuild.yaml
steps:
  # Build Rust binary
  - name: 'rust:1.75'
    entrypoint: 'cargo'
    args: ['build', '--release', '--bin', 'recommendation-engine']

  # Run tests
  - name: 'rust:1.75'
    entrypoint: 'cargo'
    args: ['test', '--all']

  # Build Docker image
  - name: 'gcr.io/cloud-builders/docker'
    args:
      - 'build'
      - '-t'
      - 'us-central1-docker.pkg.dev/$PROJECT_ID/media-gateway/recommendation-engine:$COMMIT_SHA'
      - '-t'
      - 'us-central1-docker.pkg.dev/$PROJECT_ID/media-gateway/recommendation-engine:latest'
      - '-f'
      - 'Dockerfile'
      - '.'

  # Push to Artifact Registry
  - name: 'gcr.io/cloud-builders/docker'
    args:
      - 'push'
      - '--all-tags'
      - 'us-central1-docker.pkg.dev/$PROJECT_ID/media-gateway/recommendation-engine'

  # Deploy to GKE
  - name: 'gcr.io/cloud-builders/gke-deploy'
    args:
      - 'run'
      - '--filename=k8s/layer2/recommendation-engine-deployment.yaml'
      - '--cluster=media-gateway-cluster'
      - '--location=us-central1'
      - '--namespace=layer2-intelligence'
      - '--image=us-central1-docker.pkg.dev/$PROJECT_ID/media-gateway/recommendation-engine:$COMMIT_SHA'

timeout: 1800s  # 30 minutes
options:
  machineType: 'E2_HIGHCPU_8'
```

---

## 12. REPOSITORY STRUCTURE AND BUILD ORDER

### 12.1 51 Micro-Repository Organization

**Layer 1 (Ingestion) - 23 repositories:**
- `mg-mcp-connector-{platform}` × 10 (Netflix, Prime, Disney, Hulu, Apple TV+, YouTube, Crave, HBO Max, Peacock, Paramount+)
- `mg-normalizer` (Entity normalization)
- `mg-entity-resolver` (Deduplication via EIDR/IMDb/TMDb)
- `mg-auth-service` (OAuth 2.0 + PKCE)
- `mg-sync-engine` (PubNub integration)
- `mg-mcp-server` (MCP protocol implementation)
- `mg-proto` (Protobuf definitions)
- `mg-sdk-rust` (Shared Rust SDK)
- `mg-config` (Configuration management)
- `mg-rate-limiter` (Token bucket implementation)
- `mg-health-check` (Service health monitoring)
- `mg-deep-link-validator` (HTTP verification)
- `mg-trust-scorer` (Trust calculation engine)
- `mg-aggregator-client` (JustWatch, Watchmode, etc.)

**Layer 2 (Intelligence) - 8 repositories:**
- `mg-agent-orchestrator` (Claude-Flow coordination)
- `mg-recommendation-engine` (Hybrid RRF)
- `mg-semantic-search` (Ruvector integration)
- `mg-sona-client` (SONA API client)
- `mg-lora-adapter-manager` (LoRA loading/updating)
- `mg-reasoning-bank` (Pattern storage)
- `mg-gnn-inference` (GraphSAGE)
- `mg-context-keeper` (AgentDB/Redis)

**Layer 3 (Consolidation) - 5 repositories:**
- `mg-metadata-fabric` (Unified metadata)
- `mg-availability-index` (Cross-platform availability)
- `mg-rights-engine` (Licensing validation)
- `mg-personalization-engine` (User preferences)
- `mg-search-api` (Unified search endpoint)

**Layer 4 (Applications) - 6 repositories:**
- `mg-cli` (Rust TUI)
- `mg-web` (Next.js web app)
- `mg-ios` (Swift iOS app)
- `mg-android` (Kotlin Android app)
- `mg-tizen` (Smart TV - Samsung)
- `mg-webos` (Smart TV - LG)

**Foundation - 9 repositories:**
- `mg-proto` (Protobuf schemas)
- `mg-sdk-rust` (Shared Rust utilities)
- `mg-sdk-ts` (TypeScript SDK)
- `mg-config` (Configuration management)
- `mg-error-types` (Error definitions)
- `mg-logging` (Structured logging)
- `mg-metrics` (Prometheus metrics)
- `mg-tracing` (Distributed tracing)
- `mg-test-utils` (Testing utilities)

### 12.2 Build Order for New Developers

**Step 1: Foundation (parallel builds possible):**
```bash
# Build in any order
cd mg-proto && cargo build --release
cd mg-sdk-rust && cargo build --release
cd mg-config && cargo build --release
cd mg-error-types && cargo build --release
cd mg-logging && cargo build --release
```

**Step 2: Core Infrastructure (depends on Foundation):**
```bash
cd mg-auth-service && cargo build --release
cd mg-sync-engine && cargo build --release
cd mg-entity-resolver && cargo build --release
cd mg-rate-limiter && cargo build --release
cd mg-health-check && cargo build --release
```

**Step 3: Layer 1 Connectors (depends on Core):**
```bash
# Can build in parallel
for platform in netflix prime disney hulu apple-tv youtube crave hbo-max peacock paramount; do
  cd mg-mcp-connector-$platform && cargo build --release &
done
wait

cd mg-mcp-server && cargo build --release
```

**Step 4: Layer 2 Intelligence (depends on Layer 1):**
```bash
cd mg-sona-client && cargo build --release
cd mg-lora-adapter-manager && cargo build --release
cd mg-reasoning-bank && cargo build --release
cd mg-gnn-inference && cargo build --release
cd mg-context-keeper && cargo build --release

# Then orchestration services
cd mg-recommendation-engine && cargo build --release
cd mg-semantic-search && cargo build --release
cd mg-agent-orchestrator && cargo build --release
```

**Step 5: Layer 3 Consolidation (depends on Layer 2):**
```bash
cd mg-metadata-fabric && cargo build --release
cd mg-availability-index && cargo build --release
cd mg-rights-engine && cargo build --release
cd mg-personalization-engine && cargo build --release
cd mg-search-api && cargo build --release
```

**Step 6: Layer 4 Applications (depends on Layer 3):**
```bash
cd mg-cli && cargo build --release
cd mg-web && npm install && npm run build
cd mg-ios && xcodebuild
cd mg-android && ./gradlew build
```

### 12.3 Versioning Strategy

**Semantic Versioning with Changelog Enforcement:**

Each repository maintains:
- `CHANGELOG.md` (auto-generated from commits)
- `Cargo.toml` or `package.json` with version
- Git tags for releases

**Example workflow:**
```bash
# Developer commits with conventional commit format
git commit -m "feat(recommendation): Add GNN-based similarity search"

# CI generates changelog and bumps version
npx standard-version

# Creates tag and updates CHANGELOG.md
# v1.2.0 (minor bump for feature)

# Publish to crates.io
cargo publish
```

**Breaking Change Coordination:**
```bash
# When mg-proto changes protobuf schema (breaking)
git commit -m "feat(proto)!: Add availability_confidence field to ContentMetadata

BREAKING CHANGE: All consumers must update to handle new field"

# Triggers major version bump: v1.x.x -> v2.0.0
# Dependent repos must update mg-proto dependency
```

---

## 13. HACKATHON-TV5 INTEGRATION WORKFLOWS

### 13.1 MCP Tool Exposure

**6 Core hackathon-tv5 Tools:**
1. `get_hackathon_info` - Project metadata
2. `get_tracks` - Available development tracks
3. `get_available_tools` - List integrated tools
4. `get_project_status` - Development progress
5. `check_tool_installed` - Verify tool availability
6. `get_resources` - Documentation and guides

**Media Gateway Extensions (10+ Platform Tools):**
```rust
// Each platform connector exposes MCP tools
impl MCPToolProvider for NetflixConnector {
    fn get_tool_schemas(&self) -> Vec<MCPToolSchema> {
        vec![
            MCPToolSchema {
                name: "netflix_search".to_string(),
                description: "Search Netflix catalog".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string"},
                        "region": {"type": "string"},
                        "limit": {"type": "integer", "default": 20}
                    },
                    "required": ["query"]
                }),
            },
            MCPToolSchema {
                name: "netflix_fetch_content".to_string(),
                description: "Fetch specific Netflix content metadata".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "platform_id": {"type": "string"}
                    },
                    "required": ["platform_id"]
                }),
            },
        ]
    }
}
```

**Aggregated in MCP Server:**
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mcp_server = MCPServer::new();

    // Register platform connectors
    mcp_server.register_tool_provider(Box::new(NetflixConnector::new()));
    mcp_server.register_tool_provider(Box::new(PrimeVideoConnector::new()));
    mcp_server.register_tool_provider(Box::new(DisneyPlusConnector::new()));
    // ... register all 10+ platforms

    // Start STDIO transport
    mcp_server.start_stdio().await?;

    // Alternatively, start SSE transport
    // mcp_server.start_sse("0.0.0.0:3000").await?;

    Ok(())
}
```

### 13.2 Claude Desktop Configuration

**`claude_desktop_config.json`:**
```json
{
  "mcpServers": {
    "agentics-hackathon": {
      "command": "npx",
      "args": ["agentics-hackathon", "mcp"]
    },
    "media-gateway": {
      "command": "/usr/local/bin/media-gateway",
      "args": ["mcp", "--transport", "stdio"]
    }
  }
}
```

**Installation:**
```bash
# Install hackathon-tv5 globally
npm install -g agentics-hackathon

# Build and install Media Gateway CLI
cd mg-cli
cargo build --release
sudo cp target/release/media-gateway /usr/local/bin/

# Restart Claude Desktop to load MCP servers
```

### 13.3 ARW Protocol Implementation

**Machine View Response Example:**
```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct ARWContent {
    #[serde(rename = "@context")]
    context: String,  // "https://agentics.org/arw/v1"

    id: String,       // "arw:content:tt1375666"
    title: String,
    content_type: String,  // "Movie"
    description: String,

    availability: ARWAvailability,
    embeddings: ARWEmbeddings,
    taxonomy: ARWTaxonomy,
}

#[derive(Serialize, Deserialize)]
struct ARWAvailability {
    platforms: Vec<ARWPlatform>,
}

#[derive(Serialize, Deserialize)]
struct ARWPlatform {
    name: String,
    region: String,
    pricing: ARWPricing,
    deep_link: String,
    qualities: Vec<String>,  // ["HD", "4K", "HDR"]
}

#[derive(Serialize, Deserialize)]
struct ARWPricing {
    model: String,    // "SVOD", "TVOD", "AVOD", "FREE"
    price: Option<f32>,
}

#[derive(Serialize, Deserialize)]
struct ARWEmbeddings {
    plot: Vec<f32>,   // 1536-dim
    mood: Vec<f32>,   // 256-dim
}

#[derive(Serialize, Deserialize)]
struct ARWTaxonomy {
    genres: Vec<String>,
    themes: Vec<String>,
}

// Convert internal representation to ARW format
fn to_arw_format(content: &Content) -> ARWContent {
    ARWContent {
        context: "https://agentics.org/arw/v1".to_string(),
        id: format!("arw:content:{}", content.id),
        title: content.title.clone(),
        content_type: content.content_type.clone(),
        description: content.description.clone(),

        availability: ARWAvailability {
            platforms: content.availability.iter().map(|avail| ARWPlatform {
                name: avail.platform.clone(),
                region: avail.region.clone(),
                pricing: ARWPricing {
                    model: avail.pricing_model.clone(),
                    price: avail.price,
                },
                deep_link: avail.deep_link.clone(),
                qualities: avail.qualities.clone(),
            }).collect(),
        },

        embeddings: ARWEmbeddings {
            plot: content.embeddings.plot.clone(),
            mood: content.embeddings.mood.clone(),
        },

        taxonomy: ARWTaxonomy {
            genres: content.genres.clone(),
            themes: content.themes.clone(),
        },
    }
}
```

**ARW Headers:**
```rust
use axum::{
    http::{Request, header::HeaderMap},
    middleware::Next,
    response::Response,
};

async fn arw_headers_middleware<B>(
    headers: HeaderMap,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    // Extract ARW headers
    let request_id = headers.get("AI-Request-ID")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    let agent_name = headers.get("AI-Agent-Name")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    let purpose = headers.get("AI-Purpose")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("discovery");

    let token_budget: u32 = headers.get("AI-Token-Budget")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
        .unwrap_or(50000);

    // Log agent interaction
    tracing::info!(
        request_id = request_id,
        agent_name = agent_name,
        purpose = purpose,
        token_budget = token_budget,
        "ARW agent request"
    );

    // Proceed with request
    next.run(request).await
}
```

### 13.4 Quick Start Workflow

**Initialize New Project:**
```bash
# Use hackathon-tv5 CLI
npx agentics-hackathon init

# Select Entertainment Discovery track
? Which track are you working on? › Entertainment Discovery

# Project scaffolded with:
# - package.json with Media Gateway dependencies
# - .env template
# - example configuration files
# - documentation links
```

**Install Integrated Tools:**
```bash
# Install Claude Flow (multi-agent orchestration)
npx agentics-hackathon tools --install claude-flow

# Install RuVector (hypergraph database)
npx agentics-hackathon tools --install ruvector

# Install AgentDB (memory layer)
npx agentics-hackathon tools --install agentdb

# Verify installations
npx agentics-hackathon tools --list
```

**Start MCP Servers:**
```bash
# Start hackathon-tv5 MCP server with SSE transport
npx agentics-hackathon mcp sse --port 3000

# In separate terminal, start Media Gateway MCP server
media-gateway mcp --transport stdio

# Check status
npx agentics-hackathon status
```

---

## 14. KEY IMPLEMENTATION INSIGHTS

### 14.1 Token Reduction via ARW

**Problem:** HTML scraping consumes excessive tokens (e.g., 50KB HTML → 12K tokens)

**Solution:** ARW machine views reduce payload size by 85%

**Example Comparison:**

**HTML Response (~12K tokens):**
```html
<!DOCTYPE html>
<html>
<head><title>Inception</title></head>
<body>
  <div class="header">...</div>
  <div class="content">
    <h1>Inception</h1>
    <p>A thief who steals corporate secrets through the use of dream-sharing technology...</p>
    <div class="cast">...</div>
    <div class="reviews">...</div>
    <!-- ... hundreds of lines of layout HTML ... -->
  </div>
</body>
</html>
```

**ARW JSON Response (~1.8K tokens = 85% reduction):**
```json
{
  "@context": "https://agentics.org/arw/v1",
  "id": "arw:content:tt1375666",
  "title": "Inception",
  "contentType": "Movie",
  "description": "A thief who steals corporate secrets through dream-sharing technology...",
  "availability": {
    "platforms": [
      {
        "name": "Netflix",
        "region": "US",
        "pricing": {"model": "SVOD"},
        "deepLink": "netflix://watch/70131314"
      }
    ]
  }
}
```

### 14.2 GraphSAGE Performance Characteristics

**Architecture:**
- **Layers:** 3 (512→256→128 neurons)
- **Attention:** 8→4→2 heads per layer
- **Parameters:** ~183K total
- **Memory:** ~45MB model size

**Inference Performance:**
- **P50 Latency:** 45ms
- **P99 Latency:** 95ms
- **Throughput:** ~22 inferences/second (single GPU)

**Training Strategy:**
- **Dataset:** User-content interactions (watch completion, ratings)
- **Loss Function:** Binary cross-entropy for edge prediction
- **Optimizer:** Adam with learning rate 0.001
- **Batch Size:** 256 subgraphs
- **Epochs:** 50 (early stopping with patience=5)

**Neighborhood Sampling:**
```python
# Pseudo-code for GraphSAGE sampling
def sample_neighbors(node_id, num_samples, layer):
    neighbors = graph.get_neighbors(node_id)

    if len(neighbors) <= num_samples:
        return neighbors
    else:
        # Importance sampling: prioritize recently interacted content
        weights = [recency_score(n) for n in neighbors]
        return weighted_sample(neighbors, weights, num_samples)

# Layer-wise sampling: 25 → 15 → 10
layer1_neighbors = sample_neighbors(user_id, 25, layer=1)
layer2_neighbors = [sample_neighbors(n, 15, layer=2) for n in layer1_neighbors]
layer3_neighbors = [[sample_neighbors(n, 10, layer=3) for n in layer2] for layer2 in layer2_neighbors]
```

### 14.3 SONA Attention Mechanism Selection

**39 Total Mechanisms Organized into 4 Categories:**

**Core Attention (12):**
- MultiHeadAttention
- FlashAttention (memory-efficient)
- LinearAttention (O(n) complexity)
- RoPE (Rotary Position Embedding)
- ALiBi (Attention with Linear Biases)
- WindowedAttention
- DilatedAttention
- ...

**Graph-Specific (10):**
- GraphRoPE (graph-aware position)
- EdgeFeaturedAttention
- GAT (Graph Attention Networks)
- GCN-based attention
- ...

**Specialized (9):**
- SparseAttention (long sequences)
- CrossAttention (multi-modal)
- Longformer
- BigBird
- ...

**Hyperbolic (8):**
- expMap (exponential map)
- mobiusAddition
- PoincaréAttention
- LorentzAttention (for hierarchies)
- ...

**Selection Logic (FastGRNN Classifier):**
```python
# Query type classification
query_embedding = encode_query(user_query)
query_type = fastgrnn_classifier.predict(query_embedding)

# Map to top-3 attention mechanisms
attention_map = {
    "similarity": ["GraphRoPE", "EdgeFeatured", "GAT"],
    "hierarchical": ["LorentzAttention", "PoincaréAttention", "expMap"],
    "cross_platform": ["CrossAttention", "LocalGlobal", "MultiHead"],
    "temporal": ["RoPE", "ALiBi", "WindowedAttention"],
    "long_context": ["Longformer", "BigBird", "LinearAttention"],
}

selected_mechanisms = attention_map[query_type]

# Weighted combination
output = sum(
    weight * mechanism(query, key, value)
    for mechanism, weight in zip(selected_mechanisms, [0.5, 0.3, 0.2])
)
```

### 14.4 Trust Score Decay Impact

**Example Calculation:**

Initial trust: 0.90 (high confidence)
Verification source: Platform API
Days since verification: 30

```
decay_rate = 0.01 per day
decayed_trust = 0.90 * (1 - 0.01 * 30) = 0.90 * 0.70 = 0.63
```

After 30 days, trust falls from 0.90 → 0.63 (still above 0.6 threshold)

After 60 days: 0.90 * (1 - 0.01 * 60) = 0.90 * 0.40 = 0.36 (below threshold, filtered)

**Mitigation Strategy:**
- Periodic re-verification (weekly for high-value content)
- Fallback to cached data with warnings
- User feedback loop (report broken links → trigger immediate re-verification)

### 14.5 E2B Sandbox Warm Pool Economics

**Monthly Usage Estimate:**
- Daily executions: 3,800
- Average duration: 15 seconds
- Monthly hours: (3,800 * 30 * 15) / 3600 ≈ 475 hours

**Warm Pool Benefit:**
- Cold start: ~150ms
- Warm start: ~5ms
- Time saved per execution: 145ms
- Monthly time saved: 3,800 * 30 * 145ms ≈ 4.9 hours of user-facing latency

**Cost-Benefit:**
- Warm pool cost: +$100-150/month (10-15 idle instances)
- User experience gain: Eliminate 145ms latency on 114K requests
- Decision: **Worthwhile trade-off** for user-facing operations

---

## 15. PERFORMANCE BENCHMARKS AND TARGETS

### 15.1 Latency Targets by Operation

| Operation | P50 Target | P99 Target | Notes |
|-----------|-----------|-----------|-------|
| Search query (full) | 250ms | 500ms | End-to-end including agent orchestration |
| Recommendation generation | 150ms | 300ms | Hybrid RRF with GNN |
| LoRA adapter loading | 5ms | 15ms | From Memorystore cache |
| Semantic routing | 0.5ms | 2ms | FastGRNN classification |
| Pattern retrieval (ReasoningBank) | 10ms | 25ms | HNSW vector search |
| Availability check | 80ms | 150ms | Parallel platform queries |
| Deep-link validation | 200ms | 400ms | HTTP HEAD request with timeout |
| CRDT sync propagation | 50ms | 100ms | PubNub message delivery |
| Multi-agent SPARC cycle | 430ms | 800ms | Specification → Completion |

### 15.2 Throughput Targets

| Service | Target QPS | Max QPS | Scaling Strategy |
|---------|-----------|---------|------------------|
| API Gateway | 1,000 | 5,000 | Cloud Run autoscaling (3-100 instances) |
| Recommendation Engine | 200 | 800 | GKE Autopilot (5-20 replicas) |
| MCP Connectors (per platform) | 10 | 50 | Rate limit + backoff |
| Semantic Search | 500 | 2,000 | Horizontal scaling (4-16 replicas) |
| Database Reads | 10,000 | 40,000 | Connection pooling + caching |
| Database Writes | 1,000 | 5,000 | Batch inserts, async writes |

### 15.3 Resource Efficiency Metrics

**GNN Inference:**
- CPU overhead: <5% on inference nodes
- GPU utilization: 60-80% during peak
- Memory per model: 45MB
- Concurrent inferences: 22/second/GPU

**LoRA Adapters:**
- Memory per adapter: 2-8MB
- Hot adapters in cache: 1M users = ~10GB Memorystore
- Cache hit rate: >95% for active users

**Pattern Search (ReasoningBank):**
- HNSW index size: ~2GB for 1M patterns
- Search latency: 10ms P50
- Index rebuild: Weekly offline process

### 15.4 Quality Metrics

**Recommendation Quality:**
- Precision@10: 0.31 (19% improvement with SONA)
- Recall@20: 0.45
- Cold-start accuracy: 47% improvement after 5 interactions
- Pattern match rate: 35% in mature deployments

**Availability Accuracy:**
- Deep-link success rate: >90%
- Trust score accuracy: 85% (validated against manual checks)
- False positive rate (content not actually available): <5%

**User Engagement:**
- Click-through rate (CTR): 25%
- Watch completion rate: 60%
- Cross-device sync success: 98%

---

## CONCLUSION

This implementation guidance summary provides comprehensive technical specifications extracted from the media-gateway-hackathon-research repository. The system architecture balances:

1. **Scale:** 51 decoupled micro-repositories with independent versioning
2. **Privacy:** Three-tier data handling with federated learning and differential privacy
3. **Intelligence:** Multi-agent orchestration (SPARC methodology) with hybrid recommendations
4. **Performance:** Sub-500ms search latency with 85% token reduction via ARW protocol
5. **Security:** OAuth 2.1 compliance, E2B sandbox isolation, sender-constrained tokens
6. **Integration:** Aggregator-based platform access (JustWatch, Watchmode) with deep-linking

**Core Algorithms:**
- Reciprocal Rank Fusion (RRF) for recommendation blending
- GraphSAGE 3-layer GNN for relationship-aware recommendations
- FastGRNN semantic routing (<5ms query classification)
- Two-Tier LoRA for runtime personalization without retraining
- HNSW vector indexing for semantic search
- HLC-based CRDT for conflict-free cross-device synchronization

**Key Data Structures:**
- Ruvector hypergraph (content, users, genres with multi-dimensional embeddings)
- OR-Sets and LWW registers for CRDT synchronization
- LoRA adapters (2-8MB per user, 30-day TTL)
- ReasoningBank patterns (HNSW-indexed, 10ms retrieval)

**Integration Patterns:**
- OAuth 2.0 + PKCE (web/mobile)
- Device Authorization Grant RFC 8628 (TV/CLI)
- Deep linking (iOS Universal Links, Android App Links)
- E2B sandboxes for secure agent code execution
- PubNub real-time sync with CRDT merge semantics

**Performance Optimizations:**
- Multi-tier caching (process → Memorystore → Cloud SQL)
- Connection pooling and prepared statements
- Async I/O with Tokio parallelization
- gRPC compression and PubNub delta encoding
- Warm E2B sandbox pools (eliminate 150ms cold starts)
- Rate limiting and circuit breakers for graceful degradation

This research provides the foundation for pseudocode design in the SPARC methodology, with concrete algorithms, data structures, and implementation patterns ready for adaptation into the Media Gateway system.

---

**Sources:**
- [GitHub Repository](https://github.com/globalbusinessadvisors/media-gateway-hackathon-research)
- [README.md](https://github.com/globalbusinessadvisors/media-gateway-hackathon-research/blob/main/README.md)
- [FINAL_ARCHITECTURE_BLUEPRINT.md](https://raw.githubusercontent.com/globalbusinessadvisors/media-gateway-hackathon-research/main/research/FINAL_ARCHITECTURE_BLUEPRINT.md)
- [SONA_INTEGRATION_SPECIFICATION.md](https://raw.githubusercontent.com/globalbusinessadvisors/media-gateway-hackathon-research/main/research/SONA_INTEGRATION_SPECIFICATION.md)
- [streaming-platform-research.md](https://raw.githubusercontent.com/globalbusinessadvisors/media-gateway-hackathon-research/main/research/streaming-platform-research.md)
- [E2B_SANDBOX_INTEGRATION.md](https://raw.githubusercontent.com/globalbusinessadvisors/media-gateway-hackathon-research/main/research/E2B_SANDBOX_INTEGRATION.md)
- [HACKATHON_TV5_INTEGRATION.md](https://raw.githubusercontent.com/globalbusinessadvisors/media-gateway-hackathon-research/main/research/HACKATHON_TV5_INTEGRATION.md)
