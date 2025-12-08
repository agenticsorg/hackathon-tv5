# Design System: Instant Media Discovery

**Version:** 1.0.0
**Last Updated:** 2025-12-05

---

## Design Direction Decision

Based on the platform's core vision of **instant, verified media discovery**, we recommend:

### Selected Aesthetic: **Option A - Dark Cinematic**

**Rationale:**
- Aligns with movie/TV watching context (dark environments)
- Creates premium, entertainment-focused feel
- Reduces eye strain for evening use (primary usage time)
- Photography-first approach highlights content over UI

**Color Palette:**

```css
/* Primary Colors */
--background-primary: #0F1419;      /* Deep slate */
--background-secondary: #1A1F2E;    /* Slightly lighter */
--background-elevated: #252B3B;     /* Cards, modals */

/* Accent Colors */
--accent-gold: #D4AF37;             /* Primary actions, highlights */
--accent-gold-hover: #F5C84C;       /* Hover state */
--accent-gold-muted: #8B7942;       /* Disabled state */

/* Text Colors */
--text-primary: #FFFFFF;            /* Headings, high emphasis */
--text-secondary: #B8BCC8;          /* Body text, medium emphasis */
--text-tertiary: #6E7485;           /* Captions, low emphasis */

/* Semantic Colors */
--success: #4CAF50;                 /* Available, confirmed */
--warning: #FF9800;                 /* Limited availability */
--error: #F44336;                   /* Unavailable, errors */
--info: #2196F3;                    /* Information, links */

/* Platform Colors (for badges) */
--netflix-red: #E50914;
--prime-blue: #00A8E1;
--hulu-green: #1CE783;
--disney-blue: #113CCF;
```

---

## Typography

### Font Stack

**Primary Font: DM Sans**
- Use: UI elements, buttons, labels
- Weights: 400 (regular), 500 (medium), 700 (bold)
- Source: Google Fonts

**Secondary Font: Merriweather**
- Use: Movie/show titles, headings
- Weights: 400 (regular), 700 (bold)
- Source: Google Fonts
- Rationale: Serif adds cinematic elegance

**Monospace: Roboto Mono**
- Use: Metadata (runtime, year), technical info
- Weight: 400 (regular)
- Source: Google Fonts

### Type Scale

```css
/* Headings */
--text-5xl: 3rem;      /* 48px - Hero headlines */
--text-4xl: 2.25rem;   /* 36px - Page titles */
--text-3xl: 1.875rem;  /* 30px - Section headers */
--text-2xl: 1.5rem;    /* 24px - Card titles */
--text-xl: 1.25rem;    /* 20px - Subheadings */

/* Body */
--text-lg: 1.125rem;   /* 18px - Large body */
--text-base: 1rem;     /* 16px - Base body */
--text-sm: 0.875rem;   /* 14px - Small text */
--text-xs: 0.75rem;    /* 12px - Captions */

/* Line Heights */
--leading-tight: 1.25;
--leading-normal: 1.5;
--leading-relaxed: 1.75;

/* Letter Spacing */
--tracking-tight: -0.025em;
--tracking-normal: 0;
--tracking-wide: 0.025em;
```

### Typography Usage

```css
/* Hero Heading */
.hero-title {
  font-family: 'Merriweather', serif;
  font-size: var(--text-5xl);
  font-weight: 700;
  line-height: var(--leading-tight);
  letter-spacing: var(--tracking-tight);
  color: var(--text-primary);
}

/* Movie Title */
.movie-title {
  font-family: 'Merriweather', serif;
  font-size: var(--text-2xl);
  font-weight: 700;
  line-height: var(--leading-tight);
  color: var(--text-primary);
}

/* Body Text */
.body-text {
  font-family: 'DM Sans', sans-serif;
  font-size: var(--text-base);
  font-weight: 400;
  line-height: var(--leading-normal);
  color: var(--text-secondary);
}

/* Metadata */
.metadata {
  font-family: 'Roboto Mono', monospace;
  font-size: var(--text-sm);
  font-weight: 400;
  letter-spacing: var(--tracking-wide);
  color: var(--text-tertiary);
}

/* Button Text */
.button-text {
  font-family: 'DM Sans', sans-serif;
  font-size: var(--text-base);
  font-weight: 500;
  letter-spacing: var(--tracking-wide);
  text-transform: uppercase;
}
```

---

## Spacing System

### Base Unit: 8px

```css
/* Spacing Scale */
--space-1: 0.25rem;   /* 4px */
--space-2: 0.5rem;    /* 8px */
--space-3: 0.75rem;   /* 12px */
--space-4: 1rem;      /* 16px */
--space-5: 1.25rem;   /* 20px */
--space-6: 1.5rem;    /* 24px */
--space-8: 2rem;      /* 32px */
--space-10: 2.5rem;   /* 40px */
--space-12: 3rem;     /* 48px */
--space-16: 4rem;     /* 64px */
--space-20: 5rem;     /* 80px */
--space-24: 6rem;     /* 96px */

/* Layout Spacing */
--section-padding: var(--space-16);
--card-padding: var(--space-4);
--button-padding-y: var(--space-3);
--button-padding-x: var(--space-6);
```

