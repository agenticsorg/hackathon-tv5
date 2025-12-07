# UI/UX Design Prompt: Multi-Agent Entertainment Discovery System

**Project**: Intelligent Multi-Agent Entertainment Discovery System
**Design Goal**: Create an intuitive, engaging user interface using YouTube's design language and cognitive science principles
**Target Framework**: Next.js / React (modern web application)

---

## 🎨 Design Brief

Create a modern, user-friendly web interface for an AI-powered entertainment discovery system that helps users find what to watch across multiple streaming platforms in seconds, not minutes.

### Core User Problem
Users spend an average of 45 minutes deciding what to watch due to:
- Too many streaming platforms (5+)
- Analysis paralysis from endless choices
- Generic, context-unaware recommendations
- Lack of trust in algorithm suggestions

### Solution
An 8-agent AI system that provides personalized, context-aware, safety-filtered recommendations with transparent reasoning and confidence scores.

---

## 🎯 Design Requirements

### Primary Goals
1. **Reduce cognitive load** - Make decisions easier, not harder
2. **Build trust** - Show AI reasoning transparently
3. **Increase engagement** - Make discovery enjoyable
4. **Ensure safety** - Family-friendly content filtering
5. **Enable sharing** - Social discovery features

### Success Metrics
- Time to first watch < 30 seconds
- User satisfaction score > 4.5/5
- Recommendation acceptance rate > 60%
- Return rate within 24 hours > 70%

---

## 🎨 YouTube Color Scheme

Use YouTube's official color palette to create familiarity and trust:

### Primary Colors
- **YouTube Red**: `#FF0000` (primary brand color)
- **Dark Red**: `#CC0000` (hover states, accents)
- **Light Red**: `#FF4444` (highlights, active states)

### Neutral Colors
- **Dark Gray (Background)**: `#0F0F0F` (dark mode primary)
- **Medium Gray**: `#272727` (cards, containers)
- **Light Gray**: `#3F3F3F` (borders, dividers)
- **White**: `#FFFFFF` (text on dark backgrounds)
- **Off-White**: `#F1F1F1` (light mode background)

### Semantic Colors
- **Success Green**: `#0F9D58` (approved, safe content)
- **Warning Yellow**: `#F4B400` (caution, pending)
- **Info Blue**: `#4285F4` (information, links)
- **Error Red**: `#DB4437` (errors, blocked content)

### Text Colors
- **Primary Text (Dark Mode)**: `#FFFFFF`
- **Secondary Text (Dark Mode)**: `#AAAAAA`
- **Primary Text (Light Mode)**: `#030303`
- **Secondary Text (Light Mode)**: `#606060`

---

## 🧠 Cognitive Science Principles

### 1. **Hick's Law** - Reduce Decision Time
- **Principle**: Time to decide increases logarithmically with choices
- **Application**:
  - Show 3-5 recommendations per view (not 20+)
  - Use progressive disclosure (show more on demand)
  - Implement "Not interested" quick actions
  - Provide clear categorization

### 2. **Miller's Law** - Limit Information Chunks
- **Principle**: People can hold 7±2 items in working memory
- **Application**:
  - Maximum 5-7 recommendation cards visible at once
  - Group related information (ratings, platform, genre)
  - Use chunking: "Top picks" / "Trending" / "Because you watched X"

### 3. **Fitts's Law** - Target Size & Distance
- **Principle**: Time to acquire a target depends on distance and size
- **Application**:
  - Large, touch-friendly buttons (minimum 44x44px)
  - Primary actions at bottom-right (thumb zone on mobile)
  - Quick actions near content cards
  - Reduce cursor travel distance

### 4. **Von Restorff Effect** - Make Important Items Stand Out
- **Principle**: Distinctive items are more memorable
- **Application**:
  - Highlight "Very High Confidence" recommendations
  - Use visual hierarchy (size, color, position)
  - Emphasize trending/hot content with 🔥 indicators
  - Make "Watch Now" button prominent

### 5. **Serial Position Effect** - Primacy & Recency
- **Principle**: First and last items are remembered best
- **Application**:
  - Place best recommendation at top
  - Secondary call-to-action at bottom
  - Key information at beginning/end of descriptions

