import { useMemo } from 'react'
import type { PricePoint } from '../api/client'
import { formatPrice } from '../lib/format'
import { cardBase } from './cardStyles'
import { useT } from '../i18n/store'
import { RadarSpinner } from './RadarSpinner'

interface PriceSparklineProps {
  /** Точки истории цены за год (null — ещё грузится, [] — пусто). */
  points: PricePoint[] | null
  /** Ошибка загрузки истории. */
  error: boolean
}

const W = 360
const H = 96

/**
 * Тултип-спарклайн цены FLUX за год: inline-SVG polyline (без сторонних либ)
 * + подписи min/max/now. Glassmorphism-стиль (cardBase), позиционируется
 * вызывающим (Header) под ценой.
 */
export function PriceSparkline({ points, error }: PriceSparklineProps) {
  const t = useT()

  const chart = useMemo(() => {
    if (!points || points.length < 2) return null
    const usd = points.map((p) => p.usd)
    const min = Math.min(...usd)
    const max = Math.max(...usd)
    const span = max - min || 1
    const stepX = W / (points.length - 1)
    const line = points
      .map((p, i) => {
        const x = i * stepX
        // Инвертируем Y (SVG: 0 сверху). Отступ 4px сверху/снизу под обводку.
        const y = 4 + (H - 8) * (1 - (p.usd - min) / span)
        return `${x.toFixed(1)},${y.toFixed(1)}`
      })
      .join(' ')
    const now = usd[usd.length - 1]
    return { line, min, max, now }
  }, [points])

  return (
    <div className={`${cardBase} w-[400px] p-4`}>
      <div className="mb-2 font-mono text-[11px] font-semibold uppercase tracking-[0.08em] text-text-dim">
        {t('price.history.title')}
      </div>

      {error && (
        <div className="py-4 text-center text-xs text-text-dim">{t('price.history.error')}</div>
      )}

      {!error && !chart && (
        <div className="flex items-center justify-center gap-2 py-4 text-center text-xs text-text-dim">
          <RadarSpinner label={t('price.history.loading')} size="sm" />
          {t('price.history.loading')}
        </div>
      )}

      {!error && chart && (
        <>
          <svg
            viewBox={`0 0 ${W} ${H}`}
            width="100%"
            height={H}
            preserveAspectRatio="none"
            role="img"
            aria-label={t('price.history.title')}
            className="overflow-visible"
          >
            <defs>
              <linearGradient id="sparkFill" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stopColor="var(--flux-cyan)" stopOpacity="0.25" />
                <stop offset="100%" stopColor="var(--flux-cyan)" stopOpacity="0" />
              </linearGradient>
            </defs>
            <polygon
              points={`0,${H} ${chart.line} ${W},${H}`}
              fill="url(#sparkFill)"
              stroke="none"
            />
            <polyline
              points={chart.line}
              fill="none"
              stroke="var(--flux-cyan)"
              strokeWidth="2"
              strokeLinejoin="round"
              strokeLinecap="round"
            />
          </svg>

          <div className="mt-3 flex items-center justify-between gap-2 font-mono text-[11px]">
            <Stat label={t('price.history.low')} value={formatPrice(chart.min)} />
            <Stat label={t('price.history.now')} value={formatPrice(chart.now)} accent />
            <Stat label={t('price.history.high')} value={formatPrice(chart.max)} />
          </div>
        </>
      )}
    </div>
  )
}

function Stat({ label, value, accent }: { label: string; value: string; accent?: boolean }) {
  return (
    <div className="flex flex-col gap-0.5">
      <span className="text-[10px] uppercase tracking-[0.06em] text-text-dim">{label}</span>
      <span className={accent ? 'font-semibold text-flux-cyan' : 'text-text-secondary'}>{value}</span>
    </div>
  )
}