---

## Component Library

### Buttons

```css
/* Primary Button */
.btn-primary {
  background: var(--accent-gold);
  color: var(--background-primary);
  padding: var(--button-padding-y) var(--button-padding-x);
  border-radius: 8px;
  font: var(--button-text);
  border: none;
  cursor: pointer;
  transition: all 200ms ease;
}

.btn-primary:hover {
  background: var(--accent-gold-hover);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(212, 175, 55, 0.3);
}

.btn-primary:active {
  transform: translateY(0);
}

.btn-primary:disabled {
  background: var(--accent-gold-muted);
  cursor: not-allowed;
  opacity: 0.5;
}

/* Secondary Button */
.btn-secondary {
  background: transparent;
  color: var(--text-primary);
  border: 2px solid var(--text-secondary);
  /* ... same size/spacing as primary */
}

.btn-secondary:hover {
  border-color: var(--accent-gold);
  color: var(--accent-gold);
}

/* Ghost Button */
.btn-ghost {
  background: transparent;
  color: var(--text-secondary);
  border: none;
  padding: var(--space-2) var(--space-4);
}

.btn-ghost:hover {
  color: var(--text-primary);
  background: rgba(255, 255, 255, 0.05);
}
```

### Cards

```css
/* Movie/Show Card */
.media-card {
  background: var(--background-elevated);
  border-radius: 12px;
  overflow: hidden;
  transition: transform 200ms ease, box-shadow 200ms ease;
}

.media-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
}

.media-card__poster {
  width: 100%;
  aspect-ratio: 2/3;
  object-fit: cover;
}

.media-card__content {
  padding: var(--space-4);
}

.media-card__title {
  font-family: 'Merriweather', serif;
  font-size: var(--text-xl);
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: var(--space-2);
}

.media-card__meta {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  font-size: var(--text-sm);
  color: var(--text-tertiary);
}
```

### Input Fields

```css
/* Text Input */
.input-text {
  background: var(--background-secondary);
  border: 2px solid transparent;
  border-radius: 8px;
  padding: var(--space-3) var(--space-4);
  font-family: 'DM Sans', sans-serif;
  font-size: var(--text-base);
  color: var(--text-primary);
  width: 100%;
  transition: border-color 200ms ease;
}

.input-text::placeholder {
  color: var(--text-tertiary);
}

.input-text:focus {
  outline: none;
  border-color: var(--accent-gold);
  box-shadow: 0 0 0 3px rgba(212, 175, 55, 0.2);
}

/* Search Input (special styling) */
.input-search {
  font-size: var(--text-lg);
  padding: var(--space-4) var(--space-6);
  padding-left: var(--space-12); /* Space for search icon */
  border-radius: 12px;
}
```

### Filter Chips

```css
/* Filter Chip */
.chip {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-4);
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 24px;
  font-size: var(--text-sm);
  color: var(--text-primary);
  cursor: pointer;
  transition: all 150ms ease;
}

.chip:hover {
  background: rgba(255, 255, 255, 0.15);
  border-color: var(--accent-gold);
}

.chip--active {
  background: var(--accent-gold);
  color: var(--background-primary);
  border-color: var(--accent-gold);
}

.chip__remove {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  margin-left: var(--space-1);
  cursor: pointer;
}
```

---

## Iconography

**Icon Library:** Phosphor Icons
- Style: Regular weight for most icons
- Size: 24px standard, 20px small, 32px large
- Color: Inherit from parent or use --text-secondary

**Common Icons:**
- 🔍 MagnifyingGlass - Search
- 🎤 Microphone - Voice input
- ⭐ Star - Ratings
- 🍅 (custom) - Rotten Tomatoes
- ▶ Play - Watch action
- ➕ Plus - Add to watchlist
- ✕ X - Close/remove
- ⚙ Gear - Settings
- 👤 User - Profile

---

## Elevation & Shadows

```css
/* Elevation system (0-5) */
--shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.3);
--shadow-md: 0 4px 8px rgba(0, 0, 0, 0.4);
--shadow-lg: 0 8px 16px rgba(0, 0, 0, 0.5);
--shadow-xl: 0 12px 24px rgba(0, 0, 0, 0.6);
--shadow-2xl: 0 24px 48px rgba(0, 0, 0, 0.7);

/* Usage */
.card { box-shadow: var(--shadow-md); }
.modal { box-shadow: var(--shadow-2xl); }
.dropdown { box-shadow: var(--shadow-lg); }
```

---

## Motion & Animation

### Timing Functions

```css
--ease-in: cubic-bezier(0.4, 0, 1, 1);
--ease-out: cubic-bezier(0, 0, 0.2, 1);
--ease-in-out: cubic-bezier(0.4, 0, 0.2, 1);
--spring: cubic-bezier(0.34, 1.56, 0.64, 1);
```

