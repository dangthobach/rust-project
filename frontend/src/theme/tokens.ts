/**
 * Neo-Brutalist Design Tokens
 * Centralized design system constants
 */

export const colors = {
  // Primary Colors
  primary: {
    DEFAULT: '#00FF00', // Neon Green
    dark: '#00CC00',
    light: '#66FF66',
  },
  secondary: {
    DEFAULT: '#0080FF', // Electric Blue
    dark: '#0066CC',
    light: '#4DA6FF',
  },
  accent: {
    yellow: '#FFFF00',
    pink: '#FF10F0',
    orange: '#FF6B00',
    purple: '#9D00FF',
  },
  neutral: {
    beige: '#E8E3D6',
    concrete: '#C0B5A4',
    gray: '#8B8680',
    darkGray: '#5A5550',
  },
  // Semantic Colors
  success: '#00FF00',
  warning: '#FFFF00',
  error: '#FF0000',
  info: '#0080FF',
  // Base
  background: '#E8E3D6',
  surface: '#FFFFFF',
  border: '#000000',
  text: '#000000',
} as const;

export const typography = {
  fontFamily: {
    heading: ['Space Grotesk', 'DM Sans', 'system-ui', 'sans-serif'],
    body: ['Inter', 'Work Sans', 'system-ui', 'sans-serif'],
    mono: ['JetBrains Mono', 'Fira Code', 'Courier New', 'monospace'],
  },
  fontSize: {
    xs: '0.75rem',    // 12px
    sm: '0.875rem',   // 14px
    base: '1rem',     // 16px
    lg: '1.125rem',   // 18px
    xl: '1.25rem',    // 20px
    '2xl': '1.5rem',  // 24px
    '3xl': '1.875rem', // 30px
    '4xl': '2.25rem', // 36px
    '5xl': '3rem',    // 48px
    '6xl': '3.75rem', // 60px
  },
  fontWeight: {
    normal: '400',
    medium: '500',
    semibold: '600',
    bold: '700',
    extrabold: '800',
    black: '900',
  },
  lineHeight: {
    tight: '1.1',
    snug: '1.3',
    normal: '1.5',
    relaxed: '1.6',
    loose: '1.8',
  },
} as const;

export const spacing = {
  0: '0',
  px: '1px',
  0.5: '0.125rem',  // 2px
  1: '0.25rem',     // 4px
  2: '0.5rem',      // 8px
  3: '0.75rem',     // 12px
  4: '1rem',        // 16px
  5: '1.25rem',     // 20px
  6: '1.5rem',      // 24px
  8: '2rem',        // 32px
  10: '2.5rem',     // 40px
  12: '3rem',       // 48px
  16: '4rem',       // 64px
  20: '5rem',       // 80px
  24: '6rem',       // 96px
  32: '8rem',       // 128px
} as const;

export const borders = {
  width: {
    none: '0',
    thin: '2px',
    medium: '3px',
    thick: '5px',
  },
  radius: {
    none: '0',
    sm: '2px',
    md: '4px',
    lg: '8px',
  },
} as const;

export const shadows = {
  brutal: {
    sm: '4px 4px 0px 0px #000000',
    md: '8px 8px 0px 0px #000000',
    lg: '12px 12px 0px 0px #000000',
    xl: '16px 16px 0px 0px #000000',
  },
  colored: {
    primary: '8px 8px 0px 0px #00FF00',
    secondary: '8px 8px 0px 0px #0080FF',
    yellow: '8px 8px 0px 0px #FFFF00',
    pink: '8px 8px 0px 0px #FF10F0',
  },
} as const;

export const breakpoints = {
  sm: '640px',
  md: '768px',
  lg: '1024px',
  xl: '1280px',
  '2xl': '1536px',
} as const;

export const transitions = {
  duration: {
    fast: '150ms',
    normal: '300ms',
    slow: '500ms',
  },
  timing: {
    ease: 'ease',
    easeIn: 'ease-in',
    easeOut: 'ease-out',
    easeInOut: 'ease-in-out',
    linear: 'linear',
  },
} as const;

export const zIndex = {
  base: 0,
  dropdown: 10,
  sticky: 20,
  fixed: 30,
  modalBackdrop: 40,
  modal: 50,
  popover: 60,
  tooltip: 70,
  notification: 80,
} as const;

// Status colors for task/client statuses
export const statusColors = {
  task: {
    todo: colors.neutral.concrete,
    in_progress: colors.secondary.DEFAULT,
    done: colors.success,
    cancelled: colors.neutral.gray,
  },
  client: {
    active: colors.success,
    inactive: colors.neutral.gray,
    prospect: colors.accent.yellow,
    customer: colors.primary.DEFAULT,
  },
  priority: {
    low: colors.neutral.concrete,
    medium: colors.accent.yellow,
    high: colors.accent.orange,
    urgent: colors.error,
  },
} as const;

// Export all tokens as a single object
export const tokens = {
  colors,
  typography,
  spacing,
  borders,
  shadows,
  breakpoints,
  transitions,
  zIndex,
  statusColors,
} as const;

export type Tokens = typeof tokens;
