# LinkedIn Post: Media Discovery Platform

---

## POST 1: Marketing-Focused (Hook + Story)

---

**45 seconds.**

That's how long it takes to understand your movie taste.

Not 45 minutes of questionnaires.
Not weeks of watch history.
Not scrolling paralysis wondering what to watch.

---

We just built something for the TV5MONDE Global Hackathon that challenges how streaming platforms think about the cold-start problem.

**The insight:** Traditional platforms ASK you what you like. We OBSERVE.

When you land on a new streaming service, you're typically faced with:
- "Rate these 20 movies" (homework)
- "Select your favorite genres" (guessing)
- Generic recommendations until you've watched enough

Our approach: Show you 8 visual thumbnails. No labels. No text. Just images.

Then we watch what YOU watch:
- Where does your cursor slow down?
- Which image do you hover on longest?
- What do you click first, second, third?
- What do you scroll past quickly?

In under 60 seconds, we've built a behavioral profile that feels like magic.

The timer runs live on screen. Users see exactly how fast it happens.

"12 seconds to profile you. Not 45 minutes."

---

**Why this matters for TV5MONDE:**

Streaming is a brutal market. Netflix has 200M+ subscribers worth of data. How does a niche francophone platform compete?

Not on content volume. Not on recommendation algorithm sophistication.

On SPEED and RESPECT.

"We respect your time" is a brand identity, not a feature. Features get copied. Positioning persists.

---

**Technical highlights for the builders:**

- Real-time behavioral signal capture at 60fps
- Zustand state management for millisecond-precision tracking
- Weighted scoring: return visits (3x), clicks (2.5x), micro-hesitations (2x)
- Next.js 15 with App Router
- Portrait thumbnails (9:16) for visual impact
- TMDB API for content data

The prototype is live. The thesis is proven.

---

Huge thanks to the Agentics Foundation London team for the sprint.

What's next? AgentDB integration for persistent profiles. Visual similarity search with CLIP embeddings. Emotional arc matching.

The cold-start problem is solved. Now we make it continuous.

---

#AI #Hackathon #Streaming #ProductDesign #UX #TV5MONDE #AgenticsFoundation

---

## POST 2: Technical Deep-Dive

---

**We solved the streaming cold-start problem in 45 seconds. Here's the technical breakdown.**

The TV5MONDE Global Hackathon challenge: How do you personalize content for a brand new user with zero watch history?

Traditional approaches:
- Questionnaires (high friction, low completion)
- Collaborative filtering (needs existing data)
- Content-based filtering (generic until trained)

Our approach: **Behavioral micro-signal capture.**

---

**The Architecture:**

```
Browser Events (60fps)
        │
        ▼
   Event Debounce (16ms)
        │
        ▼
   Signal Processor (weighted)
        │
        ▼
   Profile Builder (genre ranking)
        │
        ▼
   Personalized Feed
```

**Signals we capture:**

| Signal | Weight | Why |
|--------|--------|-----|
| Return visit | 3.0x | User came BACK to this thumbnail |
| Explicit click | 2.5x | Conscious selection |
| Micro-hesitation | 2.0x | Cursor slowed >500ms |
| Hover duration | 1.5x | >1000ms sustained attention |
| Skip velocity | -1.5x | Rapid scroll = rejection |

---

**The Key Insight:**

No genre labels during profiling. Pure visual reaction.

System 1 thinking (fast, intuitive) is 100x faster than System 2 (reading, analyzing).

When you see a dark, moody thumbnail vs. a bright, colorful one—your cursor tells us everything before you consciously decide.

Labels are revealed AFTER profiling. Creates an "aha moment": "The system understood me without asking."

---

**Stack:**

- Next.js 15 (App Router)
- Zustand (state with immer)
- Tailwind CSS
- TMDB API
- Framer Motion
- Canvas-confetti (because celebrating profile completion matters)

**Key React patterns:**

- `performance.now()` for millisecond-precision timestamps
- `requestAnimationFrame` for smooth timer updates
- `mousemove` velocity tracking via position deltas
- Debounced hover events to prevent signal noise

---

**What surprised us:**

The TIMER is the product.

Showing users exactly how fast profiling happens—"0.000s" counting up in real-time—proves the value proposition live. No marketing claim needed. They experience it.

---

**Phase 2 roadmap:**

1. **AgentDB** — Vector persistence for behavioral embeddings
2. **CLIP embeddings** — "More like this" via visual similarity
3. **Emotional arc matching** — Narrative trajectory recommendations
4. **Multi-device signals** — TV (dwell time), Mobile (swipe velocity)

---

The cold-start problem is a data problem. We solved it by creating data faster.

Not asking. Observing.

Code: github.com/agenticsorg/hackathon-tv5

---

#Engineering #AI #MachineLearning #Streaming #NextJS #React #Hackathon

---

## POST 3: Short Version (Quick Hook)

---

Traditional streaming platforms: "Rate these 20 movies to get started"

Us: *watches where your cursor hovers for 45 seconds*

"Here's your personalized feed."

---

Just shipped a behavioral profiling system for the TV5MONDE hackathon.

No questionnaires. No watch history needed. Pure micro-signal capture.

- Hover duration
- Click sequence
- Cursor velocity
- Return visits

The timer runs live. Users watch their profile being built in real-time.

12 seconds average. Not 12 minutes.

Cold-start problem → solved.

---

#Hackathon #AI #UX #Streaming

---
