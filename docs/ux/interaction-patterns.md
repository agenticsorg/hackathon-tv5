# Interaction Patterns Reference

**Version:** 1.0.0
**Last Updated:** 2025-12-05

---

## Table of Contents

1. [Search Interactions](#search-interactions)
2. [Navigation Patterns](#navigation-patterns)
3. [Content Discovery](#content-discovery)
4. [Filtering & Sorting](#filtering--sorting)
5. [Detail Views](#detail-views)
6. [Watchlist Management](#watchlist-management)
7. [Voice Interactions](#voice-interactions)
8. [Social Features](#social-features)
9. [Feedback & Notifications](#feedback--notifications)
10. [Error States](#error-states)

---

## Search Interactions

### 1. Autocomplete Search

**Pattern:** Progressive disclosure with intelligent suggestions

**Interaction Flow:**
```
User types: "thr"
  ↓ (150ms delay)
Suggestions appear:
  - Thriller (genre)
  - Three Billboards Outside Ebbing, Missouri (movie)
  - The throne room scene (query)
  - Thrilling documentaries (category)
  ↓
User continues typing: "thriller"
  ↓
Suggestions refine:
  - Psychological thriller (genre)
  - Thriller (1973) (movie)
  - Thriller movies on Netflix (query)
  ↓
User presses Enter or clicks suggestion
  ↓
Results appear
```

**Implementation Details:**
- **Debounce delay:** 150ms after user stops typing
- **Max suggestions:** 5-8 items
- **Keyboard navigation:** ↑↓ arrows to navigate, Enter to select
- **Categories:** Group suggestions by type (genres, titles, actors)
- **Highlight:** Bold the matching portion of text

**Example Code:**
```javascript
const [query, setQuery] = useState('');
const [suggestions, setSuggestions] = useState([]);

const fetchSuggestions = useDebounce((value) => {
  if (value.length >= 2) {
    api.autocomplete(value).then(setSuggestions);
  }
}, 150);

useEffect(() => {
  fetchSuggestions(query);
}, [query]);
```

---

### 2. Voice Search

**Pattern:** Multimodal feedback (visual + audio)

**Interaction Flow:**
```
User taps microphone icon
  ↓
Microphone activates (pulsing animation)
Haptic feedback (mobile)
  ↓
User speaks: "Find me a comedy"
  ↓
Real-time transcription appears
Waveform visualizes audio input
  ↓
2 seconds of silence detected
  ↓
System confirms: "Searching for comedies..."
  ↓
Results appear
Optional: Verbal summary of results
```

**States:**
1. **Idle:** Static microphone icon
2. **Listening:** Pulsing animation, waveform
3. **Processing:** Spinner, "Understanding..."
4. **Completed:** Checkmark, transition to results
5. **Error:** X icon, "Didn't catch that, try again"

**Accessibility:**
- Provide text alternative (edit transcription)
- Visual indicators for all states
- Option to switch to text input anytime

---

### 3. Visual Search

**Pattern:** Upload → Analysis → Results

**Interaction Flow:**
```
User uploads movie poster image
  ↓
Loading state with progress bar
  ↓
Image analysis (OCR + visual recognition)
  ↓
System identifies elements:
  - Movie title detected
  - Actors recognized
  - Visual style analyzed
  ↓
Results:
  "This appears to be [Movie Name]"
  + Similar movies based on visual style
  + Movies with same actors
  + Movies in same genre
```

**Upload Methods:**
- Drag and drop
- File picker
- Camera capture (mobile)
- Paste from clipboard
- URL input

---

## Navigation Patterns

### 4. Breadcrumb Navigation

**Pattern:** Location awareness with quick backtracking

**Structure:**
```
Home > Search Results > The Dark Knight > Similar Movies
 ↑        ↑                  ↑                    ↑
Link    Link            Current Page        Current Page
```

**Behavior:**
- Current page not clickable
- Separator: > or /
- Mobile: Show only current + parent
- Desktop: Show full path, truncate middle if too long

---

### 5. Tab Navigation

**Pattern:** Switching between related content sections

**Example: Movie Detail Page**
```
┌──────────────────────────────────────────┐
│ [Overview] [Cast] [Reviews] [Similar]   │ ← Tabs
├──────────────────────────────────────────┤
│                                          │
│ Content for selected tab                 │
│                                          │
└──────────────────────────────────────────┘
```

**States:**
- **Active:** Accent color, underline
- **Inactive:** Muted color
- **Hover:** Brighter color
- **Disabled:** Greyed out, no pointer

**Accessibility:**
- ARIA role="tablist", role="tab", role="tabpanel"
- Arrow key navigation
- Focus management

---

### 6. Infinite Scroll

**Pattern:** Continuous loading as user scrolls

**Interaction Flow:**
```
User scrolls down
  ↓
Reaches 80% of page height
  ↓
Trigger "load more"
  ↓
Loading indicator appears
  ↓
New results append to list
  ↓
Repeat
```

**Best Practices:**
- Threshold: 80% scroll position
- Batch size: 10-20 items per load
- Loading indicator: Skeleton screens or spinner
- "Back to top" button after 2-3 loads
- Preserve scroll position on back navigation

**Accessibility:**
- Announce new items to screen readers
- Keyboard: Provide "Load More" button alternative
- Stop infinite scroll after 100 items, require manual action

---

## Content Discovery

### 7. Horizontal Carousel

**Pattern:** Swipeable content rows

**Layout:**
```
┌──────────────────────────────────────────────────┐
│ Trending This Week               [← →] [See All]│
│ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐               │
│ │     │ │     │ │     │ │     │  →            │
│ └─────┘ └─────┘ └─────┘ └─────┘               │
└──────────────────────────────────────────────────┘
```

**Behaviors:**
- **Touch:** Swipe left/right
- **Mouse:** Drag or click arrows
- **Keyboard:** Tab to arrows, press Enter
- **Scroll snap:** Snap to item boundaries
- **Momentum:** Continue scrolling after swipe

**Indicators:**
- Navigation arrows (hide on mobile)
- Scroll progress dots (optional)
- Fade effect on edges to indicate more content

---

### 8. Mood-Based Discovery

**Pattern:** Emotion-driven content filtering

**UI Flow:**
```
"How are you feeling?"

[😊 Happy] [😢 Sad] [😱 Thrilled] [😌 Relaxed]
[🤔 Thoughtful] [😂 Playful] [😤 Intense]

User selects: 😌 Relaxed
  ↓
Refined mood options:
[🌿 Nature documentaries]
[☕ Cozy comedies]
[🎨 Art house films]
  ↓
User selects: ☕ Cozy comedies
  ↓
Results: Comfort movies perfect for unwinding
```

**Mood Categories:**
- Positive: Happy, Playful, Excited
- Negative: Sad, Anxious, Angry
- Neutral: Thoughtful, Curious, Bored
- Energy: Energetic, Relaxed, Sleepy

---

### 9. "Surprise Me" Feature

**Pattern:** Random recommendations with explanation

**Interaction Flow:**
```
User clicks "Surprise Me"
  ↓
Loading animation (fun, playful)
"Finding something special for you..."
  ↓
Single recommendation appears:
┌──────────────────────────────────────┐
│   [Large poster image]               │
│                                      │
│   The Grand Budapest Hotel (2014)   │
│   ⭐ 8.1  •  Comedy, Drama           │
│                                      │
│   Why we picked this:                │
│   "You loved Moonrise Kingdom, and   │
│   this has the same whimsical Wes    │
│   Anderson style."                   │
│                                      │
│   [Watch Now] [Not Interested]       │
│   [Surprise Me Again]                │
└──────────────────────────────────────┘
```

**Features:**
- Personalized explanation
- Easy accept/reject
- Generate new recommendation quickly
- Learn from rejections

---

## Filtering & Sorting

### 10. Multi-Select Filters

**Pattern:** Cumulative filtering with live results

**UI Example:**
```
Filters:

Genre (Multi-select)
☑ Comedy
☑ Drama
☐ Action
☐ Horror

Year (Range slider)
1990 ━━━●━━━━━━ 2024

Rating (Dropdown)
[7.0+ ▼]

Results update: 47 movies → 23 movies
```

**Behaviors:**
- Filters combine with AND logic
- Results count updates in real-time
- Clear all button
- Save filter combinations

---

### 11. Sort Options

**Pattern:** Dropdown with common sort criteria

**Options:**
```
Sort by: [Relevance ▼]

Dropdown options:
- Relevance (default for search)
- Rating: High to Low
- Rating: Low to High
- Release Date: Newest First
- Release Date: Oldest First
- Title: A to Z
- Title: Z to A
- Runtime: Shortest First
- Runtime: Longest First
- Popularity
```

**Persistence:**
- Remember user's last sort preference
- Per-context sorting (search vs. browse)

---

### 12. Filter Chips (Active Filters)

**Pattern:** Visual representation of active filters

**Layout:**
```
Active Filters:
[Comedy ×] [1990s ×] [Netflix ×] [7.0+ ×]    [Clear All]
```

**Interactions:**
- Click X to remove individual filter
- Click chip body to edit filter value
- Drag to reorder (desktop)
- "Clear All" removes everything

---

## Detail Views

### 13. Modal Detail View

**Pattern:** Overlay with rich information

**Structure:**
```
┌─────────────────────────────────────────────┐
│                                      [× Close]│
│  [Hero image backdrop]                      │
│                                             │
│  Title, Year, Rating                        │
│  [▶ Watch Now]  [+ Watchlist]  [Share]     │
│                                             │
│  Synopsis...                                │
│                                             │
│  Cast: [Actor] [Actor] [Actor]             │
│                                             │
│  [Trailer] [Reviews] [Similar]             │
│                                             │
└─────────────────────────────────────────────┘
```

**Opening Animation:**
- Scale from card position
- Background blur
- Fade in backdrop

**Closing:**
- ESC key
- Click outside
- Close button
- Swipe down (mobile)

---

### 14. Expandable Cards

**Pattern:** In-place expansion without modal

**Collapsed State:**
```
┌─────────┬────────────────┐
│ Poster  │ Title          │
│         │ Rating • Year  │
│         │ [More ▼]       │
└─────────┴────────────────┘
```

**Expanded State:**
```
┌─────────┬──────────────────────────────┐
│ Poster  │ Title                        │
│         │ Rating • Year • Runtime      │
│         │                              │
│         │ Synopsis paragraph...        │
│         │                              │
│         │ Cast: Actor, Actor...        │
│         │                              │
│         │ [Watch] [Watchlist] [Less ▲]│
└─────────┴──────────────────────────────┘
```

**Use Case:** List views where modals would be disruptive

---

### 15. Trailer Preview

**Pattern:** Hover-triggered video preview

**Desktop Behavior:**
```
User hovers on card for 1 second
  ↓
Trailer thumbnail/GIF appears
Sound muted, auto-plays
  ↓
User moves mouse away
  ↓
Trailer stops, returns to poster
```

**Mobile Behavior:**
```
User long-presses card
  ↓
Haptic feedback
Trailer preview modal opens
  ↓
User releases or taps X
  ↓
Modal closes
```

---

## Watchlist Management

### 16. Add to Watchlist

**Pattern:** One-tap save with confirmation

**Interaction Flow:**
```
User clicks "+ Watchlist" button
  ↓
Button changes: "✓ Added to Watchlist"
Background color changes (subtle green)
Haptic feedback (mobile)
Toast notification: "Added to Watchlist"
  ↓
After 2 seconds, button reverts to "✓ In Watchlist"
```

**States:**
- Not in watchlist: "+ Add to Watchlist"
- In watchlist: "✓ In Watchlist"
- Hover (when in watchlist): "✕ Remove"

---

### 17. Watchlist Organization

**Pattern:** Folders/categories for watchlist items

**UI:**
```
My Watchlist

[+ New Folder]

📁 To Watch This Week (5)
📁 Date Night Ideas (12)
📁 Documentary Queue (8)
📁 Rewatches (3)

Uncategorized (23)
```

**Features:**
- Drag and drop to folders
- Multiple items can be in multiple folders
- Sort within folders
- Share folders with friends

---

## Voice Interactions

### 18. Wake Word Detection

**Pattern:** Always-listening mode (opt-in)

**Flow:**
```
User says: "Hey [App Name]"
  ↓
Visual indicator activates
  ↓
System responds: "Yes?"
  ↓
User: "Find me a comedy"
  ↓
System: [Executes search]
```

**Privacy:**
- Opt-in only
- Local processing for wake word
- Visual indicator when listening
- Easy disable toggle

---

### 19. Voice Refinement

**Pattern:** Conversational follow-ups

**Conversation:**
```
User: "Find action movies"
System: "I found 47 action movies."
User: "Only on Netflix"
System: "Filtering to Netflix. 12 action movies."
User: "From the last 5 years"
System: "Here are 8 recent action movies on Netflix."
```

**Context Retention:**
- Remember previous 3-5 queries in conversation
- Clear context after 2 minutes of inactivity
- User can say "Start over" to clear

---

## Social Features

### 20. Group Watch Session

**Pattern:** Collaborative decision making

**Flow:**
```
User creates session
  ↓
Gets shareable link
  ↓
Friends join (no account needed)
  ↓
Everyone submits preferences
  ↓
System shows matched results
  ↓
Group votes (thumbs up/down)
  ↓
First to 3 votes wins
  ↓
Option to start watch party
```

**Real-time Updates:**
- Live voting counts
- New participant joins
- Someone changed their vote
- Decision reached

---

### 21. Share Recommendations

**Pattern:** Native share with preview

**Share Content:**
```
Check out "Inception" on [App Name]!

[Preview card with poster]

⭐ 8.8 • Sci-Fi, Thriller • 2h 28m
"A thief who steals corporate secrets..."

[Open in App] [Find Where to Watch]
```

**Share Targets:**
- Message apps (iMessage, WhatsApp)
- Social media (Twitter, Facebook)
- Email
- Copy link

---

## Feedback & Notifications

### 22. Toast Notifications

**Pattern:** Non-intrusive, temporary messages

**Placement:**
- Desktop: Bottom-right corner
- Mobile: Top center (below status bar)

**Duration:**
- Success/Info: 3 seconds
- Warning: 5 seconds
- Error: 7 seconds or manual dismiss

**Example:**
```
┌────────────────────────────────┐
│ ✓ Added to Watchlist           │
│ [Undo] [×]                     │
└────────────────────────────────┘
```

**Features:**
- Auto-dismiss with countdown bar
- Action button (Undo, View, etc.)
- Stack multiple toasts
- Pause on hover

---

### 23. Loading States

**Pattern:** Skeleton screens over spinners

**Skeleton Card:**
```
┌─────────────────┐
│                 │
│  ░░░░░░░░░░░░  │ ← Animated shimmer
│  ░░░░░░░░░░░░  │
│                 │
└─────────────────┘
  ░░░░░░░░
  ░░░░░░░░░░░░░
```

**Principles:**
- Match layout of loaded content
- Subtle shimmer animation
- Show immediately (no delay)
- Graceful transition to real content

---

### 24. Empty States

**Pattern:** Helpful guidance when no content

**Example: Empty Watchlist**
```
┌─────────────────────────────────────┐
│                                     │
│         📋                          │
│         (Icon)                      │
│                                     │
│   Your Watchlist is Empty          │
│                                     │
│   Save movies and shows you want   │
│   to watch later.                  │
│                                     │
│   [Browse Recommendations]          │
│                                     │
└─────────────────────────────────────┘
```

**Components:**
- Icon or illustration
- Headline explaining state
- Helpful description
- Primary action to resolve

---

## Error States

### 25. Form Validation

**Pattern:** Inline, real-time feedback

**Valid State:**
```
Email: user@example.com ✓
```

**Invalid State:**
```
Email: user@example
⚠ Please enter a valid email address
```

**Timing:**
- Validate on blur (after field loses focus)
- Show success immediately when valid
- Show error after user stops typing (500ms debounce)

---

### 26. Network Error Recovery

**Pattern:** Automatic retry with manual option

**UI:**
```
┌─────────────────────────────────────┐
│   ⚠️ Connection Lost                │
│                                     │
│   Retrying in 5 seconds...          │
│   [Retry Now] [View Offline]        │
└─────────────────────────────────────┘
```

**Retry Strategy:**
- Attempt 1: Immediate
- Attempt 2: 5 seconds
- Attempt 3: 15 seconds
- After 3 attempts: Manual retry only

---

### 27. Progressive Enhancement

**Pattern:** Core functionality without JavaScript

**Baseline (No JS):**
- Basic search form submission
- Server-side rendering of results
- Traditional pagination links
- Full page reloads

**Enhanced (With JS):**
- Real-time autocomplete
- Infinite scroll
- Smooth animations
- Optimistic UI updates

---

## Best Practices Summary

### ✅ Do:
- Provide immediate feedback (<100ms)
- Use skeleton screens over spinners
- Remember user's context and preferences
- Offer keyboard shortcuts for power users
- Preserve state during navigation
- Make all features accessible via keyboard
- Test with real content (long titles, missing images)

### ❌ Don't:
- Don't block user interactions during loading
- Don't use generic error messages ("Error 500")
- Don't auto-play audio without user consent
- Don't remove focus indicators
- Don't nest modals more than 2 levels deep
- Don't use hover-only interactions on touch devices

---

**Document Version:** 1.0.0
**Maintained By:** UX Team
**Last Review:** 2025-12-05
