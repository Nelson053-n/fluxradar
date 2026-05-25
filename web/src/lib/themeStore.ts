import { createContext, useContext } from 'react'

export type Theme = 'dark' | 'light'

export const THEME_STORAGE_KEY = 'fluxscope:theme'

/** Текущая тема из <html data-theme> (её выставляет анти-FOUC скрипт в index.html). */
export function readTheme(): Theme {
  return document.documentElement.getAttribute('data-theme') === 'light' ? 'light' : 'dark'
}

export interface ThemeContextValue {
  theme: Theme
  toggle: () => void
}

export const ThemeContext = createContext<ThemeContextValue | null>(null)

export function useTheme(): ThemeContextValue {
  const ctx = useContext(ThemeContext)
  if (!ctx) throw new Error('useTheme must be used within ThemeProvider')
  return ctx
}
