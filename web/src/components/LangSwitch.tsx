import { useI18n, type Lang } from '../i18n/store'

const OPTIONS: { code: Lang; label: string }[] = [
  { code: 'en', label: 'EN' },
  { code: 'ru', label: 'RU' },
  { code: 'zh', label: '中文' },
]

/** Сегментный переключатель языка (RU / EN / 中文) для Header. */
export function LangSwitch() {
  const { lang, setLang, t } = useI18n()

  return (
    <div
      role="group"
      aria-label={t('lang.label')}
      className="flex items-center gap-0.5 rounded-xl border border-border bg-[var(--bg-card)] p-0.5 backdrop-blur-xl"
    >
      {OPTIONS.map((o) => (
        <button
          key={o.code}
          type="button"
          onClick={() => setLang(o.code)}
          aria-pressed={lang === o.code}
          className={
            lang === o.code
              ? 'rounded-lg bg-subtle-hover px-2 py-1 text-xs font-semibold text-text-primary'
              : 'rounded-lg px-2 py-1 text-xs font-medium text-text-secondary transition-colors hover:text-text-primary'
          }
        >
          {o.label}
        </button>
      ))}
    </div>
  )
}
