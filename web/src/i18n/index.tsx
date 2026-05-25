import { useCallback, useMemo, useState, type ReactNode } from 'react'
import en from './en'
import ru from './ru'
import {
  I18nContext,
  LANG_STORAGE_KEY,
  detectLang,
  interpolate,
  type I18nContextValue,
  type Keys,
  type Lang,
  type Translate,
} from './store'

// Реэкспорт типов (значения-хуки импортируются из './i18n/store' напрямую,
// чтобы не ломать react-refresh: файл с компонентом экспортирует только компонент).
export type { Lang, Keys, Translate } from './store'

const DICTS: Record<Lang, Record<Keys, string>> = { en, ru }

export function I18nProvider({ children }: { children: ReactNode }) {
  const [lang, setLangState] = useState<Lang>(detectLang)

  const setLang = useCallback((next: Lang) => {
    setLangState(next)
    localStorage.setItem(LANG_STORAGE_KEY, next)
    document.documentElement.lang = next
  }, [])

  const t = useCallback<Translate>(
    (key, params) => interpolate(DICTS[lang][key], params),
    [lang],
  )

  const value = useMemo<I18nContextValue>(() => ({ lang, setLang, t }), [lang, setLang, t])

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>
}
