//! Оценки дохода и денежные конвертации (§4.1 ТЗ).
//!
//! Реализуется с нуля по публичным параметрам сети Flux (idea–expression, см.
//! `docs/LICENSE_NOTES.md`). Точную формулу выплат API не отдаёт — она выводится
//! из наблюдаемой механики наград и фиксируется golden-тестами (§13.10), чтобы
//! не сломать молча при изменении констант сети.
//!
//! Модель (PoUW v2): на каждый блок (~30с) фиксированная награда тиру делится
//! поровну между активными нодами этого тира. Месячный доход одной ноды:
//!   reward_тира / число_активных_нод_тира × блоков_в_месяц.
//! Текущие ставки: с блока 3_071_200 (26.10.2026), 9 активных PA.
//! Источники: https://runonflux.com/first-annual-block-reward-reduction
//! https://docs.runonflux.com/fluxnodes/claim-parallel-assets/
//! Все оценки помечаются на фронте как «est.» — это прогноз, не факт выплат.

use crate::Tier;

/// Денежная величина в FLUX с USD-эквивалентом.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct Money {
    pub flux: f64,
    pub usd: f64,
}

impl Money {
    /// Собрать из количества FLUX и текущей цены в USD.
    pub fn from_flux(flux: f64, price_usd: f64) -> Self {
        Money {
            flux,
            usd: flux * price_usd,
        }
    }
}

/// Перевести баланс из сатоши Flux (1 FLUX = 1e8 сатоши) в FLUX.
///
/// Flux API `/explorer/balance` отдаёт целое число сатоши (факт, проверено).
pub fn satoshi_to_flux(satoshi: u64) -> f64 {
    satoshi as f64 / 1e8
}

// --- Параметры сети (PoUW v2). Вынесены в константы: при изменении сети
// правится здесь, golden-тесты ниже ловят расхождение. ---

/// Награда за блок одной ноде тира до деления на число нод (FLUX/блок на тир).
/// С блока 3_071_200: 12.6 FLUX/блок, из них 0.9/3.15/8.1 — нодам.
fn tier_block_reward(tier: Tier) -> f64 {
    match tier {
        Tier::Cumulus => 0.9,
        Tier::Nimbus => 3.15,
        Tier::Stratus => 8.1,
    }
}

/// Залог (collateral) ноды тира в FLUX — база для APY (факт сети Flux).
/// Cumulus 1000 / Nimbus 12500 / Stratus 40000.
pub fn tier_collateral(tier: Tier) -> f64 {
    match tier {
        Tier::Cumulus => 1000.0,
        Tier::Nimbus => 12500.0,
        Tier::Stratus => 40000.0,
    }
}

/// Время блока в секундах (PoUW v2).
const BLOCK_SECS: f64 = 30.0;
/// Секунд в сутках / месяце (30 дней) / году (365 дней).
const SECS_PER_DAY: f64 = 86_400.0;
const DAYS_PER_MONTH: f64 = 30.0;
const DAYS_PER_YEAR: f64 = 365.0;

// --- Parallel Assets (PA) — «боковая» доходность Flux. ---
// Каждый активный PA даёт 10% native-награды, до комиссий claim.
// Ergo больше не активен после 21.09.2026; старые остатки не входят в новый прогноз.
const PA_REWARD_SHARE: f64 = 0.10;
const ACTIVE_PARALLEL_ASSETS: u32 = 9;

/// Native + начисленные PA; фактический claim и комиссии здесь не учитываются.
fn total_reward_multiplier() -> f64 {
    1.0 + ACTIVE_PARALLEL_ASSETS as f64 * PA_REWARD_SHARE
}

/// Блоков за период из числа дней.
fn blocks_in(days: f64) -> f64 {
    days * SECS_PER_DAY / BLOCK_SECS
}

