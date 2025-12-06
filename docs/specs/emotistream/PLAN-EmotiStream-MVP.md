# EmotiStream Nexus MVP - Implementation Plan

**Document Version**: 1.0
**Created**: 2025-12-05
**Hackathon Duration**: ~70 hours
**Target**: Fully functional MVP with live demo

---

## Executive Summary

This implementation plan breaks down the EmotiStream Nexus MVP into **5 strategic phases** across ~70 hours, with **40 granular tasks**, **parallel work streams**, and **aggressive risk mitigation**. The plan prioritizes critical path items, provides hourly checkpoints, and includes fallback strategies for common blockers.

**Success Definition**: A working system where a user inputs emotional state → receives RL-optimized recommendations → provides feedback → system learns (Q-values update) → repeat shows improvement.

---

## Table of Contents

1. [Implementation Phases](#implementation-phases)
2. [Detailed Task Breakdown](#detailed-task-breakdown)
3. [Critical Path Analysis](#critical-path-analysis)
4. [Parallel Work Streams](#parallel-work-streams)
5. [Risk Mitigation Timeline](#risk-mitigation-timeline)
6. [Hourly Checkpoints](#hourly-checkpoints)
7. [Resource Allocation](#resource-allocation)
8. [Definition of Done](#definition-of-done)
9. [Fallback Plan](#fallback-plan)
10. [Demo Script](#demo-script)

---

## Implementation Phases

### Phase 1: Foundation (Hours 0-8)
**Goal**: Working development environment with all dependencies initialized

**Key Deliverables**:
- Project scaffolding with TypeScript + Node.js
- AgentDB initialized with basic schemas
- RuVector client configured with HNSW index
- Gemini API integration tested
- Mock content catalog (100 items) with emotional profiles
- Basic CLI interface

**Success Criteria**:
- ✅ `npm run dev` starts without errors
- ✅ AgentDB stores/retrieves test data
- ✅ RuVector semantic search returns results
- ✅ Gemini API responds to test emotion analysis
- ✅ Mock content catalog loads successfully

---

### Phase 2: Emotion Detection (Hours 8-20)
**Goal**: Accurate emotion detection from text input via Gemini

**Key Deliverables**:
- Gemini API integration (text → emotion)
- Emotion detector service with retry logic
- Valence-arousal mapping (Russell's Circumplex)
- 8D emotion vector mapping (Plutchik's Wheel)
- State hashing for Q-table lookup
- Unit tests for emotion detection

**Success Criteria**:
- ✅ Text input "I'm stressed" → valence: -0.5, arousal: 0.6
- ✅ Gemini API timeout fallback works
- ✅ Invalid JSON response handled gracefully
- ✅ Emotion detection latency <2s (p95)
- ✅ 10+ emotion detection tests pass

---

### Phase 3: RL Engine (Hours 20-40)
**Goal**: Working Q-learning engine that updates from feedback

**Key Deliverables**:
- Q-learning implementation (state-action-reward)
- State hashing algorithm (discretized valence-arousal-stress)
- Reward function (direction alignment + magnitude)
- Experience replay buffer (AgentDB)
- Policy update logic (TD-learning)
- ε-greedy exploration (15% exploration rate)
- Q-value storage in AgentDB

**Success Criteria**:
- ✅ User feedback → Q-value updates in AgentDB
- ✅ Repeated queries show Q-values increase for good content
- ✅ Exploration vs exploitation balances correctly
- ✅ Reward calculation matches PRD formula
- ✅ Q-value variance <0.05 after 50 simulated experiences

---

### Phase 4: Recommendations (Hours 40-52)
**Goal**: RL + content-based fusion produces ranked recommendations

**Key Deliverables**:
- Content emotional profiling (batch Gemini)
- RuVector semantic search (emotion transition vectors)
- RL policy selection (Q-value + UCB exploration)
- Recommendation ranking (Q-value 70% + similarity 30%)
- GraphQL API endpoints (submitEmotionalInput, emotionalDiscover)
- Post-viewing feedback API (trackEmotionalOutcome)

**Success Criteria**:
- ✅ API returns 20 recommendations in <3s
- ✅ Top recommendations have highest Q-values
- ✅ RuVector search finds relevant content (>0.7 similarity)
- ✅ Feedback loop updates Q-values correctly
- ✅ API responds to all PRD query/mutation specs

---

### Phase 5: Demo & Polish (Hours 52-70)
**Goal**: Polished demo that runs flawlessly for 5 minutes

**Key Deliverables**:
- CLI demo interface (interactive prompts)
- Demo script (7-step flow, 3 minutes)
- Bug fixes and edge case handling
- Documentation (README, API docs)
- Presentation slides (5-minute pitch)
- Rehearsal and timing

**Success Criteria**:
- ✅ Demo runs without crashes (3 full rehearsals)
- ✅ Q-values visibly change after feedback
- ✅ Recommendations improve on second query
- ✅ All 7 demo steps complete in 3 minutes
- ✅ Presentation explains value proposition clearly

---

## Detailed Task Breakdown

### Phase 1: Foundation (Hours 0-8)

| Task ID | Name | Est. Hours | Dependencies | Deliverable | Acceptance Criteria |
|---------|------|------------|--------------|-------------|---------------------|
| **T-001** | Project scaffolding | 1h | None | TypeScript + Node.js boilerplate | `npm run build` succeeds |
| **T-002** | Dependency installation | 0.5h | T-001 | package.json with all deps | `npm install` completes |
| **T-003** | AgentDB initialization | 1.5h | T-002 | AgentDB client + schemas | Store/retrieve test data |
| **T-004** | RuVector setup | 1.5h | T-002 | RuVector client + HNSW index | Semantic search works |
| **T-005** | Gemini API integration | 1.5h | T-002 | Gemini client + test | API responds to test prompt |
| **T-006** | Mock content catalog | 1.5h | T-003, T-004 | 100 items with emotional profiles | Catalog loads in AgentDB |
| **T-007** | Basic CLI interface | 0.5h | T-002 | Inquirer.js prompts | CLI starts and accepts input |

**Total Phase 1**: 8 hours

---

### Phase 2: Emotion Detection (Hours 8-20)

| Task ID | Name | Est. Hours | Dependencies | Deliverable | Acceptance Criteria |
|---------|------|------------|--------------|-------------|---------------------|
| **T-008** | Gemini emotion analysis | 2h | T-005 | Emotion detector service | Text → emotion JSON |
| **T-009** | Valence-arousal mapping | 1.5h | T-008 | Russell's Circumplex mapper | Valence/arousal in [-1, 1] |
| **T-010** | 8D emotion vector | 1.5h | T-008 | Plutchik's Wheel mapper | 8D Float32Array |
| **T-011** | State hashing algorithm | 1h | T-009 | State discretizer | Same state → same hash |
| **T-012** | Error handling | 1.5h | T-008 | Timeout + fallback logic | API timeout → neutral state |
| **T-013** | Emotion detection tests | 2h | T-008-T-012 | Jest tests (10+ cases) | All tests pass |
| **T-014** | Confidence scoring | 1h | T-008 | Confidence calculator | Confidence in [0, 1] |
| **T-015** | Integration with CLI | 1.5h | T-007, T-008 | CLI → emotion detection | User input → emotion output |

**Total Phase 2**: 12 hours

---

### Phase 3: RL Engine (Hours 20-40)

| Task ID | Name | Est. Hours | Dependencies | Deliverable | Acceptance Criteria |
|---------|------|------------|--------------|-------------|---------------------|
| **T-016** | Q-table schema | 1h | T-003, T-011 | AgentDB Q-value storage | Q-values persist |
| **T-017** | Reward function | 2.5h | T-009 | Reward calculator (PRD formula) | Reward in [-1, 1] |
| **T-018** | Q-learning update | 3h | T-016, T-017 | TD-learning implementation | Q-values update correctly |
| **T-019** | Experience replay buffer | 2h | T-003 | Replay buffer in AgentDB | Store experiences |
| **T-020** | ε-greedy exploration | 2h | T-018 | Exploration strategy | 15% random actions |
| **T-021** | UCB exploration | 2h | T-020 | UCB bonus calculation | High uncertainty → bonus |
| **T-022** | Policy selection | 2.5h | T-018, T-021 | Exploit vs explore logic | Select best action |
| **T-023** | Batch policy update | 2h | T-019, T-018 | Batch learning (32 samples) | Q-values converge |
| **T-024** | RL tests | 2h | T-016-T-023 | Jest tests (RL logic) | All tests pass |
| **T-025** | Q-value debugging | 1h | T-018 | Logging + visualization | Q-values visible in CLI |

**Total Phase 3**: 20 hours

---

### Phase 4: Recommendations (Hours 40-52)

| Task ID | Name | Est. Hours | Dependencies | Deliverable | Acceptance Criteria |
|---------|------|------------|--------------|-------------|---------------------|
| **T-026** | Content profiling (batch) | 2h | T-005, T-006 | Gemini batch profiler | Profile 100 items |
| **T-027** | Emotion embeddings | 2h | T-004, T-026 | RuVector embeddings | 100 embeddings in RuVector |
| **T-028** | Transition vector search | 2h | T-004, T-011 | Semantic search query | Top 30 candidates |
| **T-029** | Q-value re-ranking | 1.5h | T-022, T-028 | Hybrid ranking (Q+sim) | Top 20 recommendations |
| **T-030** | GraphQL schema | 1h | T-002 | Type definitions | Schema compiles |
| **T-031** | API resolvers | 2.5h | T-030, T-008, T-029 | Query/mutation logic | API responds correctly |
| **T-032** | Feedback API | 1.5h | T-031, T-018 | trackEmotionalOutcome | Feedback → Q-update |
| **T-033** | API tests | 1.5h | T-031, T-032 | Jest + Supertest | All API tests pass |

**Total Phase 4**: 12 hours

---

### Phase 5: Demo & Polish (Hours 52-70)

| Task ID | Name | Est. Hours | Dependencies | Deliverable | Acceptance Criteria |
|---------|------|------------|--------------|-------------|---------------------|
| **T-034** | CLI demo flow | 3h | T-015, T-031, T-032 | Interactive demo script | Full demo works |
| **T-035** | Q-value visualization | 2h | T-025, T-034 | CLI output formatter | Q-values print nicely |
| **T-036** | Demo rehearsal 1 | 1h | T-034 | Timing + bugs found | <5 min runtime |
| **T-037** | Bug fixes | 4h | T-036 | Bug fixes from rehearsal | Demo stable |
| **T-038** | Demo rehearsal 2 | 1h | T-037 | Polish + timing | <4 min runtime |
| **T-039** | Documentation | 2h | All | README + API docs | Clear setup instructions |
| **T-040** | Presentation slides | 2h | All | 5-min pitch deck | Explains value prop |
| **T-041** | Demo rehearsal 3 (final) | 1h | T-038, T-040 | Final polish | <3.5 min runtime |
| **T-042** | Backup demo video | 2h | T-041 | Pre-recorded video | Fallback if live fails |
| **T-043** | Contingency buffer | 2h | All | Last-minute fixes | Demo ready |

**Total Phase 5**: 18 hours (includes 2h buffer)

---

## Critical Path Analysis

**Critical Path** (tasks that MUST complete on time, zero slack):

```
T-001 → T-002 → T-003 → T-005 → T-008 → T-011 → T-016 → T-018 → T-022 → T-029 → T-031 → T-032 → T-034 → T-041
```

**Critical Path Timeline**:
- **Hour 0**: T-001 (scaffolding)
- **Hour 1**: T-002 (deps)
- **Hour 2-3**: T-003 (AgentDB)
- **Hour 4-5**: T-005 (Gemini)
- **Hour 8-10**: T-008 (emotion detection)
- **Hour 12**: T-011 (state hashing)
- **Hour 20**: T-016 (Q-table schema)
- **Hour 21-24**: T-018 (Q-learning)
- **Hour 27-29**: T-022 (policy selection)
- **Hour 44-46**: T-029 (recommendation ranking)
- **Hour 48-51**: T-031 + T-032 (API + feedback)
- **Hour 52-55**: T-034 (demo flow)
- **Hour 68**: T-041 (final rehearsal)

**Bottleneck Tasks** (high risk, low slack):
- **T-018** (Q-learning): Complex logic, 3h estimate, potential for bugs
- **T-029** (Hybrid ranking): Integration point, depends on RL + RuVector
- **T-034** (Demo flow): Must work end-to-end

---

## Parallel Work Streams

### Stream A: Emotion Detection (Independent)
**Tasks**: T-008 → T-009 → T-010 → T-014 → T-013
**Duration**: 8 hours (Hours 8-16)
**Owner**: Dev 1
**Deliverable**: Emotion detector service with tests

---

### Stream B: Content Profiling (Independent)
**Tasks**: T-026 → T-027
**Duration**: 4 hours (Hours 40-44)
**Owner**: Dev 2
**Deliverable**: 100 content items profiled + embedded in RuVector

---

### Stream C: RL Engine (Depends on Stream A)
**Tasks**: T-016 → T-017 → T-018 → T-019 → T-020 → T-021 → T-022 → T-023 → T-024
**Duration**: 18 hours (Hours 20-38)
**Owner**: Dev 1
**Deliverable**: Working Q-learning policy

---

### Stream D: Recommendation API (Depends on A, B, C)
**Tasks**: T-028 → T-029 → T-030 → T-031 → T-032 → T-033
**Duration**: 11 hours (Hours 40-51)
**Owner**: Dev 2
**Deliverable**: GraphQL API with RL + content fusion

---

### Stream E: Demo UI (Depends on D)
**Tasks**: T-034 → T-035 → T-036 → T-037 → T-038 → T-041
**Duration**: 13 hours (Hours 52-65)
**Owner**: All
**Deliverable**: Polished demo

---

## Risk Mitigation Timeline

| Hour | Risk Check | Mitigation Strategy |
|------|------------|---------------------|
| **8** | Gemini API working? | ✅ Test API with 5 prompts<br>❌ **Fallback**: Mock emotion responses |
| **12** | Emotion detection accurate? | ✅ Manual validation (10 test inputs)<br>❌ **Fallback**: Lower confidence thresholds |
| **20** | AgentDB persisting Q-values? | ✅ Write + read test<br>❌ **Fallback**: In-memory Q-table (lose learning) |
| **30** | RL policy updating Q-values? | ✅ Log Q-values before/after<br>❌ **Fallback**: Random recommendations (no learning) |
| **40** | RL learning visible? | ✅ Simulate 50 experiences, check convergence<br>❌ **Fallback**: Pre-train Q-values from mock data |
| **45** | RuVector search fast enough? | ✅ Benchmark search latency<br>❌ **Fallback**: Reduce topK to 10 |
| **52** | Demo flow working end-to-end? | ✅ Full integration test<br>❌ **Fallback**: Simplify demo (drop post-viewing analysis) |
| **60** | Bugs blocking demo? | ✅ Bug triage, prioritize critical<br>❌ **Fallback**: Feature freeze, polish existing |
| **65** | Demo rehearsal smooth? | ✅ 3rd rehearsal with timer<br>❌ **Fallback**: Pre-record video demo |

---

## Hourly Checkpoints

### Hour 8 Checkpoint (End of Phase 1)
**Expected State**:
- ✅ Project compiles without errors
- ✅ `npm run dev` starts successfully
- ✅ AgentDB initialized (test data stored/retrieved)
- ✅ RuVector client connected (test search works)
- ✅ Gemini API responds (test emotion analysis)
- ✅ Mock content catalog (100 items) loaded

**Go/No-Go Decision**:
- **GO**: All checkboxes ✅ → Proceed to Phase 2
- **NO-GO**: Gemini API failing → Switch to mock emotion responses
- **NO-GO**: AgentDB not working → Use in-memory storage (lose persistence)

---

### Hour 20 Checkpoint (End of Phase 2)
**Expected State**:
- ✅ Emotion detection works for text input
- ✅ Valence/arousal mapped correctly
- ✅ State hashing produces consistent hashes
- ✅ Error handling (timeout, invalid JSON) works
- ✅ 10+ unit tests passing
- ✅ Emotion detection latency <2s

**Go/No-Go Decision**:
- **GO**: All checkboxes ✅ → Proceed to Phase 3
- **NO-GO**: Emotion detection inaccurate → Lower confidence threshold, add manual override
- **NO-GO**: Latency >5s → Cache Gemini responses, reduce prompt complexity

---

### Hour 40 Checkpoint (End of Phase 3)
**Expected State**:
- ✅ Q-table schema in AgentDB
- ✅ Reward function calculates correctly (test cases)
- ✅ Q-values update after feedback
- ✅ Experience replay buffer stores experiences
- ✅ ε-greedy exploration balances correctly
- ✅ RL tests passing (Q-value convergence)

**Go/No-Go Decision**:
- **GO**: All checkboxes ✅ → Proceed to Phase 4
- **NO-GO**: Q-values not updating → Debug TD-learning, simplify to basic Q-learning
- **NO-GO**: Reward function broken → Use simple rating (1-5) instead of emotion delta
- **CRITICAL**: If RL fully broken → **Fallback to content-based filtering only**

---

### Hour 52 Checkpoint (End of Phase 4)
**Expected State**:
- ✅ Content profiling completed (100 items)
- ✅ RuVector embeddings stored
- ✅ Semantic search returns relevant content
- ✅ Hybrid ranking (Q-value + similarity) works
- ✅ GraphQL API endpoints responding
- ✅ Feedback API updates Q-values

**Go/No-Go Decision**:
- **GO**: All checkboxes ✅ → Proceed to Phase 5 (Demo)
- **NO-GO**: RuVector search slow → Reduce topK, use simpler queries
- **NO-GO**: API broken → Use CLI-only demo (skip GraphQL)
- **CRITICAL**: If recommendations not working → **Use mock recommendations**

---

### Hour 65 Checkpoint (End of Demo Development)
**Expected State**:
- ✅ Demo flow works end-to-end (3 full runs)
- ✅ Q-values visibly change in CLI output
- ✅ Recommendations improve on second query
- ✅ Bug fixes applied
- ✅ Documentation complete
- ✅ Presentation slides ready

**Go/No-Go Decision**:
- **GO**: All checkboxes ✅ → Final rehearsal + polish
- **NO-GO**: Demo crashes → Pre-record backup video
- **NO-GO**: Q-values not visible → Hard-code demo Q-values to show learning
- **CRITICAL**: Feature freeze at Hour 65, polish only

---

### Hour 70 Checkpoint (DEMO READY)
**Expected State**:
- ✅ Demo rehearsed 3 times without crashes
- ✅ Demo runtime <3.5 minutes
- ✅ Presentation explains value proposition clearly
- ✅ Backup video recorded (if needed)
- ✅ Team ready to present

**Definition of Done**: MVP is complete when all 7 demo steps run successfully for 5 minutes without crashes.

---

## Resource Allocation

### Solo Developer Plan (Recommended)
**Total Time**: 70 hours (critical path + buffer)

**Hour 0-8**: Foundation (scaffolding, deps, AgentDB, RuVector, Gemini, mock catalog)
**Hour 8-20**: Emotion Detection (Gemini integration, valence-arousal, tests)
**Hour 20-40**: RL Engine (Q-learning, reward, policy, tests)
**Hour 40-52**: Recommendations (content profiling, RuVector search, API)
**Hour 52-70**: Demo & Polish (CLI demo, bug fixes, rehearsal, slides)

**Focus Strategy**:
- **Hours 0-40**: 100% focus on critical path (no distractions)
- **Hours 40-52**: Parallel content profiling + API development
- **Hours 52-65**: Integration + demo flow
- **Hours 65-70**: Polish + rehearsal only (no new features)

---

### Team Plan (2-3 Developers)

**Dev 1: RL & Core Logic** (40 hours)
- Phase 1: AgentDB setup (Hours 0-3)
- Phase 2: Emotion detection (Hours 8-16)
- Phase 3: RL engine (Hours 20-38)
- Phase 5: Demo integration (Hours 52-60)

**Dev 2: Content & API** (35 hours)
- Phase 1: RuVector setup (Hours 0-3)
- Phase 1: Mock catalog (Hours 4-6)
- Phase 4: Content profiling + embeddings (Hours 40-44)
- Phase 4: GraphQL API (Hours 45-51)
- Phase 5: API testing (Hours 52-55)

**Dev 3: Infrastructure & Demo** (30 hours)
- Phase 1: Project scaffolding (Hours 0-2)
- Phase 2: Error handling + tests (Hours 12-16)
- Phase 3: RL tests (Hours 36-38)
- Phase 5: CLI demo + visualization (Hours 52-65)
- Phase 5: Presentation + rehearsal (Hours 66-70)

**Handoff Points**:
- **Hour 16**: Dev 1 → Dev 3 (emotion detection module ready for testing)
- **Hour 38**: Dev 1 → Dev 2 (RL policy ready for API integration)
- **Hour 51**: Dev 2 → Dev 3 (API ready for CLI demo)

---

## Definition of Done

### MVP is DONE when these 7 criteria are met:

1. ✅ **User Input**: User can input text emotional state via CLI
2. ✅ **Emotion Detection**: System detects emotion via Gemini (valence, arousal, emotion vector)
3. ✅ **Recommendations**: System recommends 20 content items based on emotion + RL policy
4. ✅ **Feedback**: User can provide post-viewing feedback (rating 1-5 or emoji)
5. ✅ **RL Update**: Q-values update in AgentDB after feedback
6. ✅ **Learning**: Repeat query shows Q-values changed (learning visible)
7. ✅ **Demo Stability**: Demo runs for 5 minutes without crashes (3 successful rehearsals)

### Additional Success Metrics (Nice-to-Have):

- ✅ Emotion detection accuracy >70% (manual validation on 10 test inputs)
- ✅ Q-value convergence after 50 simulated experiences (variance <0.05)
- ✅ Recommendation latency <3s (p95)
- ✅ RL policy outperforms random baseline (mean reward >0.6 vs 0.3)

---

## Fallback Plan

### If Behind Schedule (Aggressive Scope Reduction):

#### **Hour 30**: Drop Post-Viewing Emotion Analysis
**Trigger**: RL engine not working by Hour 30
**Action**: Use simple rating (1-5) instead of emotion delta for reward
**Impact**: Still demonstrates RL learning, simpler reward function
**Time Saved**: 3 hours

---

#### **Hour 45**: Drop RuVector Semantic Search
**Trigger**: RuVector search not working or too slow
**Action**: Use random content selection + RL re-ranking only
**Impact**: Recommendations less relevant, but RL still learns
**Time Saved**: 4 hours

---

#### **Hour 55**: Drop GraphQL API
**Trigger**: API integration broken, demo not working
**Action**: CLI-only demo with direct function calls
**Impact**: Demo still shows all features, just no API
**Time Saved**: 5 hours

---

#### **Hour 60**: Simplify Demo Script
**Trigger**: Demo flow too complex, crashes frequent
**Action**: Reduce to 3 steps: input → recommend → feedback
**Impact**: Minimal viable demo, still shows learning
**Time Saved**: 3 hours

---

#### **Hour 65**: Pre-Record Demo Video
**Trigger**: Live demo unstable, high crash risk
**Action**: Record 3-minute video walkthrough as backup
**Impact**: No live demo risk, less impressive but safe
**Time Saved**: Reduces presentation stress

---

## Demo Script

### 7-Step Demo Flow (Target: 3 minutes)

**Setup**: Pre-loaded mock content catalog (100 items), fresh Q-table (no history)

---

#### **[00:00-00:30] Step 1: Introduction**
**Script**:
> "EmotiStream Nexus is an emotion-driven recommendation system that learns what content actually improves your mood. Unlike Netflix or YouTube, which optimize for watch time, we optimize for emotional wellbeing using reinforcement learning."

**Action**: Show title slide

---

#### **[00:30-01:00] Step 2: Emotional Input**
**Script**:
> "I just finished a stressful workday. Let me tell the system how I feel."

**CLI Interaction**:
```
> How are you feeling? (describe in your own words)
User: "I'm exhausted and stressed after a long day"

> Analyzing your emotional state...
Detected Emotion: Sadness/Stress
Valence: -0.6 (negative)
Arousal: 0.4 (moderate)
Stress Level: 0.8 (high)
```

**Action**: Paste pre-written input for consistency

---

#### **[01:00-01:30] Step 3: Desired State Prediction**
**Script**:
> "The system predicts I want to feel calm and positive, not more stressed."

**CLI Output**:
```
> Predicting desired emotional state...
Desired Valence: +0.5 (positive)
Desired Arousal: -0.3 (calm)
Confidence: 0.7 (learned from similar users)
```

**Action**: Show prediction logic (heuristic: stressed → calm)

---

#### **[01:30-02:00] Step 4: Recommendations**
**Script**:
> "Here are content recommendations optimized for my emotional transition from stressed to calm."

**CLI Output**:
```
Top 5 Recommendations (Ranked by RL Policy + Content Match):

1. "Nature Sounds: Ocean Waves" (Q-value: 0.0, Similarity: 0.89)
   Emotional Profile: Calming (valence: +0.4, arousal: -0.5)

2. "Planet Earth: Forests" (Q-value: 0.0, Similarity: 0.85)
   Emotional Profile: Uplifting nature (valence: +0.5, arousal: -0.3)

3. "The Great British Bake Off" (Q-value: 0.0, Similarity: 0.78)
   Emotional Profile: Cozy comfort (valence: +0.6, arousal: 0.0)
```

**Note**: Q-values are 0 (no learning yet), similarity drives ranking

---

#### **[02:00-02:30] Step 5: Viewing & Feedback**
**Script**:
> "After watching 'Ocean Waves', I feel much calmer. Let me give feedback."

**CLI Interaction**:
```
> You selected: "Nature Sounds: Ocean Waves"
> (Simulating viewing... completed)

> How do you feel now?
User: "Much better, very calm"

> Analyzing post-viewing emotional state...
Post-Viewing Valence: +0.5 (positive) ✅
Post-Viewing Arousal: -0.4 (calm) ✅
Emotional Improvement: +1.1 (large improvement)
```

**Action**: Paste pre-written feedback

---

#### **[02:30-02:45] Step 6: RL Learning**
**Script**:
> "The system calculates a reward and updates its Q-value for this recommendation."

**CLI Output**:
```
> Calculating reward...
Direction Alignment: 0.92 (moved toward desired state)
Improvement Magnitude: 1.1
Proximity Bonus: 0.18 (reached desired state)
Total Reward: +0.88 🎯

> Updating Q-value...
Previous Q-value: 0.0
New Q-value: +0.088 (learning rate: 0.1)
Experience stored in replay buffer.
```

**Action**: Show Q-value update in AgentDB (log output)

---

#### **[02:45-03:00] Step 7: Demonstrating Learning**
**Script**:
> "Now when I repeat the same emotional state, the system recommends 'Ocean Waves' higher because it learned it works for me."

**CLI Interaction**:
```
> How are you feeling? (describe in your own words)
User: "Stressed again after another long day"

> Top 5 Recommendations:

1. "Nature Sounds: Ocean Waves" (Q-value: 0.088 ⬆️, Similarity: 0.89)
   ^ Ranked #1 because RL learned it works for me!

2. "Planet Earth: Forests" (Q-value: 0.0, Similarity: 0.85)
3. "The Great British Bake Off" (Q-value: 0.0, Similarity: 0.78)
```

**Key Insight**: Q-value increased from 0.0 → 0.088, so "Ocean Waves" now ranks higher!

---

### Demo Timing Breakdown:
- **Step 1**: 30 sec (intro)
- **Step 2**: 30 sec (input emotion)
- **Step 3**: 30 sec (predict desired state)
- **Step 4**: 30 sec (show recommendations)
- **Step 5**: 30 sec (feedback)
- **Step 6**: 15 sec (Q-value update)
- **Step 7**: 15 sec (show learning)

**Total**: 3 minutes

---

### Demo Rehearsal Checklist:

**Before Demo**:
- ✅ Fresh AgentDB (Q-values = 0)
- ✅ Mock content catalog loaded (100 items)
- ✅ Gemini API key set
- ✅ CLI prompts pre-written (copy-paste ready)
- ✅ Backup video recorded (if live demo fails)

**During Demo**:
- ✅ Paste inputs quickly (no typing)
- ✅ Highlight Q-value changes (point at screen)
- ✅ Explain "learning" clearly (not just "Q-value increased")

**After Demo**:
- ✅ Answer questions about RL, Gemini, AgentDB

---

## Appendix: Technology Stack

### Core Dependencies:
- **Language**: TypeScript (Node.js 20+)
- **Emotion Detection**: Gemini 2.0 Flash API
- **RL Storage**: AgentDB (Q-tables, experiences, user profiles)
- **Semantic Search**: RuVector (HNSW index, 1536D embeddings)
- **API**: GraphQL (Apollo Server)
- **CLI**: Inquirer.js + Chalk
- **Testing**: Jest + Supertest

### File Structure:
```
emotistream-mvp/
├── src/
│   ├── emotion/
│   │   ├── detector.ts          # Gemini emotion analysis
│   │   ├── mapper.ts            # Valence-arousal mapping
│   │   └── state.ts             # State hashing
│   ├── rl/
│   │   ├── q-learning.ts        # Q-learning engine
│   │   ├── reward.ts            # Reward function
│   │   ├── policy.ts            # Policy selection
│   │   └── replay.ts            # Experience replay
│   ├── content/
│   │   ├── profiler.ts          # Content emotional profiling
│   │   ├── embeddings.ts        # RuVector embeddings
│   │   └── catalog.ts           # Mock content catalog
│   ├── recommendations/
│   │   ├── ranker.ts            # Hybrid ranking (Q + sim)
│   │   └── search.ts            # RuVector search
│   ├── api/
│   │   ├── schema.ts            # GraphQL schema
│   │   ├── resolvers.ts         # Query/mutation resolvers
│   │   └── server.ts            # Apollo Server
│   ├── cli/
│   │   ├── demo.ts              # CLI demo interface
│   │   └── prompts.ts           # Inquirer prompts
│   └── db/
│       ├── agentdb.ts           # AgentDB client
│       └── ruvector.ts          # RuVector client
├── tests/
│   ├── emotion.test.ts
│   ├── rl.test.ts
│   ├── recommendations.test.ts
│   └── api.test.ts
├── package.json
├── tsconfig.json
└── README.md
```

---

## Success Metrics

### MVP Success (Hour 70):
- ✅ **Demo Stability**: 3 successful rehearsals without crashes
- ✅ **RL Learning**: Q-values visibly change after feedback
- ✅ **Recommendation Quality**: Top recommendation has highest Q-value
- ✅ **Latency**: Emotion detection <2s, recommendations <3s
- ✅ **Accuracy**: Emotion detection manually validated (10 test cases)

### Post-Hackathon Goals (Optional):
- 🎯 **Beta Users**: 50 users, 200 experiences
- 🎯 **Mean Reward**: ≥0.60 (vs random baseline 0.30)
- 🎯 **Convergence**: Q-values stabilize after 100 experiences (variance <0.05)

---

**End of Implementation Plan**

**Last Updated**: 2025-12-05
**Status**: Ready for execution
**Next Steps**: Begin Phase 1 (Hour 0) → Project scaffolding
