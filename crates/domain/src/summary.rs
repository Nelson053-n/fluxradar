//! Агрегация флота нод в сводку для дашборда (§4.1 ТЗ).

use serde::Serialize;

use crate::earnings::{
    active_parallel_assets, fleet_apy_percent, fleet_earnings, fleet_total_mined, node_period_flux,
    parallel_assets_share, satoshi_to_flux, Money, TierCounts,
};
use crate::Tier;

/// Разбивка количества нод по tier.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TierBreakdown {
    pub cumulus: u32,
    pub nimbus: u32,
    pub stratus: u32,
}

impl TierBreakdown {
    pub fn total(&self) -> u32 {
        self.cumulus + self.nimbus + self.stratus
    }

    fn add(&mut self, tier: Tier) {
        match tier {
            Tier::Cumulus => self.cumulus += 1,
            Tier::Nimbus => self.nimbus += 1,
            Tier::Stratus => self.stratus += 1,
        }
    }
}

/// Оценка дохода флота (est., см. earnings.rs) — за день/месяц/год в FLUX+USD,
/// плюс APY от collateral и разбивка boost от Parallel Assets.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct EarningsEstimate {
    pub daily: Money,
    pub monthly: Money,
    pub yearly: Money,
    /// Годовой APY в % от вложенного collateral нод.
    pub apy_percent: f64,
    /// Доля «боковой» доходности (Parallel Assets) в совокупном доходе, %.
    pub parallel_assets_pct: f64,
    /// Число активных Parallel Assets-чейнов (для разбивки на фронте).
    pub parallel_assets_count: u32,
}

/// Совокупные итоги дохода флота. Реальные данные из Parallel Assets fusion API,
/// если доступны (`is_estimate=false`); иначе оценка по возрасту нод (`is_estimate=true`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct EarningsTotals {
    /// Всего добыто (Total Mined). Реально = maxClaimableTotal из fusion.
    pub mined: Money,
    /// Получено к настоящему (Total Claimed). Реально = claimedTotal из fusion.
    pub claimed: Money,
    /// Доступно к получению (Total Claimable) = mined − claimed.
    pub claimable: Money,
    /// true — оценка (нет данных fusion); false — реальные данные Parallel Assets.
    pub is_estimate: bool,
}

/// Разбивка Parallel Assets по чейнам (реальные данные fusion).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PaChain {
    pub chain: String,
    pub claimable: f64,
    pub claimed: f64,
    pub received: f64,
    /// Уплаченная комиссия сети чейна.
    pub fees: f64,
}

/// Реальная сводка Parallel Assets (если получена из fusion API).
#[derive(Debug, Clone, PartialEq)]
pub struct RealPaTotals {
    pub mined_flux: f64,
    pub claimed_flux: f64,
    pub claimable_flux: f64,
    pub chains: Vec<PaChain>,
}

/// Сводка по сети для контекста дашборда (`getfluxnodecount`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct NetworkSummary {
    pub total: u32,
    pub cumulus: u32,
    pub nimbus: u32,
    pub stratus: u32,
}

/// Сводка по кошельку — ответ `/api/v1/wallet/:address/summary` (§8).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WalletSummary {
    pub total_nodes: u32,
    pub tiers: TierBreakdown,
    pub balance: Money,
    /// Оценка дохода (est.) — прогноз по механике наград, не факт выплат.
    pub earnings: EarningsEstimate,
    /// Совокупные итоги (mined/claimed/claimable).
    pub totals: EarningsTotals,
    /// Разбивка Parallel Assets по чейнам (реальные данные fusion; пусто если нет).
    pub pa_chains: Vec<PaChain>,
    /// Сводка сети (для контекста, доля флота).
    pub network: NetworkSummary,
    /// Возраст старейшей ноды флота в секундах (для «Best Uptime»-инсайта).
    pub oldest_node_age_secs: i64,
    /// Доля нод флота «в норме» (CONFIRMED) — честная метрика здоровья флота, %.
    pub fleet_uptime_pct: f64,
    /// Изменение цены FLUX за 24ч, % (для бейджа).
    pub price_change_24h: f64,
    /// Сколько нод флота прошли бенчмарк (из общего числа).
    pub bench_passed: u32,
    /// Всего приложений, хостится на нодах флота.
    pub hosted_apps: u32,
}

