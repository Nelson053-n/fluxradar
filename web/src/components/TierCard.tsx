import { ChartIcon } from './icons'
import { cardBase } from './cardStyles'
import { formatInt } from '../lib/format'
import { useT, type Keys } from '../i18n/store'

interface TierCardProps {
  cumulus: number
  nimbus: number
  stratus: number
}

const ROWS = [
  { key: 'cumulus', nameKey: 'tierName.cumulus', dot: 'var(--flux-cyan)', from: 'var(--flux-cyan)', to: 'rgba(79,215,232,0.4)' },
  { key: 'nimbus', nameKey: 'tierName.nimbus', dot: 'var(--flux-glow)', from: 'var(--flux-glow)', to: 'rgba(91,141,239,0.4)' },
  { key: 'stratus', nameKey: 'tierName.stratus', dot: 'var(--flux-purple)', from: 'var(--flux-purple)', to: 'rgba(123,91,255,0.4)' },
] as const satisfies readonly { key: string; nameKey: Keys; dot: string; from: string; to: string }[]

/** Tier Breakdown: три полоски с долями тиров (ширина = доля от total). */
export function TierCard({ cumulus, nimbus, stratus }: TierCardProps) {
  const t = useT()
  const counts = { cumulus, nimbus, stratus }
  const total = cumulus + nimbus + stratus

  return (
    <div className={cardBase}>
      <div className="mb-[18px] flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.08em] text-text-dim">
        <span className="flex h-8 w-8 items-center justify-center rounded-lg border border-[rgba(79,215,232,0.2)] bg-gradient-to-br from-[rgba(43,97,209,0.2)] to-[rgba(79,215,232,0.15)] text-flux-cyan">
          <ChartIcon />
        </span>
        {t('tier.title')}
      </div>
      <div className="mt-2 flex flex-col gap-3.5">
        {ROWS.map((row) => {
          const count = counts[row.key]
          const pct = total > 0 ? (count / total) * 100 : 0
          return (
            <div key={row.key}>
              <div className="mb-1.5 flex justify-between text-[13px]">
                <span className="flex items-center gap-2 font-medium">
                  <span
                    className="h-1.5 w-1.5 rounded-full"
                    style={{ background: row.dot, boxShadow: `0 0 8px ${row.dot}` }}
                  />
                  {t(row.nameKey)}
                </span>
                <span className="font-mono text-xs text-text-dim">{formatInt(count)}</span>
              </div>
              <div className="h-1 overflow-hidden rounded-full bg-subtle">
                <div
                  className="h-full rounded-full"
                  style={{
                    width: `${pct}%`,
                    background: `linear-gradient(90deg, ${row.from}, ${row.to})`,
                  }}
                />
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}
