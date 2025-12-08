# EmotiStream Nexus MVP - SPARC Specification Package

**Generated**: 2025-12-05
**SPARC Phase**: 1 - Specification
**Hackathon Duration**: ~70 hours
**Status**: Ready for Implementation

---

## Quick Start

```bash
# Start implementation immediately
cd /workspaces/hackathon-tv5

# Review the implementation plan first
cat docs/specs/emotistream/PLAN-EmotiStream-MVP.md

# Then follow the critical path in order
```

---

## Specification Documents

| Document | Purpose | Read When |
|----------|---------|-----------|
| [SPEC-EmotiStream-MVP.md](./SPEC-EmotiStream-MVP.md) | **Feature specifications** - What to build | First - understand scope |
| [ARCH-EmotiStream-MVP.md](./ARCH-EmotiStream-MVP.md) | **Architecture design** - How it fits together | Second - understand design |
| [PLAN-EmotiStream-MVP.md](./PLAN-EmotiStream-MVP.md) | **Implementation plan** - Hour-by-hour tasks | Third - plan your work |
| [API-EmotiStream-MVP.md](./API-EmotiStream-MVP.md) | **API contracts** - Endpoint & data models | During implementation |

---

## MVP Scope Summary

### Included (P0 - Must Have)
- Text-based emotion detection via Gemini API
- Q-learning RL recommendation engine
- Content emotional profiling (200 items)
- Post-viewing feedback & reward calculation
- AgentDB persistence (Q-tables, profiles)
- RuVector semantic search
- CLI demo interface

### Excluded (Defer to Phase 2)
- Voice/biometric emotion detection
- Full web/mobile UI
- Wellbeing crisis detection
- Multi-platform content integration (Netflix, YouTube)
- Advanced RL (actor-critic, prioritized replay)
- A/B testing framework

---

## Time Budget (70 Hours)

```
Phase 1: Foundation        ████░░░░░░  8 hours   (Hour 0-8)
Phase 2: Emotion Detection █████████░ 12 hours   (Hour 8-20)
Phase 3: RL Engine         ████████████████████ 20 hours (Hour 20-40)
Phase 4: Recommendations   █████████░ 12 hours   (Hour 40-52)
Phase 5: Demo & Polish     ████████████████████ 18 hours (Hour 52-70)
```

---

## Critical Path

```
Setup → Gemini API → Emotion Detector → Q-Learning → Reward Function →
→ Content Profiling → RuVector → Recommendation Engine → API Layer →
→ CLI Demo → Integration Tests → Demo Rehearsal → PRESENTATION
```

---

## Checkpoints

| Hour | Checkpoint | Go/No-Go Criteria |
|------|------------|-------------------|
| 8 | Foundation | Project compiles, Gemini connected |
| 20 | Emotion Detection | Text → valence/arousal works |
| 40 | RL Engine | Q-values update on feedback |
| 52 | Recommendations | End-to-end flow works |
| 65 | Demo Ready | 5-minute demo without crashes |
| 70 | **PRESENTATION** | Rehearsed 3 times |

---

## Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| Runtime | Node.js 20+ / TypeScript | Core application |
| AI | Gemini 2.0 Flash Exp | Emotion detection |
| Vector DB | RuVector (HNSW) | Semantic search |
| Persistence | AgentDB | Q-tables, profiles |
| API | Express REST | Endpoints |
| Demo | CLI (Inquirer.js) | Interactive demo |

---

## Demo Flow (3 minutes)

```
1. [00:00] "EmotiStream Nexus predicts content for emotional wellbeing"
2. [00:30] User inputs: "I'm feeling stressed after work"
3. [01:00] System detects: Valence -0.5, Arousal 0.6, Stress 0.7
4. [01:15] Predicts desired state: Calm & Positive (Valence 0.5, Arousal -0.2)
5. [01:30] Shows 5 recommendations with Q-values
6. [02:00] User watches "Ocean Waves" → Feedback: "Much better!"
7. [02:15] Shows Q-value update: 0.0 → 0.08 (learning!)
8. [02:30] Next session: "Ocean Waves" now ranks #1 (improvement!)
9. [02:45] Closing: "RL learns what content improves YOUR emotional state"
```

---

## Success Criteria

| Metric | Target | Measurement |
|--------|--------|-------------|
| Emotion detection | ≥70% accuracy | Gemini classification |
| RL improvement | 0.3 → 0.6 mean reward | After 15 experiences |
| Q-value convergence | Variance <0.1 | Last 20 updates |
| Demo stability | 5 min no crashes | 3 rehearsals |
| Recommendation latency | <3 seconds | End-to-end |

---

## Fallback Plan

| Trigger | Action |
|---------|--------|
| Hour 30 behind | Drop post-viewing emotion analysis, use 1-5 rating |
| Hour 45 behind | Drop RuVector, use mock recommendations |
| Hour 55 behind | Drop API, CLI-only demo |
| Hour 65 behind | Feature freeze, use pre-recorded demo backup |

---

## Quick Reference

### Core API Endpoints

```bash
# Detect emotion
curl -X POST http://localhost:3000/api/v1/emotion/detect \
  -H "Content-Type: application/json" \
  -d '{"userId": "demo", "text": "I am feeling stressed"}'

# Get recommendations
curl -X POST http://localhost:3000/api/v1/recommend \
  -H "Content-Type: application/json" \
  -d '{"userId": "demo", "emotionalStateId": "state_123"}'

# Submit feedback
curl -X POST http://localhost:3000/api/v1/feedback \
  -H "Content-Type: application/json" \
  -d '{"userId": "demo", "contentId": "content_ocean", "emotionalStateId": "state_123", "postViewingState": {"explicitRating": 5}}'
```

### Key Data Models

```typescript
interface EmotionalState {
  valence: number;    // -1 to +1
  arousal: number;    // -1 to +1
  stressLevel: number; // 0 to 1
}

interface QTableEntry {
  stateHash: string;
  contentId: string;
  qValue: number;
}
```

---

## Next Steps

1. **Read SPEC** → Understand what we're building
2. **Read ARCH** → Understand how it fits together
3. **Read PLAN** → Follow the hour-by-hour tasks
4. **Use API doc** → Reference during implementation
5. **Build MVP** → Follow critical path
6. **Demo** → Rehearse 3 times before presentation

---

## Team Allocation (if multiple developers)

| Developer | Responsibilities | Hours |
|-----------|-----------------|-------|
| **Dev 1** | Emotion Detection + Content Profiling | 20h |
| **Dev 2** | RL Engine + Recommendations | 26h |
| **Dev 3** | API Layer + CLI Demo | 20h |
| **All** | Integration + Demo Prep | 4h |

---

**Good luck with the hackathon!**

*Generated by SPARC Specification Swarm*