### 6. **Gestalt Principles** - Visual Grouping
- **Principle**: Elements are perceived as unified wholes
- **Application**:
  - Group related metadata (platform + genre + rating)
  - Use white space to separate recommendations
  - Consistent card layouts create patterns
  - Proximity indicates relationships

### 7. **Progressive Disclosure** - Reveal Information Gradually
- **Principle**: Show only necessary information initially
- **Application**:
  - Collapsed detailed view, expand on click
  - "Why this recommendation?" revealed on hover
  - Advanced filters hidden until requested
  - Settings accessible but not prominent

### 8. **Feedback Loops** - Acknowledge User Actions
- **Principle**: Users need immediate feedback
- **Application**:
  - Loading states during AI processing
  - Success animations on actions
  - Real-time agent activity indicators
  - Micro-interactions (hover, click, swipe)

### 9. **Recognition Over Recall** - Visual Memory
- **Principle**: Recognition is easier than recall
- **Application**:
  - Show poster images (not just titles)
  - Use platform logos (not just names)
  - Visual genre indicators
  - Previously watched content marked

### 10. **Aesthetic-Usability Effect** - Beauty = Usability
- **Principle**: Beautiful interfaces feel easier to use
- **Application**:
  - Smooth animations and transitions
  - High-quality poster images
  - Consistent spacing and alignment
  - Professional typography

---

## 📱 Interface Layout Structure

### Homepage Layout

```
┌─────────────────────────────────────────────────┐
│  [Logo] Multi-Agent Discovery    [Profile] [⚙️] │  ← Header (sticky)
├─────────────────────────────────────────────────┤
│                                                 │
│  🎬 What do you want to watch?                  │  ← Hero Section
│  ┌─────────────────────────────────────────┐   │
│  │ "I'm in the mood for something..."      │   │  ← Smart Search Input
│  └─────────────────────────────────────────┘   │
│  [🎭 Comedy] [⚡ Action] [💭 Drama] [🌟 Top]    │  ← Quick Filters
│                                                 │
├─────────────────────────────────────────────────┤
│  🌟 TOP PICKS FOR YOU                           │  ← Section Header
│  ─────────────────────────────────────────────  │
│                                                 │
│  ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐│
│  │ [IMG]  │  │ [IMG]  │  │ [IMG]  │  │ [IMG]  ││  ← Recommendation Cards
│  │ Title  │  │ Title  │  │ Title  │  │ Title  ││
│  │ ⭐ 9.0 │  │ ⭐ 8.8 │  │ ⭐ 8.7 │  │ ⭐ 8.6 ││
│  │ Netflix│  │ HBO Max│  │ Disney+│  │ Prime  ││
│  └────────┘  └────────┘  └────────┘  └────────┘│
│  [See All →]                                    │
│                                                 │
├─────────────────────────────────────────────────┤
│  🔥 TRENDING NOW                                │
│  ─────────────────────────────────────────────  │
│  [Horizontal scroll of trending content...]    │
│                                                 │
├─────────────────────────────────────────────────┤
│  📊 BECAUSE YOU WATCHED "Stranger Things"       │
│  ─────────────────────────────────────────────  │
│  [Horizontal scroll of similar content...]     │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Recommendation Card Design (Expanded)

```
┌─────────────────────────────────────┐
│ ┌─────────────────────────────────┐ │  ← Poster Image (16:9)
│ │                                 │ │
│ │      [Movie/Show Poster]        │ │
│ │                                 │ │
│ │  ⭐ 8.8  🔥 Trending  ✓ Safe    │ │  ← Overlay Badges
│ └─────────────────────────────────┘ │
│                                     │
│ Stranger Things                     │  ← Title (bold, 18px)
│ 📺 Netflix • Sci-Fi                 │  ← Platform & Genre (14px)
│                                     │
│ ⭐⭐⭐⭐⭐ 8.8/10                      │  ← Visual Rating
│ 🎯 Confidence: Very High            │  ← AI Confidence Score
│                                     │
│ ┌─────────────────────────────┐   │
│ │  💡 Why you'll like this:   │   │  ← Collapsible Reasoning
│ │  • Highly rated (8.8/10)    │   │
│ │  • Trending in Sci-Fi       │   │
│ │  • 12 friends watching      │   │
│ │  • Strong critical consensus│   │
│ └─────────────────────────────┘   │
│                                     │
│ [▶ Watch Now]  [+ Watchlist]  [×]  │  ← Action Buttons
└─────────────────────────────────────┘
```

### AI Agent Activity Indicator (During Search)

```
┌─────────────────────────────────────┐
│  🤖 Finding perfect recommendations... │
│                                        │
│  ✓ Understanding your preferences      │  ← PersonalizationAgent
│  ✓ Detecting your mood & context       │  ← MoodDetectionAgent
│  ⏳ Searching 5 platforms...           │  ← ResearchAgent (in progress)
│  ⏳ Aggregating reviews...             │  ← ReviewAggregationAgent
│  ⏳ Analyzing trends...                │  ← TrendAnalysisAgent
│  ○ Filtering content...               │  ← Pending
│  ○ Ranking results...                 │  ← Pending
│  ○ Generating recommendations...      │  ← Pending
│                                        │
│  [Progress: 50%] ████████░░░░░░░░     │
└─────────────────────────────────────────┘
```

---

## 🎨 Component Specifications

### 1. Search Input (Smart Query Box)

**Visual Design:**
- Large input field: min-height 56px
- Rounded corners: border-radius 28px (pill shape)
- Background: `#F1F1F1` (light) / `#3F3F3F` (dark)
- Placeholder: "What's your mood? What do you want to watch?"
- Icon: 🔍 on left, 🎤 (voice) on right
- Focus state: YouTube Red outline

