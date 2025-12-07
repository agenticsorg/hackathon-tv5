import type { Config } from 'tailwindcss';

export default {
  content: [
    './src/pages/**/*.{js,ts,jsx,tsx,mdx}',
    './src/components/**/*.{js,ts,jsx,tsx,mdx}',
    './src/app/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        // Core Colors - Cool Midnight
        'bg-deep': '#0D1117',
        'bg-primary': '#1A2332',
        'bg-elevated': '#242D3D',
        'text-primary': '#F0F6FC',
        'text-secondary': '#8B949E',
        'accent-cyan': '#4ECDC4',
        'accent-cyan-dim': '#2A9D8F',
        'border-subtle': '#30363D',

        // Genre Colors
        'genre-romance': '#FF6B6B',
        'genre-thriller': '#4ECDC4',
        'genre-comedy': '#FFE66D',
        'genre-scifi': '#A855F7',
        'genre-drama': '#F97316',
        'genre-action': '#EF4444',
        'genre-horror': '#6B7280',
        'genre-documentary': '#10B981',

        // Legacy support
        background: 'var(--background)',
        foreground: 'var(--foreground)',
      },
      fontFamily: {
        sans: ['var(--font-dm-sans)', '-apple-system', 'BlinkMacSystemFont', 'sans-serif'],
        mono: ['var(--font-jetbrains)', 'monospace'],
      },
      fontSize: {
        'hero': ['3rem', { lineHeight: '1.1', letterSpacing: '-0.04em', fontWeight: '700' }],
        'h1': ['2.25rem', { lineHeight: '1.2', letterSpacing: '-0.04em', fontWeight: '700' }],
        'h2': ['1.5rem', { lineHeight: '1.3', letterSpacing: '-0.02em', fontWeight: '700' }],
      },
      animation: {
        'screen-enter': 'screenEnter 500ms cubic-bezier(0.16, 1, 0.3, 1) forwards',
        'screen-exit': 'screenExit 400ms ease-in forwards',
        'card-fade-in': 'cardFadeIn 600ms cubic-bezier(0.16, 1, 0.3, 1) forwards',
        'card-reveal': 'cardReveal 650ms cubic-bezier(0.16, 1, 0.3, 1) forwards',
        'section-fade-up': 'sectionFadeUp 500ms cubic-bezier(0.16, 1, 0.3, 1) forwards',
        'header-slide': 'headerSlide 400ms cubic-bezier(0.16, 1, 0.3, 1) forwards',
        'badge-pop': 'badgePop 300ms cubic-bezier(0.34, 1.56, 0.64, 1)',
        'button-reveal': 'buttonReveal 400ms cubic-bezier(0.16, 1, 0.3, 1) forwards',
      },
      keyframes: {
        screenEnter: {
          '0%': { opacity: '0', transform: 'translateY(40px) scale(0.98)' },
          '100%': { opacity: '1', transform: 'translateY(0) scale(1)' },
        },
        screenExit: {
          '0%': { opacity: '1', transform: 'translateY(0) scale(1)' },
          '100%': { opacity: '0', transform: 'translateY(-30px) scale(0.97)' },
        },
        cardFadeIn: {
          '0%': { opacity: '0', transform: 'translateY(20px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        cardReveal: {
          '0%': { opacity: '0', transform: 'translateY(40px) scale(0.92)' },
          '60%': { transform: 'translateY(-8px) scale(1.02)' },
          '100%': { opacity: '1', transform: 'translateY(0) scale(1)' },
        },
        sectionFadeUp: {
          '0%': { opacity: '0', transform: 'translateY(30px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        headerSlide: {
          '0%': { opacity: '0', transform: 'translateX(-15px)' },
          '100%': { opacity: '1', transform: 'translateX(0)' },
        },
        badgePop: {
          '0%': { transform: 'scale(0)' },
          '50%': { transform: 'scale(1.2)' },
          '100%': { transform: 'scale(1)' },
        },
        buttonReveal: {
          '0%': { opacity: '0', transform: 'translateY(10px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
      },
      transitionTimingFunction: {
        'bounce-out': 'cubic-bezier(0.34, 1.56, 0.64, 1)',
        'smooth-out': 'cubic-bezier(0.16, 1, 0.3, 1)',
      },
    },
  },
  plugins: [],
} satisfies Config;
