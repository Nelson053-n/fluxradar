import type { FluxNode } from '../api/client'
import { NA, formatAge, formatDuration } from '../lib/format'
import { useI18n, type Keys } from '../i18n/store'

interface SpotlightProps {
  nodes: FluxNode[]
  /** Возраст старейшей ноды флота, сек (из summary). */
  oldestAgeSecs: number
  /**
   * Значение «Apps on a single node» (макс. приложений на одной ноде).
   * Медленный запрос грузится отдельно: спиннер пока считаем, число когда готово,
   * «—» при ошибке/отсутствии.
   */
  mostHostedValue: React.ReactNode
  /** Нода с максимумом приложений (IP + tier) — для подписи под значением; null если нет. */
  mostHostedNode: { ip: string; tier: string; count: number } | null
}

const tierTag =
  'rounded bg-subtle px-2 py-0.5 text-[11px] uppercase tracking-[0.05em] text-flux-cyan'

const tierKey: Record<string, Keys> = {
  CUMULUS: 'tierName.cumulus',
  NIMBUS: 'tierName.nimbus',
  STRATUS: 'tierName.stratus',
}

/**
 * Spotlight — три карточки мокапа. «Highest Rank» (нода с минимальным rank) и
 * «Oldest Node» (возраст старейшей ноды из summary) — реальные данные;
 * «Most Hosted» нет дёшево по флоту → «—» (вёрстка сохранена).
 */
export function Spotlight({
  nodes,
  oldestAgeSecs,
  mostHostedValue,
  mostHostedNode,
}: SpotlightProps) {
  const { t, lang } = useI18n()
  const best = nodes.length
    ? nodes.reduce((a, b) => (b.rank < a.rank ? b : a))
    : null
  // Старейшая нода по локальному списку — для подписи (IP + tier) под значением.
  const oldest = nodes.length
    ? nodes.reduce((a, b) => (b.age_secs > a.age_secs ? b : a))
    : null

  return (
    <>
      <div className="mb-5 flex items-end justify-between">
        <div>
          <h2 className="text-xl font-bold tracking-[-0.02em] md:text-2xl">{t('spotlight.title')}</h2>
          <div className="mt-1 text-[13px] text-text-dim">{t('spotlight.subtitle')}</div>
        </div>
      </div>

      <div className="stagger stagger-spot mb-8 grid grid-cols-1 gap-5 md:grid-cols-3">
        {/* Oldest Node — возраст старейшей ноды (реальные данные) */}
        <SpotCard
          ribbon={t('spotlight.oldest')}
          ribbonClass="border-[rgba(79,215,232,0.3)] bg-[rgba(79,215,232,0.15)] text-flux-cyan"
          title={t('spotlight.oldest.title')}
          value={oldestAgeSecs > 0 ? formatAge(oldestAgeSecs, lang) : NA}
          meta={
            oldest ? (
              <>
                <span>{oldest.ip}</span>
                <span className={tierTag}>{t(tierKey[oldest.tier] ?? 'tierName.cumulus')}</span>
              </>
            ) : null
          }
          noData={t('spotlight.noData')}
        />

        {/* Highest Rank — реальные данные */}
        <SpotCard
          ribbon={t('spotlight.highestRank')}
          ribbonClass="border-[rgba(245,184,71,0.3)] bg-[rgba(245,184,71,0.15)] text-warning"
          title={t('spotlight.highestRank.title')}
          value={best ? `#${best.rank.toLocaleString('en-US')}` : NA}
          meta={
            best ? (
              <>
                <span>{best.ip}</span>
                <span className={tierTag}>{t(tierKey[best.tier] ?? 'tierName.cumulus')}</span>
                <span className="ml-auto flex items-center gap-1.5 whitespace-nowrap">
                  <span className="uppercase tracking-[0.06em] text-text-dim/70">
                    {t('spotlight.payoutEta')}
                  </span>
                  <span className="text-text-secondary">
                    {best.payout_eta_secs <= 0
                      ? t('nodes.payoutSoon')
                      : formatDuration(best.payout_eta_secs, lang)}
                  </span>
                </span>
              </>
            ) : null
          }
          noData={t('spotlight.noData')}
        />

        {/* Most Hosted — макс. приложений на одной ноде (медленный отдельный запрос). */}
        <SpotCard
          ribbon={t('spotlight.mostHosted')}
          ribbonClass="border-[rgba(123,91,255,0.3)] bg-[rgba(123,91,255,0.15)] text-flux-purple"
          title={t('spotlight.mostHosted.title')}
          value={mostHostedValue}
          meta={
            mostHostedNode ? (
              <>
                <span>{mostHostedNode.ip}</span>
                <span className={tierTag}>
                  {t(tierKey[mostHostedNode.tier.toUpperCase()] ?? 'tierName.cumulus')}
                </span>
              </>
            ) : null
          }
          noData={t('spotlight.noData')}
        />
      </div>
    </>
  )
}

interface SpotCardProps {
  ribbon: string
  ribbonClass: string
  title: string
  value: React.ReactNode
  meta: React.ReactNode
  noData: string
}

function SpotCard({ ribbon, ribbonClass, title, value, meta, noData }: SpotCardProps) {
  return (
    <div className="relative overflow-hidden rounded-2xl border border-border bg-bg-card p-6 backdrop-blur-xl">
      <span
        className={`absolute right-4 top-4 rounded-full border px-2.5 py-1 font-mono text-[10px] uppercase tracking-[0.12em] ${ribbonClass}`}
      >
        {ribbon}
      </span>
      <div className="mb-3 text-[13px] font-medium text-text-secondary">{title}</div>
      <div className="mb-3 font-mono text-[28px] font-extrabold tracking-[-0.03em]">
        {value}
      </div>
      <div className="flex min-h-[18px] flex-wrap items-center gap-x-3 gap-y-1.5 border-t border-border pt-4 font-mono text-xs text-text-dim">
        {meta ?? <span className="text-text-dim/60">{noData}</span>}
      </div>
    </div>
  )
}
