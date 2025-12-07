# Media Discovery Platform PRD
## TV5MONDE Hackathon - Cold-Start Solution

---

## 1. Product Vision & Value Proposition Matrix

### Product Vision

Transform the streaming cold-start problem from a 45-minute friction point into a 45-second competitive advantage. By observing micro-behaviors instead of asking questions, we create instant user profiles that feel like magic—not interrogation.

### Value Proposition Matrix

| Stakeholder | Problem | Our Solution | Measurable Impact |
|-------------|---------|--------------|-------------------|
| **End User** (Overwhelmed Browser) | Wastes 20+ minutes browsing, defaults to safe rewatches | Profile built in <60 seconds through passive observation | 95% reduction in time-to-first-relevant-recommendation |
| **TV5MONDE Business** | 40% trial abandonment during cold-start; licensed content underutilized | Higher conversion, better catalog surfacing | +15% trial-to-paid conversion; +30% catalog coverage in recommendations |
| **TV5MONDE Brand** | Can't outspend Netflix on content | Out-personalize on niche (francophone) audiences; "respects your time" positioning | Defensible brand identity in crowded market |
| **TV5MONDE Strategy** | Dependent on commoditized metadata providers | Proprietary behavioral signals + francophone content embeddings | Owned recommendation intelligence; strategic asset |

### Core Thesis

> *Traditional platforms ask. We observe.*
> *Questionnaires take minutes and feel like homework. Attention takes seconds and feels like browsing.*

---

## 2. Success Criteria & Primary User Persona

### Success Criteria

| Priority | Metric | Target | Measurement |
|----------|--------|--------|-------------|
| **Primary** | Speed vs. traditional onboarding | 95% faster than questionnaire-based flows | Side-by-side comparison: our 45 sec vs. competitor 5-10 min |
| Secondary | Time-to-profile | < 60 seconds | Timer from first interaction to recommendation display |
| Secondary | Profile resonance | 80%+ "this feels right" | Post-recommendation user confirmation prompt |
| Secondary | Feed engagement | First click within 10 seconds | Time from feed display to first content selection |
| Diagnostic | Behavioral signal capture | 4+ signals per user | Hover, scroll, hesitation, return-visit, time-to-first-interaction |

### Primary User Persona: The Overwhelmed Browser

**Marie, 42** — Marketing Director, Lyon

*Demographics:* Two children (8, 11). Partner works evenings. Dual income, time-poor.

*Viewing context:* 90-minute window after kids sleep (21:30–23:00). Tired but wants entertainment, not effort.

*Current behavior:*
- Opens streaming app with intent to watch something new
- Scrolls for 15-20 minutes, overwhelmed by choice
- Anxiety rises: "I'm wasting my limited time"
- Defaults to rewatching The Office or abandons entirely
- Feels frustrated, not relaxed

*What she needs:*
- Confidence that recommendations are *for her*, not generic
- Speed—every minute browsing is a minute not watching
- No cognitive load—don't make her think or answer questions

*Success for Marie:*
> "I opened the app, glanced at a few images, and within a minute I was watching something I actually enjoyed. I didn't even realize I was being profiled."

---

## 3. User Journey

### Cold-Start Discovery Flow (3 Steps)

```
┌─────────────────────────────────────────────────────────────────────────┐
│  STEP 1: ENTRY              STEP 2: DISCOVERY           STEP 3: FEED   │
│  ───────────────            ────────────────            ─────────────   │
│                                                                         │
│  "Discover Your             8 visual thumbnails         "Curated for   │
│   Next Watch"               NO LABELS - images only      You"           │
│                                                                         │
│  [Start Exploring] ──────►  User browses naturally ───► Personalized   │
│                             System captures:             results with   │
│                             • Hover duration             genre labels   │
│                             • Micro-hesitations          NOW revealed   │
│                             • Return visits                             │
│                             • Scroll velocity            Top 2 genres   │
│                             • Time-to-first-interact     highlighted    │
│                                                                         │
│                             Timer visible: 0.000s        "45 seconds.   │
│                                                           Not 45 mins." │
│                                                                         │
│  Duration: 2 sec            Duration: 30-60 sec         Duration: ∞    │
└─────────────────────────────────────────────────────────────────────────┘
```

### Moment-by-Moment Experience

