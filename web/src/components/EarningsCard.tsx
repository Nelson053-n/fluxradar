import { useState } from 'react'
import { DollarIcon } from './icons'
import type { WalletSummary } from '../api/client'
import { formatFlux, formatNum, formatUsdSmart } from '../lib/format'
import { useT, type Keys } from '../i18n/store'

type Earnings = WalletSummary['earnings']

interface EarningsCardProps {
  earnings: Earnings
}

type Period = 'daily' | 'monthly' | 'yearly'

const PERIODS: { period: Period; labelKey: Keys }[] = [
  { period: 'daily', labelKey: 'earnings.period.day' },
  { period: 'monthly', labelKey: 'earnings.period.month' },
  { period: 'yearly', labelKey: 'earnings.period.year' },
]

/**
 * Estimated Earnings — сегментированный переключатель периода (День/Месяц/Год)
 * над крупной цифрой + APY от залога. Значения — оценка (помечены «est.»),
 * не факт выплат.
 */
export function EarningsCard({ earnings }: EarningsCardProps) {
  const t = useT()
  const [period, setPeriod] = useState<Period>('monthly')
  const money = earnings[period]

  return (
    <div className="card-toptrim relative overflow-hidden rounded-2xl border border-[rgba(79,215,232,0.25)] bg-gradient-to-br from-[rgba(43,97,209,0.15)] to-[rgba(79,215,232,0.08)] p-6 backdrop-blur-xl sm:col-span-2">
      <div className="mb-[18px] flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.08em] text-text-dim">
        <span className="flex h-8 w-8 items-center justify-center rounded-lg border border-[rgba(79,215,232,0.2)] bg-gradient-to-br from-[rgba(43,97,209,0.2)] to-[rgba(79,215,232,0.15)] text-flux-cyan">
          <DollarIcon />
        </span>
        {t('earnings.title')}
        <span className="ml-1 rounded-full border border-[rgba(79,215,232,0.3)] bg-[rgba(79,215,232,0.12)] px-2 py-0.5 text-[10px] font-semibold normal-case tracking-normal text-flux-cyan">
          {t('earnings.estimate')}
        </span>
      </div>

      {/* Сегментированный переключатель периода */}
      <div
        role="tablist"
        className="mb-4 inline-flex rounded-xl border border-border bg-subtle p-1"
      >
        {PERIODS.map(({ period: p, labelKey }) => {
          const active = p === period
          return (
            <button
              key={p}
              type="button"
              role="tab"
              aria-selected={active}
              onClick={() => setPeriod(p)}
              className={`rounded-lg px-3.5 py-1.5 text-xs font-semibold transition-colors ${
                active
                  ? 'bg-flux-primary text-white shadow-[0_2px_8px_rgba(43,97,209,0.4)]'
                  : 'text-text-dim hover:text-text-secondary'
              }`}
            >
              {t(labelKey)}
            </button>
          )
        })}
      </div>

      <div className="grid grid-cols-1 items-end gap-6 sm:grid-cols-2">
        <div>
          <div className="font-mono text-[32px] font-extrabold leading-none tracking-[-0.04em] text-text-primary md:text-[40px]">
            {formatFlux(money.flux)}
            <span className="ml-1.5 text-[15px] font-semibold text-text-dim">FLUX</span>
          </div>
          <div className="mt-2 font-mono text-[13px] text-text-secondary">
            ≈ {formatUsdSmart(money.usd)}
          </div>
        </div>

        {/* APY от залога */}
        <div className="sm:text-right">
          <div className="font-mono text-[26px] font-extrabold leading-none tracking-[-0.03em] text-flux-cyan md:text-[32px]">
            {formatNum(earnings.apy_percent, 1)}%
          </div>
          <div className="mt-2 text-xs font-semibold uppercase tracking-[0.08em] text-text-dim">
            {t('earnings.apy')}
          </div>
        </div>
      </div>
    </div>
  )
}
