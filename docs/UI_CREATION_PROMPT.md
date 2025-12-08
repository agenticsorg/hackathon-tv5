# Create Entertainment Discovery UI - Complete Prompt

Create a production-ready web interface for an AI-powered entertainment discovery system using Next.js 15, React 19, TypeScript, and Tailwind CSS.

## Project Context

This is a **privacy-first, 8-agent multi-agent system** that helps users find what to watch across 5 streaming platforms (Netflix, Disney+, HBO Max, Prime Video, Apple TV+) in 6 seconds. The system uses specialized AI agents that collaborate in real-time with parallel execution for optimal performance.

## Design Requirements

### Color Scheme (YouTube Official Palette)
```css
/* Primary Colors */
--youtube-red: #FF0000;
--youtube-red-dark: #CC0000;
--youtube-red-light: #FF4444;

/* Dark Mode (Default) */
--bg-dark: #0F0F0F;
--bg-dark-card: #272727;
--bg-dark-border: #3F3F3F;

/* Text */
--text-dark-primary: #FFFFFF;
--text-dark-secondary: #AAAAAA;

/* Semantic */
--semantic-success: #0F9D58;
--semantic-warning: #F4B400;
--semantic-info: #4285F4;
--semantic-error: #DB4437;
```

### Typography
- Font Family: 'Roboto', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif
- Hero/H1: 36-48px, font-weight 700, line-height 1.2
- Section Titles/H2: 24-32px, font-weight 700
- Card Titles/H3: 18px, font-weight 600
- Body: 16px, font-weight 400, line-height 1.5
- Small/Caption: 14px, font-weight 400

### Cognitive Science Principles Applied
1. **Hick's Law**: Limit choices to 3-5 options (use 4 mood filters, not 10+)
2. **Miller's Law**: Show 5-7 recommendations at once (7±2 items in working memory)
3. **Fitts's Law**: All interactive elements minimum 44x44px touch targets
4. **Progressive Disclosure**: Hide complexity (reasoning panels collapsed by default)
5. **Recognition over Recall**: Use visual icons, badges, and indicators
6. **Von Restorff Effect**: Make important items stand out (confidence badges, trending tags)

### Accessibility
- WCAG 2.1 Level AA compliant
- Color contrast ratio 4.5:1 minimum for text
- Focus indicators on all interactive elements
- Semantic HTML structure
- ARIA labels where needed

## The 8-Agent System Architecture

Our multi-agent system consists of:

1. **PersonalizationAgent** (Priority 8) 👤 - Analyzes viewing history (ON-DEVICE for privacy)
2. **MoodDetectionAgent** (Priority 7) 🎭 - Detects current context/mood (ON-DEVICE)
3. **ResearchAgent** (Priority 7) 🔍 - Searches across 5 platforms
4. **ReviewAggregationAgent** (Priority 6) ⭐ - Collects reviews from 4 sources
5. **TrendAnalysisAgent** (Priority 6) 📈 - Analyzes social trends
6. **ContentFilterAgent** (Priority 9) 🛡️ - Applies safety filters
7. **AnalysisAgent** (Priority 8) 🧠 - Ranks recommendations (ON-DEVICE)
8. **RecommendationAgent** (Priority 9) 🎯 - Generates final picks

**Execution Flow (6 seconds total):**
- Phase 1 (Parallel): Personalization + Mood detection
- Phase 2: Research across platforms
- Phase 3 (Parallel): Reviews + Trends aggregation
- Phase 4: Safety filtering
- Phase 5: Intelligent ranking
- Phase 6: Final recommendations

## Required Components

### 1. Header Component
```typescript
// Sticky top navigation bar
// - Logo: "EntertainAI" with 8-agent branding
// - Navigation: Discover, Trending, My List
// - User profile button (right side)
// - Dark background (#0F0F0F) with border (#3F3F3F)
// - Height: 64px (16 spacing)
```