| Timestamp | User Action | System Response | Signal Captured |
|-----------|-------------|-----------------|-----------------|
| 0:00 | Clicks "Start Exploring" | Grid appears, timer starts | — |
| 0:03 | Eyes scan grid | — | Time-to-first-interaction begins |
| 0:05 | Mouse moves toward dark, moody image | — | Approach velocity |
| 0:06 | Mouse slows, hovers | Subtle border glow (no label) | Micro-hesitation detected |
| 0:08 | Moves away without clicking | Border fades | Interest without commitment logged |
| 0:12 | Returns to same image | — | **Return visit** (high signal) |
| 0:14 | Clicks | Click badge appears: "1" | Explicit selection |
| 0:18 | Scrolls past bright/comedic image quickly | — | High scroll velocity = rejection |
| 0:25 | Hovers on warm-toned image, clicks | Click badge: "2" | Second preference |
| 0:35 | Continue button appears | — | Threshold reached |
| 0:38 | Clicks "See My Recommendations" | Feed loads with labels revealed | Profile complete |

### Key Design Decisions

1. **No labels during profiling** — Pure System 1 reaction, 100x faster than reading
2. **Labels revealed in results** — "Ah, I was drawn to thrillers and drama"
3. **Timer visible** — Creates awareness of speed, reinforces value prop
4. **Subtle visual feedback only** — Border glow on hover, no text

### The "Reveal" Moment

When the user sees results, labels appear for the first time:
> *"Your Top Picks: Thriller & Drama"*

This creates a small delight: "The system understood me without asking."

---

## 4. MVP Features (Hackathon)

### Scope Philosophy

> Ship what works. The prototype already demonstrates the core innovation. Polish the narrative, not the feature set.

### Current Prototype Capabilities (Keep)

| Feature | Status | Purpose |
|---------|--------|---------|
| 3-screen flow (Entry → Discovery → Results) | ✅ Built | Core user journey |
| 8-genre thumbnail grid | ✅ Built | Visual profiling surface |
| Hover time tracking | ✅ Built | Primary behavioral signal |
| Click sequence capture | ✅ Built | Explicit preference ranking |
| Mouse velocity tracking | ✅ Built | Skip/rejection detection |
| Real-time timer display | ✅ Built | Value prop reinforcement |
| Admin widget (leaderboard) | ✅ Built | Demo/pitch tool for judges |
| Personalized feed generation | ✅ Built | Proof that profiling works |
| "Time saved" badge | ✅ Built | Dramatic comparison anchor |

### Hackathon Enhancements (Small tweaks)

| Enhancement | Effort | Impact |
|-------------|--------|--------|
| Remove genre labels from discovery grid | 30 min | Purer behavioral signal |
| Add labels to results screen only | 30 min | Creates "reveal" moment |
| Refine thumbnail selection for visual contrast | 1 hr | Better differentiation between moods |
| Add "micro-hesitation" detection to widget | 1 hr | Shows technical sophistication |

### Explicitly NOT in MVP

| Feature | Why Not |
|---------|---------|
| Real TMDB content | Adds complexity, Unsplash works for demo |
| Video/trailer playback | Different interaction model, Phase 2 |
| User accounts/persistence | Not needed for cold-start demo |
| Mobile/TV interface | Desktop proves concept, device adaptation is Phase 2 |
| AgentDB integration | Post-hackathon continuous profiling |

### Demo Script (5 minutes)

1. **Hook (30 sec):** "How long does it take Netflix to understand you? 45 minutes of watch history. We do it in 45 seconds."
2. **Problem (60 sec):** Introduce Marie. Show the scrolling paralysis problem.
3. **Live Demo (120 sec):** Walk through the 3-step flow. Let timer run visibly.
4. **Reveal (30 sec):** Show personalized feed. Point to admin widget data.
5. **Vision (60 sec):** Phase 2 roadmap. TV + Mobile. Continuous profiling.

---

## 5. Phase 2 Roadmap (Post-Hackathon)

### Phase 2 Vision

> Cold-start is solved. Now make it *continuous*. Every session refines the profile. Every interaction deepens understanding.

### Phase 2A: Continuous Profiling with AgentDB

| Capability | Implementation | User Benefit |
|------------|----------------|--------------|
| Behavioral embedding storage | AgentDB vector persistence per user | Profile survives sessions |
| Session-over-session refinement | Delta vectors merged with historical profile | "Gets smarter every time I use it" |
| Decay modeling | Recent behavior weighted higher | Adapts to mood changes, life changes |
| Cross-content signals | Track behavior on actual content, not just discovery | Deeper than initial profiling |

**Technical Architecture:**
```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  User Session   │────►│  Signal Capture  │────►│    AgentDB      │
│  (any device)   │     │  (hover, scroll, │     │  (vector store) │
└─────────────────┘     │   watch time)    │     └────────┬────────┘
                        └──────────────────┘              │
                                                          ▼
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  Personalized   │◄────│  Recommendation  │◄────│  Profile Vector │
│  Feed           │     │  Engine          │     │  (accumulated)  │
└─────────────────┘     └──────────────────┘     └─────────────────┘
```