**Cognitive Principles:**
- **Fitts's Law**: Large target, easy to click
- **Affordance**: Clearly indicates text input
- **Feedback**: Shows typing suggestions in real-time

**Example:**
```jsx
<SearchInput
  placeholder="I'm in the mood for..."
  suggestions={["something exciting", "highly rated sci-fi", "family comedy"]}
  onSearch={handleSearch}
  voice={true}
/>
```

### 2. Recommendation Card

**Visual Design:**
- Card width: 280px (desktop), 160px (mobile)
- Aspect ratio: 16:9 for poster
- Shadow: `0 4px 8px rgba(0,0,0,0.2)`
- Hover: Lift effect (transform: translateY(-4px))
- Border-radius: 8px

**Information Hierarchy:**
1. Poster image (primary visual)
2. Title (bold, 16-18px)
3. Platform + Genre (secondary, 12-14px)
4. Rating (stars + number)
5. Confidence score
6. Action buttons

**Cognitive Principles:**
- **Von Restorff**: High confidence items have subtle glow
- **Gestalt**: Related info grouped visually
- **Progressive Disclosure**: Details on hover/click

**States:**
- Default: Clean, minimal
- Hover: Show more details, lift effect
- Active: YouTube Red border
- Watched: 50% opacity + checkmark

### 3. Confidence Score Indicator

**Visual Design:**
```
Very High: 🎯 Green badge (#0F9D58)
High:      ⭐ Blue badge (#4285F4)
Medium:    ⚡ Yellow badge (#F4B400)
Low:       ℹ️  Gray badge (#606060)
```

**Cognitive Principles:**
- **Color psychology**: Green = trust, Yellow = caution
- **Icon + text**: Multi-sensory feedback
- **Consistent placement**: Always same location

### 4. Platform Badge

**Visual Design:**
- Small logo icons (24x24px)
- Background color matches platform brand
- Netflix: Red, Disney+: Blue, HBO: Purple, etc.
- Hover: Platform name tooltip

**Example:**
```jsx
<PlatformBadge platform="netflix" showName={false} />
// Renders: [N] with red background
```

### 5. "Why This Recommendation?" Expandable Panel