### 2. SearchSection Component
```typescript
// Hero search interface
// - Large heading: "Find What to Watch in 6 Seconds" (YouTube red on "6 Seconds")
// - Subtitle: "Our 8 specialized AI agents search across 5 platforms..."
// - Search input:
//   * Pill-shaped (border-radius: 28px)
//   * Height: 56px
//   * Placeholder: "What do you feel like watching? (e.g., 'action movies with strong female leads')"
//   * YouTube red focus border (#FF0000)
//   * Search button inside input (right side)
// - Quick mood filters (4 buttons):
//   * ⚡ Energetic (blue #4285F4)
//   * 😌 Relaxed (green #0F9D58)
//   * 🎢 Thrilling (yellow #F4B400)
//   * 🤔 Thoughtful (red #FF0000)
//   * Pill-shaped, 32px height
// - Platform indicators: "Searching 5 platforms • 8 AI agents active • Real-time analysis"
```

### 3. AgentActivity Component
```typescript
// Real-time agent visualization (shows during 6-second search)
// - Title: "Multi-Agent System at Work"
// - 8 agent cards in vertical list:
//   * Agent emoji + name + description
//   * Status indicator: Pending (⏳ gray), Active (⚡ red pulsing), Complete (✅ green)
//   * Progress bar when active (800ms animation)
// - Timeline simulation:
//   * 0-500ms: Personalization + Mood (parallel)
//   * 500-1500ms: Research
//   * 1500-3000ms: Reviews + Trends (parallel)
//   * 3000-4500ms: Content Filter
//   * 4500-5000ms: Analysis
//   * 5000-6000ms: Recommendations
```

### 4. RecommendationSection Component
```typescript
// Section wrapper for recommendations
// - Section title + subtitle
// - Two layouts:
//   * Grid: 4 columns on desktop, 2 on tablet, 1 on mobile
//   * Scroll: Horizontal overflow with snap points
// - Gap between cards: 24px
```

### 5. RecommendationCard Component
```typescript
// Individual content card (280px width)
// - Poster image: 2:3 aspect ratio (280x420px)
// - Confidence badge (top-right):
//   * Very High (green), High (blue), Medium (yellow), Low (gray)
//   * Pill-shaped, 8px padding
// - Platform badge (top-left): e.g., "Netflix", "HBO Max"
// - Tags (bottom-left on hover): 🔥 Trending, ⭐ Critics' Choice
// - Card content (below image):
//   * Title (18px bold, 2 lines max with ellipsis)
//   * Year + Genres (14px gray, single line)
//   * Review scores: IMDb 8.9, RT 96%, Metacritic 92 (star icons)
//   * Social proof: "12 of your friends are watching"
//   * "Why this?" button (YouTube red, full width)
// - Reasoning panel (expandable):
//   * Accordion that slides down when "Why this?" clicked
//   * Dark background (#0F0F0F), 12px padding
//   * Explanation text (14px, gray)
// - Hover effects:
//   * Image scales to 105%
//   * Card shadow elevates (shadow-card-hover)
//   * Smooth 200ms transition
```

## Mock Data Structure

```typescript
interface Recommendation {
  id: string;
  title: string;
  year: number;
  platform: 'Netflix' | 'Disney+' | 'HBO Max' | 'Prime Video' | 'Apple TV+';
  poster: string; // Image URL
  rating: number; // IMDb score
  confidence: 'Very High' | 'High' | 'Medium' | 'Low';
  genres: string[]; // e.g., ['Action', 'Drama', 'Sci-Fi']
  reasoning: string; // AI explanation
  reviews: {
    source: 'IMDb' | 'RT' | 'Metacritic' | 'Google';
    score: number; // 0-10 or 0-100 scale
  }[];
  tags?: string[]; // e.g., ['🔥 Trending', '⭐ Critics\' Choice']
  socialProof?: string; // e.g., "12 of your friends are watching"
}
```