### Phase 2B: Visual Similarity Search (Thumbnail Embeddings)

| Capability | Implementation | User Benefit |
|------------|----------------|--------------|
| Thumbnail/still frame embeddings | CLIP or similar vision model | "More like this" by visual mood |
| Color palette matching | Extract dominant colors, match to preferences | Warm-tone lovers get warm content |
| Composition analysis | Frame structure, character placement | Visual storytelling preferences |
| Poster vs. still frames | Use actual movie frames, not marketing posters | Authentic visual representation |

**User Interaction:**
> User hovers long on a dark, desaturated thriller image
> System learns: "prefers cool palette, high contrast, isolated figures"
> Future recommendations prioritize visually similar content—even across genres

### Phase 2C: Emotional Arc Matching

| Capability | Implementation | User Benefit |
|------------|----------------|--------------|
| Narrative shape embeddings | LLM-extracted story arcs (rags-to-riches, tragedy, redemption) | "Find content with similar emotional journey" |
| Mood trajectory mapping | Beginning → middle → end emotional states | Match viewer's desired emotional experience |
| Theme knowledge graphs | Neo4j: Movie → Theme → Related concepts | Deep thematic connections |

**Example:**
> User loved *Amélie* (whimsical → melancholy → hopeful)
> System finds: *The Secret Life of Walter Mitty*, *Paddington*, *Chef*
> Not genre-matched (different categories), but emotionally resonant

### Phase 2 Priority Order

| Phase | Focus | Dependency | Timeline |
|-------|-------|------------|----------|
| 2A | AgentDB continuous profiling | Hackathon MVP | Month 1-2 |
| 2B | Visual similarity search | 2A (needs storage) | Month 2-3 |
| 2C | Emotional arc matching | 2B (needs embeddings infra) | Month 3-4 |

---

## 6. Technical Architecture (Millisecond Monitoring)

### Architecture Philosophy

> Capture everything. Process in real-time. Store intelligently. The behavioral fingerprint is the product.

### Signal Capture Layer

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         BEHAVIORAL SIGNALS                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │   HOVER     │  │   SCROLL    │  │ HESITATION  │  │   RETURN    │    │
│  │   TIME      │  │  VELOCITY   │  │  DETECTION  │  │   VISITS    │    │
│  ├─────────────┤  ├─────────────┤  ├─────────────┤  ├─────────────┤    │
│  │ ms per item │  │ px/s scan   │  │ deceleration│  │ item revisit│    │
│  │ cumulative  │  │ direction   │  │ threshold   │  │ count + gap │    │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘    │
│                                                                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │   TIME TO   │  │   CLICK     │  │   SKIP      │  │   DWELL     │    │
│  │   FIRST     │  │  SEQUENCE   │  │  PATTERNS   │  │   DEPTH     │    │
│  ├─────────────┤  ├─────────────┤  ├─────────────┤  ├─────────────┤    │
│  │ ms to first │  │ order + gap │  │ rapid-pass  │  │ time in     │    │
│  │ interaction │  │ between     │  │ items       │  │ viewport    │    │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘    │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Signal Weights & Scoring

| Signal | Weight | Rationale |
|--------|--------|-----------|
| Return visit to same item | 3.0x | Strongest intent indicator—came back |
| Click (explicit selection) | 2.5x | Conscious commitment |
| Micro-hesitation (>500ms slowdown) | 2.0x | Subconscious interest |
| Hover duration (>1000ms) | 1.5x | Sustained attention |
| Scroll deceleration near item | 1.2x | Peripheral interest |
| Time-to-first-interaction | 1.0x | Baseline confidence signal |
| High scroll velocity past item | -1.5x | Active rejection |
| Zero hover on item | -1.0x | Passive disinterest |

### Real-Time Processing Pipeline

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Browser    │     │   Event      │     │   Signal     │     │   Profile    │
│   Events     │────►│   Debounce   │────►│   Processor  │────►│   Builder    │
│   (60fps)    │     │   (16ms)     │     │   (weights)  │     │   (ranking)  │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
       │                                                              │
       │  mousemove, scroll,                                          │
       │  mouseenter, mouseleave,                                     ▼
       │  click, timestamp                                   ┌──────────────┐
       │                                                     │   Genre      │
       └─────────────────────────────────────────────────────│   Leaderboard│
                                                             │   (live)     │
                                                             └──────────────┘
