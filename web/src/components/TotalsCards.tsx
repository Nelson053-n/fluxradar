import type { WalletSummary } from '../api/client'
import type { Money } from '../api/client'
import { cardBase } from './cardStyles'
import { formatFlux, formatUsdSmart } from '../lib/format'
import { useT, type Keys } from '../i18n/store'

interface TotalsCardsProps {
  totals: WalletSummary['totals']
}

// Только денежные поля totals (is_estimate — флаг, не карточка).
type MoneyKey = 'mined' | 'claimed' | 'claimable'

const ITEMS: { key: MoneyKey; labelKey: Keys }[] = [
  { key: 'mined', labelKey: 'totals.mined' },
  { key: 'claimed', labelKey: 'totals.claimed' },
  { key: 'claimable', labelKey: 'totals.claimable' },
]

/**
 * Totals — три карточки накопленных сумм за жизнь нод:
 * Только подтверждённые суммы Fusion. Оценка native+PA по возрасту нод
 * не заменяет историю PA при недоступном источнике.
 */
export function TotalsCards({ totals }: TotalsCardsProps) {
  const t = useT()
  return (
    <>
      {ITEMS.map(({ key, labelKey }) => (
        <TotalCard
          key={key}
          label={t(labelKey)}
          money={totals.is_estimate ? null : totals[key]}
          note={totals.is_estimate ? t('earnings.notAvailable') : t('totals.noteReal')}
          estimate={totals.is_estimate ? t('earnings.notAvailable') : null}
        />
      ))}
    </>
  )
}

interface TotalCardProps {
  label: string
  money: Money | null
  note: string
  /** Бейдж недоступности источника или null. */
  estimate: string | null
}

function TotalCard({ label, money, note, estimate }: TotalCardProps) {
  return (
    <div className={cardBase}>
      <div className="mb-[18px] flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.08em] text-text-dim">
        {label}
        {estimate && (
          <span className="ml-1 rounded-full border border-[rgba(79,215,232,0.3)] bg-[rgba(79,215,232,0.12)] px-2 py-0.5 text-[10px] font-semibold normal-case tracking-normal text-flux-cyan">
            {estimate}
          </span>
        )}
      </div>
      <div className="mb-1.5 font-mono text-[28px] font-extrabold leading-none tracking-[-0.04em] text-text-primary md:text-[34px]">
        {money ? formatFlux(money.flux) : '—'}
        <span className="ml-1.5 text-[14px] font-semibold text-text-dim">FLUX</span>
      </div>
      <div className="font-mono text-[13px] text-text-secondary">{money ? `≈ ${formatUsdSmart(money.usd)}` : '—'}</div>
      <div className="mt-2 text-[11px] text-text-dim">{note}</div>
    </div>
  )
}
