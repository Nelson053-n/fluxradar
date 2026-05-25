import { createContext, useContext } from 'react'
import en from './en'

export type Lang = 'en' | 'ru' | 'zh'
export type Keys = keyof typeof en

export const LANG_STORAGE_KEY = 'fluxscope:lang'

/** navigator.language → поддерживаемый язык. ru* → ru, zh* → zh, иначе en. */
export function detectLang(): Lang {
  const stored = localStorage.getItem(LANG_STORAGE_KEY)
  if (stored === 'en' || stored === 'ru' || stored === 'zh') return stored
  const nav = (navigator.language || 'en').toLowerCase()
  if (nav.startsWith('ru')) return 'ru'
  if (nav.startsWith('zh')) return 'zh'
  return 'en'
}

/** Подстановка плейсхолдеров вида {name} из params. */
export function interpolate(
  template: string,
  params?: Record<string, string | number>,
): string {
  if (!params) return template
  return template.replace(/\{(\w+)\}/g, (_, k: string) =>
    k in params ? String(params[k]) : `{${k}}`,
  )
}

export type Translate = (key: Keys, params?: Record<string, string | number>) => string

export interface I18nContextValue {
  lang: Lang
  setLang: (lang: Lang) => void
  t: Translate
}

export const I18nContext = createContext<I18nContextValue | null>(null)

export function useI18n(): I18nContextValue {
  const ctx = useContext(I18nContext)
  if (!ctx) throw new Error('useI18n must be used within I18nProvider')
  return ctx
}

/** Удобный хук, когда нужна только функция перевода. */
export function useT(): Translate {
  return useI18n().t
}