/// Оценка дохода одной ноды тира за период (в днях), в FLUX-эквиваленте
/// (включая Parallel Assets — совокупная «боковая» доходность).
///
/// `tier_node_count` — число активных нод этого тира в сети (делитель награды).
/// При нулевом счётчике возвращает 0 (нет данных — нет оценки, не делим на ноль).
pub fn node_period_flux(tier: Tier, tier_node_count: u32, days: f64) -> f64 {
    if tier_node_count == 0 {
        return 0.0;
    }
    tier_block_reward(tier) / tier_node_count as f64 * blocks_in(days) * total_reward_multiplier()
}

/// Счётчики активных нод сети по тирам (из `getfluxnodecount`).
#[derive(Debug, Clone, Copy, Default)]
pub struct TierCounts {
    pub cumulus: u32,
    pub nimbus: u32,
    pub stratus: u32,
}

impl TierCounts {
    fn count_for(&self, tier: Tier) -> u32 {
        match tier {
            Tier::Cumulus => self.cumulus,
            Tier::Nimbus => self.nimbus,
            Tier::Stratus => self.stratus,
        }
    }

    /// Публичный доступ к счётчику тира (для расчётов в других модулях).
    pub fn count_for_pub(&self, tier: Tier) -> u32 {
        self.count_for(tier)
    }
}

/// Оценка дохода флота за день/месяц/год (FLUX-эквивалент, включая PA).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FleetEarnings {
    pub daily: f64,
    pub monthly: f64,
    pub yearly: f64,
}

/// Оценка дохода флота нод владельца за день/месяц/год, FLUX (с учётом PA).
///
/// `owner_tiers` — тиры нод владельца; `network` — счётчики нод сети по тирам.
pub fn fleet_earnings(owner_tiers: &[Tier], network: TierCounts) -> FleetEarnings {
    let mut e = FleetEarnings {
        daily: 0.0,
        monthly: 0.0,
        yearly: 0.0,
    };
    for &t in owner_tiers {
        let n = network.count_for(t);
        e.daily += node_period_flux(t, n, 1.0);
        e.monthly += node_period_flux(t, n, DAYS_PER_MONTH);
        e.yearly += node_period_flux(t, n, DAYS_PER_YEAR);
    }
    e
}

/// Совместимость: месячный и годовой доход (используется существующим кодом).
pub fn fleet_estimate_flux(owner_tiers: &[Tier], network: TierCounts) -> (f64, f64) {
    let e = fleet_earnings(owner_tiers, network);
    (e.monthly, e.yearly)
}

/// Число активных Parallel Assets (публичный геттер для UI-разбивки).
pub fn active_parallel_assets() -> u32 {
    ACTIVE_PARALLEL_ASSETS
}

/// Доля «боковой» (Parallel Assets) части в совокупном доходе, [0..1].
/// PA / (1 + PA): при 9 активных сетях это 0.9/1.9.
pub fn parallel_assets_share() -> f64 {
    let pa = ACTIVE_PARALLEL_ASSETS as f64 * PA_REWARD_SHARE;
    pa / (1.0 + pa)
}

/// Накопленный (lifetime) доход флота по возрасту нод, FLUX (оценка «Total mined»).
///
/// Для каждой ноды: дневной доход тира × возраст в днях. Это оценка всего, что нода
/// «намайнила» с момента активации (помечается est.; реальная история — в explorer).
pub fn fleet_total_mined(nodes: &[(Tier, i64)], network: TierCounts) -> f64 {
    nodes
        .iter()
        .map(|(tier, age_secs)| {
            let n = network.count_for(*tier);
            let age_days = (*age_secs as f64 / SECS_PER_DAY).max(0.0);
            node_period_flux(*tier, n, age_days)
        })
        .sum()
}

/// Годовой APY флота в % от вложенного collateral (как в оригинале Fluxnode):
/// сумма годового дохода (FLUX, с PA) / сумма collateral нод × 100.
/// База — залог нод, а не баланс кошелька: это доходность вложения в ноды.
pub fn fleet_apy_percent(owner_tiers: &[Tier], network: TierCounts) -> f64 {
    let yearly = fleet_earnings(owner_tiers, network).yearly;
    let collateral: f64 = owner_tiers.iter().map(|&t| tier_collateral(t)).sum();
    if collateral <= 0.0 {
        0.0
    } else {
        yearly / collateral * 100.0
    }
}

