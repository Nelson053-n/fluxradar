import { useState } from 'react'
import { ChartIcon } from './icons'
import { cardBase } from './cardStyles'
import { formatFlux, formatNum, formatUsd, formatUsdSmart } from '../lib/format'
import { calcEarnings, COLLATERAL, type TierCounts, type TierKey } from '../lib/earningsCalc'
import { useT, type Keys } from '../i18n/store'

interface CalculatorProps {
  /** Число нод сети по тирам — делитель частоты выплат. */
  network: TierCounts | null
  priceUsd: number | null
  /** Кол-во нод пользователя (если кошелёк загружен) — стартовые значения ползунков. */
  defaultCounts: TierCounts
}

type Period = 'daily' | 'monthly' | 'yearly'

const PERIODS: { period: Period; labelKey: Keys }[] = [
  { period: 'daily', labelKey: 'earnings.period.day' },
  { period: 'monthly', labelKey: 'earnings.period.month' },
  { period: 'yearly', labelKey: 'earnings.period.year' },
]

const TIER_ROWS: { tier: TierKey; labelKey: Keys; accent: string }[] = [
  { tier: 'cumulus', labelKey: 'calc.cumulus', accent: 'text-flux-cyan' },
  { tier: 'nimbus', labelKey: 'calc.nimbus', accent: 'text-flux-glow' },
  { tier: 'stratus', labelKey: 'calc.stratus', accent: 'text-flux-purple' },
]

const SLIDER_MAX = 100

/**
 * Калькулятор доходности: ползунки/поля кол-ва нод по тирам → стоимость флота
 * (collateral) + доходность день/месяц/год во FLUX (с учётом PA) и в USD + APY.
 * Формулы — lib/earningsCalc.ts (порт из domain/earnings.rs).
 */
export function Calculator({ network, priceUsd, defaultCounts }: CalculatorProps) {
  const t = useT()
  const [period, setPeriod] = useState<Period>('monthly')
  const [counts, setCounts] = useState<TierCounts>(defaultCounts)
  const [touched, setTouched] = useState(false)

  // Подставить число нод пользователя, когда кошелёк загрузился (если юзер ещё не трогал ползунки).
  // «adjust state during render» вместо setState в эффекте (react-hooks/set-state-in-effect).
  const [seenDefault, setSeenDefault] = useState(defaultCounts)
  if (!touched && defaultCounts !== seenDefault) {
    setSeenDefault(defaultCounts)
    setCounts(defaultCounts)
  }

  function setTier(tier: TierKey, raw: number) {
    setTouched(true)
    const v = Number.isFinite(raw) ? Math.max(0, Math.floor(raw)) : 0
    setCounts((c) => ({ ...c, [tier]: v }))
  }

  // Сеть: загруженная или нейтральный fallback (1 — чтобы не делить на 0 до прихода данных).
  const net = network ?? { cumulus: 1, nimbus: 1, stratus: 1 }
  const r = calcEarnings(counts, net, priceUsd)

  const periodFlux = { daily: r.dailyFlux, monthly: r.monthlyFlux, yearly: r.yearlyFlux }[period]
  const periodUsd = { daily: r.dailyUsd, monthly: r.monthlyUsd, yearly: r.yearlyUsd }[period]

  return (
    <div className={cardBase}>
      <div className="mb-[18px] flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.08em] text-text-dim">
        <span className="flex h-8 w-8 items-center justify-center rounded-lg border border-[rgba(79,215,232,0.2)] bg-gradient-to-br from-[rgba(43,97,209,0.2)] to-[rgba(79,215,232,0.15)] text-flux-cyan">
          <ChartIcon />
        </span>
        {t('calc.title')}
        <span className="ml-1 rounded-full border border-[rgba(79,215,232,0.3)] bg-[rgba(79,215,232,0.12)] px-2 py-0.5 text-[10px] font-semibold normal-case tracking-normal text-flux-cyan">
          {t('earnings.estimate')}
        </span>
      </div>
      <div className="mb-6 text-[13px] text-text-secondary">{t('calc.subtitle')}</div>

      <div className="grid grid-cols-1 gap-8 lg:grid-cols-2">
        {/* Слева: ввод кол-ва нод + стоимость флота */}
        <div className="flex flex-col gap-5">
          {TIER_ROWS.map(({ tier, labelKey, accent }) => (
            <div key={tier}>
              <div className="mb-2 flex items-center justify-between gap-3">
                <span className={`text-sm font-semibold ${accent}`}>{t(labelKey)}</span>
                <input
                  type="number"
                  min={0}
                  value={counts[tier]}
                  onChange={(e) => setTier(tier, e.target.valueAsNumber)}
                  className="w-20 rounded-lg border border-border bg-subtle px-2.5 py-1.5 text-right font-mono text-sm text-text-primary focus:border-border-strong focus:outline-none"
                />
              </div>
              <input
                type="range"
                min={0}
                max={SLIDER_MAX}
                value={Math.min(counts[tier], SLIDER_MAX)}
                onChange={(e) => setTier(tier, e.target.valueAsNumber)}
                className="w-full accent-flux-primary"
              />
              <div className="mt-1 text-[11px] text-text-dim">
                {t('calc.collateralPer', {
                  flux: formatNum(COLLATERAL[tier], 0),
                })}
              </div>
            </div>
          ))}

          <div className="mt-2 rounded-xl border border-border bg-subtle px-4 py-3">
            <div className="text-[11px] font-semibold uppercase tracking-[0.08em] text-text-dim">
              {t('calc.collateral')}
            </div>
            <div className="mt-1 font-mono text-2xl font-extrabold tracking-[-0.03em] text-text-primary">
              {formatFlux(r.collateralFlux)}
              <span className="ml-1.5 text-[13px] font-semibold text-text-dim">FLUX</span>
            </div>
            <div className="mt-0.5 font-mono text-[13px] text-text-secondary">
              ≈ {formatUsd(r.collateralUsd)}
            </div>
          </div>
        </div>

        {/* Справа: период + доходность + APY */}
        <div className="flex flex-col">
          <div role="tablist" className="mb-4 inline-flex self-start rounded-xl border border-border bg-subtle p-1">
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

          <div className="text-[11px] font-semibold uppercase tracking-[0.08em] text-text-dim">
            {t('calc.earnings')}
          </div>
          <div className="mt-1 font-mono text-[32px] font-extrabold leading-none tracking-[-0.04em] text-text-primary md:text-[40px]">
            {formatFlux(periodFlux)}
            <span className="ml-1.5 text-[15px] font-semibold text-text-dim">FLUX</span>
          </div>
          <div className="mt-2 font-mono text-[13px] text-text-secondary">
            ≈ {formatUsdSmart(periodUsd)}
          </div>

          <div className="mt-auto pt-6">
            <div className="font-mono text-[26px] font-extrabold leading-none tracking-[-0.03em] text-flux-cyan md:text-[30px]">
              {formatNum(r.apyPercent, 1)}%
            </div>
            <div className="mt-2 text-xs font-semibold uppercase tracking-[0.08em] text-text-dim">
              {t('earnings.apy')}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