```

### Latency Targets

| Stage | Target | Rationale |
|-------|--------|-----------|
| Event capture | <16ms | Match browser frame rate |
| Signal processing | <50ms | Real-time widget updates |
| Profile generation | <100ms | Instant on "Continue" click |
| Feed rendering | <200ms | Perceived as instantaneous |
| **Total cold-start** | **<60 seconds** | Primary success metric |

### Data Schema (MVP)

```javascript
UserSession {
  sessionId: string
  startTime: timestamp

  signals: {
    [itemId]: {
      hoverTime: number (ms)
      hoverCount: number
      clickOrder: number | null
      returnVisits: number
      hesitations: number (count of slowdowns)
      scrollVelocityPast: number (px/s, if skipped)
    }
  }

  meta: {
    totalTime: number (ms)
    timeToFirstInteraction: number (ms)
    avgMouseVelocity: number (px/s)
    totalHovers: number
    totalClicks: number
    skipEvents: number
  }

  output: {
    rankedGenres: [{ id, score }]
    profileConfidence: number (0-1)
  }
}
```

### Phase 2 Schema Extension (AgentDB)

```javascript
UserProfile {
  userId: string
  embedding: Float32Array (behavioral vector)

  sessions: SessionSummary[]

  preferences: {
    visualStyle: { palette, contrast, composition }
    emotionalArc: { trajectory, intensity }
    genreAffinities: { [genre]: score }
  }

  decay: {
    lastUpdated: timestamp
    sessionCount: number
    confidenceScore: number
  }
}
```

---

## 7. Interface Evolution (5-Year Horizon)

### Device Priority

| Priority | Device | User Context | Current State |
|----------|--------|--------------|---------------|
| **Primary** | Smart TV | Lean-back, 10-foot UI, shared viewing | Target platform |
| **Primary** | Mobile | Thumb-driven, vertical, personal | Target platform |
| Secondary | Desktop | Mouse precision, work breaks | Prototype platform |

### Challenge: Signal Capture Without a Mouse

The prototype leverages cursor tracking—rich, precise, millisecond data. TV and mobile have no cursor. How do we maintain signal quality?

---

### Smart TV Interface (Lean-Back)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           SMART TV DISCOVERY                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│    ┌───────┐  ┌───────┐  ┌───────┐  ┌───────┐                          │
│    │       │  │       │  │ ████  │  │       │   ◄── D-pad focus        │
│    │   1   │  │   2   │  │   3   │  │   4   │       (highlighted)      │
│    │       │  │       │  │       │  │       │                          │
│    └───────┘  └───────┘  └───────┘  └───────┘                          │
│                                                                         │
│    ┌───────┐  ┌───────┐  ┌───────┐  ┌───────┐                          │
│    │       │  │       │  │       │  │       │                          │
│    │   5   │  │   6   │  │   7   │  │   8   │                          │
│    │       │  │       │  │       │  │       │                          │
│    └───────┘  └───────┘  └───────┘  └───────┘                          │
│                                                                         │
│                        [OK to select]                                   │
└─────────────────────────────────────────────────────────────────────────┘
```

**TV Signal Capture:**

| Signal | Desktop Equivalent | TV Implementation |
|--------|-------------------|-------------------|
| Hover duration | Mouse hover time | **Dwell time** on D-pad selection |
| Micro-hesitation | Mouse slowdown | **Pause before pressing OK** |
| Return visits | Mouse returns to item | **Navigate back** to same item |
| Scroll velocity | Mouse speed past | **D-pad repeat speed** (rapid clicks = skip) |
| Skip patterns | Fast mouse movement | **Items passed without pause** |

**TV-Specific Signals:**

| New Signal | Capture Method | Meaning |
|------------|----------------|---------|
| Direction bias | Track up/down/left/right patterns | Spatial preference mapping |
| Voice hesitation | "Play... um..." pause before command | Uncertainty indicator |
| Co-viewing detection | Multiple "OK" presses in quick succession | Shared decision-making mode |

---

### Mobile Interface (Thumb-Driven)

```
┌─────────────────────┐
│                     │
│   ┌─────────────┐   │
│   │             │   │
│   │     1       │   │  ◄── Full-width card
│   │             │   │
│   └─────────────┘   │
│                     │
│   ┌─────────────┐   │
│   │             │   │
│   │     2       │   │  ◄── Scroll to reveal
│   │             │   │
│   └─────────────┘   │
│                     │
│   ┌─────────────┐   │
│   │             │   │  ◄── Vertical swipe
│   │     3       │   │       feed pattern
│   │             │   │
│   └─────────────┘   │
│                     │
│        ●○○○○        │  ◄── Progress dots
└─────────────────────┘
```

**Mobile Signal Capture:**

