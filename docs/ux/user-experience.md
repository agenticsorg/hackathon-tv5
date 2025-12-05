# User Experience Design: Instant Media Discovery

**Version:** 1.0.0
**Last Updated:** 2025-12-05
**Status:** Design Specification

---

## Executive Summary

This document outlines the user experience design for an instant media discovery platform that provides verified answers to natural language queries about what to watch. The design prioritizes speed, accuracy, and delightful interactions across multiple input modalities.

**Core Vision:** Users should get instant, verified answers to natural language queries about what to watch.

**Design Principles:**
1. **Instant Clarity** - Results appear in <1s with clear, actionable information
2. **Natural Conversation** - Interaction feels like talking to a knowledgeable friend
3. **Adaptive Intelligence** - System learns and personalizes over time
4. **Universal Access** - Works seamlessly across devices and interaction methods
5. **Trust & Verification** - All recommendations include source verification

---

## Table of Contents

1. [Primary Interaction Patterns](#1-primary-interaction-patterns)
2. [User Journey Maps](#2-user-journey-maps)
3. [Query Input Interface Design](#3-query-input-interface-design)
4. [Results Presentation](#4-results-presentation)
5. [Filtering & Refinement](#5-filtering--refinement)
6. [Personalization Onboarding](#6-personalization-onboarding)
7. [Wireframe Descriptions](#7-wireframe-descriptions)
8. [Multi-Platform Design](#8-multi-platform-design)
9. [Error Handling & Fallbacks](#9-error-handling--fallbacks)
10. [Accessibility Requirements](#10-accessibility-requirements)

---

## 1. Primary Interaction Patterns

### 1.1 Chat Interface (Primary)

**Purpose:** Natural language queries with conversational follow-ups

**Key Features:**
- Free-form text input with intelligent autocomplete
- Contextual suggestions based on partial input
- Multi-turn conversation support
- Quick action buttons for common refinements

**Interaction Flow:**
```
User types: "something funny to watch with my family tonight"
  ↓
System shows typing indicator (200-300ms)
  ↓
Results appear with conversational response:
"Here are 3 family-friendly comedies perfect for tonight..."
  ↓
Quick refinements appear: [Make it shorter] [More recent] [Animated]
```

**Design Considerations:**
- Input field expands as user types (single line → multi-line)
- Support for voice dictation via browser API
- Keyboard shortcuts (⌘K to focus, Enter to send, ⌘↑ for previous query)
- Suggested queries appear below input when empty

**Visual Metaphor:** Modern messaging app meets search engine

---

### 1.2 Voice Interface (Secondary)

**Purpose:** Hands-free interaction, especially useful on mobile and TV platforms

**Key Features:**
- Wake word activation ("Hey [App Name]" or button press)
- Real-time speech-to-text visualization
- Verbal confirmation of understood query
- Audio response with visual results

**Interaction Flow:**
```
User says: "Find me a thriller like Gone Girl"
  ↓
Visual feedback shows transcription in real-time
  ↓
System responds verbally: "I found 5 psychological thrillers similar to Gone Girl"
  ↓
Visual results display simultaneously
  ↓
User can refine: "Only on Netflix" (hands-free refinement)
```

**Design Considerations:**
- Visual waveform animation during listening
- Confidence indicator for transcription accuracy
- Option to edit transcribed text before submitting
- Multimodal output (audio + visual)
- Noise cancellation and error recovery

**Technical Requirements:**
- Web Speech API for browser-based voice
- Fallback to Google Cloud Speech-to-Text for accuracy
- Wake word detection for ambient mode
- Local processing for privacy

---

### 1.3 Visual Search (Tertiary)

**Purpose:** Upload/capture movie posters, screenshots, or memes to find similar content

**Key Features:**
- Camera capture for posters/screenshots
- Drag-and-drop image upload
- URL-based image search
- Multi-image comparison

**Interaction Flow:**
```
User uploads screenshot of a show
  ↓
Image analysis (visual + OCR for text)
  ↓
System identifies: "This appears to be from Stranger Things"
  ↓
Results: "More shows like Stranger Things..." + visual similarity grid
```

**Design Considerations:**
- Large drop zone with clear affordances
- Preview of uploaded image with identified elements highlighted
- Progressive disclosure: Show what was detected (actors, setting, mood)
- Option to add text query alongside image

---

### 1.4 Hybrid Interaction Model

**Recommended Approach:** Seamlessly blend all three modalities

**Example Scenarios:**

**Scenario 1: Multi-modal refinement**
```
Voice: "Find me a comedy"
  ↓ (results appear)
Touch: Taps filter chip "1990s"
  ↓ (results update)
Text: Types "with Jim Carrey"
  ↓ (final refined results)
```

**Scenario 2: Visual + text combination**
```
Upload: Movie poster image
  ↓ (system identifies movie)
Text: "but make it animated"
  ↓ (animated movies with similar themes)
```

---

## 2. User Journey Maps

### 2.1 Journey Map: The Decisive Viewer

**Persona:** Sarah, 32, knows what she wants but needs help finding where to watch it

**Scenario:** Friday evening, wants to watch a specific type of movie

**Journey Stages:**

| Stage | User Action | User Emotion | System Response | Design Opportunity |
|-------|-------------|--------------|-----------------|-------------------|
| **Trigger** | Sits down to relax after work | Relaxed, ready to unwind | Landing page loads instantly | Show personalized greeting based on time of day |
| **Query Formation** | Types: "romantic comedy from the 90s with strong female lead" | Confident, specific | Autocomplete suggests: "...like Notting Hill" | Intelligent suggestions validate her taste |
| **Results Review** | Browses 8 curated results | Evaluating, comparing | Shows availability across her streaming services | Clear availability indicators prevent frustration |
| **Filtering** | Clicks "Available on Netflix" | Narrowing focus | Results filter to 3 movies | Smooth animation, no page reload |
| **Decision** | Hovers over "While You Were Sleeping" | Interested, curious | Expands card with trailer, reviews, similar titles | Rich preview without leaving page |
| **Action** | Clicks "Watch on Netflix" | Excited, decisive | Opens Netflix in new tab to exact title page | Seamless handoff to streaming service |
| **Follow-up** | Returns to app | Satisfied | Prompt: "How was While You Were Sleeping?" | Builds long-term relationship |

**Pain Points Addressed:**
- ✅ No endless scrolling through streaming apps
- ✅ No uncertainty about availability
- ✅ No switching between multiple services
- ✅ No forgetting title after seeing it

**Key Metrics:**
- Time to decision: <2 minutes
- Query satisfaction: 95%+
- Conversion to watch: 80%+

---

### 2.2 Journey Map: The Browser

**Persona:** Marcus, 24, doesn't know what he wants, needs inspiration

**Scenario:** Sunday afternoon, wants to discover something new

**Journey Stages:**

| Stage | User Action | User Emotion | System Response | Design Opportunity |
|-------|-------------|--------------|-----------------|-------------------|
| **Trigger** | Opens app while lounging on couch | Bored, uninspired | Shows mood-based quick prompts | Reduce decision paralysis with curated options |
| **Exploration** | Taps mood chip: "Mind-bending" | Curious, open | Swipeable carousel of psychological thrillers | Gamified discovery feels fun, not overwhelming |
| **Engagement** | Swipes through 5 options | Engaged, browsing | Each swipe pre-loads next card, smooth 60fps | Delightful interactions encourage exploration |
| **Interest** | Pauses on "Coherence" card | Intrigued | Card auto-expands after 2s, shows trailer | Reward attention with deeper content |
| **Comparison** | Asks: "What else is like this but scarier?" | Refining taste | 4 new options appear with intensity indicators | Conversational refinement feels natural |
| **Decision** | Voice: "Tell me more about Midsommar" | Committed to learning | Verbal summary plays while visual details display | Multimodal response feels premium |
| **Action** | Adds to watchlist for later | Satisfied with discovery | Haptic feedback, visual confirmation | Positive reinforcement |

**Pain Points Addressed:**
- ✅ No paradox of choice
- ✅ No commitment pressure (can save for later)
- ✅ No boring list interfaces
- ✅ No feeling lost in endless options

**Key Metrics:**
- Engagement time: 5-10 minutes
- Items added to watchlist: 2-3 per session
- Return rate: 70%+

---

### 2.3 Journey Map: The Group Planner

**Persona:** Jenna, 28, organizing movie night with 4 friends with different tastes

**Scenario:** Planning Friday group watch session

**Journey Stages:**

| Stage | User Action | User Emotion | System Response | Design Opportunity |
|-------|-------------|--------------|-----------------|-------------------|
| **Trigger** | Group chat discussing movie night | Responsible, slight pressure | Creates shareable session link | Collaborative features reduce coordination burden |
| **Setup** | Shares link with group | Organized | Each friend joins session anonymously or with profile | Low-friction participation |
| **Input** | 4 people submit preferences:<br>"action", "not too long", "something fun", "sci-fi" | Collaborative | System finds intersection of preferences in real-time | Smart algorithm finds common ground |
| **Results** | Group sees 3 options that match everyone | Relieved, impressed | Results sorted by "group fit score" | Transparency builds trust |
| **Voting** | Group votes on top choice | Democratic, engaged | Live voting results, countdown timer for decision | Gamification makes decision fun |
| **Consensus** | "Edge of Tomorrow" wins 3-1 | Satisfied, ready | Celebration animation, streaming links for all | Positive emotional peak |
| **Watch** | Group starts watching | Excited | Optional: Watch party sync feature | Extended value beyond discovery |

**Pain Points Addressed:**
- ✅ No endless debate
- ✅ No one person dictating choice
- ✅ No checking multiple preferences manually
- ✅ No defaulting to "whatever"

**Key Metrics:**
- Time to consensus: <5 minutes
- Group satisfaction: 90%+
- Feature stickiness: 60% use it regularly

---

### 2.4 Journey Map: The Accessibility User

**Persona:** David, 45, vision impaired, uses screen reader

**Scenario:** Evening after work, wants to find a documentary

**Journey Stages:**

| Stage | User Action | User Emotion | System Response | Design Opportunity |
|-------|-------------|--------------|-----------------|-------------------|
| **Trigger** | Opens app via screen reader | Habitual | Clear heading hierarchy, skip to main content link | Immediate orientation reduces frustration |
| **Navigation** | Tabs to search input (3 tab stops) | Efficient | Focus clearly indicated, announces "Search for movies and shows" | Logical tab order respects user's time |
| **Query** | Voice input: "nature documentary narrated by David Attenborough" | Confident | Announces: "Searching..." then "Found 6 results" | Audio feedback confirms system is working |
| **Results** | Screen reader announces first result | Listening | Descriptive aria-labels include: title, year, rating, availability | Rich context without visual dependence |
| **Filtering** | Navigates to filter region (via landmark) | In control | All filters keyboard accessible, state announced | Equal access to all features |
| **Details** | Activates "Planet Earth II" details | Interested | Modal opens with ARIA live region announcing key info | Information hierarchy prioritizes what matters |
| **Action** | Selects "Watch on BBC iPlayer" | Ready | Announces destination before navigation, confirmation | Predictability builds confidence |

**Pain Points Addressed:**
- ✅ No image-only buttons
- ✅ No mystery click targets
- ✅ No unreachable keyboard traps
- ✅ No missing context for screen readers

**Key Metrics:**
- WCAG 2.1 AAA compliance
- Screen reader user satisfaction: 95%+
- Keyboard-only task completion: 100%

---

## 3. Query Input Interface Design

### 3.1 Conversational UI (Recommended Primary)

**Design Philosophy:** Make querying feel like texting a knowledgeable friend

**Layout Structure:**

```
┌─────────────────────────────────────────────────┐
│  [Logo]                    [Profile] [Settings] │
├─────────────────────────────────────────────────┤
│                                                 │
│  Conversation History (if any)                 │
│  ┌──────────────────────────────────┐          │
│  │ You: "comedy for family night"   │          │
│  └──────────────────────────────────┘          │
│                                                 │
│  ┌──────────────────────────────────────────┐  │
│  │ System: Here are 5 family comedies...   │  │
│  │ [Results Grid Below]                     │  │
│  └──────────────────────────────────────────┘  │
│                                                 │
├─────────────────────────────────────────────────┤
│  Input Area (Fixed Bottom)                     │
│  ┌───────────────────────────────────────────┐ │
│  │ What do you want to watch?                │ │
│  │                                    [🎤][⬆]│ │
│  └───────────────────────────────────────────┘ │
│  Suggested: [Trending] [Mood-based] [Surprise]│
└─────────────────────────────────────────────────┘
```

**Key Components:**

1. **Input Field**
   - Auto-expanding textarea (1-4 lines max)
   - Placeholder rotates through example queries
   - Character limit: 500 characters
   - Real-time validation for context

2. **Smart Suggestions (Contextual)**
   - Empty state: Show trending queries + personalized suggestions
   - Partial input: Show autocomplete with → key navigation
   - Post-result: Show refinement chips based on current results

3. **Voice Button**
   - Single tap to activate
   - Pulsing animation while listening
   - Tap again to cancel
   - Auto-submit when speech ends (2s silence)

4. **Visual Feedback**
   - Typing indicator (animated dots)
   - Query appears in chat bubble above
   - Results slide in from bottom with spring animation

---

### 3.2 Structured Form (Alternative for Power Users)

**Design Philosophy:** Give control to users who know exactly what they want

**Layout Structure:**

```
┌─────────────────────────────────────────────────┐
│  Advanced Search                                │
├─────────────────────────────────────────────────┤
│                                                 │
│  Genre          [Dropdown: Action ▼]           │
│  Released       [Range: 2000 ━━━━○━━ 2024]    │
│  Rating         [Min: 7.0 ▼]                   │
│  Runtime        [○ Short <90min               │
│                  ○ Medium 90-120min            │
│                  ◉ Any]                        │
│  Available on   [☑ Netflix ☑ Prime ☐ Hulu]   │
│  Language       [Multi-select: English ▼]     │
│  Mood/Theme     [Tags: Dark, Suspenseful...]  │
│                                                 │
│  [Clear All]              [Search 247 results →]│
└─────────────────────────────────────────────────┘
```

**When to Show:**
- User clicks "Advanced" toggle
- User has performed 3+ searches in session (offer as option)
- User is on desktop (more screen real estate)

**Key Features:**
- All fields optional
- Live result count updates as fields change
- Save filter combinations as presets
- One-click toggle between simple/advanced

---

### 3.3 Hybrid Approach (Recommended Implementation)

**Best of Both Worlds:** Start conversational, reveal structure on demand

**Primary Interface:**
```
Large search input with natural language
  ↓
User types partial query
  ↓
Smart suggestions appear (conversational)
  + "Need more control? Try advanced filters"
  ↓
User can continue conversationally OR switch to structured
```

**Interaction Patterns:**

1. **Natural → Structured Transition**
   ```
   User: "action movie"
   System: Shows results + "Refine by year, rating, or streaming service?"
   User: Clicks "year"
   System: Structured year picker appears inline
   ```

2. **Structured → Natural Refinement**
   ```
   User: Sets filters (Genre: Thriller, Year: 2020+)
   System: Shows results + input field
   User: Types "with strong female lead"
   System: Applies natural language refinement to structured filters
   ```

3. **Voice + Form Hybrid**
   ```
   User: Voice "Find comedies"
   System: Populates genre field automatically
   User: Manually adjusts other filters
   System: Remembers this pattern for future queries
   ```

---

### 3.4 Input Enhancements

**Smart Query Understanding:**

| User Input | System Interpretation | Visual Feedback |
|------------|----------------------|-----------------|
| "something like Inception" | Similar movies search | Pill: "Similar to Inception [×]" |
| "comedy from the 90s" | Genre + year filter | Pills: "Comedy [×]" "1990-1999 [×]" |
| "on Netflix" | Platform filter | Pill: "Netflix [×]" |
| "under 2 hours" | Runtime constraint | Pill: "Runtime <120min [×]" |
| "funny but not silly" | Mood + exclusion | Pills: "Witty humor [×]" "Exclude slapstick [×]" |

**Autocomplete Intelligence:**

- **Typed: "some"**
  - something funny → [Comedy genre]
  - something scary → [Horror genre]
  - something like [recent search] → [Similar to...]

- **Typed: "mov"**
  - movies with [common actor]
  - movies directed by [auteur]
  - movie night ideas → [Curated list]

- **Typed: "I want"**
  - I want to cry → [Emotional dramas]
  - I want to laugh → [Comedies]
  - I want something new → [Recent releases]

**Context Awareness:**

- Time of day: Evening = "relaxing", Morning = "energizing"
- Day of week: Friday = "weekend starter", Sunday = "feel-good"
- Season: Winter = "cozy", Summer = "adventure"
- Previous searches: Learn preferences over time

---

## 4. Results Presentation

### 4.1 Card Layout (Recommended Primary)

**Design Philosophy:** Visual-first, scannable, actionable

**Card Anatomy:**

```
┌─────────────────────────────────┐
│   ┌───────────────────────┐     │ ← Poster Image (2:3 ratio)
│   │                       │     │   High-quality, lazy-loaded
│   │    Movie Poster      │     │   Hover: Subtle scale (1.05x)
│   │                       │     │
│   └───────────────────────┘     │
│   Title (2 lines max)            │ ← Large, bold, truncated
│   Year • Rating • Runtime        │ ← Metadata row
│   ⭐ 8.2/10  🍅 94%             │ ← Scores with icons
│   ┌─────────────────────────┐   │
│   │ [▶ Watch on Netflix]    │   │ ← Primary CTA
│   └─────────────────────────┘   │
│   Also on: Prime, Hulu...       │ ← Secondary availability
└─────────────────────────────────┘
```

**Grid Layout:**

- **Desktop:** 4 columns, 24px gap
- **Tablet:** 3 columns, 16px gap
- **Mobile:** 2 columns, 12px gap (1 column for list view)

**Card States:**

1. **Default**
   - Subtle shadow
   - Flat design (no depth effects)
   - Crisp borders

2. **Hover** (Desktop)
   - Poster scales slightly (1.05x)
   - Shadow increases
   - Quick preview appears after 1s (trailer thumbnail)
   - Transition: 200ms ease-out

3. **Expanded** (Click/Tap)
   - Card expands to modal overlay (full details)
   - Smooth scale animation from card position
   - Background blurs behind modal
   - Close via X, ESC, or click outside

**Information Hierarchy:**

1. **Primary:** Poster (visual hook)
2. **Secondary:** Title (identification)
3. **Tertiary:** Rating/scores (credibility)
4. **Quaternary:** Streaming availability (action)
5. **Hidden until hover/expand:** Synopsis, cast, reviews

---

### 4.2 List Layout (Alternative for Dense Information)

**Design Philosophy:** Information-dense, efficient scanning

**List Item Anatomy:**

```
┌──────────────────────────────────────────────────────┐
│ [Poster] Title (Year)               ⭐8.2  🍅94%    │
│ 100x150  Drama, Thriller • 2h 28m                   │
│          ┌──────────────────────────────────────┐   │
│          │ Brief synopsis (2 lines max)...      │   │
│          └──────────────────────────────────────┘   │
│          [▶ Netflix] [+Watchlist] [...More]         │
└──────────────────────────────────────────────────────┘
```

**When to Use:**
- User preference (toggle view)
- Search results >20 items (easier to scan)
- Accessibility mode (more linear for screen readers)
- Slow connections (less image loading)

**List Features:**
- Alternating row backgrounds for scannability
- Sticky header with sort controls
- Infinite scroll with "Back to top" button after 10 items
- Keyboard navigation (J/K for up/down, Enter to expand)

---

### 4.3 Grid Layout (For Visual Discovery)

**Design Philosophy:** Pinterest-style browsing for mood-based discovery

**Masonry Grid:**
```
┌────┐ ┌────────┐ ┌────┐
│    │ │        │ │    │
└────┘ │        │ └────┘
┌────┐ └────────┘ ┌────────┐
│    │ ┌────┐     │        │
│    │ │    │     │        │
└────┘ └────┘     └────────┘
```

**Characteristics:**
- Variable height cards based on content
- Posters displayed at original aspect ratios
- Dense packing algorithm (minimize gaps)
- Hover reveals title + quick info overlay

**Best For:**
- Mood-based browsing ("Show me cozy movies")
- Visual similarity search (uploaded image)
- Genre exploration (Horror, Animation, etc.)
- "Surprise me" mode

**Grid Behavior:**
- Lazy load images as user scrolls
- Parallax effect on scroll for depth
- Tap card to expand with context
- Drag to reorder (personalization signal)

---

### 4.4 Interactive Map View (Experimental)

**Design Philosophy:** Spatial exploration of media landscape

**Concept:**
- Movies/shows positioned in 2D space by similarity
- Clusters represent genres, themes, moods
- User can pan, zoom, explore relationships
- Click node to see details

**Example Layout:**
```
        Comedy Cluster
           ●  ●
          ●  ●  ●
             ●

Sci-Fi          Romance
  ●  ●          ●  ●
 ●  ●  ●       ●  ●
  ●                ●

     Thriller Cluster
        ●  ●  ●
       ●  ●  ●  ●
```

**Use Cases:**
- Discovering unexpected connections
- Exploring subgenres
- Finding niche content
- Understanding personal taste map

**Implementation Notes:**
- Use force-directed graph algorithm
- WebGL for performance (thousands of nodes)
- Cluster labels appear on zoom
- Color coding by genre/mood

---

### 4.5 Recommended Multi-View Strategy

**Default View by Context:**

| Query Type | Default View | Why |
|------------|--------------|-----|
| Specific search ("Inception-like") | Card grid | Visual comparison important |
| Broad search ("comedy") | List view | Many results to scan |
| Mood-based ("cozy movie") | Masonry grid | Visual browsing |
| Voice query result | Card grid (large) | Optimized for lean-back viewing |
| Accessibility mode | List view | Linear, keyboard-friendly |

**User Controls:**
- View toggle (top-right): [Grid icon] [List icon] [Map icon]
- Preference persists per session
- Quick switch via keyboard: G (grid), L (list), M (map)

---

## 5. Filtering & Refinement

### 5.1 Filter Categories

**Primary Filters (Always Visible):**

1. **Genre**
   - Multi-select with common combinations
   - Examples: Action, Comedy, Drama, Horror, Sci-Fi
   - Smart suggestions: "Dark Comedy", "Sci-Fi Thriller"

2. **Mood/Vibe**
   - Curated mood tags (not standard genre)
   - Examples: Cozy, Mind-bending, Feel-good, Intense, Whimsical
   - Can combine with genre for nuanced results

3. **Runtime**
   - Quick chips: "Quick (<90min)", "Standard (90-120min)", "Epic (>2hr)"
   - Custom range slider for precision
   - Remember user's typical preferences

4. **Release Year**
   - Decade chips: [1980s] [1990s] [2000s] [2010s] [2020s]
   - Custom range slider
   - "Last 3 months" for new releases

5. **Rating/Quality**
   - Minimum score slider (IMDb, Rotten Tomatoes, user rating)
   - "Critically acclaimed" (>80% RT)
   - "Hidden gems" (high rating, low popularity)

**Secondary Filters (Expandable "More Filters"):**

6. **Streaming Platform**
   - Icons for visual recognition
   - Shows user's subscriptions first (if connected)
   - "Free" filter (ad-supported, library, etc.)

7. **Language**
   - Multi-select with search
   - Includes subtitle availability
   - "Original language" vs "Dubbed available"

8. **Content Rating**
   - G, PG, PG-13, R, NC-17 (US)
   - Kids safe, Family friendly, Mature
   - International rating systems (BBFC, etc.)

9. **Awards**
   - Oscar winners/nominees
   - Emmy winners
   - Festival favorites (Sundance, Cannes, etc.)

10. **People**
    - Actor, Director, Writer search
    - Auto-complete with popular names
    - "Directed by women" or similar diversity filters

---

### 5.2 Filter Interaction Patterns

**Pattern 1: Chip-based Filters (Mobile-friendly)**

```
┌─────────────────────────────────────────────────┐
│ Active Filters:                                 │
│ [Comedy ×] [1990s ×] [Netflix ×]               │
│                                       [Clear all]│
├─────────────────────────────────────────────────┤
│ Add Filters:                                    │
│ [+ Genre] [+ Mood] [+ Year] [+ Platform]       │
└─────────────────────────────────────────────────┘
```

**Behavior:**
- Tap filter category → Opens bottom sheet with options
- Select option → Appears as chip in "Active" area
- Tap X on chip → Removes filter
- Results update in real-time with smooth transition

---

**Pattern 2: Sidebar Filters (Desktop)**

```
┌───────────┬─────────────────────────────────┐
│ FILTERS   │ Results (247 movies)           │
│           │                                 │
│ Genre     │ [Card Grid]                    │
│ ☑ Comedy  │                                 │
│ ☐ Drama   │                                 │
│ ☐ Action  │                                 │
│           │                                 │
│ Mood      │                                 │
│ ○ Cozy    │                                 │
│ ○ Intense │                                 │
│           │                                 │
│ Year      │                                 │
│ 1990──●───│                                 │
│ ─────2024 │                                 │
└───────────┴─────────────────────────────────┘
```

**Behavior:**
- Left sidebar (250px) with sticky positioning
- Collapsible sections (accordion style)
- Live result count updates
- "Apply" button optional (instant filtering preferred)

---

**Pattern 3: Conversational Refinement (AI-powered)**

```
User sees results, then types:
"Make them more recent"
  ↓
System: Adds filter [2020+] and updates results

User: "Only on Netflix"
  ↓
System: Adds filter [Netflix] and updates

User: "Shorter"
  ↓
System: Adds filter [<90min] and updates
```

**Benefits:**
- Natural for voice interaction
- No need to learn filter UI
- Contextual to current results
- Can describe filters in plain language

---

### 5.3 Smart Filter Recommendations

**Based on Query Context:**

| Initial Query | Suggested Refinements |
|---------------|----------------------|
| "action movie" | [Recent] [Highly rated] [On Netflix] |
| "something funny" | [Mood: Feel-good] [Family friendly] [<2hr] |
| "like Inception" | [Mind-bending] [Sci-fi] [Thriller] |

**Based on User Behavior:**

- User often filters by Netflix → Show Netflix toggle prominently
- User prefers movies <90min → Default runtime filter
- User browses horror at night → Suggest horror in evening

**Dynamic Refinement Suggestions:**

```
After showing results:
"Looking for something specific? Try refining by:"
[🎭 Mood] [📅 Year] [⭐ Higher rating] [🎬 Director]
```

---

### 5.4 Filter Persistence & Memory

**Session Memory:**
- Filters persist while browsing
- Clear when new unrelated query starts
- "Continue from last search" option on return

**User Preferences:**
- Remember favorite streaming platforms
- Learn typical runtime preferences
- Save custom filter presets

**Preset Filters:**
```
User creates: "Friday Night Movie"
  → Comedy + Recent + Family-friendly + <2hr + Netflix

User clicks preset → Instant results
User can create unlimited presets
Presets sync across devices (if logged in)
```

---

## 6. Personalization Onboarding

### 6.1 Onboarding Philosophy

**Core Principle:** Progressive personalization - deliver value immediately, learn over time

**Anti-patterns to Avoid:**
- ❌ Long forms before showing any content
- ❌ Forced account creation
- ❌ Generic "what genres do you like" questions
- ❌ Personality quizzes that feel like work

**Recommended Approach:**
- ✅ Show value immediately (guest mode)
- ✅ Learn from behavior, not surveys
- ✅ Optional account for sync/advanced features
- ✅ Transparent about what's being personalized

---

### 6.2 Onboarding Flow: "Soft Start"

**Step 1: Immediate Value (0 clicks)**

```
┌─────────────────────────────────────────────────┐
│  Welcome to [App Name]                          │
│  Find what to watch in seconds                  │
│                                                 │
│  ┌───────────────────────────────────────────┐ │
│  │ What do you want to watch?                │ │
│  └───────────────────────────────────────────┘ │
│                                                 │
│  Try: [Trending now] [Surprise me] [90s comedy]│
│                                                 │
│  No account needed to start                    │
│  [Create account] or [Continue as guest]       │
└─────────────────────────────────────────────────┘
```

**User can immediately:**
- Perform searches
- See results
- Click through to streaming services
- Explore features

**No barriers to entry.**

---

**Step 2: Contextual Personalization Prompts**

After user performs 2-3 searches:

```
┌─────────────────────────────────────────────────┐
│  💡 Want better recommendations?                │
│                                                 │
│  We noticed you like comedies and thrillers.   │
│  Connect your streaming services to see        │
│  personalized picks from your subscriptions.   │
│                                                 │
│  [Connect Netflix] [Connect Prime] [Skip]      │
└─────────────────────────────────────────────────┘
```

**Timing:**
- Show after user engagement (not immediately)
- Dismiss-able without penalty
- Remember dismissal (don't nag)

---

**Step 3: Platform Connection (Optional)**

```
┌─────────────────────────────────────────────────┐
│  Connect Your Streaming Services                │
│  (We'll show what's available on your platforms)│
│                                                 │
│  ┌─────────────┐  ┌─────────────┐              │
│  │ [Netflix]   │  │ [Prime]     │              │
│  │ ☐ Connected │  │ ☐ Connected │              │
│  └─────────────┘  └─────────────┘              │
│                                                 │
│  ┌─────────────┐  ┌─────────────┐              │
│  │ [Hulu]      │  │ [Disney+]   │              │
│  │ ☐ Connected │  │ ☐ Connected │              │
│  └─────────────┘  └─────────────┘              │
│                                                 │
│  Privacy: We never access your watch history.  │
│  We only check what's available to stream.     │
│                                                 │
│  [Save Selections] [Skip for now]              │
└─────────────────────────────────────────────────┘
```

**Implementation:**
- Simple checkbox selection (no OAuth required initially)
- Manual subscription management
- Optional: OAuth integration for advanced features
- Clear privacy explanation

---

**Step 4: Taste Profile Building (Passive)**

**Don't ask - observe and infer:**

After user interacts with results:
- Click on thriller → Increase thriller weight
- Skip horror results → Decrease horror weight
- Watch older movies → Prefer classic content
- Always filter to Netflix → Remember platform preference

**User can view/edit taste profile:**
```
┌─────────────────────────────────────────────────┐
│  Your Taste Profile                             │
│                                                 │
│  We've learned you enjoy:                       │
│  ━━━━━━━━━━ Thrillers (90%)                    │
│  ━━━━━━━━ Comedies (80%)                       │
│  ━━━━━━ Dramas (60%)                           │
│  ━━━━ Sci-Fi (40%)                             │
│  ━━ Horror (20%)                               │
│                                                 │
│  Typical preferences:                           │
│  • Runtime: 90-120 minutes                      │
│  • Era: 1990s-2010s                             │
│  • Rating: 7.5+                                 │
│                                                 │
│  [Edit Manually] [Reset Profile]               │
└─────────────────────────────────────────────────┘
```

**User has control:**
- View what system learned
- Manually adjust weights
- Reset and start over
- Export data

---

### 6.3 Account Creation (When Needed)

**Trigger Account Creation When:**
- User wants to save watchlist
- User wants to sync across devices
- User wants to create filter presets
- User wants to participate in group sessions

**Quick Account Creation:**

```
┌─────────────────────────────────────────────────┐
│  Save Your Watchlist?                           │
│  Create a free account to access your list      │
│  on any device.                                 │
│                                                 │
│  ┌───────────────────────────────────────────┐ │
│  │ Email: ___________________________       │ │
│  └───────────────────────────────────────────┘ │
│                                                 │
│  ┌───────────────────────────────────────────┐ │
│  │ Password: ________________________       │ │
│  └───────────────────────────────────────────┘ │
│                                                 │
│  Or sign in with:                               │
│  [Google] [Apple] [GitHub]                      │
│                                                 │
│  [Create Account] [Maybe Later]                 │
└─────────────────────────────────────────────────┘
```

**Features:**
- Social sign-in for speed
- Passwordless options (magic link)
- No email verification required initially
- Account created in <30 seconds

---

### 6.4 Personalization Features

**What Gets Personalized:**

1. **Search Results Ranking**
   - Prioritize user's preferred genres
   - Surface content from connected platforms
   - Boost items similar to previous likes

2. **Homepage Recommendations**
   - "Picked for You" section
   - "Because you liked X"
   - "Trending in [favorite genre]"

3. **Query Suggestions**
   - Autocomplete based on taste
   - "You might be looking for..."
   - Context-aware (time, day, season)

4. **Filter Defaults**
   - Pre-select common platforms
   - Default runtime ranges
   - Typical rating thresholds

5. **Notification Preferences**
   - New releases in favorite genres
   - Price drops on watchlist items
   - Friend recommendations (if social features)

**What Doesn't Get Personalized (Privacy):**
- No tracking of actual watch behavior on streaming platforms
- No sharing data with third parties
- No selling personal information
- User can delete all data anytime

---

### 6.5 Onboarding Success Metrics

**Key Metrics:**

| Metric | Target | Measurement |
|--------|--------|-------------|
| Time to first search | <10 seconds | From landing to query submission |
| Guest conversion rate | 30% | Guests who create account within 7 days |
| Platform connection rate | 60% | Users who connect at least one service |
| Personalization satisfaction | 85% | Survey: "Recommendations match my taste" |
| Profile accuracy | 90% | User confirms taste profile is accurate |

**A/B Testing Opportunities:**
- Onboarding flow variations
- Prompt timing and copy
- Account creation friction
- Platform connection UI

---

## 7. Wireframe Descriptions

### 7.1 Mobile App - Home Screen

**Screen Dimensions:** 375x812 (iPhone reference)

```
┌─────────────────────────────────────────────────┐
│ [☰]          [App Logo]              [Profile] │ ← Header (60px)
├─────────────────────────────────────────────────┤
│                                                 │
│  ┌───────────────────────────────────────────┐ │ ← Search Input (56px)
│  │ 🔍 What do you want to watch?      [🎤]  │ │   Large, always visible
│  └───────────────────────────────────────────┘ │   Tap to expand
│                                                 │
│  Quick Actions:                                 │ ← Horizontal scroll
│  [🎯 Trending] [🎲 Surprise] [❤️ Watchlist]   │   80px tall chips
│                                                 │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                                 │
│  For You                              [See All]│ ← Section header
│                                                 │
│  ┌────────┐  ┌────────┐  ┌────────┐          │ ← Horizontal card scroll
│  │ Poster │  │ Poster │  │ Poster │  →       │   160x240 cards
│  │        │  │        │  │        │          │   12px gap
│  └────────┘  └────────┘  └────────┘          │
│   Title       Title       Title               │
│   ⭐8.2       ⭐7.9       ⭐9.1               │
│                                                 │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                                 │
│  Trending Now                         [See All]│
│                                                 │
│  ┌────────┐  ┌────────┐  ┌────────┐          │
│  │ Poster │  │ Poster │  │ Poster │  →       │
│  └────────┘  └────────┘  └────────┘          │
│   Title       Title       Title               │
│                                                 │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                                 │
│  New This Week                        [See All]│
│                                                 │
│  [Scrollable content continues...]            │
│                                                 │
└─────────────────────────────────────────────────┘
│ [🏠 Home] [🔍 Search] [📋 Watchlist] [👤 You]│ ← Bottom nav (60px)
└─────────────────────────────────────────────────┘
```

**Interaction Notes:**
- Tap search → Keyboard appears, search expands to full screen
- Pull down from top → Refresh recommendations
- Swipe right from left edge → Open menu drawer
- Long press card → Quick actions (Add to watchlist, Share)
- Double tap card → Expand to full details

---

### 7.2 Mobile App - Search Results

```
┌─────────────────────────────────────────────────┐
│ [←]          Search Results              [Filter]│ ← Header with back
├─────────────────────────────────────────────────┤
│                                                 │
│  ┌───────────────────────────────────────────┐ │ ← Search input
│  │ 🔍 comedy from the 90s            [×]    │ │   (sticky)
│  └───────────────────────────────────────────┘ │
│                                                 │
│  Active: [Comedy ×] [1990-1999 ×]     Clear all│ ← Filter chips
│                                                 │   (horizontal scroll)
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                                 │
│  43 results                                     │ ← Result count
│                                                 │
│  ┌─────────┬────────────────────────────────┐  │ ← Card (2-column)
│  │ Poster  │ The Big Lebowski (1998)        │  │   120px × 180px poster
│  │         │ ⭐ 8.1  🍅 79%                 │  │   + details
│  │         │ Comedy • 1h 57m                │  │
│  │         │ [▶ Netflix]                    │  │
│  └─────────┴────────────────────────────────┘  │
│                                                 │
│  ┌─────────┬────────────────────────────────┐  │
│  │ Poster  │ Groundhog Day (1993)           │  │
│  │         │ ⭐ 8.0  🍅 96%                 │  │
│  │         │ Comedy, Fantasy • 1h 41m       │  │
│  │         │ [▶ Prime] [▶ Hulu]            │  │
│  └─────────┴────────────────────────────────┘  │
│                                                 │
│  [More results continue...]                    │
│                                                 │
│  ┌───────────────────────────────────────────┐ │ ← Loading indicator
│  │     Loading more results...  [spinner]    │ │   (infinite scroll)
│  └───────────────────────────────────────────┘ │
│                                                 │
└─────────────────────────────────────────────────┘
│ [🏠 Home] [🔍 Search] [📋 Watchlist] [👤 You]│
└─────────────────────────────────────────────────┘
```

**Interaction Notes:**
- Tap [Filter] → Bottom sheet with filter options
- Tap card → Expand to detail view
- Swipe left on card → Quick actions (Watchlist, Share, Not interested)
- Pull to refresh → Re-run search with latest data
- Scroll to top button appears after scrolling past 5 results

---

### 7.3 Mobile App - Detail View (Modal)

```
┌─────────────────────────────────────────────────┐
│                                          [× Close]│ ← Close button
│  ┌─────────────────────────────────────────────┐ │
│  │                                             │ │ ← Poster backdrop
│  │           Movie Poster                      │ │   Full width
│  │           (Hero Image)                      │ │   300px tall
│  │                                             │ │
│  └─────────────────────────────────────────────┘ │
│                                                 │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                                 │
│  The Big Lebowski (1998)                        │ ← Title (large, bold)
│  ⭐ 8.1/10  🍅 79%  ❤️ 94% liked              │ ← Ratings row
│                                                 │
│  ┌─────────────────────────────────────────┐   │ ← Primary CTA
│  │  ▶  Watch on Netflix                    │   │   Large button (56px)
│  └─────────────────────────────────────────┘   │
│                                                 │
│  Also available on: [Prime] [Hulu]             │ ← Secondary platforms
│                                                 │
│  [+ Watchlist]  [Share]  [More Options]        │ ← Action buttons
│                                                 │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                                 │
│  Synopsis                                       │ ← Collapsible sections
│  "The Dude" Lebowski, mistaken for a           │
│  millionaire of the same name, seeks            │
│  restitution for his ruined rug...              │
│  [Read more ▼]                                  │
│                                                 │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                                 │
│  Details                                    [▼] │
│  Director: Joel & Ethan Coen                    │
│  Cast: Jeff Bridges, John Goodman...            │
│  Genre: Comedy                                  │
│  Runtime: 1h 57m                                │
│                                                 │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                                 │
│  Trailer                                    [▼] │
│  ┌─────────────────────────────────────────┐   │ ← Embedded video
│  │  [▶ Play Trailer]                       │   │   or thumbnail
│  └─────────────────────────────────────────┘   │
│                                                 │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                                 │
│  Similar Movies                             [▼] │ ← Horizontal scroll
│  ┌────┐  ┌────┐  ┌────┐  ┌────┐             │
│  │    │  │    │  │    │  │    │  →          │
│  └────┘  └────┘  └────┘  └────┘             │
│                                                 │
│  [Scrollable content continues...]            │
│                                                 │
└─────────────────────────────────────────────────┘
```

**Interaction Notes:**
- Swipe down from top → Dismiss modal
- Tap "Watch on Netflix" → Opens Netflix app/website to exact title
- Tap "+Watchlist" → Adds with haptic feedback + visual confirmation
- Sections expand/collapse on tap
- Share opens native share sheet

---

### 7.4 Desktop Web - Home Page

**Screen Dimensions:** 1440x900 (standard desktop)

```
┌──────────────────────────────────────────────────────────────────────────────────────┐
│ [☰ Menu]    [App Logo]           [Search: What do you want to watch?]      [Profile]│ ← Header (80px)
│                                                                              [Login]  │   Sticky on scroll
├──────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                      │
│  ┌────────────────────────────────────────────────────────────────────────────────┐ │
│  │                                                                                │ │ ← Hero Section
│  │  Discover what to watch                                                       │ │   400px tall
│  │  in seconds                                                                   │ │   Gradient background
│  │                                                                                │ │
│  │  ┌─────────────────────────────────────────────────────────────┐             │ │
│  │  │ 🔍 Try: "comedy from the 90s" or "something like Inception" │  [Search →] │ │ ← Large search (60px)
│  │  └─────────────────────────────────────────────────────────────┘             │ │
│  │                                                                                │ │
│  │  Quick Start: [🔥 Trending] [🎲 Surprise Me] [🎬 By Mood] [⭐ Top Rated]     │ │
│  └────────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                      │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                                                                      │
│  Trending This Week                                                      [See All →]│ ← Section (h2)
│                                                                                      │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐    │ ← 6-column grid
│  │         │  │         │  │         │  │         │  │         │  │         │    │   200x300 cards
│  │  Poster │  │  Poster │  │  Poster │  │  Poster │  │  Poster │  │  Poster │    │   24px gap
│  │         │  │         │  │         │  │         │  │         │  │         │    │
│  │         │  │         │  │         │  │         │  │         │  │         │    │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘  └─────────┘  └─────────┘    │
│   Title        Title        Title        Title        Title        Title           │
│   ⭐ 8.5       ⭐ 7.9       ⭐ 9.0       ⭐ 8.1       ⭐ 8.7       ⭐ 7.5          │
│   Netflix      Prime        Hulu         Netflix      Disney+      HBO Max         │
│                                                                                      │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                                                                      │
│  Picked For You                                                          [See All →]│
│                                                                                      │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐    │
│  │  Poster │  │  Poster │  │  Poster │  │  Poster │  │  Poster │  │  Poster │    │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘  └─────────┘  └─────────┘    │
│                                                                                      │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                                                                      │
│  New Releases                                                            [See All →]│
│                                                                                      │
│  [Content continues...]                                                             │
│                                                                                      │
├──────────────────────────────────────────────────────────────────────────────────────┤
│  [About] [Privacy] [Terms] [Help]                              Made with ❤️ in SF  │ ← Footer
└──────────────────────────────────────────────────────────────────────────────────────┘
```

**Interaction Notes:**
- Search expands on focus with suggested queries
- Hover card → Scale up + show quick preview
- Ctrl/Cmd + K → Focus search from anywhere
- Keyboard navigation: Tab through cards, Enter to open
- Smooth scroll to sections

---

### 7.5 Desktop Web - Search Results with Filters

```
┌──────────────────────────────────────────────────────────────────────────────────────┐
│ [☰]    [Logo]         [Search: comedy from the 90s]    [🎤 Voice]    [Profile]      │ ← Header
├──────┬───────────────────────────────────────────────────────────────────────────────┤
│      │                                                                               │
│ FIL- │  Results: 127 movies                    Sort: [Relevance ▼]  View: [■][≡]  │
│ TERS │                                                                               │
│      │  Active Filters: [Comedy ×] [1990-1999 ×]                      [Clear All]  │
│ Gen- │                                                                               │
│ re   │  ┌────────┬────────────────────────────────────────────────────────────┐   │
│ ☑Com │  │        │ Groundhog Day (1993)                   ⭐ 8.0  🍅 96%     │   │
│ ☐Dra │  │ Poster │ Comedy, Fantasy, Romance • 1h 41m                         │   │
│ ☐Act │  │ 100x   │ Bill Murray, Andie MacDowell                              │   │
│      │  │ 150px  │ ┌──────────────────────────────────────────────────────┐ │   │
│ Mood │  │        │ │ "A narcissistic weatherman finds himself living...   │ │   │
│ ○Coz │  │        │ └──────────────────────────────────────────────────────┘ │   │
│ ○Fun │  │        │ [▶ Watch on Netflix] [▶ Prime]    [+ Watchlist] [···]   │   │
│ ○Int │  └────────┴────────────────────────────────────────────────────────────┘   │
│      │                                                                               │
│ Year │  ┌────────┬────────────────────────────────────────────────────────────┐   │
│ 1990 │  │        │ The Big Lebowski (1998)                ⭐ 8.1  🍅 79%     │   │
│ ●────│  │ Poster │ Comedy, Crime • 1h 57m                                    │   │
│ 2024 │  │        │ Jeff Bridges, John Goodman, Julianne Moore               │   │
│      │  │        │ ┌──────────────────────────────────────────────────────┐ │   │
│ Rat- │  │        │ │ "The Dude seeks restitution for his ruined rug...    │ │   │
│ ing  │  │        │ └──────────────────────────────────────────────────────┘ │   │
│ 7.0+ │  │        │ [▶ Watch on Netflix]              [+ Watchlist] [···]   │   │
│ ─●── │  └────────┴────────────────────────────────────────────────────────────┘   │
│ 10   │                                                                               │
│      │  ┌────────┬────────────────────────────────────────────────────────────┐   │
│ Plat │  │        │ Clueless (1995)                        ⭐ 6.9  🍅 81%     │   │
│ form │  │ Poster │ Comedy, Romance • 1h 37m                                  │   │
│ ☑Net │  │        │ Alicia Silverstone, Stacey Dash                          │   │
│ ☑Pri │  │        │ [▶ Watch on Prime] [▶ Paramount+]    [+ Watchlist]      │   │
│ ☐Hul │  └────────┴────────────────────────────────────────────────────────────┘   │
│      │                                                                               │
│ [Mo- │  [More results...]                                                           │
│ re]  │                                                                               │
│      │  ┌────────────────────────────────────────────────────────────────────┐     │
│ 250px│  │                   Loading more results...                          │     │
│ wide │  └────────────────────────────────────────────────────────────────────┘     │
│      │                                                                               │
└──────┴───────────────────────────────────────────────────────────────────────────────┘
```

**Interaction Notes:**
- Left sidebar sticky (scrolls with page)
- Filters update results instantly
- Hover result → Expand preview inline
- Click title → Open detail modal
- Click platform button → Direct link to streaming service
- Keyboard shortcuts: F for filters, S for sort

---

### 7.6 Voice Interface Screen (Mobile)

```
┌─────────────────────────────────────────────────┐
│                                          [× Exit]│
├─────────────────────────────────────────────────┤
│                                                 │
│                                                 │
│                                                 │
│              ┌─────────────┐                   │ ← Animated mic icon
│              │             │                   │   Pulsing while
│              │     🎤      │                   │   listening
│              │             │                   │   200x200px
│              └─────────────┘                   │
│                                                 │
│                                                 │
│         ═══════════════════                    │ ← Waveform animation
│         ▁▃▅▇█▇▅▃▁▃▅▇█▇▅▃▁                    │   Visualizes voice
│                                                 │
│                                                 │
│     "Find me a thriller like Gone Girl"        │ ← Real-time
│                                                 │   transcription
│                                                 │   Large text (24px)
│              [Tap to speak]                     │
│                                                 │
│                                                 │
│  Try saying:                                    │ ← Helpful examples
│  • "Something funny for kids"                   │
│  • "Action movies on Netflix"                   │
│  • "Shows like Breaking Bad"                    │
│                                                 │
│                                                 │
│                                                 │
│                                                 │
│          [Switch to text input]                 │ ← Fallback option
│                                                 │
└─────────────────────────────────────────────────┘
```

**States:**

1. **Idle State:**
   - Mic icon static
   - "Tap to speak" button
   - Example queries visible

2. **Listening State:**
   - Mic icon pulsing
   - Waveform animating
   - Real-time transcription appearing
   - "Tap to stop" button

3. **Processing State:**
   - Loading spinner
   - "Understanding your request..."
   - Transcription locked in

4. **Results State:**
   - Transition to standard results view
   - Option to refine via voice

**Interaction Notes:**
- Wake word: "Hey [App Name]" (optional)
- Tap anywhere to stop listening
- Edit transcription before submitting
- Noise indicator if too loud/quiet
- Automatic submission after 2s silence

---

### 7.7 Group Watch Planning Interface

```
┌─────────────────────────────────────────────────┐
│ [←]         Movie Night Planning          [Share]│
├─────────────────────────────────────────────────┤
│                                                 │
│  Group Session: Friday Movie Night              │
│  4 participants                                 │
│                                                 │
│  Participants:                                  │
│  👤 You  👤 Sarah  👤 Marcus  👤 Jenna         │
│                                           [+ Add]│
│                                                 │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                                 │
│  Everyone's Preferences:                        │
│                                                 │
│  👤 You: "Action movie, not too long"          │
│  👤 Sarah: "Something fun"                      │
│  👤 Marcus: "Sci-fi or thriller"                │
│  👤 Jenna: "Available on Netflix"               │
│                                                 │
│  ┌───────────────────────────────────────────┐ │
│  │ Add your preference: ________________     │ │
│  └───────────────────────────────────────────┘ │
│                                                 │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                                 │
│  Best Matches for Your Group:                   │
│                                                 │
│  ┌──────┬───────────────────────────────────┐  │
│  │      │ Edge of Tomorrow (2014)           │  │
│  │Poster│ ⭐ 7.9  •  Group Fit: 95%        │  │
│  │      │ Action, Sci-Fi • 1h 53m           │  │
│  │      │ Available on: Netflix             │  │
│  │      │ ┌──────────────────────────────┐ │  │
│  │      │ │ ✓ Matches all preferences    │ │  │
│  │      │ └──────────────────────────────┘ │  │
│  │      │ Vote: 👍 3  👎 0  ⏸ 1           │  │
│  │      │ [Cast Your Vote: 👍 👎]         │  │
│  └──────┴───────────────────────────────────┘  │
│                                                 │
│  ┌──────┬───────────────────────────────────┐  │
│  │      │ Inception (2010)                  │  │
│  │Poster│ ⭐ 8.8  •  Group Fit: 88%        │  │
│  │      │ Sci-Fi, Thriller • 2h 28m         │  │
│  │      │ Available on: Prime, Hulu         │  │
│  │      │ ⚠ Marcus: "Too long"              │  │
│  │      │ Vote: 👍 2  👎 1  ⏸ 1           │  │
│  │      │ [Cast Your Vote: 👍 👎]         │  │
│  └──────┴───────────────────────────────────┘  │
│                                                 │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                                 │
│  ┌───────────────────────────────────────────┐ │
│  │ Start voting! First to 3 votes wins.      │ │
│  │ Time remaining: 4:32                      │ │ ← Optional timer
│  └───────────────────────────────────────────┘ │
│                                                 │
└─────────────────────────────────────────────────┘
```

**Interaction Flow:**
1. User creates session → Gets shareable link
2. Friends join (no account required)
3. Everyone submits preferences (text or voice)
4. System shows matched results with "group fit" score
5. Group votes on favorites
6. First to reach threshold wins
7. Optional: Start watch party (sync feature)

**Features:**
- Anonymous or profile-based participation
- Real-time voting updates
- Conflict resolution (e.g., "Too long for 1 person")
- Timer for decision (optional pressure)
- Export decision to calendar/reminders

---

## 8. Multi-Platform Design

### 8.1 Mobile App (iOS/Android)

**Platform:** Native mobile apps + PWA

**Design Priorities:**
1. **Thumb-friendly UI**
   - Bottom navigation (easy to reach)
   - Primary actions in bottom half of screen
   - Swipe gestures for common actions

2. **Touch Targets**
   - Minimum 44×44px (iOS) / 48×48dp (Android)
   - Adequate spacing (12px minimum) between targets
   - Forgiving tap areas (extend beyond visual bounds)

3. **Native Patterns**
   - iOS: Bottom sheet modals, SF Symbols icons
   - Android: Material Design 3, FABs, navigation drawer
   - Platform-specific animations (iOS: spring, Android: ease)

4. **Performance**
   - Lazy load images below fold
   - Virtual scrolling for long lists (>100 items)
   - Skeleton screens while loading
   - Offline support (cache last search)

**Mobile-Specific Features:**

```
┌─────────────────────────────────────────────────┐
│ MOBILE-ONLY FEATURES                            │
├─────────────────────────────────────────────────┤
│                                                 │
│ 1. Camera Search                                │
│    - Point camera at poster → Instant ID        │
│    - Screenshot upload from gallery             │
│                                                 │
│ 2. Shake to Surprise                            │
│    - Shake phone → Random recommendation        │
│    - Gamified discovery                         │
│                                                 │
│ 3. Location-Based                               │
│    - "Movies playing nearby" (theaters)         │
│    - Trending in your city                      │
│                                                 │
│ 4. Share Sheet Integration                      │
│    - Share from Netflix/Prime → Get similar     │
│    - Deep link handling                         │
│                                                 │
│ 5. Widgets                                      │
│    - Home screen widget: Daily pick             │
│    - Lock screen widget: Watchlist count        │
│                                                 │
└─────────────────────────────────────────────────┘
```

**Responsive Breakpoints:**
- Small phone: 320-374px
- Medium phone: 375-414px
- Large phone: 415-767px
- Tablet portrait: 768-1023px
- Tablet landscape: 1024px+

---

### 8.2 Web Application (Desktop/Laptop)

**Platform:** Responsive web app (React/Next.js)

**Design Priorities:**
1. **Screen Real Estate**
   - Multi-column layouts
   - Sidebar navigation
   - Inline detail views (no need for modals)

2. **Mouse/Keyboard Interactions**
   - Hover states for all interactive elements
   - Keyboard shortcuts (power users)
   - Context menus (right-click)
   - Drag and drop (reorder watchlist)

3. **Progressive Enhancement**
   - Works without JavaScript (basic search)
   - Enhanced with JS (real-time filtering)
   - WebGL for advanced visualizations (map view)

**Desktop-Specific Features:**

```
┌──────────────────────────────────────────────────────────┐
│ DESKTOP-ONLY FEATURES                                    │
├──────────────────────────────────────────────────────────┤
│                                                          │
│ 1. Multi-Pane Interface                                 │
│    - Results + filters + detail in one view             │
│    - No need to navigate between screens                │
│                                                          │
│ 2. Keyboard Shortcuts                                   │
│    - ⌘K: Focus search                                   │
│    - ←→: Navigate results                               │
│    - Space: Preview                                     │
│    - Esc: Close modal                                   │
│    - J/K: Next/previous (Vim-style)                     │
│                                                          │
│ 3. Advanced Search                                      │
│    - Boolean operators (AND, OR, NOT)                   │
│    - Saved search queries                               │
│    - Batch operations (add multiple to watchlist)       │
│                                                          │
│ 4. Browser Extensions                                   │
│    - Netflix/Prime page → "Find Similar" button         │
│    - Highlight actor → See their filmography            │
│                                                          │
│ 5. Picture-in-Picture                                   │
│    - Watch trailer while browsing                       │
│    - Detached video player                              │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

**Responsive Breakpoints:**
- Small desktop: 1024-1279px (3-column grid)
- Medium desktop: 1280-1919px (4-column grid)
- Large desktop: 1920px+ (6-column grid)
- Ultrawide: 2560px+ (8-column grid with wider margins)

---

### 8.3 Voice Interface (Smart Speakers)

**Platform:** Alexa, Google Assistant, Siri

**Design Priorities:**
1. **Conversational Design**
   - Natural language understanding
   - Context retention (multi-turn conversations)
   - Confirmation before actions

2. **Audio-First UX**
   - Clear verbal responses
   - Ordered lists (first, second, third)
   - Repeat functionality

3. **Companion Display**
   - Smart displays show visual results
   - Voice-only fallback for speakers

**Voice Interaction Patterns:**

```
┌─────────────────────────────────────────────────┐
│ VOICE CONVERSATION EXAMPLES                     │
├─────────────────────────────────────────────────┤
│                                                 │
│ Example 1: Simple Query                         │
│ ─────────────────────────                       │
│ User:  "Find me a comedy"                       │
│ System: "I found 8 highly-rated comedies.       │
│          The top pick is The Grand Budapest     │
│          Hotel, rated 8.1. Would you like       │
│          to hear more options?"                 │
│ User:  "Yes"                                    │
│ System: "Second is Groundhog Day, rated 8.0.    │
│          Third is The Big Lebowski..."          │
│                                                 │
│ Example 2: Multi-Turn Refinement                │
│ ─────────────────────────────────               │
│ User:  "Find something to watch"                │
│ System: "What kind of mood are you in?"         │
│ User:  "Something thrilling"                    │
│ System: "Alright, thrillers. Any preference     │
│          for new or classic?"                   │
│ User:  "Recent"                                 │
│ System: "I have 5 recent thrillers. The top     │
│          pick is No Time to Die from 2021..."   │
│                                                 │
│ Example 3: Platform-Specific                    │
│ ─────────────────────────────                   │
│ User:  "What action movies are on Netflix?"     │
│ System: "On Netflix, the top action movie is    │
│          Extraction 2, rated 7.2. Also available│
│          are The Gray Man and..."               │
│ User:  "Tell me about Extraction 2"             │
│ System: [Reads synopsis, asks if user wants to  │
│          add to watchlist or open Netflix]      │
│                                                 │
└─────────────────────────────────────────────────┘
```

**Voice-Specific Commands:**
- "Alexa, ask [App Name] for a comedy"
- "Hey Google, what should I watch tonight?"
- "Siri, find movies like Inception on [App Name]"
- "Add to watchlist"
- "Open on Netflix"

**Verbal Response Design:**
- Keep responses under 30 seconds
- Offer "hear more" for longer content
- Provide 3-5 options max per response
- Use natural language (avoid jargon)
- Include ratings and availability in summary

---

### 8.4 TV/Streaming Device Interface

**Platform:** Apple TV, Android TV, Fire TV, Roku

**Design Priorities:**
1. **10-Foot UI**
   - Large text (min 24px)
   - High contrast colors
   - Generous spacing (4x normal)
   - Focus states super clear

2. **D-Pad Navigation**
   - Arrow keys for navigation
   - Large click targets (100x100px minimum)
   - Logical focus flow
   - Wrap-around navigation

3. **Remote-Friendly**
   - Minimal text input (use voice)
   - Quick actions (watchlist, share)
   - Predictable button mapping

**TV Interface Layout:**

```
┌────────────────────────────────────────────────────────────────────────┐
│ [App Logo]                                               [Search] [⚙]  │
├────────────────────────────────────────────────────────────────────────┤
│                                                                        │
│  ┌──────────────────────────────────────────────────────────────────┐ │
│  │                                                                  │ │
│  │  What do you want to watch?                                     │ │
│  │                                                                  │ │
│  │  [Use voice remote or select below]                             │ │
│  │                                                                  │ │
│  └──────────────────────────────────────────────────────────────────┘ │
│                                                                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐               │
│  │              │  │              │  │              │               │
│  │  Trending    │  │  Surprise Me │  │  By Mood     │               │
│  │              │  │              │  │              │               │
│  └──────────────┘  └──────────────┘  └──────────────┘               │
│        [FOCUSED]                                                      │
│                                                                        │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│                                                                        │
│  Trending This Week                                                   │
│                                                                        │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  │
│  │         │  │         │  │         │  │         │  │         │  │
│  │ Poster  │  │ Poster  │  │ Poster  │  │ Poster  │  │ Poster  │  │
│  │ 240x360 │  │         │  │         │  │         │  │         │  │
│  │         │  │         │  │         │  │         │  │         │  │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘  └─────────┘  │
│   Title        Title        Title        Title        Title         │
│   ⭐ 8.5       ⭐ 7.9       ⭐ 9.0       ⭐ 8.1       ⭐ 8.7        │
│                                                                        │
│  [← Previous Row]                              [More Trending →]      │
│                                                                        │
└────────────────────────────────────────────────────────────────────────┘
```

**Remote Button Mapping:**
- **D-Pad:** Navigate focus
- **Select/OK:** Open details
- **Back:** Previous screen/close modal
- **Play:** Start watching (if on detail screen)
- **Voice Button:** Activate voice search
- **Menu:** Open app menu

**TV-Specific Features:**
- Auto-play trailers when focused (after 3s)
- Screensaver mode (rotating recommendations)
- Family profile switching
- Parental controls integration

---

### 8.5 Cross-Platform Consistency

**Unified Elements Across All Platforms:**

1. **Branding**
   - Same logo and color palette
   - Consistent voice and tone
   - Similar visual hierarchy

2. **Core Features**
   - Search, filter, watchlist available everywhere
   - Same data, synced in real-time
   - Consistent result quality

3. **Account & Sync**
   - Sign in once, work everywhere
   - Watchlist syncs across devices
   - Preferences carry over
   - Continue searching from where you left off

**Platform-Adaptive Design:**

| Feature | Mobile | Web | Voice | TV |
|---------|--------|-----|-------|-----|
| **Primary Input** | Touch | Mouse/Keyboard | Voice | Remote |
| **Screen Size** | Small | Large | N/A | Huge |
| **Layout** | Single column | Multi-column | N/A | Large cards |
| **Navigation** | Bottom bar | Top bar + sidebar | Conversational | D-pad focus |
| **Detail View** | Modal | Inline/Modal | Verbal | Full screen |
| **Filters** | Bottom sheet | Sidebar | Verbal | On-screen menu |

**Handoff Features:**
- "Continue on phone" (QR code on TV)
- "Open on TV" (cast button from mobile)
- "Send to device" (cross-device sharing)

---

## 9. Error Handling & Fallback Experiences

### 9.1 Error Types & User-Friendly Messages

**Error Philosophy:**
- Never blame the user
- Provide clear next steps
- Maintain helpful, friendly tone
- Preserve user's input/context

**Common Errors:**

| Error Type | Technical Cause | User-Friendly Message | Recovery Action |
|------------|----------------|----------------------|----------------|
| **No Results** | Query too specific, no matches | "Hmm, we couldn't find anything matching '[query]'. Try broadening your search or rephrasing." | Suggestions for broader search |
| **Network Error** | API timeout, offline | "Looks like we lost connection. Check your internet and try again." | Retry button, offline mode |
| **Invalid Query** | Malformed input, unsupported characters | "We didn't quite understand that. Try describing what you want to watch." | Clear input, show examples |
| **Rate Limited** | Too many requests | "Whoa, slow down! Take a breath and try again in a moment." | Countdown timer, queue request |
| **Service Unavailable** | Streaming API down | "We're having trouble checking availability on [Platform]. Try another platform or check back soon." | Show results without that platform |
| **Voice Recognition Failure** | Speech-to-text error | "Sorry, we didn't catch that. Try again or type your query instead." | Switch to text input |

---

### 9.2 Error UI Patterns

**Pattern 1: No Results Found**

```
┌─────────────────────────────────────────────────┐
│ No results for "obscure-film-that-doesnt-exist" │
│                                                 │
│         ┌─────────────┐                         │
│         │   📭        │                         │
│         │  Empty Box  │                         │
│         └─────────────┘                         │
│                                                 │
│  We couldn't find anything matching that.      │
│                                                 │
│  Try:                                           │
│  • Using different keywords                     │
│  • Checking your spelling                       │
│  • Searching for a genre or mood instead        │
│                                                 │
│  Similar searches:                              │
│  [obscure thriller] [indie films] [hidden gems] │
│                                                 │
│  ┌───────────────────────────────────────────┐ │
│  │ [Try a different search]                  │ │
│  └───────────────────────────────────────────┘ │
│                                                 │
└─────────────────────────────────────────────────┘
```

**Smart Suggestions:**
- Fuzzy matching for typos ("Incepton" → "Did you mean Inception?")
- Related searches based on partial match
- Trending queries in same genre
- "Browse instead" option (curated categories)

---

**Pattern 2: Network Error**

```
┌─────────────────────────────────────────────────┐
│                                                 │
│         ┌─────────────┐                         │
│         │   📡💔      │                         │
│         │  No Signal  │                         │
│         └─────────────┘                         │
│                                                 │
│  Connection Lost                                │
│                                                 │
│  Check your internet connection and try again.  │
│                                                 │
│  ┌───────────────────────────────────────────┐ │
│  │ [↻ Retry]                                 │ │
│  └───────────────────────────────────────────┘ │
│                                                 │
│  [View offline watchlist]                       │
│                                                 │
└─────────────────────────────────────────────────┘
```

**Offline Capabilities:**
- Cache last 10 searches (show stale data with indicator)
- Offline watchlist access
- "Save for later" queue (syncs when online)
- Graceful degradation (show basic info without fresh availability data)

---

**Pattern 3: Service Unavailable**

```
┌─────────────────────────────────────────────────┐
│  ⚠️ Partial Results                             │
│                                                 │
│  We're having trouble connecting to Netflix     │
│  right now, so availability info might not      │
│  be up to date.                                 │
│                                                 │
│  Results from other platforms:                  │
│                                                 │
│  [Results grid with "Netflix: Unknown" status]  │
│                                                 │
│  [Check Netflix directly ↗]                     │
│                                                 │
└─────────────────────────────────────────────────┘
```

**Degraded Experience:**
- Show results without unavailable data
- Clear indicator of what's missing
- Provide alternative platforms
- Link directly to problematic service

---

### 9.3 Voice-Specific Error Handling

**Voice Errors:**

| Error | Voice Response | Visual Fallback |
|-------|---------------|----------------|
| **No speech detected** | "I didn't hear anything. Could you try again?" | Show tap-to-speak button |
| **Ambient noise** | "It's a bit noisy. Could you repeat that?" | Noise level indicator |
| **Unclear speech** | "I heard '[partial transcription]'. Did I get that right?" | Show transcription, allow edit |
| **Unsupported query** | "I'm not sure about that. Try asking for a movie or show genre, or actor name." | Show example queries |

**Voice Error Recovery:**

```
System: "Sorry, I didn't catch that. Did you say
         'comedy' or 'drama'?"

User:   "Comedy"

System: "Got it! Searching for comedies..."
```

---

### 9.4 Fallback Content Strategies

**When Personalization Fails:**
- Show trending content instead of "For You"
- Generic recommendations (crowd-sourced ratings)
- Curated editorial picks

**When Search Fails:**
- Suggest browsing by category
- Show recently popular searches
- Offer "Surprise Me" feature

**When Platform Data Missing:**
- Show "Availability unknown" status
- Provide direct links to check manually
- Historical data (last known availability)

**When Images Fail to Load:**
```
┌─────────┐
│         │
│  [🎬]   │  ← Fallback icon
│  Title  │  ← Text still readable
│         │
└─────────┘
```

---

### 9.5 Proactive Error Prevention

**Input Validation:**
```
User types: "show me @#$%"
  ↓
System: [Inline message] "Special characters aren't needed.
         Try describing what you want to watch."
```

**Smart Autocorrect:**
```
User types: "Incepton"
  ↓
Autocomplete: "Inception" (with confidence)
```

**Contextual Guidance:**
```
Empty search box shows rotating placeholders:
- "Try: 'comedy from the 90s'"
- "Try: 'something like Inception'"
- "Try: 'action movies on Netflix'"
```

**Rate Limiting Warnings:**
```
After 10 searches in 1 minute:
"Whoa, you're searching fast! Take a moment to review
 your results before searching more."
```

---

### 9.6 Error Analytics & Improvement

**Track Error Metrics:**
- Error frequency by type
- Recovery success rate (user retried and succeeded)
- Abandonment after error
- Most common error paths

**Use Errors to Improve:**
- Frequent "no results" for specific queries → Expand dataset
- High voice recognition errors → Improve acoustic model
- Common typos → Add to autocorrect dictionary
- Service downtime patterns → Cache strategy optimization

---

## 10. Accessibility Requirements

### 10.1 WCAG 2.1 Compliance Targets

**Compliance Level:** AAA where possible, AA minimum

**Key Standards:**

| Criterion | Level | Implementation |
|-----------|-------|---------------|
| **1.4.3 Contrast (Minimum)** | AA | 4.5:1 for normal text, 3:1 for large text |
| **1.4.6 Contrast (Enhanced)** | AAA | 7:1 for normal text, 4.5:1 for large text |
| **2.1.1 Keyboard** | A | All functionality available via keyboard |
| **2.4.7 Focus Visible** | AA | Clear focus indicators (3px outline, high contrast) |
| **3.2.4 Consistent Identification** | AA | UI elements function consistently |
| **4.1.2 Name, Role, Value** | A | All components properly labeled for AT |

---

### 10.2 Visual Accessibility

**Color & Contrast:**

```css
/* High contrast color palette */
--text-primary: #000000;        /* Black on white: 21:1 */
--text-secondary: #404040;      /* Dark grey: 12:1 */
--background: #FFFFFF;
--accent-primary: #0052CC;      /* Blue: 8.6:1 */
--accent-secondary: #C41E3A;    /* Red: 5.9:1 */
--success: #006644;             /* Green: 7.3:1 */
--warning: #B54800;             /* Orange: 5.1:1 */

/* Focus states */
--focus-outline: 3px solid #0052CC;
--focus-offset: 2px;
```

**Never rely on color alone:**
```
❌ Bad:  [Button in red] [Button in green]
✅ Good: [✕ Cancel] [✓ Confirm]
         (Icons + color + text)
```

**Text Sizing:**
- Base font: 16px minimum
- Allow user scaling up to 200% without breaking layout
- Use relative units (rem, em)
- Respect user's browser font size preferences

**Visual Indicators:**
```
Status indicators must combine:
- Color (e.g., green for available)
- Icon (e.g., ✓ checkmark)
- Text (e.g., "Available on Netflix")
```

---

### 10.3 Keyboard Accessibility

**Navigation Requirements:**

1. **Tab Order**
   - Logical, predictable sequence
   - Matches visual layout (left-to-right, top-to-bottom)
   - Skip links to main content
   - No keyboard traps

2. **Keyboard Shortcuts**
   ```
   Global:
   - Tab: Next focusable element
   - Shift+Tab: Previous element
   - Enter/Space: Activate
   - Esc: Close modal/cancel
   - ⌘/Ctrl+K: Focus search

   Results Grid:
   - Arrow keys: Navigate cards
   - Home/End: First/last item
   - Page Up/Down: Scroll

   Detail View:
   - Tab: Cycle through actions
   - Esc: Close detail view
   ```

3. **Focus Management**
   ```javascript
   // When opening modal, trap focus
   modal.open() {
     previousFocus = document.activeElement;
     focusFirstElement(modal);
     trapFocus(modal);
   }

   // When closing, restore focus
   modal.close() {
     previousFocus.focus();
   }
   ```

**Visual Focus Indicators:**
```css
/* Clear, high-contrast focus states */
.focusable:focus {
  outline: 3px solid var(--accent-primary);
  outline-offset: 2px;
  box-shadow: 0 0 0 3px rgba(0, 82, 204, 0.2);
}

/* Never remove focus styles without replacement */
:focus {
  outline: none; /* ❌ BAD */
}

:focus-visible {
  outline: 3px solid blue; /* ✅ GOOD */
}
```

---

### 10.4 Screen Reader Accessibility

**Semantic HTML:**
```html
<!-- Use proper HTML structure -->
<header>
  <nav aria-label="Main navigation">
    <ul role="list">
      <li><a href="/">Home</a></li>
      <li><a href="/search">Search</a></li>
    </ul>
  </nav>
</header>

<main id="main-content">
  <section aria-labelledby="results-heading">
    <h2 id="results-heading">Search Results</h2>
    <!-- Results here -->
  </section>
</main>

<footer>
  <!-- Footer content -->
</footer>
```

**ARIA Labels & Descriptions:**
```html
<!-- Button without visible text -->
<button aria-label="Close dialog">
  <svg>...</svg> <!-- X icon -->
</button>

<!-- Search input -->
<label for="search-input" class="sr-only">Search for movies and shows</label>
<input
  id="search-input"
  type="text"
  aria-describedby="search-instructions"
  placeholder="What do you want to watch?"
/>
<span id="search-instructions" class="sr-only">
  Enter a movie title, actor, genre, or describe what you're looking for
</span>

<!-- Result card -->
<article
  role="article"
  aria-labelledby="movie-title-123"
  aria-describedby="movie-meta-123"
>
  <h3 id="movie-title-123">Inception</h3>
  <div id="movie-meta-123">
    <span aria-label="Rating">8.8 out of 10</span>
    <span aria-label="Runtime">2 hours 28 minutes</span>
    <span aria-label="Available on">Netflix, Amazon Prime</span>
  </div>
</article>
```

**Live Regions for Dynamic Content:**
```html
<!-- Announce search results -->
<div aria-live="polite" aria-atomic="true" class="sr-only">
  Found 27 results for "comedy from the 90s"
</div>

<!-- Announce filtering changes -->
<div aria-live="polite" class="sr-only">
  Results filtered. Now showing 14 movies.
</div>

<!-- Error announcements -->
<div aria-live="assertive" role="alert" class="sr-only">
  Connection lost. Please check your internet.
</div>
```

**Screen Reader Only Text:**
```css
/* Utility class for SR-only content */
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}
```

---

### 10.5 Motor Accessibility

**Touch Target Sizing:**
```css
/* All interactive elements minimum 44x44px */
.button, .link, .card {
  min-height: 44px;
  min-width: 44px;
  padding: 12px 16px;
}

/* Adequate spacing between targets */
.button-group > * {
  margin: 8px; /* 16px total spacing */
}
```

**Reduce Motion:**
```css
/* Respect prefers-reduced-motion */
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}

/* Disable parallax and complex animations */
@media (prefers-reduced-motion: reduce) {
  .parallax {
    transform: none !important;
  }

  .auto-play-video {
    display: none; /* Don't autoplay for motion sensitivity */
  }
}
```

**Voice Control Support:**
```html
<!-- Unique, descriptive labels for voice commands -->
<button aria-label="Add Inception to watchlist">
  Add to Watchlist
</button>

<!-- Voice command: "Click add inception to watchlist" -->
```

---

### 10.6 Cognitive Accessibility

**Simplicity & Clarity:**
- Use plain language (8th grade reading level)
- Short sentences and paragraphs
- Clear headings and structure
- Avoid jargon

**Consistency:**
- Predictable navigation placement
- Consistent button labeling
- Familiar UI patterns
- Same actions perform same way

**Error Prevention & Recovery:**
- Clear form validation
- Confirmation for destructive actions
- Undo functionality where possible
- Helpful error messages with solutions

**Avoid Cognitive Overload:**
```
❌ Bad:  Show 100 results at once, 20 filter options
✅ Good: Show 10 results, 5 primary filters, expandable for more
```

---

### 10.7 Deaf/Hard of Hearing Support

**Captions & Transcripts:**
- All video content (trailers) must have captions
- Provide transcripts for audio-only content
- Visual indicators for audio cues

**Visual Feedback:**
```html
<!-- Don't rely on sound alone -->
❌ Bad:  [Plays "ding" sound on successful search]
✅ Good: [Shows checkmark + "Search complete" text + optional sound]
```

---

### 10.8 Internationalization (i18n)

**Language Support:**
- Text direction support (LTR, RTL)
- Dynamic font loading for non-Latin scripts
- Date/time formatting per locale
- Currency formatting

**Cultural Considerations:**
- Color meanings vary by culture
- Icon interpretation differences
- Appropriate imagery

```html
<!-- Language declaration -->
<html lang="en">

<!-- RTL support -->
<html lang="ar" dir="rtl">

<!-- Language switcher -->
<select aria-label="Choose language">
  <option value="en">English</option>
  <option value="es">Español</option>
  <option value="fr">Français</option>
</select>
```

---

### 10.9 Accessibility Testing Checklist

**Automated Testing:**
- [ ] axe DevTools (Chrome extension)
- [ ] Lighthouse Accessibility audit (Chrome DevTools)
- [ ] WAVE (WebAIM accessibility tool)
- [ ] Pa11y (CI/CD integration)

**Manual Testing:**
- [ ] Keyboard-only navigation
- [ ] Screen reader testing (NVDA, JAWS, VoiceOver)
- [ ] Color contrast verification (Contrast Checker)
- [ ] Text scaling to 200%
- [ ] Reduce motion preference
- [ ] Mobile touch target sizing

**User Testing:**
- [ ] Recruit users with disabilities
- [ ] Conduct usability testing
- [ ] Gather feedback on pain points
- [ ] Iterate based on findings

---

### 10.10 Accessibility Statement

**Publish an Accessibility Statement:**
```markdown
# Accessibility Commitment

We're committed to making [App Name] accessible to everyone.

**Current Compliance:**
- WCAG 2.1 Level AA compliant
- Working toward AAA where possible

**Features:**
- Keyboard navigation support
- Screen reader optimization
- High contrast color scheme
- Reduced motion option
- Resizable text up to 200%

**Known Issues:**
- [List any current limitations]

**Feedback:**
Contact accessibility@[app].com to report issues.

**Last Updated:** 2025-12-05
```

---

## Conclusion

This UX design specification provides a comprehensive foundation for building an instant media discovery platform that is:

✅ **Fast** - Instant results, minimal friction
✅ **Intuitive** - Natural language queries, multiple input methods
✅ **Accessible** - WCAG 2.1 AA/AAA compliant, inclusive design
✅ **Delightful** - Smooth interactions, helpful intelligence
✅ **Trustworthy** - Clear sourcing, transparent recommendations

**Next Steps:**
1. **Prototype**: Build interactive prototypes for key flows
2. **User Test**: Validate designs with target users
3. **Iterate**: Refine based on feedback
4. **Develop**: Implement with design system
5. **Measure**: Track metrics and continuously improve

---

**Document Version:** 1.0.0
**Design Team:** [Your Team]
**Review Cycle:** Quarterly
**Feedback:** ux-team@[app].com
