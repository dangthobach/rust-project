import type { Config } from 'tailwindcss';

export default {
  content: ['./src/**/*.{js,ts,jsx,tsx,mdx}'],
  theme: {
    extend: {
      colors: {
        // Neo-Brutalist Color Palette
        primary: {
          DEFAULT: '#00FF00', // Neon Green
          dark: '#00CC00',
        },
        secondary: {
          DEFAULT: '#0080FF', // Electric Blue
          dark: '#0066CC',
        },
        accent: {
          yellow: '#FFFF00',
          pink: '#FF10F0',
          orange: '#FF6B00',
        },
        neutral: {
          beige: '#E8E3D6',
          concrete: '#C0B5A4',
          gray: '#8B8680',
        },
        background: '#E8E3D6',
        surface: '#C0B5A4',
        border: '#000000',
      },
      fontFamily: {
        heading: ['Space Grotesk', 'DM Sans', 'system-ui', 'sans-serif'],
        body: ['Inter', 'Work Sans', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'Fira Code', 'monospace'],
      },
      fontSize: {
        'display-1': ['4rem', { lineHeight: '1', fontWeight: '900' }],
        'display-2': ['3rem', { lineHeight: '1.1', fontWeight: '900' }],
        'heading-1': ['2.5rem', { lineHeight: '1.2', fontWeight: '800' }],
        'heading-2': ['2rem', { lineHeight: '1.2', fontWeight: '800' }],
        'heading-3': ['1.5rem', { lineHeight: '1.3', fontWeight: '700' }],
        body: ['1rem', { lineHeight: '1.6', fontWeight: '400' }],
      },
      boxShadow: {
        brutal: '8px 8px 0px 0px #000000',
        'brutal-sm': '4px 4px 0px 0px #000000',
        'brutal-lg': '12px 12px 0px 0px #000000',
        'brutal-xl': '16px 16px 0px 0px #000000',
        'brutal-primary': '8px 8px 0px 0px #00FF00',
        'brutal-secondary': '8px 8px 0px 0px #0080FF',
      },
      borderWidth: {
        '3': '3px',
        '5': '5px',
      },
      spacing: {
        '18': '4.5rem',
        '88': '22rem',
      },
    },
  },
  plugins: [],
} satisfies Config;
