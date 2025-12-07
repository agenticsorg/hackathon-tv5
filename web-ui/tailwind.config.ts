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
          light: '#F1F1F1',
          'light-card': '#FFFFFF',
        },
        text: {
          'dark-primary': '#FFFFFF',
          'dark-secondary': '#AAAAAA',
          'light-primary': '#030303',
          'light-secondary': '#606060',
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
