import type { PaChain, WalletSummary } from '../api/client'
import { cardBase } from './cardStyles'
import { formatInt, formatNum } from '../lib/format'
import { useT } from '../i18n/store'

type Earnings = WalletSummary['earnings']

interface ParallelAssetsProps {
  earnings: Earnings
  /** Реальная разбивка по чейнам (если бэкенд её прислал). */
  paChains: PaChain[]
}

// Тикеры PA-чейнов (10 шт.) — статичный список для fallback-режима, не переводится.
const PA_CHAINS = [
  'KDA',
  'ETH',
  'BSC',
  'SOL',
  'TRX',
  'AVAX',
  'ERG',
  'ALGO',
  'BTC',
  'MATIC',
] as const

// Малые суммы FLUX показываем с дробной частью, крупные — целыми с разделителями.
function paAmount(flux: number): string {
  return flux > 0 && flux < 1000 ? formatNum(flux, 2) : formatInt(flux)
}

/**
 * Parallel Assets — боковая доходность Flux-нод по PA-чейнам.
 * Если бэкенд прислал реальную разбивку (`paChains`) — показываем по каждому
 * чейну его claimable/claimed/received (реальные данные, без «est.»).
 * Иначе — fallback на оценку: делим годовую долю PA поровну между чейнами («est.»).
 */
export function ParallelAssets({ earnings, paChains }: ParallelAssetsProps) {
  const t = useT()

  if (paChains.length > 0) {
    return (
      <div className={cardBase}>
        <div className="mb-[18px] flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.08em] text-text-dim">
          {t('pa.title')}
        </div>

        <div className="mb-5 text-[13px] text-text-secondary">{t('pa.realSubtitle')}</div>

        <div className="grid grid-cols-1 gap-2.5 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5">
          {paChains.map((c) => (
            <div
              key={c.chain}
              className="rounded-xl border border-border bg-subtle px-3 py-2.5"
            >
              <div className="font-mono text-[13px] font-bold tracking-[0.04em] text-flux-cyan">
                {c.chain.toUpperCase()}
              </div>
              <div className="mt-2 space-y-1 font-mono text-[12px]">
                <ChainRow label={t('pa.claimable')} value={paAmount(c.claimable)} accent />
                <ChainRow label={t('pa.claimed')} value={paAmount(c.claimed)} />
                <ChainRow label={t('pa.fees')} value={paAmount(c.fees)} />
              </div>
            </div>
          ))}
        </div>
      </div>
    )
  }

  // Fallback (оценка): нет реальной разбивки — делим годовую долю PA поровну.
  const count = earnings.parallel_assets_count || PA_CHAINS.length
  const sharePct = earnings.parallel_assets_pct / count
  const perChainYearFlux =
    (earnings.yearly.flux * (earnings.parallel_assets_pct / 100)) / count

  return (
    <div className={cardBase}>
      <div className="mb-[18px] flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.08em] text-text-dim">
        {t('pa.title')}
        <span className="ml-1 rounded-full border border-[rgba(123,91,255,0.3)] bg-[rgba(123,91,255,0.12)] px-2 py-0.5 text-[10px] font-semibold normal-case tracking-normal text-flux-purple">
          {t('earnings.estimate')}
        </span>
      </div>

      <div className="mb-1 text-[13px] text-text-secondary">
        {t('pa.summary', {
          pct: formatNum(earnings.parallel_assets_pct, 0),
          count,
        })}
      </div>
      <div className="mb-5 text-xs text-text-dim">
        {t('pa.share', { pct: formatNum(sharePct, 1) })} · {t('pa.perChainYear')}
      </div>

      <div className="grid grid-cols-2 gap-2.5 sm:grid-cols-3 md:grid-cols-5">
        {PA_CHAINS.slice(0, count).map((ticker) => (
          <div
            key={ticker}
            className="rounded-xl border border-border bg-subtle px-3 py-2.5"
          >
            <div className="font-mono text-[13px] font-bold tracking-[0.04em] text-flux-cyan">
              {ticker}
            </div>
            <div className="mt-1 font-mono text-[15px] font-extrabold tracking-[-0.02em] text-text-primary">
              {formatInt(perChainYearFlux)}
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}

function ChainRow({ label, value, accent }: { label: string; value: string; accent?: boolean }) {
  return (
    <div className="flex items-center justify-between gap-2">
      <span className="text-text-dim">{label}</span>
      <span className={accent ? 'font-semibold text-text-primary' : 'text-text-secondary'}>
        {value}
      </span>
    </div>
  )
}
