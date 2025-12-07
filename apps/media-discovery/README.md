# Media Discovery Platform

**TV5MONDE Global Hackathon Entry** | Agentics Foundation London Team

> 45 seconds to profile you. Not 45 minutes.

## The Problem

Every streaming platform faces the **cold-start problem**: new users land on a sea of content with no personalization. Traditional solutions ask users to fill out questionnaires (5-10 minutes) or wait for weeks of watch history to accumulate.

**The result?** 40% trial abandonment. 20+ minutes of frustrated browsing. Users default to rewatching safe favorites instead of discovering new content.

## Our Solution

**Behavioral observation instead of questionnaires.**

We profile users in under 60 seconds by observing micro-behaviors as they browse a visual grid—no questions asked. The system captures:

- Hover duration on thumbnails
- Click sequence and timing
- Mouse velocity (skip patterns)
- Return visits to the same item
- Micro-hesitations (cursor slowdowns)

Traditional platforms ask. We observe.

## User Flow

```
STEP 1: ENTRY               STEP 2: DISCOVERY              STEP 3: RESULTS
─────────────               ─────────────────              ───────────────

"Discover Your              8 visual thumbnails            "Curated for You"
 Next Watch"                NO LABELS - pure visuals
                                                           Personalized feed
[Start Exploring] ────────► User browses naturally ──────► with genre labels
                            Timer: 0.000s visible          NOW revealed

                            System captures:               Top 2 genres
                            • Hover patterns               highlighted
                            • Click order (1, 2, 3)
                            • Attention signals            "12s to profile you,
                                                            not 45 minutes"
```

### Key Design Decisions

1. **No labels during profiling** — Users react to visuals (System 1 thinking), not text
2. **Labels revealed in results** — Creates an "aha" moment: "The system understood me"
3. **Live timer** — Proves the speed claim in real-time
4. **Large centered numbers** — Clear visual feedback when thumbnails are selected

## Technical Architecture

### Signal Capture

```
Browser Events (60fps)
        │
        ▼
┌──────────────────┐
│  Event Debounce  │  (16ms batching)
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Signal Processor │  (weighted scoring)
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Profile Builder  │  (genre ranking)
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  Personalized    │
│  Recommendations │
└──────────────────┘
```

### Signal Weights

| Signal | Weight | Rationale |
|--------|--------|-----------|
| Return visit to same item | 3.0x | Strongest intent—user came back |
| Click (explicit selection) | 2.5x | Conscious commitment |
| Micro-hesitation (>500ms) | 2.0x | Subconscious interest |
| Hover duration (>1000ms) | 1.5x | Sustained attention |
| High scroll velocity past | -1.5x | Active rejection |

### Tech Stack

- **Framework:** Next.js 15 (App Router)
- **State:** Zustand
- **Styling:** Tailwind CSS
- **Content API:** TMDB
- **Animations:** CSS keyframes + Framer Motion
- **Deployment:** Netlify / Vercel

## Features Built

| Feature | Purpose |
|---------|---------|
| 3-screen flow (Entry → Discovery → Results) | Core user journey |
| 8-genre visual grid with portrait thumbnails | Profiling surface |
| Real-time hover/click tracking | Behavioral signals |
| Mouse velocity detection | Skip pattern analysis |
| Live millisecond timer | Value prop demonstration |
| Large centered number badges (1, 2, 3) | Clear selection feedback |
| Personalized recommendations | Proof that profiling works |
| Admin widget with leaderboard | Demo tool for judges |
| Skip/Continue flow handling | Graceful degradation |
| Disclosure banner for skipped users | Transparency |

## Recent Development (Last 8 Hours)

- Netflix-style premium animations with card hover effects
- Portrait thumbnails (9:16 aspect ratio) for visual impact
- Skip/Recommendations button with animated gradient border
- Genre labels removed from discovery grid (pure visual profiling)
- Disclosure banner when user skips or doesn't select genres
- Large centered number badges on thumbnail selection
- Confetti celebration when profile is ready
- Conditional hero messaging based on profiling completion

## Running Locally

```bash
cd apps/media-discovery
npm install
npm run dev
```

Open [http://localhost:3000](http://localhost:3000)

## Environment Variables

```env
TMDB_API_KEY=your_tmdb_api_key
```

## Success Metrics

| Metric | Target |
|--------|--------|
| Time-to-profile | < 60 seconds |
| Speed vs. questionnaires | 95% faster |
| Profile resonance | 80%+ "this feels right" |
| Feed engagement | First click within 10 seconds |

## Phase 2 Roadmap

1. **AgentDB Integration** — Persistent behavioral profiles across sessions
2. **Visual Similarity Search** — CLIP embeddings for thumbnail matching
3. **Emotional Arc Matching** — Story trajectory recommendations
4. **Multi-device Support** — Smart TV (dwell time), Mobile (swipe velocity)

## Team

**Agentics Foundation London** — [london.agentics.org](https://london.agentics.org)

## License

MIT

---

*Powered by [TMDB](https://www.themoviedb.org/) | Built with [ARW](https://arw.dev)*