/// Оценка времени до выплаты ноды по её рангу (позиция в очереди round-robin).
///
/// `rank` — позиция в очереди ВНУТРИ тира (нумеруется с 0 для каждого тира).
/// Механика подтверждена по `last_paid_height`: каждый тир платит ровно одну ноду
/// за блок (~1.00 блока/ноду на всех тирах), блок 30с. Значит время до выплаты =
/// rank × 30с. При rank<=0 → 0 (выплата вот-вот).
const PAYOUT_SECS_PER_RANK: f64 = BLOCK_SECS;

pub fn payout_eta_secs(rank: i64) -> i64 {
    if rank <= 0 {
        0
    } else {
        (rank as f64 * PAYOUT_SECS_PER_RANK) as i64
    }
}

/// Период переподтверждения ноды в блоках (факт сети Flux: 480 блоков ≈ 240 мин).
const MAINTENANCE_BLOCK_RATE: i64 = 480;

/// Окно обслуживания ноды в секундах: сколько осталось до необходимости
/// переподтверждения (иначе нода уйдёт в EXPIRED). Формула сети Flux:
/// `480 − (current_height − last_confirmed_height)` блоков × 30с.
/// Возвращает None, если окно закрыто (win<=0) или нет данных высоты.
pub fn maintenance_window_secs(last_confirmed_height: i64, current_height: i64) -> Option<i64> {
    if last_confirmed_height <= 0 || current_height <= 0 {
        return None;
    }
    let win_blocks = MAINTENANCE_BLOCK_RATE - (current_height - last_confirmed_height);
    if win_blocks <= 0 {
        None // окно закрыто — нужно переподтвердить
    } else {
        Some((win_blocks as f64 * BLOCK_SECS) as i64)
    }
}