| Signal | Desktop Equivalent | Mobile Implementation |
|--------|-------------------|----------------------|
| Hover duration | Mouse hover time | **Thumb pause** on item |
| Micro-hesitation | Mouse slowdown | **Scroll deceleration** near item |
| Return visits | Mouse returns to item | **Scroll back** to previous item |
| Scroll velocity | Mouse speed | **Swipe velocity** (px/s) |
| Skip patterns | Fast mouse movement | **Rapid swipe-through** |

**Mobile-Specific Signals:**

| New Signal | Capture Method | Meaning |
|------------|----------------|---------|
| Hold-to-preview | Long press triggers preview | Strong curiosity |
| Swipe direction | Up vs. left swipe | Different intent (dismiss vs. skip) |
| Scroll-back depth | How far back they scroll | How much they regret passing |
| Session time-of-day | Timestamp | Commute vs. evening mood |

---

### Desktop Interface (Current Prototype)

Remains the richest signal source. Used for:
- Development and testing
- Power users / work-from-home viewers
- Benchmarking signal quality across devices

---

### 5-Year Interface Roadmap

| Year | Milestone | Interface Focus |
|------|-----------|-----------------|
| 2024 | Hackathon MVP | Desktop prototype proves concept |
| 2025 | Production launch | Smart TV + Mobile primary |
| 2026 | Cross-device sync | Unified profile across all devices |
| 2027 | Ambient signals | Room lighting, time-of-day, co-viewing detection |
| 2028 | Eye-tracking | Smart TV cameras, gaze data |
| 2029 | Predictive pre-loading | System queues content before user decides |

### Emerging: Eye-Tracking (3-5 Year Bet)

| Platform | Status | Opportunity |
|----------|--------|-------------|
| Smart TV cameras | Emerging (Samsung, LG) | Living room eye-tracking |
| Mobile eye-tracking | 2-3 years | Front camera gaze estimation |
| VR/AR headsets | Growing market | True gaze data, immersive viewing |

**Eye-tracking replaces cursor as the gold standard:**
- Where you look = what you consider
- How long you look = interest level
- What you skip visually = rejection
- Pupil dilation = emotional response (advanced)

---

## 8. Success Metrics

### Metrics Hierarchy

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         NORTH STAR METRIC                               │
│                                                                         │
│              Time-to-Relevant-Content: < 60 seconds                     │
│                                                                         │
│         "From app open to watching something you'll enjoy"              │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┼───────────────┐
                    ▼               ▼               ▼
            ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
            │   SPEED     │ │  ACCURACY   │ │ ENGAGEMENT  │
            │   METRICS   │ │  METRICS    │ │  METRICS    │
            └─────────────┘ └─────────────┘ └─────────────┘