/// Параметры построения сводки, не относящиеся к тирам флота.
pub struct SummaryInputs {
    pub balance_satoshi: u64,
    pub flux_price_usd: f64,
    pub price_change_24h: f64,
    pub network: NetworkSummary,
    pub oldest_node_age_secs: i64,
    /// Число нод флота в статусе CONFIRMED (для fleet_uptime_pct).
    pub confirmed_nodes: u32,
    /// Per-node данные для оценки итогов (fallback): (тир, возраст_сек, сек_с_выплаты).
    pub nodes_age_paid: Vec<(Tier, i64, i64)>,
    /// Реальные Parallel Assets totals из fusion API (приоритетнее оценки).
    pub real_pa: Option<RealPaTotals>,
    /// Сколько нод флота прошли бенчмарк (агрегат по IP из fluxinfo).
    pub bench_passed: u32,
    /// Всего приложений на нодах флота (агрегат по IP из fluxinfo).
    pub hosted_apps: u32,
}

/// Построить сводку из тиров нод владельца и доп. входных данных.
pub fn build(node_tiers: &[Tier], inputs: &SummaryInputs) -> WalletSummary {
    let mut tiers = TierBreakdown::default();
    for &t in node_tiers {
        tiers.add(t);
    }
    let total = tiers.total();

    let net = TierCounts {
        cumulus: inputs.network.cumulus,
        nimbus: inputs.network.nimbus,
        stratus: inputs.network.stratus,
    };
    let price = inputs.flux_price_usd;

    let e = fleet_earnings(node_tiers, net);
    let apy = fleet_apy_percent(node_tiers, net);

    // Totals: приоритет — реальные данные Parallel Assets (fusion API).
    // Fallback — оценка по возрасту нод, если fusion недоступен.
    let (mined_flux, claimed_flux, claimable_flux, totals_is_estimate, pa_chains) =
        match &inputs.real_pa {
            Some(pa) => (
                pa.mined_flux,
                pa.claimed_flux,
                pa.claimable_flux,
                false,
                pa.chains.clone(),
            ),
            None => {
                let mined_nodes: Vec<(Tier, i64)> = inputs
                    .nodes_age_paid
                    .iter()
                    .map(|(t, age, _)| (*t, *age))
                    .collect();
                let mined = fleet_total_mined(&mined_nodes, net);
                let claimable: f64 = inputs
                    .nodes_age_paid
                    .iter()
                    .map(|(t, _, since_paid)| {
                        let days = (*since_paid as f64 / 86_400.0).max(0.0);
                        node_period_flux(*t, net.count_for_pub(*t), days)
                    })
                    .sum();
                let claimed = (mined - claimable).max(0.0);
                (mined, claimed, claimable, true, Vec::new())
            }
        };

    let fleet_uptime_pct = if total == 0 {
        0.0
    } else {
        inputs.confirmed_nodes as f64 / total as f64 * 100.0
    };

    WalletSummary {
        total_nodes: total,
        tiers,
        balance: Money::from_flux(satoshi_to_flux(inputs.balance_satoshi), price),
        earnings: EarningsEstimate {
            daily: Money::from_flux(e.daily, price),
            monthly: Money::from_flux(e.monthly, price),
            yearly: Money::from_flux(e.yearly, price),
            apy_percent: apy,
            parallel_assets_pct: parallel_assets_share() * 100.0,
            parallel_assets_count: active_parallel_assets(),
        },
        totals: EarningsTotals {
            mined: Money::from_flux(mined_flux, price),
            claimed: Money::from_flux(claimed_flux, price),
            claimable: Money::from_flux(claimable_flux, price),
            is_estimate: totals_is_estimate,
        },
        pa_chains,
        network: inputs.network,
        oldest_node_age_secs: inputs.oldest_node_age_secs,
        fleet_uptime_pct,
        price_change_24h: inputs.price_change_24h,
        bench_passed: inputs.bench_passed,
        hosted_apps: inputs.hosted_apps,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs(balance: u64, price: f64) -> SummaryInputs {
        SummaryInputs {
            balance_satoshi: balance,
            flux_price_usd: price,
            price_change_24h: 0.0,
            network: NetworkSummary {
                total: 7089,
                cumulus: 3795,
                nimbus: 1708,
                stratus: 1586,
            },
            oldest_node_age_secs: 0,
            confirmed_nodes: 0,
            nodes_age_paid: Vec::new(),
            real_pa: None,
            bench_passed: 0,
            hosted_apps: 0,
        }
    }

    #[test]
    fn aggregates_tiers_and_balance() {
        let nodes = [Tier::Cumulus, Tier::Cumulus, Tier::Nimbus, Tier::Stratus];
        let s = build(&nodes, &inputs(107_299_992_160, 0.0751));
        assert_eq!(s.total_nodes, 4);
        assert_eq!(s.tiers.cumulus, 2);
        assert_eq!(s.tiers.nimbus, 1);
        assert_eq!(s.tiers.stratus, 1);
        assert!((s.balance.flux - 1_072.999_921_6).abs() < 1e-9);
    }

    #[test]
    fn empty_fleet() {
        let s = build(&[], &inputs(0, 0.0751));
        assert_eq!(s.total_nodes, 0);
        assert_eq!(s.balance.usd, 0.0);
        assert_eq!(s.fleet_uptime_pct, 0.0);
    }

    #[test]
    fn earnings_and_uptime_filled() {
        let nodes = [Tier::Cumulus, Tier::Stratus];
        let mut inp = inputs(0, 0.10);
        inp.confirmed_nodes = 2; // обе ноды CONFIRMED
        let s = build(&nodes, &inp);
        // earnings monthly > 0 и USD = flux * price
        assert!(s.earnings.monthly.flux > 0.0);
        assert!((s.earnings.monthly.usd - s.earnings.monthly.flux * 0.10).abs() < 1e-9);
        assert_eq!(s.fleet_uptime_pct, 100.0);
    }

    #[test]
    fn earnings_periods_and_apy_and_pa() {
        let nodes = [Tier::Cumulus];
        let s = build(&nodes, &inputs(0, 0.10));
        // day < month < year
        assert!(s.earnings.daily.flux < s.earnings.monthly.flux);
        assert!(s.earnings.monthly.flux < s.earnings.yearly.flux);
        // APY > 0, PA = 50% при 10 активных PA
        assert!(s.earnings.apy_percent > 0.0);
        assert!((s.earnings.parallel_assets_pct - 50.0).abs() < 1e-6);
        assert_eq!(s.earnings.parallel_assets_count, 10);
    }

    #[test]
    fn totals_mined_claimed_claimable() {
        // 1 Cumulus, возраст 100 дней, последняя выплата 1 день назад.
        let mut inp = inputs(0, 0.10);
        inp.nodes_age_paid = vec![(Tier::Cumulus, 100 * 86_400, 86_400)];
        let s = build(&[Tier::Cumulus], &inp);
        assert!(s.totals.mined.flux > 0.0);
        assert!(s.totals.claimable.flux > 0.0);
        // claimed = mined − claimable
        assert!(
            (s.totals.claimed.flux - (s.totals.mined.flux - s.totals.claimable.flux)).abs() < 1e-6
        );
        // claimable за 1 день << mined за 100 дней
        assert!(s.totals.claimable.flux < s.totals.mined.flux);
        // нет real_pa → это оценка
        assert!(s.totals.is_estimate);
        assert!(s.pa_chains.is_empty());
    }

    #[test]
    fn totals_use_real_pa_when_present() {
        // Реальные данные fusion имеют приоритет над оценкой.
        let mut inp = inputs(0, 0.10);
        inp.nodes_age_paid = vec![(Tier::Nimbus, 100 * 86_400, 86_400)];
        inp.real_pa = Some(RealPaTotals {
            mined_flux: 8942.0,
            claimed_flux: 6752.0,
            claimable_flux: 2190.0,
            chains: vec![PaChain {
                chain: "kda".into(),
                claimable: 219.0,
                claimed: 675.0,
                received: 645.0,
                fees: 30.0,
            }],
        });
        let s = build(&[Tier::Nimbus], &inp);
        assert_eq!(s.totals.mined.flux, 8942.0);
        assert_eq!(s.totals.claimed.flux, 6752.0);
        assert_eq!(s.totals.claimable.flux, 2190.0);
        assert!(!s.totals.is_estimate); // реальные данные
        assert_eq!(s.pa_chains.len(), 1);
        assert_eq!(s.pa_chains[0].chain, "kda");
    }
}