### Sample Mock Data (6 recommendations)
```typescript
[
  {
    id: '1',
    title: 'The Last of Us',
    year: 2023,
    platform: 'HBO Max',
    poster: 'https://images.unsplash.com/photo-1536440136628-849c177e76a1?w=400&h=600&fit=crop',
    rating: 8.9,
    confidence: 'Very High',
    genres: ['Action', 'Drama', 'Sci-Fi'],
    reasoning: 'Based on your preference for post-apocalyptic narratives and character-driven stories. The show has received critical acclaim (96% on Rotten Tomatoes) and matches your evening viewing mood. Similar to The Walking Dead which you rated highly.',
    reviews: [
      { source: 'IMDb', score: 8.9 },
      { source: 'RT', score: 96 }
    ],
    tags: ['🔥 Trending', '⭐ Critics\' Choice'],
    socialProof: '12 of your friends are watching'
  },
  {
    id: '2',
    title: 'Wednesday',
    year: 2022,
    platform: 'Netflix',
    poster: 'https://images.unsplash.com/photo-1518676590629-3dcbd9c5a5c9?w=400&h=600&fit=crop',
    rating: 8.2,
    confidence: 'Very High',
    genres: ['Comedy', 'Fantasy', 'Mystery'],
    reasoning: 'Your viewing history shows strong interest in dark comedy and supernatural themes. This Tim Burton series combines both elements with a fresh take on the Addams Family. Currently trending with 341.2M hours viewed in first month.',
    reviews: [
      { source: 'IMDb', score: 8.2 },
      { source: 'RT', score: 73 }
    ],
    tags: ['🔥 Trending'],
    socialProof: '8 of your friends are watching'
  },
  {
    id: '3',
    title: 'The Bear',
    year: 2023,
    platform: 'Disney+',
    poster: 'https://images.unsplash.com/photo-1414235077428-338989a2e8c0?w=400&h=600&fit=crop',
    rating: 8.6,
    confidence: 'High',
    genres: ['Drama', 'Comedy'],
    reasoning: 'Matches your interest in character-driven dramas with fast-paced storytelling. The show\'s intense kitchen environment and personal growth themes align with your recent viewing of Succession. 23 Emmy nominations including Outstanding Drama Series.',
    reviews: [
      { source: 'IMDb', score: 8.6 },
      { source: 'Metacritic', score: 92 }
    ],
    tags: ['⭐ Critics\' Choice'],
    socialProof: '6 of your friends are watching'
  },
  {
    id: '4',
    title: 'Severance',
    year: 2022,
    platform: 'Apple TV+',
    poster: 'https://images.unsplash.com/photo-1497215728101-856f4ea42174?w=400&h=600&fit=crop',
    rating: 8.7,
    confidence: 'High',
    genres: ['Sci-Fi', 'Drama', 'Mystery'],
    reasoning: 'Your high ratings for Black Mirror and Westworld suggest you enjoy thought-provoking sci-fi. This psychological thriller explores similar themes of identity and reality. Currently holds 97% on Rotten Tomatoes with universal critical acclaim.',
    reviews: [
      { source: 'IMDb', score: 8.7 },
      { source: 'RT', score: 97 }
    ],
    tags: ['⭐ Critics\' Choice'],
    socialProof: 'Won 2 Emmy Awards'
  },
  {
    id: '5',
    title: 'Succession',
    year: 2023,
    platform: 'HBO Max',
    poster: 'https://images.unsplash.com/photo-1611162617474-5b21e879e113?w=400&h=600&fit=crop',
    rating: 8.9,
    confidence: 'Very High',
    genres: ['Drama'],
    reasoning: 'Perfect match for your preference for prestige dramas with complex family dynamics. The show\'s sharp writing and stellar performances align with your viewing history. Final season won 6 Emmy Awards including Outstanding Drama Series.',
    reviews: [
      { source: 'IMDb', score: 8.9 },
      { source: 'Metacritic', score: 93 }
    ],
    tags: ['🔥 Trending', '⭐ Critics\' Choice'],
    socialProof: 'Won 19 Emmy Awards total'
  },
  {
    id: '6',
    title: 'The Night Agent',
    year: 2023,
    platform: 'Netflix',
    poster: 'https://images.unsplash.com/photo-1574267432644-f85fd0a82475?w=400&h=600&fit=crop',
    rating: 7.5,
    confidence: 'Medium',
    genres: ['Action', 'Thriller'],
    reasoning: 'Your watch history includes several political thrillers like Jack Ryan. This fast-paced series offers similar high-stakes action with conspiracy themes. Most-watched show on Netflix in Q1 2023 with 812M hours viewed.',
    reviews: [
      { source: 'IMDb', score: 7.5 },
      { source: 'RT', score: 78 }
    ],
    tags: ['🔥 Trending'],
    socialProof: '15 of your friends are watching'
  }
]
```

## Privacy-First Architecture (Key Differentiator)

**ON-DEVICE (Private):**
- PersonalizationAgent - Watch history never leaves device
- MoodDetectionAgent - Context analysis local
- AnalysisAgent - Ranking happens on device

**SERVER-SIDE (Anonymized):**
- ResearchAgent - Uses differential privacy for queries
- ReviewAggregationAgent - Public data aggregation
- TrendAnalysisAgent - Social signals