```

### Speed Metrics (Proving the "45 seconds" claim)

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Cold-start profile time | < 60 sec | Timer: first interaction → recommendations shown |
| Comparison vs. questionnaire | 95% faster | Benchmark against Netflix/competitor onboarding |
| Time-to-first-click on feed | < 10 sec | Timer: feed display → first content selection |
| Discovery grid engagement | 30-90 sec | Healthy range—too fast = random, too slow = confusion |

### Accuracy Metrics (Proving profiles are correct)

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Self-reported resonance | 80%+ "this feels right" | Post-feed micro-survey (optional) |
| Top-2 genre match | 70%+ correct | Compare predicted vs. actual watched content |
| Feed click-through rate | > 40% | Clicks on recommended content / total recommendations |
| Content completion rate | > 60% | Users finish what they started watching |

### Engagement Metrics (Proving business value)

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Trial-to-paid conversion | +15% vs. baseline | A/B test: with vs. without cold-start system |
| Session duration | +20% vs. baseline | Time spent in app per session |
| Return rate (7-day) | > 50% | Users who return within 7 days |
| Catalog coverage | +30% | Unique titles surfaced in recommendations |

### Diagnostic Metrics (System health)

| Metric | Target | Purpose |
|--------|--------|---------|
| Signals captured per session | 4+ distinct signals | Data richness indicator |
| Profile confidence score | > 0.7 | Internal validity check |
| Genre distribution in recommendations | No single genre > 50% | Avoid filter bubbles |
| Edge case handling | < 5% "no signal" sessions | Users who don't interact enough |

### Anti-Metrics (What we don't optimize for)

| Anti-Metric | Why We Ignore It |
|-------------|------------------|
| Time spent browsing | We want *less* browsing, more watching |
| Number of recommendations shown | Quality over quantity |
| Scroll depth | Deep scrolling = failure, not engagement |
| Session count | Fewer, more satisfying sessions > many frustrating ones |

### Hackathon Demo Metrics (What judges will see)

| Visible Metric | Display Location | Purpose |
|----------------|------------------|---------|
| Live timer | Discovery screen | Proves speed claim in real-time |
| Genre leaderboard | Admin widget | Shows signal capture working |
| Behavioral signals count | Admin widget | Technical sophistication |
| "Time saved" badge | Results screen | Dramatic comparison anchor |

---

## 9. Risks & Mitigations

### Risk Matrix

| Risk | Severity | Likelihood | Priority |
|------|----------|------------|----------|
| C: Cold-start for cold-start | High | High | **Critical** |
| Signal quality on TV/mobile | High | Medium | High |
| Privacy perception | Medium | Medium | Medium |
| Content catalog dependency | Medium | Low | Low |
| Competitor response | Low | High | Low |

---

### CRITICAL: Cold-Start for Cold-Start

*The system itself needs data to tune signal weights. How do you bootstrap before you have user behavior data?*

**The Problem:**
```
┌─────────────────────────────────────────────────────────────────────────┐
│  CHICKEN-AND-EGG                                                        │
│                                                                         │
│  Good recommendations ◄─── Accurate signal weights                      │
│         │                          ▲                                    │
│         │                          │                                    │
│         ▼                          │                                    │
│  Users engage ──────────────► Behavioral data                           │
│                                                                         │
│  But without good recommendations, users won't engage...                │
└─────────────────────────────────────────────────────────────────────────┘
```

**Mitigation Strategies:**

| Strategy | Approach | Trade-off |
|----------|----------|-----------|
| **1. Expert priors** | Start with human-curated signal weights based on UX intuition | May not match real user behavior |
| **2. Simulation testing** | Generate synthetic behavior patterns, tune weights | Fake data ≠ real data |
| **3. Soft launch cohort** | Beta users provide initial data, refine before wide launch | Slower rollout |
| **4. Transfer learning** | Use published research on visual attention, hover behavior | Generic, not domain-specific |
| **5. A/B weight testing** | Run parallel weight configurations, measure outcomes | Requires traffic volume |

**Recommended Approach:**

```
Phase 1 (Hackathon): Expert priors
├── Hover time: 1.5x weight (established UX pattern)
├── Return visits: 3.0x weight (strong intent signal)
├── Click sequence: 2.5x weight (explicit commitment)
└── Skip velocity: -1.5x weight (rejection indicator)

Phase 2 (Soft launch): Validate with real users
├── 500-1000 beta users
├── Compare predicted vs. actual viewing behavior
├── Adjust weights based on correlation data
└── Build confidence intervals per signal

