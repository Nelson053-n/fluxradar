import { useTheme } from '../lib/themeStore'
import { useT } from '../i18n/store'
import { SunIcon, MoonIcon } from './icons'

/** Переключатель тёмной/светлой темы (солнце/луна) для Header. */
export function ThemeToggle() {
  const { theme, toggle } = useTheme()
  const t = useT()
  const isDark = theme === 'dark'

  return (
    <button
      type="button"
      onClick={toggle}
      title={isDark ? t('theme.toggleToLight') : t('theme.toggleToDark')}
      aria-label={isDark ? t('theme.toggleToLight') : t('theme.toggleToDark')}
      className="flex h-9 w-9 items-center justify-center rounded-xl border border-border bg-[var(--bg-card)] text-text-secondary backdrop-blur-xl transition-colors hover:border-border-strong hover:text-text-primary"
    >
      {isDark ? <SunIcon /> : <MoonIcon />}
    </button>
  )
}