**Visual Design:**
- Collapsed: "💡 See why" link
- Expanded: Card background (#F9F9F9 light / #2A2A2A dark)
- Bullet points with icons
- Smooth accordion animation (200ms ease-out)

**Content:**
- Personalization factors ("You liked similar shows")
- Quality signals ("Highly rated at 8.8/10")
- Social proof ("12 friends watching")
- Trend data ("Trending in Sci-Fi")

**Cognitive Principles:**
- **Transparency**: Builds AI trust
- **Progressive disclosure**: Details on demand
- **Recognition**: Icons make scanning easy

### 6. Filter Controls

**Visual Design:**
- Chip/pill style buttons
- Unselected: `#E5E5E5` background
- Selected: YouTube Red background + white text
- Multi-select with checkmark icons
- Categories: Genre, Platform, Rating, Mood

**Cognitive Principles:**
- **Hick's Law**: Grouped by category
- **Recognition**: Icons + labels
- **Feedback**: Immediate visual response

**Example:**
```jsx
<FilterChips>
  <Chip label="Comedy" icon="🎭" selected={true} />
  <Chip label="Action" icon="⚡" selected={false} />
  <Chip label="Drama" icon="💭" selected={false} />
</FilterChips>
```

### 7. Loading States

**Visual Design:**
- Skeleton screens (not spinners)
- Shimmer animation (left to right)
- Agent activity indicator (list of steps)
- Progress bar: YouTube Red gradient

**Cognitive Principles:**
- **Feedback**: User knows system is working
- **Transparency**: Shows AI agents in action
- **Engagement**: Interesting to watch

**Agent Activity Display:**
```jsx
<AgentActivity agents={[
  { name: "PersonalizationAgent", status: "complete" },
  { name: "ResearchAgent", status: "in_progress", progress: 75 },
  { name: "AnalysisAgent", status: "pending" }
]} />
```

### 8. Action Buttons

**Primary Action (Watch Now):**
- YouTube Red background (#FF0000)
- White text, bold
- Height: 48px, padding: 16px 24px
- Border-radius: 24px (pill)
- Hover: Darken to #CC0000
- Active: Scale slightly (0.98)

**Secondary Actions:**
- Outline style (border only)
- Gray text/border
- Same size as primary
- Hover: Fill with light gray

**Micro-interactions:**
- Ripple effect on click
- Success state: Checkmark animation
- Error state: Shake animation

---

## 📐 Typography

### Font Family
```css
font-family: 'Roboto', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
```

### Type Scale
```css
/* Hero/H1 */
font-size: 36px;
font-weight: 700;
line-height: 1.2;

/* Section Headers/H2 */
font-size: 24px;
font-weight: 600;
line-height: 1.3;

/* Card Titles/H3 */
font-size: 18px;
font-weight: 600;
line-height: 1.4;

/* Body Text */
font-size: 16px;
font-weight: 400;
line-height: 1.5;

/* Secondary Text */
font-size: 14px;
font-weight: 400;
line-height: 1.5;
color: #606060 (light) / #AAAAAA (dark);

/* Small Text/Captions */
font-size: 12px;
font-weight: 400;
line-height: 1.4;
```

---

## 🎭 Interaction Patterns

### 1. Micro-interactions
- **Hover**: Lift cards 4px with shadow increase
- **Click**: Ripple effect from click point
- **Success**: Green checkmark with bounce
- **Error**: Red X with shake
- **Loading**: Shimmer animation

### 2. Transitions
```css
/* Default transition */
transition: all 0.2s cubic-bezier(0.4, 0.0, 0.2, 1);

/* Card hover */
transition: transform 0.3s ease-out, box-shadow 0.3s ease-out;

/* Modal/overlay */
transition: opacity 0.2s ease-in-out, transform 0.3s ease-out;
```

### 3. Animations
- **Page load**: Fade in + slide up (stagger children)
- **Card appearance**: Scale from 0.9 to 1.0
- **Modal**: Backdrop fade + content slide up
- **Agent activity**: Progress bar with easing

### 4. Gestures (Mobile)
- **Swipe left**: "Not interested"
- **Swipe right**: "Add to watchlist"
- **Pull down**: Refresh recommendations
- **Long press**: Show quick actions menu

---

## ♿ Accessibility Requirements

### WCAG 2.1 Level AA Compliance

**Color Contrast:**
- Normal text: 4.5:1 minimum
- Large text (18px+): 3:1 minimum
- Test all color combinations

**Keyboard Navigation:**
- All interactive elements focusable
- Visible focus indicators (YouTube Red outline)
- Logical tab order
- Keyboard shortcuts (? to show help)

**Screen Readers:**
- Semantic HTML (header, nav, main, section)
- ARIA labels for icons
- Alt text for all images
- Live regions for dynamic content

**Motor Accessibility:**
- Touch targets minimum 44x44px
- No hover-only interactions
- Generous click areas
- Timeout warnings (auto-dismiss modals)

**Visual Accessibility:**
- Scalable text (no fixed pixel sizes for body)
- High contrast mode support
- Reduced motion mode (prefers-reduced-motion)
- Clear visual hierarchy

---

## 📱 Responsive Design

### Breakpoints
```css
/* Mobile */
@media (max-width: 640px) {
  /* 1 column, vertical stack */
}

/* Tablet */
@media (min-width: 641px) and (max-width: 1024px) {
  /* 2-3 columns, hybrid layout */
}

/* Desktop */
@media (min-width: 1025px) {
  /* 4-5 columns, full features */
}

/* Large Desktop */
@media (min-width: 1440px) {
  /* 5-6 columns, max content width 1400px */
}
```

### Mobile-First Approach
- Design for mobile first
- Progressive enhancement for larger screens
- Touch-friendly interactions
- Bottom sheet modals (not center modals)

---

## 🎯 User Flows

### Primary Flow: Quick Discovery

1. **Land on homepage**
   - See personalized "Top Picks"
   - View trending content
   - Browse curated categories

2. **Search with natural language**
   - Type: "I want something exciting"
   - See agent activity indicator
   - Wait 3-6 seconds

3. **Review recommendations**
   - See 5 recommendations with confidence scores
   - Expand "Why this?" for reasoning
   - Compare options visually

4. **Take action**
   - Click "Watch Now" → Redirects to platform
   - Or "Add to Watchlist" → Saves for later
   - Or "Not interested" → Removes from view

5. **Provide feedback**
   - Rate recommendation (thumbs up/down)
   - System learns for next time

### Secondary Flow: Advanced Filtering

1. **Click "Advanced Filters"**
2. **Select preferences:**
   - Genre (multi-select)
   - Platform (available subscriptions)
   - Content rating (family-safe options)
   - Mood/tone
3. **Apply filters**
4. **See refined recommendations**

---

## 🧪 A/B Testing Opportunities

### Variants to Test

1. **Card Layout:**
   - Horizontal vs vertical cards
   - Image size: large vs small
   - Information density: minimal vs detailed

2. **Recommendation Count:**
   - 3 vs 5 vs 7 initial recommendations
   - Infinite scroll vs pagination

3. **Agent Transparency:**
   - Show agent activity vs hide (just loading spinner)
   - Detailed reasoning vs simple tags

4. **Primary CTA:**
   - "Watch Now" vs "Play" vs "Stream"
   - Button color: Red vs Blue vs Green

5. **Personalization Prompts:**
   - Ask upfront vs learn passively
   - Quiz-style vs natural language

---

## 📊 Analytics & Tracking

### Key Metrics to Track

**Engagement:**
- Time to first recommendation view
- Recommendation card click rate
- "Why this?" expansion rate
- Watchlist add rate

**Quality:**
- Recommendation acceptance rate
- Thumbs up/down ratio
- Search refinement frequency
- Filter usage patterns

**Performance:**
- Agent processing time
- Page load speed
- Time to interactive
- Error rates

**Business:**
- Platform affinity (which streaming services)
- Content type preferences
- Session duration
- Return visitor rate

---

## 🎨 Example Component Code Structure

### React Component Hierarchy

```jsx
<App>
  <Header>
    <Logo />
    <SearchBar />
    <UserMenu />
  </Header>

  <Main>
    <HeroSection>
      <SmartSearchInput />
      <QuickFilters />
    </HeroSection>

    {isSearching && (
      <AgentActivityIndicator agents={activeAgents} />
    )}

    <RecommendationSection title="Top Picks for You">
      <RecommendationGrid>
        {recommendations.map(rec => (
          <RecommendationCard
            key={rec.id}
            data={rec}
            onWatch={handleWatch}
            onWatchlist={handleWatchlist}
            onDismiss={handleDismiss}
          />
        ))}
      </RecommendationGrid>
    </RecommendationSection>

    <RecommendationSection title="Trending Now">
      <HorizontalScroll>
        {trending.map(item => (
          <TrendingCard key={item.id} data={item} />
        ))}
      </HorizontalScroll>
    </RecommendationSection>

    <PersonalizedSection title="Because You Watched 'Stranger Things'">
      <RecommendationGrid>
        {similar.map(item => (
          <RecommendationCard key={item.id} data={item} />
        ))}
      </RecommendationGrid>
    </PersonalizedSection>
  </Main>

  <Footer>
    <Links />
    <SocialSharing />
  </Footer>
</App>
```

---

## 🎯 Implementation Checklist

### Phase 1: Core UI (Week 1)
- [ ] Setup Next.js project with TypeScript
- [ ] Implement YouTube color scheme
- [ ] Create base components (Button, Card, Input)
- [ ] Build responsive grid system
- [ ] Add dark mode toggle

### Phase 2: Recommendation Display (Week 2)
- [ ] Recommendation card component
- [ ] Grid layout with filtering
- [ ] Agent activity indicator
- [ ] Loading states & skeletons
- [ ] Error handling UI

### Phase 3: Interactions (Week 3)
- [ ] Search with natural language
- [ ] Expand/collapse reasoning
- [ ] Watchlist functionality
- [ ] Quick actions (swipe, hover)
- [ ] Micro-interactions & animations

### Phase 4: Polish (Week 4)
- [ ] Accessibility audit (WCAG AA)
- [ ] Performance optimization
- [ ] Analytics integration
- [ ] A/B test framework
- [ ] User testing & refinement

---

## 📚 Design References

### YouTube-Style Elements
- **Homepage layout**: YouTube.com main feed
- **Card design**: YouTube video thumbnails
- **Search bar**: YouTube search with suggestions
- **Dark mode**: YouTube dark theme

### Cognitive Science Resources
- Nielsen Norman Group: UX research
- Laws of UX: Cognitive principles for designers
- Google Material Design: Interaction patterns
- Apple HIG: Mobile best practices

### Accessibility
- WCAG 2.1 Guidelines
- WebAIM Contrast Checker
- Lighthouse accessibility audits
- Screen reader testing (NVDA, JAWS, VoiceOver)

---

## 🎬 Success Criteria

### User Experience
✅ Time to first recommendation < 30 seconds
✅ Clear visual hierarchy (F-pattern eye tracking)
✅ Zero accessibility violations (WCAG AA)
✅ Smooth performance (60fps animations)

### Business Metrics
✅ Recommendation acceptance rate > 60%
✅ Session duration > 5 minutes
✅ Return visitor rate > 70% (24 hours)
✅ Watchlist conversion rate > 30%

### Technical Quality
✅ Lighthouse score > 90 (all categories)
✅ Page load time < 2 seconds
✅ Time to interactive < 3 seconds
✅ Zero console errors

---

## 🚀 Next Steps After UI Creation

1. **User Testing** (5-10 users)
   - Task completion rate
   - Time on task
   - Error recovery
   - Satisfaction surveys

2. **A/B Testing**
   - Test recommendation count
   - Test card layouts
   - Test CTA copy
   - Test agent transparency

3. **Iteration**
   - Analyze heatmaps
   - Review session recordings
   - Gather qualitative feedback
   - Implement improvements

4. **Scale**
   - Multi-language support
   - Platform-specific optimizations
   - Advanced personalization
   - Social features

---

## 💡 Pro Tips

1. **Start Simple** - Basic layout first, polish later
2. **Mobile First** - Most users will be on mobile
3. **Use Real Content** - Test with actual movie posters and titles
4. **Test Early** - Show users rough prototypes
5. **Measure Everything** - You can't improve what you don't measure
6. **Iterate Fast** - Weekly updates based on data
7. **Trust the Science** - Cognitive principles are proven
8. **Ask Users** - They'll tell you what's confusing
9. **Be Accessible** - Design for everyone from the start
10. **Have Fun** - Good design should be enjoyable to create!

---

**Created**: 2025-12-05
**Purpose**: Guide UI/UX implementation for multi-agent entertainment discovery system
**Framework**: Next.js / React with TypeScript
**Design System**: YouTube color scheme + Cognitive science principles