Phase 3 (Production): Continuous learning
├── AgentDB stores outcomes (did they watch? complete? return?)
├── Feedback loop refines weights per user segment
└── Eventually: personalized weight profiles
```

---

### HIGH: Signal Quality on TV/Mobile

*Can you capture enough behavioral data without mouse precision?*

**The Problem:**
- Desktop: 60fps cursor tracking, sub-pixel precision
- TV: D-pad navigation, binary (selected / not selected)
- Mobile: Touch scrolling, less granular than cursor

**Mitigation Strategies:**

| Strategy | Approach |
|----------|----------|
| **Compensate with time** | TV/mobile may need 90 sec vs. desktop's 45 sec |
| **Device-specific weights** | Dwell time more important on TV; swipe velocity more important on mobile |
| **Richer visual stimuli** | Larger thumbnails, higher contrast, more distinct choices |
| **Supplementary signals** | Time-of-day, session frequency, content completion history |
| **Graceful degradation** | If signal quality low, fall back to broader recommendations |

**Signal Quality Targets by Device:**

| Device | Minimum Signals | Profile Confidence Target |
|--------|-----------------|---------------------------|
| Desktop | 6+ | > 0.8 |
| Mobile | 4+ | > 0.7 |
| Smart TV | 3+ | > 0.6 |

---

### MEDIUM: Privacy Perception

*"You're tracking my every micro-movement" feels creepy to some users.*

**The Problem:**
- Millisecond tracking sounds invasive when described technically
- GDPR/privacy regulations require transparency
- Trust is essential for francophone audience

**Mitigation Strategies:**

| Strategy | Implementation |
|----------|----------------|
| **Frame as "attention, not surveillance"** | "We notice what catches your eye" vs. "We track your mouse" |
| **Transparency without overwhelm** | Simple explanation: "We personalize based on what you browse" |
| **No PII in behavioral data** | Signals are anonymous; tied to session, not identity |
| **Opt-out available** | "Skip personalization" option (falls back to editorial picks) |
| **Data locality** | Process on-device where possible; minimize server-side storage |

**Messaging Framework:**

> "We learn your taste by watching what catches your attention—no questions asked"

---

### LOW: Content Catalog Dependency

*Recommendations are only as good as the content library.*

**The Problem:**
- If TV5MONDE's catalog has genre gaps, no algorithm fixes that
- User profiled as "thriller lover" but catalog is 80% drama

**Mitigation:**
- Acknowledge limitation in pitch: "Algorithm surfaces the best of what exists"
- Phase 2 opportunity: "Identifies catalog gaps based on unmet user demand"
- Hybrid curation: Human editors fill gaps algorithm can't solve

---

### LOW: Competitor Response

*Netflix/Prime could copy this in 6 months if it works.*

**Why It's Low Priority:**

| Moat | Defensibility |
|------|---------------|
| **Speed-to-value brand** | Identity, not feature—hard to copy positioning |
| **Francophone specialization** | Netflix won't invest in niche optimization |
| **Editorial + algorithm hybrid** | Requires human taste, not just engineering |
| **Behavioral signal IP** | Specific weights/combinations are proprietary |
| **First-mover in segment** | TV5MONDE owns the narrative for their audience |

---

## 10. Future-Proofing

### Two Moats Strategy

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         DEFENSIBILITY FRAMEWORK                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   MOAT 1: BRAND IDENTITY              MOAT 2: OPERATIONAL ADVANTAGE     │
│   ──────────────────────              ─────────────────────────────     │
│                                                                         │
│   "The platform that                  Algorithm suggests,               │
│    respects your time"                editors refine                    │
│                                                                         │
│   ┌─────────────────────┐             ┌─────────────────────┐          │
│   │  Not a feature.     │             │  Human taste +      │          │
│   │  An identity.       │             │  machine scale      │          │
│   │                     │             │                     │          │
│   │  Features get       │             │  Competitors can    │          │
│   │  copied.            │             │  copy algorithm,    │          │
│   │  Positioning        │             │  not your curators. │          │
│   │  persists.          │             │                     │          │
│   └─────────────────────┘             └─────────────────────┘          │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

### Moat 1: Speed-to-Value as Brand

**The Positioning:**
> *"Other platforms make you work for entertainment. We get you watching in under a minute."*

**Why This Persists:**

| Factor | Explanation |
|--------|-------------|
| Emotional resonance | Time-poor users *feel* this problem daily |
| Simple to communicate | "45 seconds vs. 45 minutes" is instantly understood |
| Hard to out-claim | Competitors would need to prove faster—we set the benchmark |
| Compounds over time | Every satisfied user reinforces the reputation |

**Brand Execution:**

| Touchpoint | Message |
|------------|---------|
| App Store description | "Find your next watch in under a minute" |
| Onboarding | Timer visible—proving the claim live |
| Marketing | "We respect your time" campaign |
| Word-of-mouth | Users share: "It just knew what I wanted" |

---

### Moat 2: Hybrid AI + Editorial Curation

**The Model:**

```
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│   ALGORITHM LAYER                    EDITORIAL LAYER                    │
│   ───────────────                    ──────────────                     │
│                                                                         │
│   • Behavioral profiling             • Thematic collections             │
│   • Signal processing                • Cultural context                 │
│   • Similarity matching              • Quality curation                 │
│   • Personalization at scale         • "Editor's Picks" trust signal    │
│                                                                         │
│                        ┌───────────┐                                    │
│                        │  HYBRID   │                                    │
│                        │  OUTPUT   │                                    │
│                        └───────────┘                                    │
│                              │                                          │
│                              ▼                                          │
│              "Personalized AND trustworthy"                             │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

**Why Competitors Can't Copy This:**

| Component | Netflix/Prime | TV5MONDE |
|-----------|---------------|----------|
| Algorithm | Better resourced | Sufficient for niche |
| Scale | Billions of data points | Smaller dataset |
| Francophone expertise | Not a priority | Core competency |
| Editorial taste | Algorithmic-first culture | Curatorial heritage |
| Niche audience trust | Generic global brand | Cultural authority |

**Editorial Integration Points:**

| Stage | Algorithm Role | Editor Role |
|-------|----------------|-------------|
| Discovery thumbnails | Select from catalog | Approve visual quality |
| Recommendation ranking | Score by behavioral fit | Boost hidden gems |
| Collection creation | Cluster similar content | Name and contextualize |
| New content onboarding | Cold-start embedding | Editorial positioning |

---

### Technology Future-Proofing

| Trend | Preparation | Timeline |
|-------|-------------|----------|
| **Multimodal AI** | Embeddings architecture supports text, image, video | Ready now |
| **On-device ML** | Signal processing can run client-side | 1-2 years |
| **Real-time personalization** | AgentDB enables instant profile updates | Phase 2 |
| **Voice interfaces** | Behavioral signals from voice hesitation, queries | 2-3 years |
| **Spatial computing** | Gaze tracking as primary signal | 3-5 years |