**Privacy Features:**
- Auto-expire watch history after 60 days
- No cross-device tracking
- Contextual ads only (no behavioral targeting)
- E2EE for social features
- "Wipe persona" button in settings

## Technical Implementation

### Stack
- **Framework**: Next.js 15 (App Router)
- **UI Library**: React 19
- **Language**: TypeScript 5.6+
- **Styling**: Tailwind CSS 3.4
- **Package Manager**: npm

### File Structure
```
web-ui/
├── app/
│   ├── layout.tsx          # Root layout with dark mode
│   ├── page.tsx            # Main page with search + results
│   └── globals.css         # Tailwind + custom styles
├── components/
│   ├── Header.tsx
│   ├── SearchSection.tsx
│   ├── AgentActivity.tsx
│   ├── RecommendationSection.tsx
│   └── RecommendationCard.tsx
├── lib/
│   └── mockData.ts         # Sample recommendations
├── tailwind.config.ts      # YouTube color system
├── tsconfig.json
└── package.json
```

### Tailwind Config
```typescript
// tailwind.config.ts
import type { Config } from "tailwindcss";

const config: Config = {
  content: [
    "./pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./components/**/*.{js,ts,jsx,tsx,mdx}",
    "./app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        youtube: {
          red: '#FF0000',
          'red-dark': '#CC0000',
          'red-light': '#FF4444',
        },
        bg: {
          dark: '#0F0F0F',
          'dark-card': '#272727',
          'dark-border': '#3F3F3F',
        },
        text: {
          'dark-primary': '#FFFFFF',
          'dark-secondary': '#AAAAAA',
        },
        semantic: {
          success: '#0F9D58',
          warning: '#F4B400',
          info: '#4285F4',
          error: '#DB4437',
        }
      },
      fontFamily: {
        sans: ['Roboto', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'sans-serif'],
      },
      borderRadius: {
        'card': '8px',
        'pill': '28px',
      },
      boxShadow: {
        'card': '0 4px 8px rgba(0,0,0,0.2)',
        'card-hover': '0 8px 16px rgba(0,0,0,0.3)',
      },
    },
  },
  plugins: [],
  darkMode: 'class',
};
export default config;
```

### Custom CSS (globals.css)
```css
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  html {
    font-family: Roboto, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  }
}

/* Custom scrollbar for dark mode */
::-webkit-scrollbar {
  width: 12px;
}

::-webkit-scrollbar-track {
  background: #0F0F0F;
}

::-webkit-scrollbar-thumb {
  background: #3F3F3F;
  border-radius: 6px;
}

::-webkit-scrollbar-thumb:hover {
  background: #555;
}

/* Smooth transitions */
* {
  transition-property: transform, box-shadow, background-color;
  transition-duration: 0.2s;
  transition-timing-function: cubic-bezier(0.4, 0.0, 0.2, 1);
}

button, a {
  transition-property: all;
}
```

## User Flow

1. **Landing**: User sees hero search and trending recommendations
2. **Search**: User types query or clicks mood filter
3. **Agent Activity**: 6-second visualization showing 8 agents working
4. **Results**: Grid of personalized recommendations with confidence scores
5. **Exploration**: User clicks "Why this?" to understand AI reasoning
6. **Selection**: User clicks card to watch (would open streaming platform)

## Key Features to Implement

✅ Dark mode by default (YouTube aesthetic)
✅ Responsive design (mobile-first)
✅ Smooth animations and transitions
✅ Progressive disclosure (reasoning panels)
✅ Real-time agent status simulation
✅ Confidence scoring visualization
✅ Multi-source review aggregation display
✅ Social proof indicators
✅ Platform badges
✅ Trending/featured tags
✅ 44x44px minimum touch targets
✅ WCAG 2.1 AA color contrast

## Expected Output

Create a complete, production-ready Next.js application with:
- All 5 components fully implemented
- Mock data with 10+ recommendations
- Responsive design working on mobile/tablet/desktop
- Smooth animations and transitions
- Working search interaction (6-second agent simulation)
- Expandable reasoning panels
- TypeScript type safety throughout
- Zero accessibility violations

The final application should be runnable with:
```bash
npm install
npm run dev
```

And accessible at `http://localhost:3000` with a polished, professional interface ready for a hackathon demo.
