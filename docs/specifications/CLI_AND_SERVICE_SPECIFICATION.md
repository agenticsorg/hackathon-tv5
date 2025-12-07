# Media Gateway - CLI and Service Specification

**Version:** 1.0.0
**Date:** 2025-12-06
**Status:** Draft
**Research Sources:**
- https://github.com/agenticsorg/hackathon-tv5
- https://github.com/globalbusinessadvisors/media-gateway-research

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [CLI Behavior Specifications](#cli-behavior-specifications)
3. [Service Expectations](#service-expectations)
4. [Agent Orchestration Goals](#agent-orchestration-goals)
5. [API Surface](#api-surface)
6. [Technical Architecture](#technical-architecture)
7. [Integration Requirements](#integration-requirements)

---

## 1. Executive Summary

The Media Gateway is a unified TV content discovery platform designed to aggregate 10+ streaming services (Netflix, Prime Video, Disney+, Hulu, Apple TV+, YouTube, Crave, HBO Max, etc.) into a single intelligent interface. Built on Agentics Foundation with Agent-Ready Web (ARW) protocol integration, it addresses the critical user problem: **45 minutes average decision time** selecting content across fragmented streaming platforms.

**Core Value Proposition:** Reduce content discovery friction through AI-powered semantic search, multi-agent orchestration, and intelligent recommendation systems.

**Technology Stack:**
- **Primary Language:** Rust (100% for core services)
- **Foundation:** hackathon-tv5 toolkit (TypeScript-based CLI/tooling)
- **Intelligence:** SONA (Self-Optimizing Neural Architecture)
- **Protocol:** ARW (Agent-Ready Web) with 85% token reduction vs HTML scraping
- **Infrastructure:** GCP (GKE Autopilot, Cloud Run, Cloud SQL, Memorystore)

---

## 2. CLI Behavior Specifications

### 2.1 Command Structure and Hierarchy

The Media Gateway CLI follows a hierarchical command structure with two primary interfaces:

#### 2.1.1 Developer/Platform CLI (Rust-based TUI)

**Purpose:** Platform operators and infrastructure management

```bash
media-gateway [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS] [ARGS]
```

**Global Options:**
```
--config <PATH>          Custom configuration file path
--env <ENVIRONMENT>      Target environment (dev|staging|prod)
--verbose, -v            Enable verbose logging
--quiet, -q              Suppress non-error output
--json                   Output in JSON format
--help, -h               Display help information
--version, -V            Display version information
```

**Command Categories:**

##### A. Initialization & Setup
```bash
media-gateway init [OPTIONS]
  --interactive, -i      Interactive project setup (default)
  --track <TRACK>        Competition track (entertainment|multi-agent|workflow|open)
  --tools                Install recommended toolchain
  --skip-deps            Skip dependency installation
  --template <NAME>      Use project template

# Example:
media-gateway init --track multi-agent --tools
```

##### B. Service Management
```bash
media-gateway service <SUBCOMMAND>
  start [SERVICE...]     Start specified services (or all if none specified)
  stop [SERVICE...]      Stop specified services
  restart [SERVICE...]   Restart specified services
  status [SERVICE...]    Display service health status
  logs [SERVICE...]      Stream service logs
  scale <SERVICE> <N>    Scale service to N replicas

# Service identifiers:
#   - mcp-server          Model Context Protocol server
#   - api-gateway         REST/GraphQL API gateway
#   - metadata-service    Global metadata consolidation
#   - auth-service        OAuth2/PKCE authentication
#   - search-service      Semantic search engine
#   - recommendation      SONA-powered recommendation engine
#   - normalizer-*        Platform-specific normalizers (netflix, prime, disney, etc.)
#   - orchestrator        Multi-agent orchestration layer

# Examples:
media-gateway service start mcp-server api-gateway
media-gateway service status
media-gateway service logs search-service --follow --tail 100
media-gateway service scale recommendation 3
```

##### C. Stream Management
```bash
media-gateway stream <SUBCOMMAND>
  list [OPTIONS]         List available content streams
    --platform <NAME>    Filter by platform (netflix|prime|disney|hulu|apple|...)
    --category <CAT>     Filter by category (movie|series|documentary|...)
    --available          Show only currently available content

  search <QUERY>         Semantic search across platforms
    --platforms <LIST>   Comma-separated platform filter
    --threshold <FLOAT>  Similarity threshold (0.0-1.0, default: 0.75)
    --limit <N>          Max results (default: 20)
    --format <FMT>       Output format (table|json|compact)

  cache <SUBCOMMAND>     Manage content cache
    refresh [PLATFORM]   Refresh platform metadata
    clear [PLATFORM]     Clear cache entries
    stats                Display cache statistics

# Examples:
media-gateway stream list --platform netflix --category movie
media-gateway stream search "sci-fi thriller" --platforms netflix,prime,hulu --limit 10
media-gateway stream cache refresh netflix
media-gateway stream cache stats
```

##### D. Monitoring & Status
```bash
media-gateway status [OPTIONS]
  --services             Show service health matrix
  --agents               Show active agent status
  --metrics              Display performance metrics
  --integrations         Show platform integration status
  --all                  Show comprehensive status

media-gateway health [SERVICE]
  # Returns health check status with exit codes:
  # 0 = healthy, 1 = degraded, 2 = unhealthy, 3 = unknown

media-gateway metrics <SUBCOMMAND>
  export [FILE]          Export metrics to file
  dashboard              Launch metrics dashboard (TUI)
  query <METRIC>         Query specific metric

# Examples:
media-gateway status --all --json
media-gateway health mcp-server
media-gateway metrics dashboard
```

##### E. Configuration Management
```bash
media-gateway config <SUBCOMMAND>
  get <KEY>              Retrieve configuration value
  set <KEY> <VALUE>      Update configuration value
  list [SECTION]         List configuration values
  validate [FILE]        Validate configuration file
  export [FILE]          Export current configuration
  import <FILE>          Import configuration

# Configuration sections:
#   - services           Service-specific settings
#   - platforms          Streaming platform credentials/endpoints
#   - agents             Agent orchestration parameters
#   - mcp                Model Context Protocol settings
#   - intelligence       SONA/recommendation engine tuning
#   - cache              Caching strategies
#   - security           Authentication/authorization

# Examples:
media-gateway config get services.mcp-server.port
media-gateway config set platforms.netflix.enabled true
media-gateway config list platforms
media-gateway config validate ./custom-config.toml
```

##### F. Agent Operations
```bash
media-gateway agent <SUBCOMMAND>
  list [OPTIONS]         List active agents
    --type <TYPE>        Filter by agent type
    --status <STATUS>    Filter by status (active|idle|error)

  spawn <TYPE> [ARGS]    Spawn new agent instance
  terminate <AGENT_ID>   Terminate specific agent
  inspect <AGENT_ID>     Display agent details

  orchestrate <TASK>     Orchestrate multi-agent task
    --topology <TYPE>    Topology (hierarchical|mesh|adaptive)
    --max-agents <N>     Maximum agents to spawn
    --timeout <SEC>      Task timeout in seconds

# Agent types:
#   - search-agent       Content discovery
#   - recommendation     Personalization
#   - metadata-resolver  Entity resolution
#   - auth-manager       Platform authentication
#   - cache-optimizer    Cache strategy tuning
#   - anomaly-detector   Usage pattern analysis

# Examples:
media-gateway agent list --status active
media-gateway agent spawn search-agent --platform netflix
media-gateway agent orchestrate "find action movies under 2 hours" --topology mesh
```

##### G. Tools & Utilities
```bash
media-gateway tools <SUBCOMMAND>
  list [CATEGORY]        List available tools
    # Categories: ai-assistants, orchestration, cloud, databases, frameworks, advanced

  install <TOOL>         Install specific tool
  uninstall <TOOL>       Uninstall tool
  update [TOOL]          Update tool(s)
  info <TOOL>            Display tool information

# Available tools (17+ from hackathon-tv5):
#   AI Assistants: cursor, windsurf, cline, etc.
#   Orchestration: claude-flow, agentic-flow, google-adk
#   Cloud: e2b-sandbox, firecracker
#   Databases: ruvector, postgresql, memorystore
#   Frameworks: next.js, react-native, rust-axum
#   Advanced: sona-engine, arw-protocol

# Examples:
media-gateway tools list orchestration
media-gateway tools install claude-flow
media-gateway tools info sona-engine
```

#### 2.1.2 Foundation CLI (hackathon-tv5 wrapper)

**Purpose:** Development setup and MCP server management

```bash
npx agentics-hackathon <COMMAND> [OPTIONS]
```

**Commands:**
```bash
init                     Interactive project setup with track selection
  # Prompts for:
  # - Competition track (entertainment|multi-agent|workflow|open)
  # - Required tools installation
  # - Platform credentials configuration
  # - Development environment setup

tools                    Browse and install 17+ development tools
  # Categories:
  # - AI Assistants (Cursor, Windsurf, Cline, Aide, Aider, Continue)
  # - Orchestration (Claude Flow, Agentic Flow, Google ADK)
  # - Cloud (E2B Sandboxes, Firecracker)
  # - Databases (Ruvector, PostgreSQL, Memorystore)
  # - Frameworks (Next.js, React Native, Tizen/WebOS)
  # - Advanced (SONA Engine, ARW Protocol)

status                   View project configuration and installed tools
  # Displays:
  # - Selected track
  # - Installed tools and versions
  # - Platform integration status
  # - Configuration health

mcp [TRANSPORT]          Launch Model Context Protocol server
  # Transports:
  #   stdio              Standard I/O (default, for Claude Desktop)
  #   sse [--port PORT]  Server-Sent Events (web clients, default: 3000)

  # Examples:
  npx agentics-hackathon mcp              # STDIO mode
  npx agentics-hackathon mcp sse --port 8080

info                     Access hackathon resources and documentation
  # Returns:
  # - Event details and timelines
  # - Track descriptions
  # - Prize information
  # - Educational resources

discord                  Connect to community hub
  # Opens Discord invite or displays community link

help [COMMAND]           Display help information
```

### 2.2 Interactive vs Non-Interactive Modes

#### 2.2.1 Interactive Mode Features

**When Invoked:**
- No arguments provided (e.g., `media-gateway` or `media-gateway init`)
- Explicit `--interactive` flag
- Configuration requires user decision (e.g., platform credentials)

**Interactive Capabilities:**
```bash
# Interactive initialization
$ media-gateway init

┌─────────────────────────────────────────────────┐
│   Media Gateway - Platform Initialization      │
└─────────────────────────────────────────────────┘

? Select your primary use case:
  ❯ Content Discovery Platform
    Multi-Agent Research
    Workflow Automation
    Custom Integration

? Which streaming platforms do you want to integrate?
  ◉ Netflix
  ◉ Prime Video
  ◉ Disney+
  ◯ Hulu
  ◉ Apple TV+
  ◯ YouTube
  (Use arrow keys and space to select)

? Configure authentication method:
  ❯ OAuth2 with PKCE (recommended)
    API Key
    Manual configuration

? Enable SONA intelligence engine? (Y/n)
```

**Interactive Service Management:**
```bash
$ media-gateway service

Services Status:
┌─────────────────────┬─────────┬──────────┬─────────┐
│ Service             │ Status  │ Replicas │ Uptime  │
├─────────────────────┼─────────┼──────────┼─────────┤
│ mcp-server          │ Running │ 1/1      │ 2h 15m  │
│ api-gateway         │ Running │ 2/2      │ 2h 15m  │
│ search-service      │ Degraded│ 1/2      │ 1h 03m  │
│ recommendation      │ Stopped │ 0/1      │ -       │
└─────────────────────┴─────────┴──────────┴─────────┘

? What would you like to do?
  ❯ Start a service
    Restart a service
    View logs
    Scale a service
    Exit
```

#### 2.2.2 Non-Interactive Mode Features

**Design Principles:**
- All operations support non-interactive execution
- Exit codes follow POSIX conventions (0=success, 1+=error)
- Machine-readable output formats (JSON, CSV, plain)
- Suitable for CI/CD pipelines and automation

**Examples:**
```bash
# Script-friendly initialization
media-gateway init \
  --track multi-agent \
  --platforms netflix,prime,disney \
  --auth oauth2 \
  --enable-sona \
  --no-interactive

# Automated service management
media-gateway service start mcp-server api-gateway search-service
EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
  echo "Service startup failed"
  exit $EXIT_CODE
fi

# JSON output for parsing
media-gateway status --json --services | jq '.services[] | select(.status == "unhealthy")'

# Health checks in monitoring scripts
media-gateway health api-gateway
if [ $? -eq 0 ]; then
  echo "API Gateway is healthy"
else
  # Trigger alert
fi
```

### 2.3 Configuration File Structure

**Primary Configuration:** `media-gateway.toml` (TOML format)

```toml
[metadata]
version = "1.0.0"
environment = "production"
track = "multi-agent"

[services.mcp-server]
enabled = true
port = 3000
transport = "stdio"  # or "sse"
tools = ["get_hackathon_info", "get_tracks", "get_available_tools",
         "get_project_status", "check_tool_installed", "get_resources"]

[services.api-gateway]
enabled = true
port = 8080
protocol = "https"
endpoints = ["/api/v1/search", "/api/v1/metadata", "/api/v1/recommendations"]
rate_limit = 1000  # requests per minute

[services.search-service]
enabled = true
port = 8081
engine = "sona"
index_type = "ruvector"  # hypergraph + vector + GNN
similarity_threshold = 0.75
max_results = 20

[services.recommendation]
enabled = true
port = 8082
engine = "sona"
personalization = true
lora_tier = "two-tier"  # ~10KB per user
attention_mechanisms = 39
latency_target_ms = 5

[platforms.netflix]
enabled = true
auth_method = "oauth2"
client_id = "${NETFLIX_CLIENT_ID}"
client_secret = "${NETFLIX_CLIENT_SECRET}"
normalizer = "netflix-normalizer"
cache_ttl_hours = 6

[platforms.prime-video]
enabled = true
auth_method = "oauth2"
client_id = "${PRIME_CLIENT_ID}"
client_secret = "${PRIME_CLIENT_SECRET}"
normalizer = "prime-normalizer"
cache_ttl_hours = 6

[platforms.disney-plus]
enabled = true
auth_method = "oauth2"
client_id = "${DISNEY_CLIENT_ID}"
client_secret = "${DISNEY_CLIENT_SECRET}"
normalizer = "disney-normalizer"
cache_ttl_hours = 6

[agents]
max_concurrent = 10
topology = "hierarchical"  # hierarchical|mesh|adaptive
coordination_namespace = "aqe/*"
spawn_timeout_sec = 30
task_timeout_sec = 300

[agents.types]
search-agent = { enabled = true, max_instances = 5 }
recommendation-agent = { enabled = true, max_instances = 3 }
metadata-resolver = { enabled = true, max_instances = 2 }
auth-manager = { enabled = true, max_instances = 1 }
cache-optimizer = { enabled = true, max_instances = 1 }

[intelligence.sona]
enabled = true
attention_mechanisms = [
  "core", "graph", "specialized", "hyperbolic"
]
lora_config = { tier = "two-tier", size_per_user_kb = 10 }
reasoning_bank = { enabled = true, cache_successful_patterns = true }
ewc_enabled = true  # Elastic Weight Consolidation
latency_target_ms = 5

[cache]
backend = "memorystore"  # Valkey (Redis-compatible)
ttl_default_hours = 6
max_size_gb = 10
eviction_policy = "lru"

[cache.strategies]
metadata = { ttl_hours = 24, priority = "high" }
search_results = { ttl_hours = 1, priority = "medium" }
recommendations = { ttl_hours = 6, priority = "medium" }

[security]
auth_required = true
oauth2_pkce = true
tls_enabled = true
cert_path = "/etc/media-gateway/certs/server.crt"
key_path = "/etc/media-gateway/certs/server.key"
secret_manager = "gcp-secret-manager"

[logging]
level = "info"  # trace|debug|info|warn|error
format = "json"
output = "stdout"

[logging.sinks]
console = { enabled = true, level = "info" }
file = { enabled = true, path = "/var/log/media-gateway/app.log", level = "debug" }
cloud = { enabled = true, provider = "gcp-logging", level = "warn" }

[metrics]
enabled = true
exporter = "prometheus"
port = 9090
interval_sec = 15

[deployment.gcp]
project_id = "${GCP_PROJECT_ID}"
region = "us-central1"
kubernetes_cluster = "media-gateway-prod"
use_autopilot = true
estimated_monthly_cost_usd = [2400, 3650]

[deployment.gcp.services]
api_gateway = { type = "cloud-run", min_instances = 1, max_instances = 10 }
database = { type = "cloud-sql", tier = "postgresql-15-ha", storage_gb = 100 }
cache = { type = "memorystore", tier = "valkey", size_gb = 5 }
```

### 2.4 Environment Variables

**Required:**
```bash
# Platform Authentication
NETFLIX_CLIENT_ID=<oauth2-client-id>
NETFLIX_CLIENT_SECRET=<oauth2-client-secret>
PRIME_CLIENT_ID=<oauth2-client-id>
PRIME_CLIENT_SECRET=<oauth2-client-secret>
DISNEY_CLIENT_ID=<oauth2-client-id>
DISNEY_CLIENT_SECRET=<oauth2-client-secret>
HULU_CLIENT_ID=<oauth2-client-id>
HULU_CLIENT_SECRET=<oauth2-client-secret>

# GCP Infrastructure
GCP_PROJECT_ID=<project-id>
GCP_SERVICE_ACCOUNT_KEY=<path-to-key.json>

# Security
JWT_SECRET=<secret-key>
ENCRYPTION_KEY=<encryption-key>
```

**Optional:**
```bash
# MCP Configuration
MCP_TRANSPORT=stdio  # stdio|sse
MCP_PORT=3000

# Feature Flags
ENABLE_SONA=true
ENABLE_ARW_PROTOCOL=true
ENABLE_E2B_SANDBOX=false

# Performance Tuning
MAX_CONCURRENT_AGENTS=10
CACHE_TTL_HOURS=6
SIMILARITY_THRESHOLD=0.75

# Logging
LOG_LEVEL=info
LOG_FORMAT=json
```

### 2.5 Exit Codes

**Standard Exit Codes:**
```
0   Success
1   General error
2   Misuse of shell command (invalid arguments)
3   Configuration error
4   Service startup failure
5   Service runtime error
6   Agent orchestration failure
7   Platform integration error
8   Authentication/authorization failure
9   Network/connectivity error
10  Database/cache error
11  Resource exhaustion (memory, CPU, disk)
12  Timeout
13  Dependency missing or incompatible
```

---

## 3. Service Expectations

### 3.1 Core Services and Responsibilities

The Media Gateway implements a **4-layer microservices architecture** with 51 micro-repositories:

#### Layer 1: Infrastructure Services (Micro-repositories)

##### A. MCP Server (`mcp-server`)
**Responsibility:** Model Context Protocol interface for AI agent interaction

**Contract:**
```rust
pub trait McpServer {
    // Tools: Executable functions exposed to AI clients
    async fn get_hackathon_info(&self) -> Result<HackathonInfo>;
    async fn get_tracks(&self) -> Result<Vec<Track>>;
    async fn get_available_tools(&self) -> Result<Vec<ToolInfo>>;
    async fn get_project_status(&self) -> Result<ProjectStatus>;
    async fn check_tool_installed(&self, tool_name: &str) -> Result<bool>;
    async fn get_resources(&self) -> Result<Resources>;

    // Resources: Configuration and metadata access
    async fn list_resources(&self) -> Result<Vec<ResourceDescriptor>>;
    async fn read_resource(&self, uri: &str) -> Result<ResourceContent>;

    // Prompts: Guidance templates
    async fn list_prompts(&self) -> Result<Vec<PromptTemplate>>;
    async fn get_prompt(&self, name: &str) -> Result<PromptTemplate>;
}

pub struct McpServerConfig {
    transport: Transport,  // Stdio | Sse
    port: Option<u16>,     // Required for SSE
    tools_enabled: Vec<String>,
    resources_path: PathBuf,
    prompts_path: PathBuf,
}
```

**Health Metrics:**
- Uptime > 99.9%
- Request latency < 50ms (p99)
- Transport reliability (stdio: lossless, sse: auto-reconnect)

##### B. Platform Normalizers (`netflix-normalizer`, `prime-normalizer`, etc.)
**Responsibility:** Transform platform-specific APIs into unified schema

**Contract:**
```rust
pub trait PlatformNormalizer {
    async fn authenticate(&self, credentials: Credentials) -> Result<AuthToken>;
    async fn fetch_catalog(&self) -> Result<Vec<ContentItem>>;
    async fn search(&self, query: &str) -> Result<Vec<ContentItem>>;
    async fn get_metadata(&self, content_id: &str) -> Result<ContentMetadata>;
    async fn check_availability(&self, content_id: &str) -> Result<Availability>;
}

pub struct ContentItem {
    id: String,
    platform: Platform,
    title: String,
    category: Category,  // Movie | Series | Documentary | ...
    release_year: u16,
    duration_minutes: Option<u32>,
    genres: Vec<String>,
    rating: Option<f32>,
    description: String,
    thumbnail_url: String,
    availability: Availability,
}

pub struct Availability {
    is_available: bool,
    regions: Vec<String>,
    subscription_required: bool,
    rent_price: Option<Money>,
    buy_price: Option<Money>,
}
```

**Performance Requirements:**
- Platform API request rate limits respected
- Cache-first strategy (TTL: 6 hours default)
- Retry logic with exponential backoff
- Circuit breaker on platform outages

##### C. Entity Resolver (`entity-resolver`)
**Responsibility:** Deduplicate and merge content across platforms

**Contract:**
```rust
pub trait EntityResolver {
    async fn resolve(&self, items: Vec<ContentItem>) -> Result<Vec<UnifiedEntity>>;
    async fn find_duplicates(&self, item: &ContentItem) -> Result<Vec<ContentItem>>;
    async fn merge_metadata(&self, items: Vec<ContentItem>) -> Result<UnifiedEntity>;
}

pub struct UnifiedEntity {
    canonical_id: String,
    titles: HashMap<Platform, String>,  // Platform-specific titles
    platforms: Vec<Platform>,
    merged_metadata: ContentMetadata,
    availability_by_platform: HashMap<Platform, Availability>,
    confidence_score: f32,  // Deduplication confidence
}
```

**Quality Metrics:**
- Deduplication accuracy > 95%
- False positive rate < 2%
- Processing latency < 100ms per entity

##### D. Authentication Service (`auth-service`)
**Responsibility:** OAuth2/PKCE flows for platform credentials

**Contract:**
```rust
pub trait AuthService {
    async fn initiate_oauth_flow(&self, platform: Platform) -> Result<OAuthFlowState>;
    async fn handle_callback(&self, state: &str, code: &str) -> Result<AuthToken>;
    async fn refresh_token(&self, refresh_token: &str) -> Result<AuthToken>;
    async fn revoke_token(&self, token: &str) -> Result<()>;
    async fn validate_token(&self, token: &str) -> Result<TokenInfo>;
}

pub struct OAuthFlowState {
    state: String,          // CSRF token
    authorization_url: String,
    code_verifier: String,  // PKCE
    expires_at: DateTime<Utc>,
}

pub struct AuthToken {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
    token_type: String,
    platform: Platform,
}
```

**Security Requirements:**
- PKCE mandatory for public clients
- State parameter validation (CSRF protection)
- Token encryption at rest (GCP Secret Manager)
- Token rotation before expiry
- Audit logging of all auth events

#### Layer 2: Intelligence Services

##### E. Semantic Search Service (`search-service`)
**Responsibility:** AI-powered content discovery with SONA integration

**Contract:**
```rust
pub trait SearchService {
    async fn semantic_search(&self, query: &str, options: SearchOptions) -> Result<SearchResults>;
    async fn index_content(&self, items: Vec<UnifiedEntity>) -> Result<IndexStats>;
    async fn update_index(&self, entity_id: &str, metadata: ContentMetadata) -> Result<()>;
    async fn delete_from_index(&self, entity_id: &str) -> Result<()>;
}

pub struct SearchOptions {
    platforms: Option<Vec<Platform>>,
    categories: Option<Vec<Category>>,
    similarity_threshold: f32,  // 0.0-1.0, default: 0.75
    max_results: usize,         // default: 20
    personalized: bool,         // Use user history
    user_id: Option<String>,
}

pub struct SearchResults {
    results: Vec<SearchResult>,
    query_embedding: Vec<f32>,
    processing_time_ms: u64,
    total_candidates: usize,
}

pub struct SearchResult {
    entity: UnifiedEntity,
    similarity_score: f32,
    explanation: Option<String>,  // Why this was matched
    ranking_factors: RankingFactors,
}
```

**Performance Requirements:**
- Query latency < 200ms (p95)
- Throughput > 1000 queries/sec
- Index update lag < 5 minutes
- Similarity accuracy > 90% (human eval)

**SONA Integration:**
- 39 attention mechanisms (Core, Graph, Specialized, Hyperbolic)
- Two-Tier LoRA for personalization (~10KB per user)
- ReasoningBank for pattern caching
- EWC++ for continuous learning without forgetting

##### F. Recommendation Engine (`recommendation-service`)
**Responsibility:** Personalized content suggestions

**Contract:**
```rust
pub trait RecommendationService {
    async fn get_recommendations(&self, user_id: &str, options: RecoOptions) -> Result<Vec<Recommendation>>;
    async fn track_interaction(&self, user_id: &str, event: InteractionEvent) -> Result<()>;
    async fn update_user_profile(&self, user_id: &str, profile: UserProfile) -> Result<()>;
    async fn explain_recommendation(&self, recommendation_id: &str) -> Result<Explanation>;
}

pub struct RecoOptions {
    platforms: Option<Vec<Platform>>,
    categories: Option<Vec<Category>>,
    count: usize,           // default: 10
    diversity_weight: f32,  // 0.0-1.0, default: 0.3
    recency_bias: f32,      // 0.0-1.0, default: 0.5
}

pub struct Recommendation {
    id: String,
    entity: UnifiedEntity,
    score: f32,
    reasoning: Vec<ReasoningFactor>,
    freshness: Freshness,  // New | Trending | Classic
}

pub struct InteractionEvent {
    user_id: String,
    entity_id: String,
    event_type: EventType,  // View | WatchStart | WatchComplete | Rating | Save | Share
    timestamp: DateTime<Utc>,
    context: HashMap<String, String>,
}
```

**Intelligence Features:**
- Collaborative filtering (user-user, item-item)
- Content-based filtering (metadata + embeddings)
- Hybrid ensemble model
- Cold-start handling (popularity + metadata)
- A/B testing framework for model experiments

##### G. Multi-Agent Orchestrator (`orchestrator`)
**Responsibility:** Coordinate specialized agents for complex tasks

**Contract:**
```rust
pub trait AgentOrchestrator {
    async fn spawn_agent(&self, agent_type: AgentType) -> Result<AgentId>;
    async fn terminate_agent(&self, agent_id: AgentId) -> Result<()>;
    async fn orchestrate_task(&self, task: Task, topology: Topology) -> Result<TaskResult>;
    async fn get_agent_status(&self, agent_id: AgentId) -> Result<AgentStatus>;
    async fn list_active_agents(&self) -> Result<Vec<AgentInfo>>;
}

pub enum AgentType {
    SearchAgent,
    RecommendationAgent,
    MetadataResolver,
    AuthManager,
    CacheOptimizer,
    AnomalyDetector,
}

pub enum Topology {
    Hierarchical { max_depth: u8 },
    Mesh { max_agents: u8 },
    Adaptive,  // Auto-select based on task complexity
}

pub struct Task {
    id: String,
    description: String,
    parameters: HashMap<String, Value>,
    timeout_sec: u64,
    priority: Priority,
}

pub struct TaskResult {
    task_id: String,
    status: TaskStatus,  // Completed | Failed | Timeout | Cancelled
    result: Option<Value>,
    error: Option<String>,
    agents_used: Vec<AgentId>,
    execution_time_ms: u64,
}
```

**Coordination Patterns:**
- Memory namespace: `aqe/*` for cross-agent state
- Pre-task hooks: Agent assignment, resource validation
- Post-task hooks: Result aggregation, metrics collection
- Session management: Context persistence across tasks

#### Layer 3: Consolidation Services

##### H. Global Metadata Service (`metadata-service`)
**Responsibility:** Unified metadata repository

**Contract:**
```rust
pub trait MetadataService {
    async fn get_metadata(&self, entity_id: &str) -> Result<GlobalMetadata>;
    async fn update_metadata(&self, entity_id: &str, metadata: GlobalMetadata) -> Result<()>;
    async fn search_metadata(&self, filters: MetadataFilters) -> Result<Vec<GlobalMetadata>>;
    async fn enrich_metadata(&self, entity_id: &str, source: MetadataSource) -> Result<()>;
}

pub struct GlobalMetadata {
    entity_id: String,
    imdb_id: Option<String>,
    tmdb_id: Option<String>,
    canonical_title: String,
    international_titles: HashMap<String, String>,
    cast: Vec<Person>,
    crew: Vec<Person>,
    genres: Vec<String>,
    themes: Vec<String>,
    ratings: HashMap<RatingSource, f32>,
    reviews: Vec<Review>,
    awards: Vec<Award>,
    metadata_quality_score: f32,
    last_updated: DateTime<Utc>,
}
```

##### I. Availability Index (`availability-index`)
**Responsibility:** Real-time content availability tracking

**Contract:**
```rust
pub trait AvailabilityIndex {
    async fn check_availability(&self, entity_id: &str, region: &str) -> Result<Vec<PlatformAvailability>>;
    async fn update_availability(&self, updates: Vec<AvailabilityUpdate>) -> Result<()>;
    async fn subscribe_to_updates(&self, entity_id: &str) -> Result<UpdateStream>;
}

pub struct PlatformAvailability {
    platform: Platform,
    is_available: bool,
    subscription_required: bool,
    pricing: Option<Pricing>,
    last_verified: DateTime<Utc>,
}
```

##### J. Rights Engine (`rights-engine`)
**Responsibility:** Content licensing and regional restrictions

**Contract:**
```rust
pub trait RightsEngine {
    async fn check_rights(&self, entity_id: &str, region: &str) -> Result<RightsInfo>;
    async fn get_licensing_windows(&self, entity_id: &str) -> Result<Vec<LicensingWindow>>;
}

pub struct RightsInfo {
    can_stream: bool,
    can_download: bool,
    restrictions: Vec<Restriction>,
    expires_at: Option<DateTime<Utc>>,
}
```

#### Layer 4: End-User Services

##### K. API Gateway (`api-gateway`)
**Responsibility:** REST/GraphQL API for client applications

**Endpoints:**
```
POST   /api/v1/search
GET    /api/v1/metadata/{entity_id}
GET    /api/v1/recommendations
POST   /api/v1/auth/oauth/{platform}
GET    /api/v1/availability/{entity_id}
POST   /api/v1/events/interaction

WebSocket /ws/updates  (real-time availability updates via PubNub)
```

See **Section 5: API Surface** for detailed endpoint specifications.

### 3.2 Service Lifecycle Management

#### 3.2.1 Startup Sequence

```
1. Configuration Loading
   ├── Read media-gateway.toml
   ├── Validate configuration schema
   ├── Load environment variables
   └── Initialize secret manager (GCP)

2. Infrastructure Services Bootstrap
   ├── Database migrations (Cloud SQL)
   ├── Cache warming (Memorystore)
   ├── Message broker initialization (Pub/Sub)
   └── Metrics exporter startup (Prometheus)

3. Core Services Initialization
   ├── Auth Service (credential validation)
   ├── MCP Server (tool/resource registration)
   ├── Platform Normalizers (connection testing)
   └── Entity Resolver (index loading)

4. Intelligence Services Startup
   ├── Search Service (Ruvector index loading)
   ├── Recommendation Engine (SONA model loading)
   └── Agent Orchestrator (agent pool initialization)

5. Consolidation Services
   ├── Metadata Service
   ├── Availability Index
   └── Rights Engine

6. API Gateway
   ├── Route registration
   ├── Middleware setup (auth, rate limiting)
   └── Health check endpoint activation

7. Health Check Verification
   └── All services report healthy or degraded
```

#### 3.2.2 Graceful Shutdown

```
1. Stop accepting new requests (API Gateway)
2. Drain in-flight requests (30s timeout)
3. Flush pending events to Pub/Sub
4. Persist agent state to memory namespace
5. Close database connections
6. Export final metrics
7. Terminate processes
```

#### 3.2.3 Service Health Checks

**Liveness Probe:**
```rust
pub struct LivenessCheck {
    async fn check(&self) -> HealthStatus {
        // Verify process is running and responsive
        // No external dependencies checked
        HealthStatus::Healthy
    }
}
```

**Readiness Probe:**
```rust
pub struct ReadinessCheck {
    async fn check(&self) -> HealthStatus {
        let mut status = HealthStatus::Healthy;

        // Check database connectivity
        if !self.db.ping().await.is_ok() {
            status = HealthStatus::Degraded;
        }

        // Check cache availability
        if !self.cache.ping().await.is_ok() {
            status = HealthStatus::Degraded;
        }

        // Check downstream service health
        for service in &self.dependencies {
            if !service.health_check().await.is_healthy() {
                status = HealthStatus::Degraded;
            }
        }

        status
    }
}

pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}
```

### 3.3 Inter-Service Communication

#### 3.3.1 Communication Patterns

**Synchronous (gRPC with Tonic):**
```rust
// Service definition (Protocol Buffers)
service SearchService {
    rpc SemanticSearch(SearchRequest) returns (SearchResponse);
    rpc IndexContent(IndexRequest) returns (IndexResponse);
}

// Client usage
let mut client = SearchServiceClient::connect("http://search-service:8081").await?;
let response = client.semantic_search(SearchRequest {
    query: "sci-fi thriller".to_string(),
    options: Some(SearchOptions { ... }),
}).await?;
```

**Asynchronous (Pub/Sub):**
```rust
// Event publishing
pub struct EventPublisher {
    async fn publish(&self, topic: &str, event: Event) -> Result<()> {
        self.pubsub_client
            .topic(topic)
            .publish(event.to_json())
            .await
    }
}

// Event subscription
pub struct EventSubscriber {
    async fn subscribe(&self, topic: &str, handler: EventHandler) -> Result<()> {
        self.pubsub_client
            .subscription(topic)
            .receive(|message| {
                handler.handle(Event::from_json(&message.data)?);
                message.ack();
            })
            .await
    }
}

// Topics:
// - content-updates       (metadata changes)
// - availability-changes  (platform availability)
// - user-interactions     (tracking events)
// - agent-tasks           (orchestration events)
```

**Real-Time (PubNub for cross-device sync):**
```rust
pub struct RealtimeSync {
    async fn publish_update(&self, channel: &str, update: Update) -> Result<()> {
        self.pubnub_client
            .publish()
            .channel(channel)
            .message(update.to_json())
            .execute()
            .await
    }

    async fn subscribe(&self, channel: &str, callback: UpdateCallback) -> Result<()> {
        self.pubnub_client
            .subscribe()
            .channels(&[channel])
            .execute();

        // Handle incoming messages
        self.pubnub_client
            .stream()
            .for_each(|message| callback.on_update(message))
            .await
    }
}

// Channels:
// - user-{user_id}        (personalized updates)
// - global-availability   (broad content changes)
```

#### 3.3.2 Service Discovery

**Kubernetes Service DNS:**
```
mcp-server.default.svc.cluster.local:3000
api-gateway.default.svc.cluster.local:8080
search-service.default.svc.cluster.local:8081
recommendation-service.default.svc.cluster.local:8082
```

**Configuration-Based Registry:**
```toml
[service_registry]
mcp-server = "mcp-server.default.svc.cluster.local:3000"
api-gateway = "api-gateway.default.svc.cluster.local:8080"
search-service = "search-service.default.svc.cluster.local:8081"
recommendation-service = "recommendation-service.default.svc.cluster.local:8082"
```

### 3.4 Data Storage and Persistence

**Database (Cloud SQL - PostgreSQL 15 HA):**
```sql
-- Tables:
users                 (user accounts, profiles)
content_entities      (unified content catalog)
platform_mappings     (entity-to-platform relationships)
availability          (real-time availability data)
user_interactions     (event tracking)
recommendations       (cached recommendation results)
auth_tokens           (encrypted OAuth tokens)
agent_sessions        (agent orchestration state)
```

**Cache (Memorystore - Valkey/Redis-compatible):**
```
Keys:
- metadata:{entity_id}              TTL: 24h
- search:{query_hash}               TTL: 1h
- recommendations:{user_id}         TTL: 6h
- availability:{entity_id}:{region} TTL: 6h
- agent:session:{session_id}        TTL: 30m
```

**Vector/Graph Database (Ruvector):**
```
Indexes:
- content_embeddings   (semantic search vectors)
- user_embeddings      (personalization vectors)
- knowledge_graph      (entity relationships - GNN)
- hypergraph           (multi-dimensional relationships)
```

---

## 4. Agent Orchestration Goals

### 4.1 AI Agent Integration Points

The Media Gateway is designed as an **Agent-Ready Platform** with multiple integration touchpoints:

#### 4.1.1 MCP Server Integration

**Primary Interface:** Model Context Protocol (MCP) server exposing tools, resources, and prompts

**Integration Pattern:**
```json
{
  "mcpServers": {
    "media-gateway": {
      "command": "npx",
      "args": ["agentics-hackathon", "mcp"],
      "env": {
        "MCP_TRANSPORT": "stdio"
      }
    }
  }
}
```

**Available Tools for AI Agents:**
```javascript
// Tool: get_hackathon_info
{
  "name": "get_hackathon_info",
  "description": "Retrieve event details, timelines, and prize information",
  "inputSchema": { "type": "object", "properties": {} }
}

// Tool: get_tracks
{
  "name": "get_tracks",
  "description": "List available competition tracks and their descriptions",
  "inputSchema": { "type": "object", "properties": {} }
}

// Tool: get_available_tools
{
  "name": "get_available_tools",
  "description": "Browse 17+ development tools across 6 categories",
  "inputSchema": {
    "type": "object",
    "properties": {
      "category": {
        "type": "string",
        "enum": ["ai-assistants", "orchestration", "cloud", "databases", "frameworks", "advanced"]
      }
    }
  }
}

// Tool: get_project_status
{
  "name": "get_project_status",
  "description": "View project configuration, installed tools, and service health",
  "inputSchema": { "type": "object", "properties": {} }
}

// Tool: check_tool_installed
{
  "name": "check_tool_installed",
  "description": "Verify if a specific tool is installed",
  "inputSchema": {
    "type": "object",
    "properties": {
      "tool_name": { "type": "string" }
    },
    "required": ["tool_name"]
  }
}

// Tool: get_resources
{
  "name": "get_resources",
  "description": "Access configuration files and metadata",
  "inputSchema": { "type": "object", "properties": {} }
}
```

**Resources Exposed:**
```javascript
{
  "resources": [
    {
      "uri": "config://project",
      "name": "Project Configuration",
      "mimeType": "application/json"
    },
    {
      "uri": "config://platforms",
      "name": "Platform Integration Status",
      "mimeType": "application/json"
    },
    {
      "uri": "config://agents",
      "name": "Agent Orchestration Settings",
      "mimeType": "application/json"
    }
  ]
}
```

**Prompts for Guidance:**
```javascript
{
  "prompts": [
    {
      "name": "hackathon_starter",
      "description": "Onboarding guidance for new projects",
      "arguments": []
    },
    {
      "name": "choose_track",
      "description": "Help select appropriate competition track",
      "arguments": [
        {
          "name": "use_case",
          "description": "Primary project use case",
          "required": false
        }
      ]
    }
  ]
}
```

#### 4.1.2 ARW Protocol Integration

**Agent-Ready Web (ARW) Specification:**

The Media Gateway implements ARW manifests for **85% token reduction** vs HTML scraping:

**Manifest Location:** `/.well-known/arw-manifest.json`

```json
{
  "version": "0.1",
  "name": "Media Gateway",
  "description": "Unified TV content discovery across 10+ streaming platforms",
  "homepage": "https://media-gateway.example.com",
  "actions": [
    {
      "id": "semantic_search",
      "name": "Semantic Content Search",
      "description": "Search for TV shows and movies using natural language",
      "endpoint": "/api/v1/search",
      "method": "POST",
      "inputSchema": {
        "type": "object",
        "properties": {
          "query": { "type": "string", "description": "Natural language search query" },
          "platforms": { "type": "array", "items": { "type": "string" } },
          "max_results": { "type": "integer", "default": 20 }
        },
        "required": ["query"]
      },
      "outputSchema": {
        "type": "object",
        "properties": {
          "results": {
            "type": "array",
            "items": { "$ref": "#/definitions/SearchResult" }
          }
        }
      }
    },
    {
      "id": "get_recommendations",
      "name": "Personalized Recommendations",
      "description": "Get AI-powered content recommendations",
      "endpoint": "/api/v1/recommendations",
      "method": "GET",
      "inputSchema": {
        "type": "object",
        "properties": {
          "user_id": { "type": "string" },
          "count": { "type": "integer", "default": 10 }
        }
      }
    },
    {
      "id": "check_availability",
      "name": "Check Content Availability",
      "description": "Verify content availability across platforms",
      "endpoint": "/api/v1/availability/{entity_id}",
      "method": "GET",
      "inputSchema": {
        "type": "object",
        "properties": {
          "entity_id": { "type": "string" },
          "region": { "type": "string", "default": "US" }
        },
        "required": ["entity_id"]
      }
    }
  ],
  "definitions": {
    "SearchResult": {
      "type": "object",
      "properties": {
        "entity_id": { "type": "string" },
        "title": { "type": "string" },
        "platforms": { "type": "array", "items": { "type": "string" } },
        "similarity_score": { "type": "number" },
        "metadata": { "$ref": "#/definitions/ContentMetadata" }
      }
    },
    "ContentMetadata": {
      "type": "object",
      "properties": {
        "category": { "type": "string" },
        "release_year": { "type": "integer" },
        "duration_minutes": { "type": "integer" },
        "genres": { "type": "array", "items": { "type": "string" } },
        "rating": { "type": "number" }
      }
    }
  },
  "authentication": {
    "type": "oauth2",
    "flows": {
      "authorizationCode": {
        "authorizationUrl": "/api/v1/auth/authorize",
        "tokenUrl": "/api/v1/auth/token",
        "scopes": {
          "read:content": "Read content metadata",
          "read:recommendations": "Access personalized recommendations",
          "write:interactions": "Track user interactions"
        }
      }
    }
  }
}
```

**ARW Benefits for AI Agents:**
- Structured action discovery (no HTML parsing)
- JSON Schema validation (type-safe interactions)
- OAuth2 enforcement (secure transactions)
- Self-documenting API surface
- 10x faster discovery vs scraping

#### 4.1.3 E2B Sandbox Integration

**Purpose:** Safe code execution for AI-generated scripts

**Use Cases:**
- Data transformation scripts (custom platform adapters)
- Recommendation algorithm experimentation
- User-defined search filters
- Automated testing of platform integrations

**Integration Pattern:**
```rust
pub struct E2BSandboxExecutor {
    async fn execute_code(&self, code: &str, language: Language) -> Result<ExecutionResult> {
        let sandbox = self.client.create_sandbox(SandboxConfig {
            runtime: match language {
                Language::Python => Runtime::Python,
                Language::JavaScript => Runtime::NodeJS,
                Language::Rust => Runtime::Rust,
            },
            timeout_sec: 30,
            memory_limit_mb: 512,
            cpu_limit: 1.0,
        }).await?;

        let result = sandbox.execute(code).await?;
        sandbox.destroy().await?;

        Ok(result)
    }
}

pub struct ExecutionResult {
    stdout: String,
    stderr: String,
    exit_code: i32,
    execution_time_ms: u64,
}
```

**Security Guarantees:**
- Firecracker microVM isolation
- No network access (air-gapped)
- Resource limits enforced
- Automatic cleanup after execution

### 4.2 Autonomous Operation Modes

#### 4.2.1 Fully Autonomous Workflows

**Scenario 1: Automated Platform Monitoring**
```rust
pub struct AutonomousPlatformMonitor {
    async fn run(&self) {
        loop {
            // Every 6 hours
            sleep(Duration::from_secs(6 * 3600)).await;

            // Spawn agents for each platform
            for platform in &self.platforms {
                self.orchestrator.spawn_agent(AgentType::MetadataResolver).await?;

                // Agent autonomously:
                // 1. Authenticates with platform
                // 2. Fetches catalog updates
                // 3. Normalizes new content
                // 4. Resolves duplicate entities
                // 5. Updates metadata service
                // 6. Invalidates stale cache entries
                // 7. Publishes availability updates
                // 8. Reports completion metrics
            }
        }
    }
}
```

**Scenario 2: Intelligent Cache Warming**
```rust
pub struct AutonomousCacheOptimizer {
    async fn optimize(&self) {
        // Spawn agent to analyze usage patterns
        let agent = self.orchestrator.spawn_agent(AgentType::CacheOptimizer).await?;

        // Agent autonomously:
        // 1. Queries user interaction logs
        // 2. Identifies trending content
        // 3. Predicts likely search queries
        // 4. Pre-fetches metadata and embeddings
        // 5. Warms cache with high-probability content
        // 6. Evicts stale, low-access entries
        // 7. Adjusts TTL based on volatility
        // 8. Reports cache hit rate improvements
    }
}
```

**Scenario 3: Anomaly Detection and Self-Healing**
```rust
pub struct AutonomousAnomalyDetector {
    async fn monitor(&self) {
        let agent = self.orchestrator.spawn_agent(AgentType::AnomalyDetector).await?;

        // Agent autonomously:
        // 1. Monitors service health metrics
        // 2. Detects unusual latency spikes
        // 3. Identifies platform API failures
        // 4. Triggers circuit breaker on outages
        // 5. Spawns additional service replicas if needed
        // 6. Reroutes traffic to healthy instances
        // 7. Alerts human operators of critical issues
        // 8. Generates incident reports
    }
}
```

#### 4.2.2 Human-in-the-Loop Scenarios

**Scenario 1: Content Rights Verification**
```rust
pub struct HumanInLoopRightsVerification {
    async fn verify_rights(&self, entity_id: &str) -> Result<RightsDecision> {
        // Agent performs initial analysis
        let agent_analysis = self.search_service.get_metadata(entity_id).await?;
        let licensing_info = self.rights_engine.get_licensing_windows(entity_id).await?;

        // If confidence is low, escalate to human
        if agent_analysis.confidence_score < 0.85 {
            // Present structured decision request
            let decision_request = DecisionRequest {
                entity_id: entity_id.to_string(),
                agent_analysis,
                licensing_info,
                required_decision: "approve_streaming_rights",
                deadline: Utc::now() + Duration::hours(24),
            };

            // Await human decision via dashboard/API
            let human_decision = self.decision_queue.await_decision(&decision_request).await?;

            // Agent learns from human feedback
            self.sona_engine.update_reasoning_bank(
                decision_request.context(),
                human_decision.rationale,
            ).await?;

            return Ok(human_decision.into());
        }

        // High confidence: autonomous decision
        Ok(RightsDecision::Approved)
    }
}
```

**Scenario 2: Ambiguous Search Query Clarification**
```rust
pub struct HumanInLoopSearchClarification {
    async fn handle_ambiguous_query(&self, query: &str) -> Result<SearchResults> {
        let agent_interpretation = self.search_service.semantic_search(query, SearchOptions::default()).await?;

        // If results are too diverse (low coherence)
        if agent_interpretation.result_coherence_score < 0.6 {
            // Request clarification
            let clarification_options = self.generate_clarification_options(query, &agent_interpretation)?;

            let user_choice = self.interaction_service.request_clarification(
                ClarificationRequest {
                    original_query: query.to_string(),
                    options: clarification_options,
                    timeout_sec: 300,
                }
            ).await?;

            // Re-run search with clarified intent
            return self.search_service.semantic_search(&user_choice.refined_query, SearchOptions::default()).await;
        }

        Ok(agent_interpretation)
    }
}
```

### 4.3 Multi-Agent Coordination

#### 4.3.1 Coordination Topologies

**Hierarchical:**
```
Orchestrator Agent (Coordinator)
├── Search Agent (Specialized task)
├── Metadata Agent (Specialized task)
└── Recommendation Agent (Specialized task)
    └── Personalization Sub-Agent (Delegated task)
```

**Use Cases:**
- Complex workflows with clear task decomposition
- Authority-based decision making
- Resource allocation optimization

**Mesh:**
```
Search Agent ←→ Metadata Agent
    ↕              ↕
Recommendation Agent ←→ Cache Agent
```

**Use Cases:**
- Peer-to-peer information sharing
- Distributed decision making
- High availability (no single point of failure)

**Adaptive:**
```
System analyzes task complexity and dynamically selects:
- Simple tasks → Single agent
- Medium tasks → Hierarchical (2-3 agents)
- Complex tasks → Mesh (4+ agents with cross-communication)
```

#### 4.3.2 Coordination Memory Namespace

**Namespace:** `aqe/*` (Agentic QE Fleet convention)

**Key Structure:**
```
aqe/test-plan/*              Test planning and requirements
aqe/coverage/*               Coverage analysis and gaps
aqe/quality/*                Quality metrics and gates
aqe/performance/*            Performance test results
aqe/security/*               Security scan findings
aqe/swarm/coordination       Cross-agent coordination state
aqe/swarm/tasks/{task_id}    Individual task state
aqe/swarm/agents/{agent_id}  Agent-specific state
```

**Coordination Protocol:**

```rust
// Pre-task hook: Register task and claim resources
pub async fn pre_task(task: &Task) -> Result<()> {
    memory::store(
        &format!("aqe/swarm/tasks/{}", task.id),
        &TaskState {
            status: TaskStatus::InProgress,
            assigned_agents: vec![],
            started_at: Utc::now(),
        }
    ).await?;

    Ok(())
}

// During task: Share intermediate results
pub async fn share_progress(agent_id: &str, progress: Progress) -> Result<()> {
    memory::store(
        &format!("aqe/swarm/agents/{}/progress", agent_id),
        &progress
    ).await?;

    // Notify other agents via Pub/Sub
    pubsub::publish("agent-progress", AgentProgressEvent {
        agent_id: agent_id.to_string(),
        progress,
    }).await?;

    Ok(())
}

// Post-task hook: Aggregate results and update metrics
pub async fn post_task(task: &Task, result: TaskResult) -> Result<()> {
    memory::store(
        &format!("aqe/swarm/tasks/{}/result", task.id),
        &result
    ).await?;

    // Update coordination state
    memory::update(
        "aqe/swarm/coordination",
        |state: &mut CoordinationState| {
            state.completed_tasks += 1;
            state.active_agents = state.active_agents.saturating_sub(result.agents_used.len());
        }
    ).await?;

    Ok(())
}
```

#### 4.3.3 Multi-Agent Task Example

**Task:** "Find the best sci-fi thriller under 2 hours available on Netflix or Prime in the US"

**Agent Decomposition:**
```rust
pub async fn orchestrate_complex_search(query: &str) -> Result<SearchResults> {
    // Coordinator spawns specialized agents
    let search_agent = spawn_agent(AgentType::SearchAgent).await?;
    let metadata_agent = spawn_agent(AgentType::MetadataResolver).await?;
    let availability_agent = spawn_agent(AgentType::AvailabilityChecker).await?;
    let recommendation_agent = spawn_agent(AgentType::RecommendationAgent).await?;

    // Step 1: Search agent finds matching content
    let semantic_matches = search_agent.search("sci-fi thriller").await?;

    // Step 2: Metadata agent enriches results (parallel)
    let enriched = join_all(
        semantic_matches.iter().map(|entity| {
            metadata_agent.enrich(entity.id)
        })
    ).await?;

    // Step 3: Filter by duration constraint
    let filtered = enriched.into_iter()
        .filter(|entity| entity.metadata.duration_minutes.unwrap_or(999) <= 120)
        .collect::<Vec<_>>();

    // Step 4: Availability agent checks platforms (parallel)
    let available = join_all(
        filtered.iter().map(|entity| {
            availability_agent.check_platforms(entity.id, &["netflix", "prime"], "US")
        })
    ).await?;

    // Step 5: Recommendation agent ranks by user preferences
    let user_id = get_current_user_id();
    let ranked = recommendation_agent.rank(available, user_id).await?;

    // Step 6: Return top results
    Ok(SearchResults {
        results: ranked.into_iter().take(10).collect(),
        agents_used: vec![search_agent.id, metadata_agent.id, availability_agent.id, recommendation_agent.id],
        execution_time_ms: /* tracked */,
    })
}
```

**Coordination Flow:**
```
1. Coordinator stores task in memory: aqe/swarm/tasks/{task_id}
2. Each agent:
   - Reads task requirements from memory
   - Executes specialized subtask
   - Writes intermediate results to memory: aqe/swarm/agents/{agent_id}/results
   - Publishes progress event
3. Coordinator:
   - Monitors agent progress via memory + Pub/Sub
   - Aggregates results from agent memory
   - Handles failures (retry or reassign)
   - Updates final result in memory
4. Cleanup:
   - Terminate agents
   - Export metrics
   - Persist session for analysis
```

---

## 5. API Surface

### 5.1 REST API Endpoints

**Base URL:** `https://api.media-gateway.example.com/api/v1`

**Authentication:** OAuth2 Bearer Token
```
Authorization: Bearer <access_token>
```

#### 5.1.1 Search Endpoints

**POST /search**

Semantic content search across platforms.

```http
POST /api/v1/search
Content-Type: application/json
Authorization: Bearer <token>

{
  "query": "sci-fi thriller under 2 hours",
  "platforms": ["netflix", "prime", "disney"],
  "categories": ["movie"],
  "similarity_threshold": 0.75,
  "max_results": 20,
  "personalized": true
}
```

**Response:**
```json
{
  "results": [
    {
      "entity_id": "ent_abc123",
      "title": "Inception",
      "platforms": ["netflix", "prime"],
      "similarity_score": 0.92,
      "metadata": {
        "category": "movie",
        "release_year": 2010,
        "duration_minutes": 148,
        "genres": ["sci-fi", "thriller", "action"],
        "rating": 8.8,
        "description": "A thief who steals corporate secrets..."
      },
      "availability": {
        "netflix": {
          "is_available": true,
          "subscription_required": true,
          "region": "US"
        },
        "prime": {
          "is_available": true,
          "subscription_required": true,
          "region": "US"
        }
      },
      "explanation": "Matched 'sci-fi thriller' with high semantic similarity. Duration: 148 minutes."
    }
  ],
  "total_results": 15,
  "processing_time_ms": 184
}
```

#### 5.1.2 Metadata Endpoints

**GET /metadata/{entity_id}**

Retrieve comprehensive metadata for a content entity.

```http
GET /api/v1/metadata/ent_abc123
Authorization: Bearer <token>
```

**Response:**
```json
{
  "entity_id": "ent_abc123",
  "canonical_title": "Inception",
  "international_titles": {
    "es": "El Origen",
    "fr": "Inception",
    "de": "Inception"
  },
  "imdb_id": "tt1375666",
  "tmdb_id": "27205",
  "category": "movie",
  "release_year": 2010,
  "duration_minutes": 148,
  "genres": ["sci-fi", "thriller", "action"],
  "themes": ["dreams", "heist", "reality-bending"],
  "cast": [
    {
      "person_id": "per_123",
      "name": "Leonardo DiCaprio",
      "role": "actor",
      "character": "Dom Cobb"
    }
  ],
  "crew": [
    {
      "person_id": "per_456",
      "name": "Christopher Nolan",
      "role": "director"
    }
  ],
  "ratings": {
    "imdb": 8.8,
    "tmdb": 8.4,
    "rotten_tomatoes": 87
  },
  "awards": [
    {
      "name": "Academy Awards",
      "category": "Best Cinematography",
      "year": 2011,
      "won": true
    }
  ],
  "platforms": ["netflix", "prime", "disney"],
  "metadata_quality_score": 0.95,
  "last_updated": "2025-12-06T10:30:00Z"
}
```

#### 5.1.3 Recommendation Endpoints

**GET /recommendations**

Get personalized content recommendations.

```http
GET /api/v1/recommendations?user_id=usr_xyz&count=10&platforms=netflix,prime&diversity_weight=0.3
Authorization: Bearer <token>
```

**Response:**
```json
{
  "recommendations": [
    {
      "id": "reco_001",
      "entity": {
        "entity_id": "ent_def456",
        "title": "The Matrix",
        "platforms": ["netflix"],
        "metadata": { ... }
      },
      "score": 0.94,
      "reasoning": [
        {
          "factor": "genre_match",
          "weight": 0.4,
          "description": "You frequently watch sci-fi movies"
        },
        {
          "factor": "collaborative_filtering",
          "weight": 0.3,
          "description": "Users with similar tastes loved this"
        },
        {
          "factor": "trending",
          "weight": 0.3,
          "description": "Popular this week"
        }
      ],
      "freshness": "trending"
    }
  ],
  "generated_at": "2025-12-06T10:35:00Z",
  "cache_ttl_seconds": 21600
}
```

**POST /recommendations/feedback**

Provide feedback on recommendations to improve future suggestions.

```http
POST /api/v1/recommendations/feedback
Content-Type: application/json
Authorization: Bearer <token>

{
  "user_id": "usr_xyz",
  "recommendation_id": "reco_001",
  "feedback": "positive",
  "action_taken": "watch_complete"
}
```

#### 5.1.4 Availability Endpoints

**GET /availability/{entity_id}**

Check content availability across platforms.

```http
GET /api/v1/availability/ent_abc123?region=US
Authorization: Bearer <token>
```

**Response:**
```json
{
  "entity_id": "ent_abc123",
  "region": "US",
  "platforms": [
    {
      "platform": "netflix",
      "is_available": true,
      "subscription_required": true,
      "pricing": null,
      "last_verified": "2025-12-06T09:00:00Z"
    },
    {
      "platform": "prime",
      "is_available": true,
      "subscription_required": false,
      "pricing": {
        "rent": { "amount": 3.99, "currency": "USD" },
        "buy": { "amount": 14.99, "currency": "USD" }
      },
      "last_verified": "2025-12-06T09:00:00Z"
    },
    {
      "platform": "disney",
      "is_available": false,
      "subscription_required": null,
      "pricing": null,
      "last_verified": "2025-12-06T09:00:00Z"
    }
  ]
}
```

#### 5.1.5 Authentication Endpoints

**POST /auth/oauth/{platform}**

Initiate OAuth2 flow for platform authentication.

```http
POST /api/v1/auth/oauth/netflix
Content-Type: application/json

{
  "user_id": "usr_xyz",
  "redirect_uri": "https://app.example.com/auth/callback"
}
```

**Response:**
```json
{
  "authorization_url": "https://netflix.com/oauth/authorize?client_id=...&state=...&code_challenge=...",
  "state": "csrf_token_abc123",
  "expires_at": "2025-12-06T11:00:00Z"
}
```

**POST /auth/callback**

Handle OAuth2 callback and exchange code for token.

```http
POST /api/v1/auth/callback
Content-Type: application/json

{
  "platform": "netflix",
  "state": "csrf_token_abc123",
  "code": "authorization_code_xyz"
}
```

**Response:**
```json
{
  "access_token": "encrypted_token",
  "refresh_token": "encrypted_refresh",
  "expires_in": 3600,
  "token_type": "Bearer",
  "platform": "netflix"
}
```

#### 5.1.6 Event Tracking Endpoints

**POST /events/interaction**

Track user interactions for recommendation improvement.

```http
POST /api/v1/events/interaction
Content-Type: application/json
Authorization: Bearer <token>

{
  "user_id": "usr_xyz",
  "entity_id": "ent_abc123",
  "event_type": "watch_complete",
  "timestamp": "2025-12-06T10:45:00Z",
  "context": {
    "device": "smart_tv",
    "session_duration_sec": 8880
  }
}
```

**Response:**
```json
{
  "event_id": "evt_123456",
  "processed": true,
  "timestamp": "2025-12-06T10:45:01Z"
}
```

### 5.2 WebSocket Interface

**Endpoint:** `wss://api.media-gateway.example.com/ws/updates`

**Purpose:** Real-time availability updates and notifications

**Connection:**
```javascript
const ws = new WebSocket('wss://api.media-gateway.example.com/ws/updates');
ws.onopen = () => {
  // Authenticate
  ws.send(JSON.stringify({
    type: 'auth',
    token: '<access_token>'
  }));

  // Subscribe to updates
  ws.send(JSON.stringify({
    type: 'subscribe',
    channels: ['user-usr_xyz', 'global-availability']
  }));
};

ws.onmessage = (event) => {
  const update = JSON.parse(event.data);
  console.log('Received update:', update);
};
```

**Update Message Types:**

**Availability Change:**
```json
{
  "type": "availability_change",
  "entity_id": "ent_abc123",
  "platform": "netflix",
  "region": "US",
  "is_available": false,
  "timestamp": "2025-12-06T11:00:00Z"
}
```

**New Content:**
```json
{
  "type": "new_content",
  "entity_id": "ent_new789",
  "title": "New Show",
  "platforms": ["prime"],
  "category": "series",
  "timestamp": "2025-12-06T11:05:00Z"
}
```

**Personalized Alert:**
```json
{
  "type": "personalized_alert",
  "user_id": "usr_xyz",
  "message": "A movie on your watchlist is now available on Netflix",
  "entity_id": "ent_xyz",
  "timestamp": "2025-12-06T11:10:00Z"
}
```

### 5.3 GraphQL API (Optional)

**Endpoint:** `https://api.media-gateway.example.com/graphql`

**Schema:**
```graphql
type Query {
  search(
    query: String!
    platforms: [Platform!]
    categories: [Category!]
    similarityThreshold: Float = 0.75
    maxResults: Int = 20
    personalized: Boolean = false
  ): SearchResults!

  entity(id: ID!): ContentEntity

  recommendations(
    userId: ID!
    count: Int = 10
    platforms: [Platform!]
    diversityWeight: Float = 0.3
  ): [Recommendation!]!

  availability(entityId: ID!, region: String = "US"): [PlatformAvailability!]!
}

type Mutation {
  trackInteraction(input: InteractionInput!): InteractionEvent!

  provideFeedback(input: FeedbackInput!): Feedback!
}

type SearchResults {
  results: [SearchResult!]!
  totalResults: Int!
  processingTimeMs: Int!
}

type SearchResult {
  entityId: ID!
  title: String!
  platforms: [Platform!]!
  similarityScore: Float!
  metadata: ContentMetadata!
  availability: [PlatformAvailability!]!
  explanation: String
}

type ContentEntity {
  entityId: ID!
  canonicalTitle: String!
  internationalTitles: [Translation!]!
  imdbId: String
  tmdbId: String
  category: Category!
  releaseYear: Int!
  durationMinutes: Int
  genres: [String!]!
  themes: [String!]!
  cast: [Person!]!
  crew: [Person!]!
  ratings: RatingCollection!
  awards: [Award!]!
  platforms: [Platform!]!
  metadataQualityScore: Float!
  lastUpdated: DateTime!
}

type Recommendation {
  id: ID!
  entity: ContentEntity!
  score: Float!
  reasoning: [ReasoningFactor!]!
  freshness: Freshness!
}

type PlatformAvailability {
  platform: Platform!
  isAvailable: Boolean!
  subscriptionRequired: Boolean
  pricing: Pricing
  lastVerified: DateTime!
}

enum Platform {
  NETFLIX
  PRIME
  DISNEY
  HULU
  APPLE_TV
  YOUTUBE
  CRAVE
  HBO_MAX
}

enum Category {
  MOVIE
  SERIES
  DOCUMENTARY
  SHORT
}

enum Freshness {
  NEW
  TRENDING
  CLASSIC
}
```

### 5.4 SDK/Client Library Expectations

**Language Support:**
- TypeScript/JavaScript (primary, for web/Node.js)
- Python (data science/ML integration)
- Rust (native performance-critical clients)
- Swift (iOS)
- Kotlin (Android)

**TypeScript SDK Example:**
```typescript
import { MediaGatewayClient } from '@media-gateway/sdk';

const client = new MediaGatewayClient({
  apiKey: process.env.MEDIA_GATEWAY_API_KEY,
  baseUrl: 'https://api.media-gateway.example.com',
});

// Semantic search
const searchResults = await client.search({
  query: 'sci-fi thriller',
  platforms: ['netflix', 'prime'],
  maxResults: 10,
  personalized: true,
});

// Get recommendations
const recommendations = await client.getRecommendations({
  userId: 'usr_xyz',
  count: 10,
  platforms: ['netflix'],
});

// Check availability
const availability = await client.checkAvailability('ent_abc123', 'US');

// Track interaction
await client.trackInteraction({
  userId: 'usr_xyz',
  entityId: 'ent_abc123',
  eventType: 'watch_complete',
});

// Real-time updates
client.on('availability_change', (update) => {
  console.log('Content availability changed:', update);
});

client.subscribe(['user-usr_xyz', 'global-availability']);
```

**SDK Guarantees:**
- Type-safe interfaces (TypeScript definitions)
- Automatic token refresh
- Retry logic with exponential backoff
- WebSocket auto-reconnect
- Request/response logging (opt-in)
- Error handling with descriptive messages

---

## 6. Technical Architecture

### 6.1 Technology Stack Summary

**Primary Language:** Rust (100%)

**Key Libraries/Frameworks:**
- **Web Framework:** Axum (async HTTP server)
- **gRPC:** Tonic (inter-service communication)
- **Database:** SQLx (PostgreSQL driver)
- **Cache:** redis-rs (Valkey/Redis client)
- **Vector/Graph:** Ruvector (custom hypergraph + vector + GNN)
- **ML/AI:** SONA Integration (Self-Optimizing Neural Architecture)
- **Real-time:** PubNub SDK (cross-device sync)
- **Messaging:** Google Cloud Pub/Sub
- **Orchestration:** Custom agent framework

**Foundation Tooling:**
- **CLI/MCP:** hackathon-tv5 (TypeScript, npx-based)
- **Sandboxes:** E2B (Firecracker microVMs)

**Infrastructure:**
- **Cloud:** Google Cloud Platform (GCP)
- **Container Orchestration:** GKE Autopilot
- **Serverless:** Cloud Run
- **Database:** Cloud SQL (PostgreSQL 15 HA)
- **Cache:** Memorystore (Valkey)
- **Secrets:** Secret Manager
- **Logging:** Cloud Logging
- **Metrics:** Prometheus + Cloud Monitoring

### 6.2 Deployment Architecture

**51 Micro-repositories:**
- Independent versioning (semantic versioning)
- Parallel CI/CD pipelines
- Blue-green deployment strategy
- Canary releases for intelligence services

**Estimated Costs (GCP):**
- **Low traffic:** $2,400/month
- **High traffic:** $3,650/month

**Scaling Characteristics:**
- API Gateway: 1-10 instances (auto-scale)
- Search Service: 1-5 instances (SONA latency target: <5ms)
- Recommendation: 1-3 instances
- Normalizers: 1 instance per platform (10+)
- Database: HA setup (read replicas)
- Cache: 5GB Valkey cluster

---

## 7. Integration Requirements

### 7.1 Platform Integration Checklist

For each streaming platform (Netflix, Prime, Disney+, etc.):

**Required:**
- [ ] OAuth2 client credentials obtained
- [ ] Normalizer implemented (`{platform}-normalizer` service)
- [ ] API rate limits documented and enforced
- [ ] Catalog refresh schedule configured (default: 6 hours)
- [ ] Error handling for platform outages (circuit breaker)
- [ ] Test account for integration testing

**Optional:**
- [ ] Webhooks for real-time availability updates
- [ ] Dedicated IP whitelisting (if required by platform)
- [ ] Custom retry logic for platform-specific quirks

### 7.2 Development Environment Setup

**Prerequisites:**
```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable

# Node.js (for hackathon-tv5 CLI)
nvm install 20
nvm use 20

# Docker (for local services)
docker --version  # 20.10+

# GCP CLI (for cloud deployment)
gcloud --version

# PostgreSQL client (for database access)
psql --version  # 15+
```

**Installation:**
```bash
# Clone repository
git clone https://github.com/your-org/media-gateway.git
cd media-gateway

# Install foundation CLI
npm install -g agentics-hackathon

# Initialize project
npx agentics-hackathon init --track multi-agent

# Install tools
npx agentics-hackathon tools install claude-flow
npx agentics-hackathon tools install sona-engine

# Configure environment
cp .env.example .env
# Edit .env with platform credentials

# Build services
cargo build --release

# Run migrations
cargo run --bin migrate

# Start services
media-gateway service start
```

### 7.3 Testing Requirements

**Unit Tests:**
```bash
cargo test --lib
```

**Integration Tests:**
```bash
cargo test --test integration -- --test-threads=1
```

**E2E Tests:**
```bash
# Requires live platform credentials (use test accounts)
cargo test --test e2e -- --ignored
```

**Performance Benchmarks:**
```bash
cargo bench
```

**Coverage Target:** > 80% line coverage

### 7.4 CI/CD Pipeline

**Stages:**
1. **Lint:** `cargo clippy --all-targets --all-features -- -D warnings`
2. **Format:** `cargo fmt -- --check`
3. **Build:** `cargo build --release`
4. **Test:** `cargo test --all-features`
5. **Security Audit:** `cargo audit`
6. **Docker Build:** `docker build -t media-gateway-{service}:{version}`
7. **Deploy to Staging:** GKE Autopilot (staging namespace)
8. **E2E Tests (Staging):** Automated test suite
9. **Deploy to Production:** Blue-green deployment with canary (10% traffic)
10. **Monitoring:** Prometheus alerts + Cloud Monitoring

---

## Appendices

### A. Glossary

- **ARW:** Agent-Ready Web protocol for structured AI-agent interaction
- **SONA:** Self-Optimizing Neural Architecture (intelligence engine)
- **PKCE:** Proof Key for Code Exchange (OAuth2 security extension)
- **LoRA:** Low-Rank Adaptation (efficient model fine-tuning)
- **EWC:** Elastic Weight Consolidation (prevents catastrophic forgetting)
- **GNN:** Graph Neural Network
- **Ruvector:** Custom database combining hypergraph, vector, and GNN indexes
- **MCP:** Model Context Protocol (Anthropic standard for AI-agent tooling)

### B. References

**Repositories:**
- https://github.com/agenticsorg/hackathon-tv5
- https://github.com/globalbusinessadvisors/media-gateway-research

**Additional Research:**
- [Multi-Agent Architecture Blueprints](https://authoritypartners.com/insights/from-single-agent-to-agent-teams-the-architecture-blueprint/)
- [Agent Operating Systems (Agent-OS)](https://www.techrxiv.org/doi/full/10.36227/techrxiv.175736224.43024590)
- [Model Context Protocol Documentation](https://modelcontextprotocol.io/docs/concepts/tools)
- [NVIDIA Agent Toolkit](https://developer.nvidia.com/agentiq-hackathon)

### C. Revision History

| Version | Date       | Changes                                     | Author        |
|---------|------------|---------------------------------------------|---------------|
| 1.0.0   | 2025-12-06 | Initial specification based on research     | Research Agent|

---

**END OF SPECIFICATION**