**Architecture Principles for Longevity:**

| Principle | Implementation |
|-----------|----------------|
| **Signal-agnostic core** | New signals (gaze, voice) plug into existing weight system |
| **Embedding-first storage** | Any content type becomes a vector—future-proof format |
| **Device-abstracted capture** | Signal capture layer per device; processing layer shared |
| **Modular recommendation engine** | Swap algorithms without rebuilding infrastructure |

---

### Strategic Roadmap Summary

| Horizon | Focus | Moat Built |
|---------|-------|------------|
| **Now (Hackathon)** | Prove cold-start solved in 45 seconds | Speed claim established |
| **6 months** | Production on TV + Mobile | Brand identity launched |
| **1 year** | Continuous profiling with AgentDB | Data asset accumulating |
| **2 years** | Visual + emotional embeddings | Technical differentiation |
| **5 years** | Hybrid AI/editorial at scale | Operational moat cemented |

---

## Appendix: Original Brainstorm Notes

*The following are the original ideation notes that led to this PRD:*

Extending our team brainstorm I ended up into the following ideas that could be answer to the Hackathon's challenge. Let me approach it zoomed out, with a brand strategy perspective:
- step 1: platforms should position its brand. Offer movies to already filtered audience (peeps know what they expect)
- reduced decision fatigue - we are halfway solved the problem
- step 2, next stage: communicate how the given media platform is DIFFERENT than the competition (example "we've got the best rom coms") -
- the positioning needs to be solved, not just serve ANY audience segment
- before the algorithms kick in, do manual curation
- but as we discussed, we don't have user profiles after they signed up (cold start). the key, we don't need to categorize them IMMEdiately. content-based filtering remains underutilized. ColdRAG could be a solution too, no questionnaires needed
- first, we group them by asking few simple questions (fav actor, genre etc)
- we now have segments, then monitor the views and likes and fine tune the algo - not YET personalised, but segmented (these steps are potentally obvious for the team :)
- next question: how do we avoid user churn? we need feedback loop (but not star reviews)
- we might not show the complete media library, just a a few recommendation
- we also display manual curation on the UI - this is the differentiator. Trust kicks in
The above use case is for a niche platform, not serving millions of people. Hybrid data/human curation.

To move away from legacy metadata providers, we need semantic understanding. This is where out embeddings-first approach comes in:
- the missing items are: thematic depth, style, cinematography, mood, tone, emotional trajectory etc
- we miss explainability: Users want to know WHY something is recommended
- 3 embedding approaches: 1. Emotional arc embeddings (rags to riches, etc) "find content with a similar emotional journey" discovery https://github.com/nchibana/moviearcs
- 2. Theme extraction with LLMs - Neo4j: Movie → Theme → Stem knowledge graphs
- 3. recco on visual similarity of thumbnails: embeddings of posters/still frames - colour palette, composition, visual mood etc.
- we are working already on our  Zero-Shot Mood-Based Discovery!
- "I want something visually stunning and emotionally complex"
- for thumbnails, TMDB library is not enough, we need stills/actual frames
- what's innovative could be the json schema based prompting with Gemini Pro 3 (nano banana)

I tested and it's working almost pixel perfect. Segment: Romance-seekers
→ Emphasize: Lead chemistry, warm color palette, intimate framing
→ De-emphasize: Action sequences, ensemble cast shots

Segment: Thriller fans
→ Emphasize: Tension moments, cool/dark palette, isolated protagonist
→ De-emphasize: Romantic subplot, comedic beats

Metadata is commoditized. Everyone has identical inputs. The transformation layer is the entire game.

Moving away from librarian brain to 'salesperson' or movie curator brain: "Position this as escapism for stressed professionals"
"Surface this when user has 90min window"
"Lead with actor X for segment Y, director Z for segment W"

What if we applied behavioral observation method, we'd show curated clips (trailers lie), we'd measure watch-through, replay, skip speed, 10 clips are enough. Or even purely thumbnails > so under a minute, we nailed the cold start problem. YouTube has trailer/clip data API.

The learning: embedding the clips + tracking

Winner:
- demoable
- can be done hybrid thumbnail generation and using exsiting video data etc
- solves the hackathon's original problem  (2 hours down to 2 mins)
- new idea (behavioral observation applied to movie discovery)
- sellable - TV5 can't do this themselves
- it enriches inputs, not replaces outputs
- no proprietary data needed
- Demonstrable value — show same movie, different contexts, different positioning
