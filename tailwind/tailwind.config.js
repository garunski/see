/** @type {import('tailwindcss').Config} */
export default {
  content: [
    // Main GUI app
    "../gui/src/**/*.rs",
    "../gui/src/**/*.html",
    
    // React Flow app
    "../react-flow-app/src/**/*.{ts,tsx,js,jsx}",
    "../react-flow-app/index.html",
    
    // TypeScript components
    "../typescript/**/*.{ts,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // Custom colors for the Speculative Execution Engine project
        primary: {
          50: '#eff6ff',
          100: '#dbeafe',
          200: '#bfdbfe',
          300: '#93c5fd',
          400: '#60a5fa',
          500: '#3b82f6',
          600: '#2563eb',
          700: '#1d4ed8',
          800: '#1e40af',
          900: '#1e3a8a',
        },
      },
    },
  },
  plugins: [],
  darkMode: 'class',
};