### Duration Scale

```css
--duration-fast: 150ms;     /* Quick feedback */
--duration-normal: 200ms;   /* Standard transitions */
--duration-slow: 300ms;     /* Page transitions */
--duration-slower: 500ms;   /* Complex animations */
```

### Common Animations

```css
/* Fade In */
@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

/* Slide Up */
@keyframes slideUp {
  from {
    transform: translateY(20px);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}

/* Scale In */
@keyframes scaleIn {
  from {
    transform: scale(0.95);
    opacity: 0;
  }
  to {
    transform: scale(1);
    opacity: 1;
  }
}

/* Pulse (for voice listening) */
@keyframes pulse {
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.7;
    transform: scale(1.1);
  }
}
```

### Usage Examples

```css
/* Page transition */
.page-enter {
  animation: slideUp var(--duration-slow) var(--ease-out);
}

/* Card hover */
.card {
  transition: transform var(--duration-normal) var(--ease-out);
}

.card:hover {
  transform: translateY(-4px);
}

/* Voice listening indicator */
.voice-active {
  animation: pulse 1.5s var(--ease-in-out) infinite;
}
```

---

## Responsive Breakpoints

```css
/* Mobile First Approach */
--breakpoint-sm: 640px;    /* Large phones */
--breakpoint-md: 768px;    /* Tablets */
--breakpoint-lg: 1024px;   /* Laptops */
--breakpoint-xl: 1280px;   /* Desktops */
--breakpoint-2xl: 1536px;  /* Large desktops */

/* Usage */
@media (min-width: 768px) {
  .container {
    padding: var(--space-8);
  }
}
```

---

## Grid System

```css
/* Results Grid */
.results-grid {
  display: grid;
  gap: var(--space-6);
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
}

/* Responsive grid adjustments */
@media (min-width: 640px) {
  .results-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (min-width: 768px) {
  .results-grid {
    grid-template-columns: repeat(3, 1fr);
  }
}

@media (min-width: 1024px) {
  .results-grid {
    grid-template-columns: repeat(4, 1fr);
  }
}

@media (min-width: 1280px) {
  .results-grid {
    grid-template-columns: repeat(6, 1fr);
  }
}
```

---

## Accessibility

### Focus States

```css
/* Global focus style */
*:focus-visible {
  outline: 3px solid var(--accent-gold);
  outline-offset: 2px;
}

/* Remove outline for mouse users */
*:focus:not(:focus-visible) {
  outline: none;
}
```

### Reduced Motion

```css
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

### High Contrast Mode

```css
@media (prefers-contrast: high) {
  :root {
    --text-primary: #FFFFFF;
    --text-secondary: #FFFFFF;
    --background-primary: #000000;
    --accent-gold: #FFD700;
  }
}
```

---

## Usage Guidelines

### Do's ✅

- Use the dark cinematic theme for primary interfaces
- Ensure text contrast meets WCAG AA (4.5:1 minimum)
- Use gold accent sparingly for primary actions only
- Leverage photography and movie stills over flat graphics
- Maintain consistent spacing using the 8px grid
- Use Merriweather for titles to add elegance
- Provide hover states for all interactive elements

### Don'ts ❌

- Don't use gold for large background areas (overwhelming)
- Don't mix the dark theme with light theme sections
- Don't use more than 3 font families
- Don't create new spacing values outside the scale
- Don't remove focus indicators without accessible replacements
- Don't use color alone to convey information
- Don't create animations longer than 500ms

---

## Implementation with Tailwind CSS

```javascript
// tailwind.config.js
module.exports = {
  theme: {
    extend: {
      colors: {
        background: {
          primary: '#0F1419',
          secondary: '#1A1F2E',
          elevated: '#252B3B',
        },
        accent: {
          gold: '#D4AF37',
          'gold-hover': '#F5C84C',
          'gold-muted': '#8B7942',
        },
        text: {
          primary: '#FFFFFF',
          secondary: '#B8BCC8',
          tertiary: '#6E7485',
        },
      },
      fontFamily: {
        sans: ['DM Sans', 'sans-serif'],
        serif: ['Merriweather', 'serif'],
        mono: ['Roboto Mono', 'monospace'],
      },
      spacing: {
        // Uses default 8px scale
      },
      boxShadow: {
        'sm': '0 1px 2px rgba(0, 0, 0, 0.3)',
        'DEFAULT': '0 4px 8px rgba(0, 0, 0, 0.4)',
        'md': '0 4px 8px rgba(0, 0, 0, 0.4)',
        'lg': '0 8px 16px rgba(0, 0, 0, 0.5)',
        'xl': '0 12px 24px rgba(0, 0, 0, 0.6)',
        '2xl': '0 24px 48px rgba(0, 0, 0, 0.7)',
      },
    },
  },
};
```

---

**Document Version:** 1.0.0
**Maintained By:** Design Team
**Last Review:** 2025-12-05
