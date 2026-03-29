import type { Config } from 'tailwindcss';

export default {
  content: ['./src/**/*.{js,ts,jsx,tsx,mdx}'],
  theme: {
    extend: {
      colors: {
        // Industrial Ledger — Neo-Brutalism
        primary: {
          DEFAULT: '#A3FF00', // Neon lime (CTA)
          dark: '#7ACC00',
        },
        secondary: {
          DEFAULT: '#0080FF',
          dark: '#0066CC',
        },
        ledger: {
          cream: '#F5F5F0',
          lime: '#A3FF00',
          orange: '#FF5C00',
          sky: '#A0C4FF',
          pale: '#FDFFB6',
          mint: '#C1FFD7',
        },
        accent: {
          yellow: '#FDFFB6',
          pink: '#FF10F0',
          orange: '#FF5C00',
        },
        neutral: {
          beige: '#E8E3D6',
          concrete: '#C0B5A4',
          gray: '#8B8680',
          lightGray: '#E8E8E4',
          darkGray: '#3D3D3D',
          black: '#000000',
        },
        background: '#F5F5F0',
        surface: '#FFFFFF',
        border: '#000000',
      },
      fontFamily: {
        heading: ['"Space Grotesk"', 'system-ui', 'sans-serif'],
        body: ['Lexend', 'system-ui', 'sans-serif'],
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
        brutal: '6px 6px 0px 0px #000000',
        'brutal-sm': '4px 4px 0px 0px #000000',
        'brutal-lg': '8px 8px 0px 0px #000000',
        'brutal-xl': '12px 12px 0px 0px #000000',
        'brutal-primary': '6px 6px 0px 0px #A3FF00',
        'brutal-secondary': '6px 6px 0px 0px #0080FF',
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