/// Возраст ноды в секундах из `activesince` (unix-timestamp) и текущего времени.
/// Если `activesince` в будущем или 0 — возвращает 0.
pub fn node_age_secs(activesince: i64, now: i64) -> i64 {
    if activesince <= 0 || now <= activesince {
        0
    } else {
        now - activesince
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn satoshi_conversion_matches_flux_api() {
        // 107299992160 сатоши из реального ответа /explorer/balance = 1072.9999216 FLUX.
        assert!((satoshi_to_flux(107_299_992_160) - 1_072.999_921_6).abs() < 1e-9);
    }

    #[test]
    fn money_usd_equiv() {
        let m = Money::from_flux(1000.0, 0.0751);
        assert!((m.usd - 75.1).abs() < 1e-9);
    }

    // --- Golden-тесты earnings (§13.10): фиксируют значения формулы.
    // Блоков в месяц = 30*86400/30 = 86400. ---

    #[test]
    fn golden_blocks_in_month_and_year() {
        assert!((blocks_in(DAYS_PER_MONTH) - 86_400.0).abs() < 1e-6);
        assert!((blocks_in(DAYS_PER_YEAR) - 1_051_200.0).abs() < 1e-6);
    }

    #[test]
    fn golden_pa_multiplier_nine_active_chains() {
        // 9 активных PA × 10% = +90% к native.
        assert!((total_reward_multiplier() - 1.9).abs() < 1e-9);
    }

    #[test]
    fn golden_cumulus_node_monthly() {
        // Текущая ставка 0.9 FLUX, 9 PA, период 30 дней.
        let flux = node_period_flux(Tier::Cumulus, 3795, DAYS_PER_MONTH);
        assert!((flux - 38.931_225_296).abs() < 1e-5, "got {flux}");
    }

    #[test]
    fn golden_stratus_node_monthly() {
        // Текущая ставка 8.1 FLUX, 9 PA, период 30 дней.
        let flux = node_period_flux(Tier::Stratus, 1586, DAYS_PER_MONTH);
        assert!((flux - 838.395_964_691).abs() < 1e-4, "got {flux}");
    }

    #[test]
    fn golden_fleet_estimate() {
        // Флот 2 Cumulus + 1 Stratus, текущие ставки и 9 PA.
        let network = TierCounts {
            cumulus: 3795,
            nimbus: 1708,
            stratus: 1586,
        };
        let (monthly, yearly) =
            fleet_estimate_flux(&[Tier::Cumulus, Tier::Cumulus, Tier::Stratus], network);
        assert!(
            (monthly - 916.258_415_283).abs() < 1e-3,
            "monthly {monthly}"
        );
        // Годовой = месячный * (365/30).
        assert!((yearly - monthly * (DAYS_PER_YEAR / DAYS_PER_MONTH)).abs() < 1e-3);
    }

    #[test]
    fn zero_tier_count_yields_zero() {
        assert_eq!(node_period_flux(Tier::Nimbus, 0, DAYS_PER_MONTH), 0.0);
    }

    #[test]
    fn node_age_basic() {
        assert_eq!(node_age_secs(1000, 5000), 4000);
        assert_eq!(node_age_secs(0, 5000), 0); // нет activesince
        assert_eq!(node_age_secs(9000, 5000), 0); // в будущем → 0
    }

    #[test]
    fn payout_eta_by_rank() {
        // rank_в_тире × 30с (1 нода тира за блок, подтверждено last_paid_height).
        // rank 100 → 3000с (50 мин), rank 474 → 14220с (≈3ч 57мин).
        assert_eq!(payout_eta_secs(100), 3000);
        assert_eq!(payout_eta_secs(474), 14220);
        assert_eq!(payout_eta_secs(0), 0);
        assert_eq!(payout_eta_secs(-5), 0);
    }

    #[test]
    fn golden_daily_period_and_fleet() {
        let network = TierCounts {
            cumulus: 3795,
            nimbus: 1708,
            stratus: 1586,
        };
        // 0.9 FLUX/блок, 3795 нод, 2880 блоков/день, PA +90%.
        let d = node_period_flux(Tier::Cumulus, 3795, 1.0);
        assert!((d - 1.297_707_51).abs() < 1e-5, "daily {d}");
        // fleet_earnings: monthly = 30 * daily, yearly = 365 * daily.
        let e = fleet_earnings(&[Tier::Cumulus], network);
        assert!((e.monthly - e.daily * 30.0).abs() < 1e-6);
        assert!((e.yearly - e.daily * 365.0).abs() < 1e-6);
    }

    #[test]
    fn golden_pa_share_nine_chains() {
        // 9 PA × 10% = +90%; доля PA в совокупном доходе — 0.9/1.9.
        assert!((parallel_assets_share() - 0.9 / 1.9).abs() < 1e-9);
        assert_eq!(active_parallel_assets(), 9);
    }

    #[test]
    fn golden_total_mined_by_age() {
        // 1 Cumulus, возраст 365 дней → mined ≈ годовой доход Cumulus.
        let network = TierCounts {
            cumulus: 3795,
            nimbus: 0,
            stratus: 0,
        };
        let mined = fleet_total_mined(&[(Tier::Cumulus, 365 * 86_400)], network);
        let yearly = fleet_earnings(&[Tier::Cumulus], network).yearly;
        assert!(
            (mined - yearly).abs() < 1e-3,
            "mined {mined} vs yearly {yearly}"
        );
    }

    #[test]
    fn golden_apy_from_collateral() {
        // APY = годовой доход(FLUX, с PA) / collateral × 100. Cumulus collateral=1000.
        let network = TierCounts {
            cumulus: 3795,
            nimbus: 1708,
            stratus: 1586,
        };
        let apy = fleet_apy_percent(&[Tier::Cumulus], network);
        let yearly = fleet_earnings(&[Tier::Cumulus], network).yearly;
        assert!((apy - yearly / 1000.0 * 100.0).abs() < 1e-6, "apy {apy}");
        assert_eq!(tier_collateral(Tier::Stratus), 40000.0);
    }
}
