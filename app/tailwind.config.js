const { heroui } = require("@heroui/theme");

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.{js,ts,jsx,tsx,mdx}",
    "./node_modules/@heroui/theme/dist/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {},
  },
  darkMode: "class",
  plugins: [heroui({
    themes: {
      dark: {
        colors: {
          background: "#0a0a1a",
          foreground: "#ECEDEE",
          primary: {
            50: "#1a1a2e",
            100: "#16213e",
            200: "#1f4068",
            300: "#3b5998",
            400: "#5b7fb8",
            500: "#6c63ff",
            600: "#8b80ff",
            700: "#a99fff",
            800: "#c8bfff",
            900: "#e6e0ff",
            DEFAULT: "#6c63ff",
            foreground: "#ffffff",
          },
          focus: "#6c63ff",
        },
      },
      light: {
        colors: {
          background: "#f5f5f5",
          foreground: "#11181C",
          primary: {
            50: "#e6e0ff",
            100: "#c8bfff",
            200: "#a99fff",
            300: "#8b80ff",
            400: "#6c63ff",
            500: "#5b4fff",
            600: "#4a3bff",
            700: "#3927ff",
            800: "#2813ff",
            900: "#1700ff",
            DEFAULT: "#6c63ff",
            foreground: "#ffffff",
          },
          focus: "#6c63ff",
        },
      },
    },
  })],
}
