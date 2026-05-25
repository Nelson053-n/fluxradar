import { useEffect, useState } from 'react'
import { fetchNodeDetail, type FluxNode, type NodeDetail } from '../api/client'
import { statusPill, statusPillTone, statusDotTone } from './cardStyles'
import { NA, countryFlag, formatDuration, formatInt, statusTone } from '../lib/format'
import { useI18n } from '../i18n/store'
import { RadarSpinner } from './RadarSpinner'

interface NodeDetailDrawerProps {
  /** Нода, для которой открыт drawer; null — закрыт. */
  node: FluxNode | null
  onClose: () => void
}

// Результат загрузки для конкретного ip; пока null — состояние «loading».
type Result =
  | { ip: string; status: 'error' }
  | { ip: string; status: 'ready'; detail: NodeDetail }

/**
 * Выезжающая справа панель с ленивыми деталями ноды (статус, гео, apps).
 * Дорогие данные тянутся по клику через fetchNodeDetail; запрос отменяется
 * при смене ноды/закрытии. Закрытие — крестик / Esc / клик по бэкдропу.
 */
export function NodeDetailDrawer({ node, onClose }: NodeDetailDrawerProps) {
  const { t, lang } = useI18n()
  // Храним только результат (после await). «loading» = результата для текущего ip ещё нет —
  // так единственные setState идут из async-колбэков, без синхронного set в теле эффекта.
  const [result, setResult] = useState<Result | null>(null)
  // reloadKey форсит повторную загрузку по кнопке Retry без смены ноды.
  const [reloadKey, setReloadKey] = useState(0)

  const ip = node?.ip ?? null

  // Загрузка деталей при открытии/смене ноды/ретрае. AbortController отменяет
  // устаревший запрос (быстрое переключение строк).
  useEffect(() => {
    if (ip == null) return
    const ctrl = new AbortController()
    fetchNodeDetail(ip, ctrl.signal)
      .then((detail) => {
        if (!ctrl.signal.aborted) setResult({ ip, status: 'ready', detail })
      })
      .catch(() => {
        if (!ctrl.signal.aborted) setResult({ ip, status: 'error' })
      })
    return () => ctrl.abort()
  }, [ip, reloadKey])

  // Результат относится к текущей ноде? Иначе — ещё грузим (loading).
  const phase: 'loading' | 'error' | 'ready' =
    result && result.ip === ip ? result.status : 'loading'

  // Esc закрывает; вешаем слушатель только когда открыт.
  useEffect(() => {
    if (ip == null) return
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') onClose()
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [ip, onClose])

  if (node == null) return null

  const detail = phase === 'ready' && result?.status === 'ready' ? result.detail : null
  const tone = statusTone(detail?.status ?? node.status)

  return (
    <div className="fixed inset-0 z-50 flex justify-end">
      {/* Бэкдроп — клик закрывает */}
      <div
        className="absolute inset-0 animate-fadeIn bg-black/50 backdrop-blur-sm"
        onClick={onClose}
        aria-hidden="true"
      />

      <aside
        role="dialog"
        aria-modal="true"
        aria-label={t('drawer.title')}
        className="relative flex h-full w-full max-w-md animate-slideInRight flex-col border-l border-border bg-[var(--bg-elevated)] shadow-[-20px_0_60px_rgba(0,0,0,0.4)]"
      >
        {/* Header */}
        <div className="flex items-center justify-between border-b border-border px-6 py-5">
          <div>
            <div className="text-xs font-semibold uppercase tracking-[0.1em] text-text-dim">
              {t('drawer.title')}
            </div>
            <div className="mt-1 font-mono text-lg font-bold text-text-primary">{node.ip}</div>
          </div>
          <button
            type="button"
            onClick={onClose}
            aria-label={t('drawer.close')}
            className="flex h-9 w-9 items-center justify-center rounded-lg border border-border text-text-dim transition-colors hover:border-border-strong hover:text-text-primary"
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>

        {/* Body */}
        <div className="thin-scroll flex-1 overflow-y-auto px-6 py-6">
          {/* Поля из списка нод (доступны сразу, без ленивой загрузки detail) */}
          <div className="mb-6 grid grid-cols-2 gap-4">
            <Field label={t('drawer.payout')}>
              <span className="font-mono text-sm font-semibold text-text-primary">
                {node.payout_eta_secs <= 0
                  ? t('nodes.payoutSoon')
                  : t('nodes.payoutIn', { dur: formatDuration(node.payout_eta_secs, lang) })}
              </span>
            </Field>
            <Field label={t('drawer.maintenance')}>
              <span className="font-mono text-sm font-semibold text-text-primary">
                {node.maintenance_window_secs != null
                  ? formatDuration(node.maintenance_window_secs, lang)
                  : t('nodes.maintClosed')}
              </span>
            </Field>
            <Field label={t('drawer.fluxos')}>
              <span className="font-mono text-sm font-semibold text-text-primary">
                {node.flux_os_version ?? NA}
              </span>
            </Field>
            <Field label={t('drawer.appsCount')}>
              <span className="font-mono text-sm font-semibold text-text-primary">
                {detail ? formatInt(detail.apps_count) : NA}
              </span>
            </Field>
          </div>

          {phase === 'loading' && (
            <div className="flex flex-col items-center justify-center gap-4 py-20 text-center">
              <RadarSpinner label={t('drawer.loading')} size="lg" />
              <div className="text-sm text-text-secondary">{t('drawer.loading')}</div>
            </div>
          )}

          {phase === 'error' && (
            <div className="flex flex-col items-center justify-center gap-4 py-20 text-center">
              <p className="text-sm text-text-secondary">{t('drawer.error')}</p>
              <button
                type="button"
                onClick={() => setReloadKey((k) => k + 1)}
                className="rounded-lg border border-[rgba(91,141,239,0.3)] bg-[rgba(43,97,209,0.12)] px-4 py-2 text-[13px] font-semibold text-flux-glow"
              >
                {t('drawer.retry')}
              </button>
            </div>
          )}

          {detail && (
            <div className="flex flex-col gap-6">
              {/* Status + Tier */}
              <div className="grid grid-cols-2 gap-4">
                <Field label={t('drawer.status')}>
                  <span className={`${statusPill} ${statusPillTone[tone]}`}>
                    <span className={`h-1.5 w-1.5 rounded-full ${statusDotTone[tone]}`} />
                    {detail.status ?? node.status}
                  </span>
                </Field>
                <Field label={t('drawer.tier')}>
                  <span className="font-mono text-sm font-semibold text-text-primary">
                    {detail.tier ?? node.tier}
                  </span>
                </Field>
              </div>

              {/* Location */}
              <Field label={t('drawer.location')}>
                {detail.geo ? (
                  <span className="flex items-center gap-2 font-mono text-sm text-text-primary">
                    <span className="text-lg leading-none">
                      {countryFlag(detail.geo.country_code)}
                    </span>
                    <span>
                      {detail.geo.city
                        ? `${detail.geo.city}, ${detail.geo.country}`
                        : detail.geo.country}
                    </span>
                  </span>
                ) : (
                  <span className="text-sm text-text-dim">{t('drawer.noLocation')}</span>
                )}
              </Field>

              {/* Apps */}
              <Field label={t('drawer.apps')}>
                {detail.apps.length > 0 ? (
                  <div className="flex flex-wrap gap-2">
                    {detail.apps.map((app) => (
                      <span
                        key={app}
                        className="rounded-lg border border-border bg-subtle px-2.5 py-1 font-mono text-[12px] text-text-secondary"
                      >
                        {app}
                      </span>
                    ))}
                  </div>
                ) : (
                  <span className="text-sm text-text-dim">{t('drawer.noApps')}</span>
                )}
              </Field>
            </div>
          )}
        </div>
      </aside>
    </div>
  )
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div>
      <div className="mb-2 text-xs font-semibold uppercase tracking-[0.08em] text-text-dim">
        {label}
      </div>
      {children}
    </div>
  )
}
