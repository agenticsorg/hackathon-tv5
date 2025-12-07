<!--
Source: https://github.com/ruvnet/agentic-flow, https://www.npmjs.com/package/agentic-flow
Fetched: 2025-12-07T00:00:00Z
Topic: agentic-flow orchestration framework, multi-agent workflows, AI agent coordination
Package: agentic-flow@2.0.1-alpha.5
-->

# Agentic Flow - Complete Reference Guide

**Production-ready AI agent orchestration platform with 66 specialized agents, 213 MCP tools, ReasoningBank learning memory, and autonomous multi-agent swarms.**

Built by @ruvnet with Claude Agent SDK, neural networks, memory persistence, and GitHub integration.

---

## Table of Contents

1. [Core Concepts](#core-concepts)
2. [Installation & Setup](#installation--setup)
3. [Workflow Patterns](#workflow-patterns)
4. [Hooks System](#hooks-system)
5. [Memory Management](#memory-management)
6. [Programmatic Integration](#programmatic-integration)
7. [Package Ecosystem](#package-ecosystem)
8. [Performance & Optimization](#performance--optimization)
9. [API Reference](#api-reference)
10. [Production Features](#production-features)

---

## Core Concepts

### Swarms & Agents

Agentic Flow implements **self-learning parallel execution with AI topology selection** achieving **3-5x speedup through auto-optimization**. Agents operate within swarms that intelligently determine optimal configurations based on learned patterns.

**Key Characteristics:**
- **66 specialized agents** for different domains (development, testing, security, DevOps)
- **Autonomous coordination** through shared memory and message passing
- **Adaptive topologies** (mesh, hierarchical, ring) selected automatically
- **Federation Hub** for ephemeral agent management (5-15 minute lifetimes)

**Topology Types:**

| Topology | Best For | Agent Count | Characteristics |
|----------|----------|-------------|-----------------|
| **Hierarchical** | Complex tasks | 5-20 | Tree-based leadership with coordinator agents |
| **Mesh** | Collaboration | 3-8 | Peer-to-peer direct communication (optimal bandwidth) |
| **Ring** | Sequential | 4-12 | Sequential task passing (bandwidth-efficient) |
| **Adaptive** | Dynamic workloads | Any | Dynamic topology switching based on learned metrics |

### Tasks & Orchestration

Tasks represent discrete work units executed by agents. The system supports:

- **Parallel execution** - Multiple agents working simultaneously
- **Sequential pipelines** - Chained task dependencies
- **Dynamic spawning** - Auto-spawning based on workload
- **Self-healing** - 97.9% degradation prevention through monitoring

### Memory Coordination

Three-layer memory architecture enables cross-agent learning:

1. **ReasoningBank** - Semantic search across historical execution patterns (46% faster execution after learning)
2. **AgentDB v2** - RuVector-powered graph database with vector search (150x faster than SQLite, sub-ms latency)
3. **Reflexion Memory** - Episode-based learning from task outcomes with self-critique

**Memory Features:**
- **Cross-session persistence** - Survives agent restarts
- **Semantic indexing** - Vector embeddings for fuzzy pattern matching
- **Namespace isolation** - Logical grouping of related memories
- **TTL management** - Automatic expiration of stale learnings
- **32.6M ops/sec** pattern search performance

---

## Installation & Setup

### NPM Installation

```bash
# Install alpha version
npm install agentic-flow@alpha

# Install specific packages
npm install @agentic-flow/agentdb@alpha
npm install @agentic-flow/agent-booster@alpha
npm install @agentic-flow/reasoningbank@alpha
```

### MCP Server Configuration

Add to Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "agentic-flow": {
      "command": "npx",
      "args": ["agentic-flow@alpha", "mcp", "start"]
    },
    "agentdb": {
      "command": "npx",
      "args": ["agentdb@alpha", "mcp", "start"]
    }
  }
}
```

### Federation Hub Setup

```bash
# Start ephemeral federation hub
npx agentic-flow@alpha federation start

# Spawn agents in federation
npx agentic-flow@alpha federation spawn --type coder --count 3

# Check federation statistics
npx agentic-flow@alpha federation stats
```

### AgentDB Initialization

```bash
# Initialize database with optimal settings
npx agentdb@alpha init --dimension 768 --preset medium

# Run diagnostics
npx agentdb@alpha doctor --verbose

# Store reflexion memory
npx agentdb@alpha reflexion store "session-1" "task-description" 0.95 true
```

---

## Workflow Patterns

### Agent Spawning & Coordination

**Claude Code Task Tool (Primary Method):**

```javascript
// Single message with concurrent agent spawning
Task("Backend Developer", "Build REST API with Express. Use hooks for coordination.", "backend-dev")
Task("Frontend Developer", "Create React UI. Coordinate with backend via memory.", "coder")
Task("Database Architect", "Design PostgreSQL schema. Store schema in memory.", "code-analyzer")
Task("Test Engineer", "Write Jest tests. Check memory for API contracts.", "tester")
```

**MCP Coordination Setup (Optional):**

```javascript
// Setup coordination topology (optional for complex tasks)
mcp__agentic-flow__swarm_init({ topology: "mesh", maxAgents: 6 })
mcp__agentic-flow__agent_spawn({ type: "researcher" })
mcp__agentic-flow__agent_spawn({ type: "coder" })
mcp__agentic-flow__task_orchestrate({ plan: "multi-step-workflow" })
```

### Deployment Strategies

Seven battle-tested patterns scored 92-99/100:

#### 1. Blue-Green Deployment (99/100)
- **Use Case:** Instant rollback capability
- **Pattern:** Maintain two identical production environments
- **Benefit:** Zero-downtime deployments with instant rollback

#### 2. Rolling Update (95/100)
- **Use Case:** Zero-downtime updates
- **Pattern:** Incrementally replace instances
- **Benefit:** Gradual rollout with continuous availability

#### 3. Progressive Delivery (97/100)
- **Use Case:** Metric-driven rollouts
- **Pattern:** Automated canary with metric gates
- **Benefit:** Data-driven deployment decisions

#### 4. Canary Deployment (92/100)
- **Use Case:** Gradual risk mitigation
- **Pattern:** Route small traffic percentage to new version
- **Benefit:** Early failure detection before full rollout

#### 5. A/B Testing (94/100)
- **Use Case:** Feature validation
- **Pattern:** Parallel version comparison with real users
- **Benefit:** Empirical feature performance data

#### 6. Shadow Deployment (93/100)
- **Use Case:** Production testing without user impact
- **Pattern:** Duplicate traffic to new version (responses discarded)
- **Benefit:** Real-world load testing without risk

#### 7. Feature Toggle (96/100)
- **Use Case:** Dark launches and gradual rollouts
- **Pattern:** Runtime feature flag control
- **Benefit:** Decouple deployment from feature activation

### Multi-Agent Coordination Example

```typescript
import { SwarmLearningOptimizer } from 'agentic-flow/hooks/swarm-learning-optimizer';

const swarm = await createSwarm({
  agents: ['coder', 'reviewer', 'tester'],
  topology: 'adaptive',      // Auto-selects optimal topology
  transport: 'quic',          // 0-RTT connectivity (50-70% faster)
  maxAgents: 10,
  learningEnabled: true
});

// Automatic topology optimization from performance metrics
const optimizer = new SwarmLearningOptimizer();
const optimalConfig = await optimizer.analyzeAndOptimize(swarm.history);

// Result: Self-configuring 3-5x faster execution
console.log(`Recommended topology: ${optimalConfig.topology}`);
console.log(`Expected speedup: ${optimalConfig.speedupFactor}x`);
```

---

## Hooks System

### Hook Types & Execution

The framework implements three categories of middleware hooks:

#### 1. Pre-Task Hooks (Sequential Execution)
Run before task execution for validation and setup:

```typescript
import { hooks } from 'agentic-flow';

await hooks.before('task', async (context) => {
  // Load historical patterns from ReasoningBank
  const patterns = await reasoningbank.queryMemories(context.taskType, {
    namespace: 'patterns',
    limit: 10
  });

  // Initialize agent memory namespace
  await memory.createNamespace(`agent-${context.agentId}`);

  // Validate resource quotas
  if (context.estimatedTokens > context.quotaRemaining) {
    throw new Error('Insufficient token quota');
  }

  // Return enriched context
  return { ...context, historicalPatterns: patterns };
});
```

**Common Pre-Task Use Cases:**
- Auto-assign agents by file type
- Validate commands for safety
- Prepare resources automatically
- Optimize topology by complexity
- Cache searches

#### 2. Post-Task Hooks (Asynchronous Execution)
Run after task completion for learning and cleanup:

```typescript
await hooks.after('task', async (context, result) => {
  // Store successful patterns to ReasoningBank
  if (result.success && result.reward > 0.8) {
    await reasoningbank.storePattern({
      taskType: context.taskType,
      approach: result.approach,
      successRate: result.reward,
      namespace: 'learned-patterns'
    });
  }

  // Update skill library with new solutions
  if (result.reusable) {
    await skillLibrary.createSkill({
      name: result.skillName,
      code: result.implementation,
      successRate: result.reward
    });
  }

  // Trigger downstream agents
  if (context.hasDownstreamTasks) {
    await swarm.spawnAgent({
      type: context.nextAgentType,
      inputContext: result.output
    });
  }
});
```

**Common Post-Task Use Cases:**
- Auto-format code
- Train neural patterns
- Update memory
- Analyze performance
- Track token usage

#### 3. Session Hooks
Manage agent lifecycle and state persistence:

```typescript
// Session start
await hooks.onSessionStart(async (session) => {
  // Restore previous session state
  const previousState = await memory.restore(`session-${session.id}`);

  // Initialize coordination topology
  await swarm.init({
    topology: previousState?.lastTopology || 'adaptive',
    agents: previousState?.activeAgents || []
  });

  return { ...session, restored: true };
});

// Session end
await hooks.onSessionEnd(async (session) => {
  // Generate session summary
  const summary = await analytics.generateSummary(session);

  // Persist state for next session
  await memory.persist(`session-${session.id}`, {
    lastTopology: session.topology,
    activeAgents: session.agents,
    metrics: session.metrics
  });

  // Export workflow for replay
  if (session.exportEnabled) {
    await export.saveWorkflow(session.workflow);
  }

  // Track token usage
  await billing.recordUsage(session.tokenCount);
});
```

### Hook Orchestration CLI

```bash
# Pre-task hook with description
npx agentic-flow@alpha hooks pre-task --description "Implement authentication API"

# Session restoration
npx agentic-flow@alpha hooks session-restore --session-id "swarm-abc123"

# Post-edit notification
npx agentic-flow@alpha hooks post-edit --file "src/api.js" --memory-key "swarm/coder/step-1"

# Broadcast message to swarm
npx agentic-flow@alpha hooks notify --message "API schema updated in memory"

# Post-task cleanup
npx agentic-flow@alpha hooks post-task --task-id "task-xyz"

# Session end with metrics export
npx agentic-flow@alpha hooks session-end --export-metrics true
```

### Agent Coordination Protocol

**Every agent spawned via Task tool MUST follow this protocol:**

**1. BEFORE Work:**
```bash
npx agentic-flow@alpha hooks pre-task --description "Build REST API endpoints"
npx agentic-flow@alpha hooks session-restore --session-id "swarm-dev-001"
```

**2. DURING Work:**
```bash
npx agentic-flow@alpha hooks post-edit --file "src/server.js" --memory-key "swarm/backend/api-schema"
npx agentic-flow@alpha hooks notify --message "API schema stored in memory namespace aqe/api-design"
```

**3. AFTER Work:**
```bash
npx agentic-flow@alpha hooks post-task --task-id "backend-api-001"
npx agentic-flow@alpha hooks session-end --export-metrics true
```

---

## Memory Management

### Three-Layer Architecture

#### Layer 1: ReasoningBank (Semantic Pattern Memory)

**Purpose:** Store and retrieve learned execution patterns with semantic search.

**Key Features:**
- **46% faster execution** after learning cycles
- **Semantic indexing** via vector embeddings (768-dimension default)
- **Namespace isolation** for logical organization
- **TTL management** for automatic cleanup
- **32.6M ops/sec** search performance

**API Usage:**

```typescript
import * as reasoningbank from 'agentic-flow/reasoningbank';

// Store successful pattern
await reasoningbank.storeMemory('api_auth_pattern', {
  taskType: 'authentication',
  approach: 'OAuth2 PKCE flow',
  implementation: 'jwt + refresh tokens',
  successRate: 0.95,
  tokensSaved: 1200
}, {
  namespace: 'api-patterns',
  ttl: 86400 * 30  // 30 days
});

// Query similar patterns
const patterns = await reasoningbank.queryMemories('user authentication', {
  namespace: 'api-patterns',
  limit: 10,
  minSuccessRate: 0.8
});

// Update pattern with new data
await reasoningbank.updateMemory('api_auth_pattern', {
  successRate: 0.97,  // Improved over time
  usageCount: patterns[0].usageCount + 1
});

// Clean up old patterns
await reasoningbank.pruneMemories({
  namespace: 'api-patterns',
  olderThan: Date.now() - (86400 * 90 * 1000)  // 90 days
});
```

**Memory Namespaces (Standard Conventions):**

```typescript
// Agent-specific namespaces
'aqe/test-plan/*'        // Test planning and requirements
'aqe/coverage/*'         // Coverage analysis and gaps
'aqe/quality/*'          // Quality metrics and gates
'aqe/performance/*'      // Performance test results
'aqe/security/*'         // Security scan findings
'aqe/swarm/coordination' // Cross-agent coordination

// Pattern categories
'api-patterns'           // API design patterns
'security-patterns'      // Security best practices
'performance-patterns'   // Optimization techniques
'testing-patterns'       // Test strategies
```

#### Layer 2: AgentDB v2 (Vector Graph Database)

**Purpose:** High-performance vector storage with six cognitive memory patterns.

**Performance Metrics:**
- **150x faster** than SQLite
- **61μs p50 latency** for vector search
- **32.6M ops/sec** pattern search
- **388K ops/sec** pattern storage
- **97.9% self-healing** degradation prevention

**Six Cognitive Memory Patterns:**

##### 1. Reflexion Memory (Self-Critique Learning)

```typescript
import { ReflexionMemory } from 'agentic-flow/agentdb';

const reflexion = new ReflexionMemory(db, embedder);

// Store episode with self-critique
await reflexion.storeEpisode({
  sessionId: 'session-dev-001',
  task: 'Fix authentication bug in OAuth flow',
  action: 'Implemented PKCE extension',
  reward: 0.95,
  success: true,
  critique: 'OAuth2 PKCE provided better security than basic flow. Should be default.',
  learnings: [
    'PKCE prevents authorization code interception',
    'Mobile apps require different OAuth flow than web'
  ]
});

// Retrieve relevant past episodes
const episodes = await reflexion.retrieveRelevant({
  task: 'authentication issues',
  k: 5,
  onlySuccesses: true,
  minReward: 0.8
});

// Learn from failures
const failures = await reflexion.retrieveRelevant({
  task: 'OAuth implementation',
  k: 10,
  onlySuccesses: false,
  maxReward: 0.5  // Get low-performing attempts
});
```

##### 2. Skill Library (Reusable Solutions)

```typescript
import { SkillLibrary } from 'agentic-flow/agentdb';

const skills = new SkillLibrary(db, embedder);

// Create reusable skill
const skillId = await skills.createSkill({
  name: 'jwt_authentication',
  description: 'Generate and validate JWT tokens with refresh token rotation',
  code: `
    async function generateTokenPair(userId) {
      const accessToken = jwt.sign({ userId }, SECRET, { expiresIn: '15m' });
      const refreshToken = jwt.sign({ userId, type: 'refresh' }, REFRESH_SECRET, { expiresIn: '7d' });
      await storeRefreshToken(userId, refreshToken);
      return { accessToken, refreshToken };
    }
  `,
  language: 'javascript',
  domain: 'authentication',
  successRate: 0.92,
  usageCount: 0,
  avgExecutionTime: 45  // milliseconds
});

// Search applicable skills
const applicableSkills = await skills.searchSkills({
  task: 'user authentication with tokens',
  k: 10,
  minSuccessRate: 0.7,
  domain: 'authentication'
});

// Update skill after usage
await skills.updateSkill(skillId, {
  successRate: 0.94,  // Improved
  usageCount: applicableSkills[0].usageCount + 1,
  lastUsed: new Date()
});
```

##### 3. Causal Memory (Intervention-Based Learning)

```typescript
import { CausalMemoryGraph } from 'agentic-flow/agentdb';

const causalMemory = new CausalMemoryGraph(db, embedder);

// Store causal relationship
await causalMemory.addCausalLink({
  cause: 'Added rate limiting to API endpoints',
  effect: 'Reduced server crashes by 85%',
  intervention: 'Implemented token bucket algorithm',
  confidence: 0.92,
  observedInstances: 15
});

// Query causal chains
const causes = await causalMemory.findCauses({
  effect: 'high API response time',
  minConfidence: 0.7
});

// Result: ['database connection pool exhaustion', 'N+1 query pattern', 'missing indexes']
```

##### 4. Explainable Recall (Merkle Proof Certificates)

```typescript
import { ExplainableRecall } from 'agentic-flow/agentdb';

const explainer = new ExplainableRecall(db);

// Retrieve with explanation
const result = await explainer.recallWithProof({
  query: 'best practices for API authentication',
  k: 5
});

// result.proof contains Merkle certificate documenting:
// - Why these results were selected
// - Similarity scores and ranking factors
// - Retrieval timestamp and version
// - Hash chain for verification

console.log(result.explanation);
// "Retrieved based on 0.94 semantic similarity to 'OAuth2 PKCE pattern'
//  stored 2024-12-01, verified via Merkle root 0x7f3a..."
```

##### 5. Utility Ranking (Effectiveness-Based Reranking)

```typescript
import { UtilityRanker } from 'agentic-flow/agentdb';

const ranker = new UtilityRanker(db);

// Search with utility reranking
const results = await ranker.searchAndRerank({
  query: 'API error handling',
  k: 20,
  rerankBy: 'actualSuccessRate',  // Not just similarity
  minUtilityScore: 0.7
});

// Results ranked by real-world effectiveness:
// 1. "Try-catch with detailed logging" (0.95 utility, 0.87 similarity)
// 2. "Custom error classes hierarchy" (0.89 utility, 0.91 similarity)
// 3. "Global error middleware" (0.85 utility, 0.93 similarity)
```

##### 6. Nightly Learner (Background Consolidation)

```typescript
import { NightlyLearner } from 'agentic-flow/agentdb';

const learner = new NightlyLearner(db, embedder);

// Schedule background learning
await learner.scheduleConsolidation({
  frequency: 'daily',
  time: '02:00',  // 2 AM
  tasks: [
    'extract-common-patterns',
    'merge-duplicate-skills',
    'prune-low-utility-memories',
    'update-success-rates',
    'rebuild-causal-graphs'
  ]
});

// Manual trigger
const insights = await learner.runConsolidation();

console.log(insights);
// {
//   patternsExtracted: 12,
//   skillsMerged: 4,
//   memoriesPruned: 87,
//   successRatesUpdated: 156,
//   newCausalLinks: 8
// }
```

#### Layer 3: Session Persistence

**Cross-Session State Management:**

```typescript
import { SessionManager } from 'agentic-flow';

const sessionMgr = new SessionManager(db);

// Save session state
await sessionMgr.saveSession('swarm-dev-001', {
  topology: 'mesh',
  activeAgents: ['coder', 'reviewer', 'tester'],
  memoryNamespaces: ['aqe/test-plan', 'aqe/coverage'],
  taskQueue: [...pendingTasks],
  metrics: {
    tokensUsed: 45000,
    tasksCompleted: 12,
    avgSuccessRate: 0.89
  },
  timestamp: new Date()
});

// Restore session
const session = await sessionMgr.restoreSession('swarm-dev-001');

// Resume from checkpoint
await swarm.init({
  topology: session.topology,
  agents: session.activeAgents,
  resumeFrom: session.taskQueue[0]
});
```

### Batch Operations (Performance Optimization)

Achieve **3-4x performance improvement** through parallel processing:

```typescript
import { BatchOperations } from 'agentic-flow/agentdb';

const batchOps = new BatchOperations(db, embedder, {
  batchSize: 100,
  parallelism: 4  // Concurrent threads
});

// Batch insert skills
await batchOps.insertSkills([
  { name: 'skill1', code: '...', successRate: 0.9 },
  { name: 'skill2', code: '...', successRate: 0.85 },
  // ... 98 more skills
]);

// Batch insert episodes
await batchOps.insertEpisodes([
  { sessionId: 's1', task: 't1', reward: 0.95 },
  // ... 99 more episodes
]);

// Batch pruning
await batchOps.pruneData({
  maxAge: 90,              // Days
  minReward: 0.3,          // Keep only successful patterns
  minSuccessRate: 0.5,
  maxRecords: 100000       // Hard limit
});
```

### Intelligent Caching

**LRU Cache with TTL:**

```typescript
import { MemoryCache } from 'agentic-flow/agentdb';

const cache = new MemoryCache({
  maxSize: 1000,
  ttl: 3600  // 1 hour
});

// Cache statistics queries (8.8x faster: 176ms → ~20ms)
const stats = await cache.getOrCompute('agent-stats', async () => {
  return await db.query('SELECT * FROM agent_metrics WHERE date > ?', [lastWeek]);
});

// Hit rates exceed 80% for frequently accessed data
console.log(cache.getHitRate());  // 0.847
```

---

## Programmatic Integration

### Core Imports

```typescript
// Memory systems
import { ReflexionMemory, SkillLibrary, CausalMemoryGraph } from 'agentic-flow/agentdb';
import * as reasoningbank from 'agentic-flow/reasoningbank';

// Optimization
import { ModelRouter } from 'agentic-flow/router';
import { AgentBooster } from 'agentic-flow/agent-booster';
import { SwarmLearningOptimizer } from 'agentic-flow/hooks/swarm-learning-optimizer';

// Transport
import { QuicTransport } from 'agentic-flow/transport/quic';

// Specialized
import { DataSharingControls } from 'agentic-flow/consent';  // HIPAA-compliant
```

### Complete Workflow Example

```typescript
import { createSwarm, SwarmLearningOptimizer } from 'agentic-flow';
import { ReasoningBank, SkillLibrary } from 'agentic-flow/agentdb';
import { ModelRouter } from 'agentic-flow/router';

async function orchestrateFullStackDevelopment(requirements: string) {
  // 1. Initialize memory systems
  const reasoningBank = new ReasoningBank(db, embedder);
  const skillLibrary = new SkillLibrary(db, embedder);

  // 2. Load historical patterns
  const patterns = await reasoningBank.queryMemories(requirements, {
    namespace: 'fullstack-patterns',
    limit: 10
  });

  // 3. Search for applicable skills
  const skills = await skillLibrary.searchSkills({
    task: requirements,
    k: 20,
    minSuccessRate: 0.7
  });

  // 4. Initialize swarm with learning optimizer
  const swarm = await createSwarm({
    agents: ['backend-dev', 'frontend-dev', 'database-architect', 'tester'],
    topology: 'adaptive',
    transport: 'quic',
    maxAgents: 10,
    learningEnabled: true
  });

  // 5. Distribute tasks
  const tasks = [
    {
      agent: 'backend-dev',
      task: 'Build REST API',
      context: { patterns, skills: skills.filter(s => s.domain === 'backend') }
    },
    {
      agent: 'frontend-dev',
      task: 'Create React UI',
      context: { patterns, skills: skills.filter(s => s.domain === 'frontend') }
    },
    {
      agent: 'database-architect',
      task: 'Design PostgreSQL schema',
      context: { patterns, skills: skills.filter(s => s.domain === 'database') }
    },
    {
      agent: 'tester',
      task: 'Write integration tests',
      context: { patterns, skills: skills.filter(s => s.domain === 'testing') }
    }
  ];

  // 6. Execute with cost optimization
  const router = new ModelRouter();
  const results = await Promise.all(tasks.map(async (task) => {
    return await router.chat({
      model: 'auto',
      priority: 'cost',  // Optimize for cost (85-99% savings)
      messages: [
        { role: 'system', content: `You are a ${task.agent}` },
        { role: 'user', content: `${task.task}\n\nContext: ${JSON.stringify(task.context)}` }
      ]
    });
  }));

  // 7. Learn from execution
  for (const [index, result] of results.entries()) {
    if (result.success) {
      // Store successful pattern
      await reasoningBank.storePattern({
        taskType: tasks[index].task,
        approach: result.approach,
        successRate: result.successRate,
        namespace: 'fullstack-patterns'
      });

      // Extract reusable skill
      if (result.reusableCode) {
        await skillLibrary.createSkill({
          name: result.skillName,
          code: result.reusableCode,
          domain: tasks[index].agent.split('-')[0],
          successRate: result.successRate
        });
      }
    }
  }

  // 8. Optimize topology for next run
  const optimizer = new SwarmLearningOptimizer();
  const optimalConfig = await optimizer.analyzeAndOptimize(swarm.history);

  console.log(`Recommended topology for next run: ${optimalConfig.topology}`);
  console.log(`Expected speedup: ${optimalConfig.speedupFactor}x`);

  return results;
}
```

### Model Router (Cost Optimization)

**85-99% cost savings** across 100+ LLM providers:

```typescript
import { ModelRouter } from 'agentic-flow/router';

const router = new ModelRouter();

// Auto-select cheapest model for task
const response = await router.chat({
  model: 'auto',
  priority: 'cost',  // Options: 'cost', 'quality', 'speed', 'balanced'
  messages: [
    { role: 'user', content: 'Review this code for security issues' }
  ]
});

// Cost comparison example:
// Claude Opus: $240/month for 100 reviews/day
// DeepSeek R1 (auto-selected): $36/month for same workload
// Savings: 85%
```

### Agent Booster (Local Code Transformation)

**350x faster than LLM-based alternatives** (352ms → 1ms per edit):

```typescript
import { AgentBooster } from 'agentic-flow/agent-booster';

const booster = new AgentBooster();

// Apply code transformation
const result = await booster.apply({
  code: 'function add(a, b) { return a + b; }',
  edit: 'function add(a: number, b: number): number { return a + b; }',
  language: 'typescript',
  strategy: 'fuzzy-match'  // Options: 'exact', 'fuzzy-match', 'template'
});

console.log(result.confidence);  // 0.85 (85% confidence)
console.log(result.latency);     // 1ms

// Batch transformations
const batchResult = await booster.applyBatch({
  files: ['src/api.js', 'src/utils.js', 'src/db.js'],
  pattern: 'add-error-handling',
  parallelism: 4
});

// Performance: 12 edits in 12ms vs 4.2 seconds with LLM API
```

### QUIC Transport (High-Performance Networking)

**50-70% faster** than TCP through zero-round-trip-time reconnection:

```typescript
import { QuicTransport } from 'agentic-flow/transport/quic';

const transport = new QuicTransport({
  multiplexing: true,     // Native stream multiplexing
  zeroRTT: true,          // 0-RTT reconnection
  congestionControl: 'bbr'  // Google BBR algorithm
});

// Use in swarm
const swarm = await createSwarm({
  agents: ['agent1', 'agent2'],
  transport: transport
});

// Benefits:
// - 50-70% faster connections
// - Native multiplexing (no head-of-line blocking)
// - Automatic connection migration (survives IP changes)
// - Built-in encryption (TLS 1.3)
```

---

## Package Ecosystem

### Core Packages

#### 1. agentdb
**RuVector-powered graph database with vector search**

**Features:**
- 150x faster than SQLite
- Sub-millisecond latency (61μs p50)
- 32.6M ops/sec pattern search
- Six cognitive memory patterns
- Automatic backend selection (RuVector → HNSWLib → SQLite → sql.js)

**Installation:**
```bash
npm install @agentic-flow/agentdb@alpha
```

**Usage:**
```typescript
import { AgentDB, ReflexionMemory, SkillLibrary } from '@agentic-flow/agentdb';

const db = new AgentDB({ dimension: 768, preset: 'medium' });
const reflexion = new ReflexionMemory(db, embedder);
const skills = new SkillLibrary(db, embedder);
```

#### 2. agent-booster
**High-performance code transformation engine (Rust/WASM)**

**Features:**
- 350x faster than LLM API (352ms → 1ms)
- Zero-cost local processing
- Support for 8 languages
- Template-based and fuzzy matching
- 50-90% confidence scoring

**Installation:**
```bash
npm install @agentic-flow/agent-booster@alpha
```

**Usage:**
```typescript
import { AgentBooster } from '@agentic-flow/agent-booster';

const booster = new AgentBooster();
const result = await booster.apply({
  code: originalCode,
  edit: modifiedCode,
  language: 'typescript'
});
```

#### 3. agentdb-onnx
**ONNX runtime integration for neural networks**

**Features:**
- 27+ neural models
- Graph Neural Network (GNN) learning
- Pattern extraction and consolidation
- Sub-linear algorithms for scaling

**Installation:**
```bash
npm install @agentic-flow/agentdb-onnx@alpha
```

#### 4. agentic-jujutsu
**Change-centric VCS for Kubernetes**

**Features:**
- <100ms reconciliation time
- Granular rollbacks (change-level, not commit-level)
- Production-ready Kubernetes operator
- Change-centric tracking (vs commit-centric)

**Installation:**
```bash
npm install @agentic-flow/agentic-jujutsu@alpha
```

#### 5. agentic-llm
**Multi-model router and optimizer**

**Features:**
- 100+ LLM provider support
- 85-99% cost savings
- Automatic quality-cost optimization
- Provider fallback and retry logic

**Installation:**
```bash
npm install @agentic-flow/agentic-llm@alpha
```

**Usage:**
```typescript
import { ModelRouter } from '@agentic-flow/agentic-llm';

const router = new ModelRouter();
const response = await router.chat({
  model: 'auto',
  priority: 'cost',
  messages: [...]
});
```

---

## Performance & Optimization

### Benchmarks

**Swarm Coordination:**
- 3-5x speedup with auto-optimization
- 32.3% token reduction through learning
- 46% faster execution after learning cycles

**AgentDB Operations:**
| Operation | Speed | Improvement |
|-----------|-------|-------------|
| Pattern search | 32.6M ops/sec | 150x vs SQLite |
| Pattern storage | 388K ops/sec | - |
| Vector query latency | 61μs p50 | Sub-millisecond |
| Statistics queries | ~20ms | 8.8x faster (vs 176ms) |

**Agent Booster Latency:**
| Metric | LLM API | Agent Booster | Improvement |
|--------|---------|---------------|-------------|
| Single edit | 352ms | 1ms | 352x faster |
| 95th percentile | 541ms | 13ms | 41.6x faster |
| 12-edit workflow | 4.2s | 12ms | 350x faster |

**QUIC Transport:**
- 50-70% faster than TCP
- 0-RTT reconnection
- Native multiplexing (no head-of-line blocking)

### Performance Tuning

#### 1. Batch Operations
```typescript
// Slow: Individual inserts
for (const skill of skills) {
  await skillLibrary.createSkill(skill);  // 388K ops/sec
}

// Fast: Batch insert (3-4x improvement)
await batchOps.insertSkills(skills);  // ~1.5M ops/sec
```

#### 2. Caching
```typescript
// Enable intelligent caching
const cache = new MemoryCache({
  maxSize: 1000,
  ttl: 3600
});

// 8.8x faster for repeated queries
const stats = await cache.getOrCompute('key', () => expensiveQuery());
```

#### 3. Topology Selection
```typescript
// Let optimizer choose topology
const optimizer = new SwarmLearningOptimizer();
const config = await optimizer.autoSelectSwarmConfig({
  taskComplexity: 'high',
  agentCount: 8,
  learningHistory: previousRuns
});

// Result: 3-5x speedup vs manual selection
```

#### 4. Model Routing
```typescript
// Optimize for cost (85-99% savings)
const router = new ModelRouter();
const response = await router.chat({
  model: 'auto',
  priority: 'cost'  // Auto-selects DeepSeek R1, Gemini Flash, etc.
});
```

#### 5. Code Transformation
```typescript
// Use Agent Booster for local edits (350x faster)
const booster = new AgentBooster();
const result = await booster.applyBatch({
  files: ['src/**/*.js'],
  pattern: 'modernize-syntax',
  parallelism: 4
});
```

### Scalability

**Agent Capacity:**
- 150+ agents spawned in <2 seconds
- 10+ concurrent agents on t3.small instances
- 100-200MB memory per agent

**Cold Start:**
- <2 seconds including MCP initialization
- Lazy-loaded MCP servers
- Resource pooling within Federation Hub

**Token Efficiency:**
- 32.3% token reduction via swarm coordination
- Shared context through memory systems
- Pattern reuse from ReasoningBank

---

## API Reference

### SwarmLearningOptimizer

```typescript
import { SwarmLearningOptimizer, autoSelectSwarmConfig } from 'agentic-flow/hooks/swarm-learning-optimizer';

const optimizer = new SwarmLearningOptimizer();

// Auto-select optimal topology
const config = await optimizer.autoSelectSwarmConfig({
  taskComplexity: 'high',      // 'low', 'medium', 'high'
  agentCount: 8,
  taskType: 'development',
  learningHistory: [...previousExecutions]
});

// Analyze and optimize from history
const optimalConfig = await optimizer.analyzeAndOptimize(swarm.history);

// Returns:
// {
//   topology: 'mesh',
//   speedupFactor: 3.8,
//   confidence: 0.92,
//   reasoning: 'Mesh optimal for 8 agents with high collaboration'
// }
```

### ReasoningBank

```typescript
import * as reasoningbank from 'agentic-flow/reasoningbank';

// Store memory
await reasoningbank.storeMemory(
  key: string,
  value: any,
  options?: {
    namespace?: string;
    ttl?: number;  // seconds
    metadata?: Record<string, any>;
  }
);

// Query memories
const results = await reasoningbank.queryMemories(
  query: string,
  options?: {
    namespace?: string;
    limit?: number;
    minSuccessRate?: number;
    similarityThreshold?: number;
  }
);

// Update memory
await reasoningbank.updateMemory(
  key: string,
  updates: Partial<any>
);

// Prune old memories
await reasoningbank.pruneMemories(
  options: {
    namespace?: string;
    olderThan?: number;  // timestamp
    minSuccessRate?: number;
  }
);
```

### AgentDB - ReflexionMemory

```typescript
import { ReflexionMemory } from 'agentic-flow/agentdb';

const reflexion = new ReflexionMemory(db, embedder);

// Store episode
await reflexion.storeEpisode({
  sessionId: string;
  task: string;
  action: string;
  reward: number;        // 0-1
  success: boolean;
  critique?: string;
  learnings?: string[];
  metadata?: Record<string, any>;
});

// Retrieve relevant episodes
const episodes = await reflexion.retrieveRelevant({
  task: string;
  k?: number;            // Default: 10
  onlySuccesses?: boolean;
  minReward?: number;
  maxReward?: number;
  sessionId?: string;
});

// Get episode by ID
const episode = await reflexion.getEpisode(episodeId: string);

// Delete episode
await reflexion.deleteEpisode(episodeId: string);
```

### AgentDB - SkillLibrary

```typescript
import { SkillLibrary } from 'agentic-flow/agentdb';

const skills = new SkillLibrary(db, embedder);

// Create skill
const skillId = await skills.createSkill({
  name: string;
  description: string;
  code: string;
  language: string;
  domain: string;
  successRate: number;
  usageCount?: number;
  avgExecutionTime?: number;  // milliseconds
  metadata?: Record<string, any>;
});

// Search skills
const results = await skills.searchSkills({
  task: string;
  k?: number;             // Default: 10
  minSuccessRate?: number;
  domain?: string;
  language?: string;
});

// Update skill
await skills.updateSkill(
  skillId: string,
  updates: Partial<Skill>
);

// Delete skill
await skills.deleteSkill(skillId: string);

// Get skill by ID
const skill = await skills.getSkill(skillId: string);
```

### AgentDB - CausalMemoryGraph

```typescript
import { CausalMemoryGraph } from 'agentic-flow/agentdb';

const causalMemory = new CausalMemoryGraph(db, embedder);

// Add causal link
await causalMemory.addCausalLink({
  cause: string;
  effect: string;
  intervention: string;
  confidence: number;     // 0-1
  observedInstances: number;
  metadata?: Record<string, any>;
});

// Find causes of an effect
const causes = await causalMemory.findCauses({
  effect: string;
  minConfidence?: number;
  k?: number;
});

// Find effects of a cause
const effects = await causalMemory.findEffects({
  cause: string;
  minConfidence?: number;
  k?: number;
});

// Get causal chain
const chain = await causalMemory.getCausalChain({
  from: string;
  to: string;
  maxDepth?: number;
});
```

### ModelRouter

```typescript
import { ModelRouter } from 'agentic-flow/router';

const router = new ModelRouter();

// Chat completion
const response = await router.chat({
  model: 'auto' | string;  // 'auto' for automatic selection
  priority: 'cost' | 'quality' | 'speed' | 'balanced';
  messages: Array<{
    role: 'system' | 'user' | 'assistant';
    content: string;
  }>;
  temperature?: number;
  maxTokens?: number;
  stream?: boolean;
});

// List available models
const models = await router.listModels({
  provider?: string;
  capability?: 'chat' | 'completion' | 'embedding';
});

// Get model info
const info = await router.getModelInfo(modelId: string);
```

### AgentBooster

```typescript
import { AgentBooster } from 'agentic-flow/agent-booster';

const booster = new AgentBooster();

// Apply transformation
const result = await booster.apply({
  code: string;
  edit: string;
  language: 'javascript' | 'typescript' | 'python' | 'rust' | 'go' | 'java' | 'c' | 'cpp';
  strategy?: 'exact' | 'fuzzy-match' | 'template';
});

// Returns:
// {
//   success: boolean;
//   transformedCode: string;
//   confidence: number;     // 0-1
//   latency: number;        // milliseconds
//   strategy: string;
// }

// Batch apply
const batchResult = await booster.applyBatch({
  files: string[];
  pattern: string;
  parallelism?: number;
});

// List available patterns
const patterns = await booster.listPatterns();
```

### Federation Hub

```typescript
import { FederationHub } from 'agentic-flow/federation';

const hub = new FederationHub();

// Start hub
await hub.start({
  port?: number;
  maxAgents?: number;
  agentLifetime?: number;  // seconds (default: 900 = 15 min)
});

// Spawn agents
await hub.spawnAgent({
  type: string;
  config?: Record<string, any>;
  lifespan?: number;  // seconds
});

// Get statistics
const stats = await hub.getStats();

// Stop hub
await hub.stop();
```

---

## Production Features

### Kubernetes Integration

**Agentic-Jujutsu Controller:**

```yaml
apiVersion: agentic.io/v1
kind: AgentSwarm
metadata:
  name: production-swarm
spec:
  topology: adaptive
  agents:
    - type: backend-dev
      replicas: 3
    - type: frontend-dev
      replicas: 2
    - type: tester
      replicas: 2
  deploymentStrategy: blue-green
  reconciliation:
    interval: 100ms
    changeTracking: true
  persistence:
    reasoningBank:
      enabled: true
      storageClass: ssd
      size: 10Gi
    agentDB:
      enabled: true
      storageClass: ssd
      size: 20Gi
```

**Features:**
- <100ms reconciliation time
- Change-centric tracking (granular rollbacks)
- Built-in deployment strategies
- Automatic scaling based on workload

### Billing & Metering

**Five Subscription Tiers:**

| Tier | Agent Hours | Deployments/mo | API Requests | Storage | GPU Hours |
|------|-------------|----------------|--------------|---------|-----------|
| Free | 10 | 5 | 10K | 1GB | - |
| Starter | 100 | 50 | 100K | 10GB | 5 |
| Pro | 1000 | 500 | 1M | 100GB | 50 |
| Team | 10K | Unlimited | 10M | 1TB | 500 |
| Enterprise | Custom | Unlimited | Unlimited | Unlimited | Custom |

**Metered Resources:**
- Agent computation hours
- Deployment operations
- API requests
- Storage (ReasoningBank + AgentDB)
- GPU hours (neural training)
- QUIC transport bandwidth
- Model router API calls
- Agent Booster transformations
- Federation hub agent spawns
- Kubernetes reconciliations

### Healthcare Compliance

**HIPAA-Compliant Platform:**

```typescript
import { DataSharingControls } from 'agentic-flow/consent';

const controls = new DataSharingControls();

// Create data sharing policy
await controls.createPolicy({
  patientId: string;
  allowedProviders: string[];
  dataCategories: Array<'demographics' | 'labs' | 'vitals' | 'medications' | 'notes'>;
  restrictions: Array<{
    type: 'time_based' | 'location_based' | 'role_based';
    config: Record<string, any>;
  }>;
  consentExpiration?: Date;
});

// Check access permission
const canAccess = await controls.checkAccess({
  providerId: string;
  patientId: string;
  dataCategory: string;
  context: {
    timestamp: Date;
    location?: string;
    purpose?: string;
  }
});

// Audit log
const auditLog = await controls.getAuditLog({
  patientId: string;
  startDate: Date;
  endDate: Date;
});
```

**Features:**
- Granular patient data sharing controls
- Consent management framework
- Audit logging (HIPAA-compliant)
- Time-based and role-based access
- Automatic PHI encryption
- Data residency controls

### Enterprise Security

**Built-in Security Features:**
- Input validation (XSS, injection protection)
- TLS 1.3 encryption (QUIC transport)
- Merkle proof certificates (explainable recall)
- Role-based access control (RBAC)
- API key rotation
- Secret management integration
- Audit logging

**Security Best Practices:**

```typescript
import { SecurityManager } from 'agentic-flow/security';

const security = new SecurityManager({
  encryptionAtRest: true,
  encryptionInTransit: true,
  auditLogging: true,
  inputValidation: true,
  rateLimiting: {
    requestsPerMinute: 100,
    tokensPerDay: 1000000
  }
});

// Validate input
const validated = await security.validateInput({
  data: userInput,
  schema: inputSchema,
  sanitize: true
});

// Encrypt sensitive data
const encrypted = await security.encrypt(sensitiveData, {
  algorithm: 'AES-256-GCM',
  keyRotation: true
});

// Audit trail
await security.logAudit({
  action: 'data_access',
  userId: userId,
  resource: resourceId,
  timestamp: new Date()
});
```

---

## Advanced Topics

### Self-Healing Workflows

**97.9% degradation prevention:**

```typescript
import { SelfHealingSwarm } from 'agentic-flow/resilience';

const swarm = new SelfHealingSwarm({
  healthCheck: {
    interval: 5000,  // milliseconds
    timeout: 3000,
    retries: 3
  },
  recovery: {
    autoRestart: true,
    maxRestarts: 5,
    backoffMultiplier: 2
  },
  fallback: {
    enabled: true,
    strategy: 'degrade-gracefully'
  }
});

// Automatic recovery on agent failure
swarm.on('agent-failure', async (agent) => {
  console.log(`Agent ${agent.id} failed, attempting recovery...`);
  await swarm.healAgent(agent.id);
});

// Graceful degradation
swarm.on('degraded', async (reason) => {
  console.log(`Swarm degraded: ${reason}, switching to fallback topology`);
  await swarm.switchTopology('hierarchical');
});
```

### Neural Pattern Training

**27+ neural models for pattern extraction:**

```typescript
import { NeuralTrainer } from 'agentic-flow/neural';

const trainer = new NeuralTrainer({
  models: ['gnn', 'transformer', 'lstm'],
  trainingData: historicalExecutions,
  hyperparameters: {
    learningRate: 0.001,
    batchSize: 32,
    epochs: 100
  }
});

// Train pattern extraction model
const model = await trainer.train({
  task: 'extract-coding-patterns',
  features: ['syntax', 'structure', 'semantics'],
  labels: successfulExecutions
});

// Use trained model for predictions
const prediction = await model.predict({
  input: newCodePattern,
  confidence: 0.8
});
```

### Bottleneck Analysis

```typescript
import { PerformanceAnalyzer } from 'agentic-flow/analytics';

const analyzer = new PerformanceAnalyzer();

// Analyze swarm execution
const analysis = await analyzer.analyzeExecution({
  swarm: swarmInstance,
  metrics: ['latency', 'throughput', 'token-usage', 'success-rate']
});

console.log(analysis.bottlenecks);
// [
//   { type: 'agent-wait', severity: 'high', recommendation: 'increase-parallelism' },
//   { type: 'memory-lookup', severity: 'medium', recommendation: 'enable-caching' }
// ]

// Apply recommendations
await analyzer.applyOptimizations(analysis.recommendations);
```

### Cross-Session Learning

```typescript
import { CrossSessionLearner } from 'agentic-flow/learning';

const learner = new CrossSessionLearner({
  persistenceLayer: 'reasoningbank',
  aggregationStrategy: 'weighted-average',
  learningRate: 0.1
});

// Learn from multiple sessions
await learner.aggregate({
  sessions: ['session-1', 'session-2', 'session-3'],
  extractPatterns: true,
  updateSkills: true,
  mergeCausalGraphs: true
});

// Apply learnings to new session
const optimizations = await learner.getOptimizations({
  taskType: 'api-development',
  confidence: 0.7
});
```

---

## Troubleshooting

### Common Issues

#### 1. Slow Pattern Search

**Symptoms:** ReasoningBank queries taking >100ms

**Solutions:**
```typescript
// Enable caching
const cache = new MemoryCache({ maxSize: 1000, ttl: 3600 });

// Use batch operations
await batchOps.insertSkills(skills);

// Reduce embedding dimension
const db = new AgentDB({ dimension: 384 });  // Instead of 768

// Add indexes
await db.createIndex('namespace', 'successRate');
```

#### 2. High Token Usage

**Symptoms:** Exceeding token quotas

**Solutions:**
```typescript
// Enable model routing (85-99% cost savings)
const router = new ModelRouter({ priority: 'cost' });

// Use Agent Booster for code edits (350x faster, $0 cost)
const booster = new AgentBooster();

// Enable memory reuse
const patterns = await reasoningBank.queryMemories(task);

// Implement caching
const cache = new MemoryCache({ ttl: 3600 });
```

#### 3. Agent Coordination Failures

**Symptoms:** Agents not sharing context

**Solutions:**
```bash
# Verify hooks are running
npx agentic-flow@alpha hooks pre-task --description "test"

# Check memory namespace
npx agentdb@alpha doctor --verbose

# Restore session
npx agentic-flow@alpha hooks session-restore --session-id "swarm-001"

# Verify QUIC transport
npx agentic-flow@alpha transport test
```

#### 4. Memory Bloat

**Symptoms:** AgentDB growing too large

**Solutions:**
```typescript
// Prune old data
await batchOps.pruneData({
  maxAge: 90,
  minReward: 0.3,
  maxRecords: 100000
});

// Enable TTL
await reasoningBank.storeMemory(key, value, { ttl: 86400 * 30 });

// Run nightly consolidation
await learner.runConsolidation();
```

### Debugging

**Enable verbose logging:**

```bash
# Set debug environment variable
export DEBUG=agentic-flow:*

# Run with verbose flag
npx agentic-flow@alpha --verbose federation start

# Check AgentDB health
npx agentdb@alpha doctor --verbose
```

**Performance profiling:**

```typescript
import { Profiler } from 'agentic-flow/diagnostics';

const profiler = new Profiler({ enabled: true });

profiler.start('swarm-execution');
// ... run swarm ...
const metrics = profiler.end('swarm-execution');

console.log(metrics);
// {
//   duration: 2345,  // ms
//   tokenUsage: 45000,
//   apiCalls: 12,
//   memoryLookups: 34,
//   cacheHitRate: 0.82
// }
```

---

## Migration Guide

### From v1.x to v2.0

**Breaking Changes:**

1. **AgentDB API**
   ```typescript
   // v1.x
   await db.storeMemory(key, value);

   // v2.0
   await reasoningBank.storeMemory(key, value, { namespace: 'default' });
   ```

2. **Swarm Initialization**
   ```typescript
   // v1.x
   const swarm = await createSwarm(['agent1', 'agent2']);

   // v2.0
   const swarm = await createSwarm({
     agents: ['agent1', 'agent2'],
     topology: 'adaptive'
   });
   ```

3. **Model Router**
   ```typescript
   // v1.x
   const response = await router.complete(prompt);

   // v2.0
   const response = await router.chat({
     model: 'auto',
     messages: [{ role: 'user', content: prompt }]
   });
   ```

**New Features in v2.0:**

- Agent Booster (350x faster code transformations)
- Six cognitive memory patterns (Reflexion, Skills, Causal, etc.)
- QUIC transport (50-70% faster)
- Self-healing workflows (97.9% uptime)
- Neural pattern training (27+ models)
- Kubernetes controller (<100ms reconciliation)

---

## Resources

### Official Links

- **GitHub Repository:** https://github.com/ruvnet/agentic-flow
- **NPM Package:** https://www.npmjs.com/package/agentic-flow
- **Documentation:** https://github.com/ruvnet/agentic-flow#readme
- **Issues:** https://github.com/ruvnet/agentic-flow/issues
- **Changelog:** https://github.com/ruvnet/agentic-flow/blob/main/CHANGELOG.md

### Community

- **Discord:** [Link if available]
- **Twitter:** @ruvnet
- **Discussions:** https://github.com/ruvnet/agentic-flow/discussions

### Related Projects

- **Flow-Nexus:** Cloud orchestration platform (https://flow-nexus.ruv.io)
- **Claude Agent SDK:** Anthropic's official SDK
- **SPARC Framework:** Systematic TDD methodology
- **RuVector:** High-performance vector database

### Keywords

ai, ai-agents, ai-orchestration, agentic, agents, agent-swarm, anthropic, anthropic-claude, automation, autonomous-agents, byzantine-consensus, claude, claude-agent-sdk, claude-code, reasoning-memory, reasoningbank, agent-learning, memory-system, consensus-protocols, coordination, crdt, devops, distributed-systems, docker, fastmcp, flow-nexus, github, github-integration, goal-planning, goap, hive-mind, huggingface, llm, llm-orchestration, machine-learning, mcp, mcp-server, mcp-tools, memory-persistence, mesh-network, multi-agent, multi-agent-systems, neural-networks, onnx, onnxruntime, orchestration, pagerank, parallel-processing, raft, ruv, ruvnet, sparc, sublinear-algorithms, swarm, swarm-intelligence, task-orchestration, typescript, workflow, workflow-automation

---

## License

MIT/Apache-2.0 dual-licensed

Copyright (c) 2024 @ruvnet

---

## Version Information

- **Package Version:** 2.0.1-alpha.5
- **Latest Stable:** 1.10.2
- **Beta Version:** 1.1.14-beta.1
- **Documentation Updated:** 2025-12-07

**Production Recommendation:** Use v1.x for production workloads. v2.0-alpha is for early adopter testing.

---

**End of Reference Guide**
