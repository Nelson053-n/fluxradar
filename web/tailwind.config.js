/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        // Mapped to CSS variables from src/styles/tokens.css (mockup :root).
        'bg-base': 'var(--bg-base)',
        'bg-deep': 'var(--bg-deep)',
        'bg-card': 'var(--bg-card)',
        'bg-card-hover': 'var(--bg-card-hover)',
        'bg-panel': 'var(--bg-panel)',
        'bg-subtle': 'var(--bg-subtle)',
        'bg-subtle-hover': 'var(--bg-subtle-hover)',
        'bg-overlay': 'var(--bg-overlay)',
        'bg-overlay-soft': 'var(--bg-overlay-soft)',
        'bg-row-hover': 'var(--bg-row-hover)',
        border: 'var(--border)',
        'border-strong': 'var(--border-strong)',
        'flux-primary': 'var(--flux-primary)',
        'flux-cyan': 'var(--flux-cyan)',
        'flux-glow': 'var(--flux-glow)',
        'flux-purple': 'var(--flux-purple)',
        'text-primary': 'var(--text-primary)',
        'text-secondary': 'var(--text-secondary)',
        'text-dim': 'var(--text-dim)',
        success: 'var(--success)',
        warning: 'var(--warning)',
        danger: 'var(--danger)',
      },
      fontFamily: {
        sans: ['Manrope', 'sans-serif'],
        mono: ['"JetBrains Mono"', 'monospace'],
      },
      borderRadius: {
        '2xl': '18px',
      },
      boxShadow: {
        glow: 'var(--shadow-glow)',
      },
      keyframes: {
        fadeUp: {
          from: { opacity: '0', transform: 'translateY(12px)' },
          to: { opacity: '1', transform: 'translateY(0)' },
        },
        spin: {
          to: { transform: 'rotate(360deg)' },
        },
        slideInRight: {
          from: { transform: 'translateX(100%)' },
          to: { transform: 'translateX(0)' },
        },
        fadeIn: {
          from: { opacity: '0' },
          to: { opacity: '1' },
        },
      },
      animation: {
        fadeUp: 'fadeUp 0.6s ease-out backwards',
        spin: 'spin 0.8s linear infinite',
        slideInRight: 'slideInRight 0.28s cubic-bezier(0.16, 1, 0.3, 1)',
        fadeIn: 'fadeIn 0.2s ease-out',
      },
    },
  },
  plugins: [],
}
