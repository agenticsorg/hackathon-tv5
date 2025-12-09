# Phase 4: Demo & Presentation - Implementation Plan

## Progressive Complexity Approach

We're building the demo experience in three progressive stages, allowing for review and iteration at each step.

---

## Option 1: Simple Discovery UI (2-4 hours)
**Status:** 🔄 In Progress

### Features
- Single-page web application
- Search bar for semantic queries
- Movie/series cards with poster placeholders
- Genre and mood tag filtering
- Platform validation badges (Netflix ✓, Amazon ✓, FAST ✓)
- AI enrichment display (moodTags, themes)

### Technical Stack
- Static HTML + Tailwind CSS + Alpine.js (lightweight)
- Calls existing Cloud Run API
- Deployed as static files on Cloud Run

### User Flow
```
1. User visits demo URL
2. Sees catalog of available content
3. Types search query: "dark thriller with twist ending"
4. Results display with similarity scores
5. Clicks on title to see full metadata
6. Sees platform validation status
```

### Deliverables
- [ ] `apps/demo-ui/index.html` - Main page
- [ ] `apps/demo-ui/styles.css` - Tailwind styles
- [ ] `apps/demo-ui/app.js` - API integration
- [ ] Dockerfile for static hosting
- [ ] Deploy to Cloud Run

---

## Option 2: Full Discovery Experience (1-2 days)
**Status:** ⏳ Planned (after Option 1 review)

### Additional Features
- User profile with preferences
- "For You" personalized recommendations
- Browse by genre, mood, year
- Similar content suggestions
- Watchlist functionality
- Viewing history tracking

### Technical Stack
- React 18 + TypeScript
- Material UI or Tailwind UI
- Local storage for user state
- React Query for API calls

### User Flow
```
1. User sets preferences (favorite genres, actors)
2. Homepage shows personalized recommendations
3. Browse sections: "Action Movies", "Dark Thrillers", etc.
4. Click any title for details + similar content
5. Add to watchlist
6. See "Because you liked X" recommendations
```

### Deliverables
- [ ] `apps/ummid-dashboard/` - React application
- [ ] User preference storage
- [ ] Recommendation algorithm
- [ ] Responsive mobile design

---

## Option 3: Real Metadata Integration (2-3 days)
**Status:** ⏳ Planned (after Option 2 review)

### Additional Features
- Connect to real metadata sources
- High-quality poster images
- Trailer playback
- Cast/crew details
- Ratings from multiple sources
- Release schedules

### Data Sources
| Source | Purpose | API |
|--------|---------|-----|
| **TMDb** | Posters, cast, crew | Free API key |
| **OMDb** | Ratings, plot | Free tier available |
| **EIDR** | Official IDs | Registry lookup |
| **YouTube** | Trailers | Data API |

### Technical Stack
- Backend proxy for API aggregation
- Image CDN for posters
- Caching layer (Redis or in-memory)

### Deliverables
- [ ] Metadata aggregation service
- [ ] Image proxy/CDN setup
- [ ] Enhanced metadata schema
- [ ] Real content catalog (100+ titles)

---

## Demo Script (For Presentation)

### 1. Introduction (1 min)
> "Every night, millions spend 30 minutes deciding what to watch. We built Nexus-UMMID to solve this."

### 2. Semantic Search Demo (2 min)
```
Query: "dark psychological thriller with twist ending"
→ Show results with AI-enriched metadata
→ Highlight mood tags and themes
```

### 3. Platform Validation (1 min)
```
Click on a title
→ Show validation status for Netflix, Amazon, FAST
→ Explain connector architecture
```

### 4. AI Enrichment (1 min)
```
Show a title with missing metadata
→ Click "Enrich"
→ Watch AI generate mood tags, themes, keywords
```

### 5. Architecture Overview (2 min)
```
Show live API endpoints
→ Explain 13-agent swarm that built it
→ Highlight production scalability (400M users)
```

### 6. Technical Deep Dive (Optional, 2 min)
```
Show Cloud Run metrics
→ Explain hypergraph data model
→ Demo RuVector similarity search
```

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Demo load time | <2s | Lighthouse |
| Search response | <500ms | API latency |
| UI responsiveness | 60fps | Chrome DevTools |
| Feature completeness | 100% | Checklist |
| Audience engagement | High | Demo feedback |

---

## Timeline

| Day | Task | Output |
|-----|------|--------|
| Today | Option 1: Simple UI | Working demo |
| +1 day | Review & iterate | Polished UI |
| +2 days | Option 2: Full experience | React app |
| +3 days | Option 3: Real metadata | Production-ready |

---

## API Endpoints for Demo

### Core Endpoints (Already Live)
```bash
# Base URL
https://nexus-ummid-api-181630922804.us-central1.run.app

# Catalog
GET /api/v1/metadata

# Search
GET /api/v1/search?q={query}

# Single item
GET /api/v1/metadata/{id}

# AI Enrichment
POST /api/v1/metadata/{id}/enrich

# Platform Validation
POST /api/v1/metadata/{id}/validate
Body: {"platform": "netflix|amazon|fast"}

# Similar content
GET /api/v1/search/similar/{id}

# Trending
GET /api/v1/search/trending
```

---

## Next Steps

1. ✅ Create this implementation plan
2. 🔄 Build Option 1 demo UI
3. ⏳ Add realistic movie data
4. ⏳ Deploy and test
5. ⏳ Review and iterate
