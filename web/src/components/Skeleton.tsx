/** Состояния загрузки/ошибки в стиле мокапа (glassmorphism-карточки). */
import { useT } from '../i18n/store'
import { RadarSpinner } from './RadarSpinner'

/** Лёгкий контур карточки (без пульсации) — фон под крутящимся радаром. */
function CardOutline() {
  return (
    <div className="h-[120px] rounded-2xl border border-border bg-bg-card backdrop-blur-xl" />
  )
}

/** Полный каркас дашборда на время первой загрузки: радар по центру + контуры карточек. */
export function DashboardSkeleton() {
  const t = useT()
  return (
    <div aria-busy="true" aria-label={t('state.loadingDashboard')} className="relative">
      <div className="pointer-events-none select-none opacity-40">
        <div className="mb-8 grid grid-cols-1 gap-5 md:grid-cols-2 xl:grid-cols-4">
          {Array.from({ length: 4 }).map((_, i) => (
            <CardOutline key={i} />
          ))}
        </div>
        <div className="grid grid-cols-1 gap-5 md:grid-cols-2 xl:grid-cols-4">
          {Array.from({ length: 4 }).map((_, i) => (
            <CardOutline key={i} />
          ))}
        </div>
      </div>
      <div className="absolute inset-0 flex flex-col items-center justify-center gap-5">
        <RadarSpinner label={t('state.loadingDashboard')} size="lg" />
        <div className="text-sm font-medium text-text-secondary">
          {t('state.loadingDashboard')}
        </div>
      </div>
    </div>
  )
}

interface ErrorStateProps {
  message: string
  onRetry: () => void
}

export function ErrorState({ message, onRetry }: ErrorStateProps) {
  const t = useT()
  return (
    <div className="mx-auto flex max-w-xl flex-col items-center gap-5 rounded-2xl border border-[rgba(255,107,107,0.25)] bg-[rgba(255,107,107,0.06)] p-10 text-center backdrop-blur-xl">
      <div className="flex h-12 w-12 items-center justify-center rounded-xl border border-[rgba(255,107,107,0.3)] bg-[rgba(255,107,107,0.1)]">
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="var(--danger)" strokeWidth="2">
          <circle cx="12" cy="12" r="10" />
          <line x1="12" y1="8" x2="12" y2="12" />
          <line x1="12" y1="16" x2="12.01" y2="16" />
        </svg>
      </div>
      <div>
        <div className="mb-1.5 text-lg font-bold text-text-primary">{t('error.title')}</div>
        <p className="text-sm text-text-secondary">{message}</p>
      </div>
      <button
        type="button"
        onClick={onRetry}
        className="rounded-lg bg-gradient-to-br from-flux-primary to-flux-glow px-5 py-2.5 text-sm font-semibold text-white shadow-[0_4px_20px_rgba(43,97,209,0.5)] transition-transform hover:-translate-y-px"
      >
        {t('error.retry')}
      </button>
    </div>
  )
}

export function EmptyFleet() {
  const t = useT()
  return (
    <div className="mx-auto flex max-w-xl flex-col items-center gap-4 rounded-2xl border border-border bg-bg-card p-12 text-center backdrop-blur-xl">
      <div className="flex h-12 w-12 items-center justify-center rounded-xl border border-border bg-subtle text-text-dim">
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <rect x="2" y="3" width="20" height="14" rx="2" />
          <line x1="8" y1="21" x2="16" y2="21" />
          <line x1="12" y1="17" x2="12" y2="21" />
        </svg>
      </div>
      <div className="text-lg font-bold text-text-primary">{t('empty.title')}</div>
      <p className="text-sm text-text-secondary">{t('empty.body')}</p>
    </div>
  )
}
