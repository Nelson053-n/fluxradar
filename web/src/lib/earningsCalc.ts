// Расчёт доходности нод — портирован 1-в-1 из crates/domain/src/earnings.rs.
// Используется калькулятором (Calculator.tsx). Сверено с реальным summary API.

export type TierKey = 'cumulus' | 'nimbus' | 'stratus'

export interface TierCounts {
  cumulus: number
  nimbus: number
  stratus: number
}

// Награда за блок одной ноде тира (FLUX/блок до деления на число нод тира в сети).
const REWARD: Record<TierKey, number> = { cumulus: 0.9, nimbus: 3.15, stratus: 8.1 }
// Залог (collateral) ноды в FLUX — база для APY и стоимости флота.
export const COLLATERAL: Record<TierKey, number> = {
  cumulus: 1000,
  nimbus: 12500,
  stratus: 40000,
}

const BLOCK_SECS = 30
const SECS_PER_DAY = 86_400
const DAYS_PER_MONTH = 30
const DAYS_PER_YEAR = 365
// С блока 3_071_200: ставки снижены на 10%; после закрытия Ergo активны 9 PA.
const PA_MULTIPLIER = 1.9

const TIERS: TierKey[] = ['cumulus', 'nimbus', 'stratus']

/** Дневной доход во FLUX для count нод тира при tierNetworkCount нод этого тира в сети. */
function dailyFluxForTier(tier: TierKey, count: number, tierNetworkCount: number): number {
  if (count <= 0 || tierNetworkCount <= 0) return 0
  const blocksPerDay = SECS_PER_DAY / BLOCK_SECS
  return count * (REWARD[tier] / tierNetworkCount) * blocksPerDay * PA_MULTIPLIER
}

export interface CalcResult {
  available: boolean
  collateralFlux: number
  collateralUsd: number
  dailyFlux: number
  monthlyFlux: number
  yearlyFlux: number
  dailyUsd: number
  monthlyUsd: number
  yearlyUsd: number
  apyPercent: number
}

/**
 * Считает стоимость флота и доходность (день/месяц/год во FLUX+USD) с учётом PA.
 * `counts` — число нод по тирам; `network` — число нод сети по тирам (делитель выплат);
 * `priceUsd` — цена FLUX (null → USD = 0).
 */
export function calcEarnings(
  counts: TierCounts,
  network: TierCounts | null,
  priceUsd: number | null,
): CalcResult {
  const available = network !== null && priceUsd !== null && Number.isFinite(priceUsd)
    && TIERS.every((tier) => counts[tier] === 0 || network[tier] > 0)
  let collateralFlux = 0
  let dailyFlux = 0
  for (const tier of TIERS) {
    collateralFlux += counts[tier] * COLLATERAL[tier]
    if (available && network) dailyFlux += dailyFluxForTier(tier, counts[tier], network[tier])
  }
  const price = priceUsd ?? 0
  const monthlyFlux = dailyFlux * DAYS_PER_MONTH
  const yearlyFlux = dailyFlux * DAYS_PER_YEAR
  return {
    available,
    collateralFlux,
    collateralUsd: collateralFlux * price,
    dailyFlux,
    monthlyFlux,
    yearlyFlux,
    dailyUsd: dailyFlux * price,
    monthlyUsd: monthlyFlux * price,
    yearlyUsd: yearlyFlux * price,
    apyPercent: collateralFlux > 0 ? (yearlyFlux / collateralFlux) * 100 : 0,
  }
}
