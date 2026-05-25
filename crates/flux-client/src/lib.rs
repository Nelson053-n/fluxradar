//! Типизированный клиент Flux API (api.runonflux.io) + цена FLUX (CoinGecko).
//!
//! Реализован с нуля по публичной документации и фактическим ответам API:
//! - https://docs.runonflux.io/fluxapi
//! - OpenAPI: https://docs.runonflux.io/_bundle/fluxapi.json
//!
//! Покрывает минимум для MVP (§11 Этап 1).

use std::time::Duration;

use serde::Deserialize;

const DEFAULT_BASE_URL: &str = "https://api.runonflux.io";
// CoinGecko: FLUX торгуется под legacy-id `zelcash` (факт, проверено).
// include_24hr_change — для бейджа изменения цены на дашборде.
const COINGECKO_PRICE_URL: &str = "https://api.coingecko.com/api/v3/simple/price?ids=zelcash&vs_currencies=usd&include_24hr_change=true";

#[derive(Debug, thiserror::Error)]
pub enum FluxError {
    #[error("HTTP-запрос не удался: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Flux API вернул статус ошибки: {0}")]
    Status(reqwest::StatusCode),
    #[error("Flux API вернул status != success: {0}")]
    ApiStatus(String),
}

/// Стандартная обёртка ответа Flux API: `{ "status": "...", "data": ... }`.
#[derive(Debug, Deserialize)]
struct FluxEnvelope<T> {
    status: String,
    data: T,
}

impl<T> FluxEnvelope<T> {
    fn into_data(self) -> Result<T, FluxError> {
        if self.status == "success" {
            Ok(self.data)
        } else {
            Err(FluxError::ApiStatus(self.status))
        }
    }
}

/// Одна нода из `viewdeterministicfluxnodelist`.
///
/// Все поля приходят одним сетевым запросом на весь список (§5.4) — никаких
/// доп. обращений на ноду. `activesince`/`lastpaid` API отдаёт строками.
#[derive(Debug, Clone, Deserialize)]
pub struct DeterministicNode {
    pub ip: String,
    pub tier: String,
    pub payment_address: String,
    pub rank: i64,
    #[serde(default)]
    pub activesince: String,
    #[serde(default)]
    pub lastpaid: String,
    #[serde(default)]
    pub last_paid_height: i64,
    /// Высота последнего переподтверждения ноды (для окна обслуживания).
    #[serde(default)]
    pub last_confirmed_height: i64,
    #[serde(default)]
    pub added_height: i64,
    #[serde(default)]
    pub amount: String,
}

/// Счётчики активных нод сети по тирам (`getfluxnodecount`).
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct NetworkCount {
    pub total: u32,
    #[serde(rename = "cumulus-enabled")]
    pub cumulus: u32,
    #[serde(rename = "nimbus-enabled")]
    pub nimbus: u32,
    #[serde(rename = "stratus-enabled")]
    pub stratus: u32,
}

/// Часть ответа `daemon/getinfo` — текущая высота блока.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct GetInfo {
    #[serde(default)]
    pub blocks: i64,
}

/// Цена FLUX с изменением за 24ч (CoinGecko).
#[derive(Debug, Clone, Copy)]
pub struct PriceInfo {
    pub usd: f64,
    pub change_24h: f64,
}

/// Статус конкретной ноды (`getfluxnodestatus?ip=`).
#[derive(Debug, Clone, Deserialize)]
pub struct NodeStatusInfo {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub tier: String,
    #[serde(default)]
    pub activesince: String,
    #[serde(default)]
    pub lastpaid: String,
}

/// Статистика по одному Parallel Asset-чейну (fusion.runonflux.io).
#[derive(Debug, Clone, Deserialize)]
pub struct PaChainStat {
    pub chain: String,
    #[serde(rename = "possibleToClaim", default)]
    pub possible_to_claim: f64,
    #[serde(rename = "claimedAmount", default)]
    pub claimed_amount: f64,
    #[serde(rename = "receivedAmount", default)]
    pub received_amount: f64,
    /// Комиссия сети чейна, уплаченная при выводе (различается по чейнам).
    #[serde(rename = "feesPaid", default)]
    pub fees_paid: f64,
}

/// Сводка Parallel Assets кошелька (fusion `coinbase/summary`).
/// Реальные данные mined/claimed по всем PA-чейнам — источник истины fluxnode.app.
#[derive(Debug, Clone, Deserialize)]
pub struct PaSummary {
    /// Всего добыто (намайнено) по всем PA — это «Total Mined».
    #[serde(rename = "maxClaimableTotal", default)]
    pub max_claimable_total: f64,
    /// Всего уже получено (claimed) — это «Total Claimed».
    #[serde(rename = "claimedTotal", default)]
    pub claimed_total: f64,
    #[serde(rename = "chainStatistics", default)]
    pub chain_statistics: Vec<PaChainStat>,
}

impl PaSummary {
    /// Доступно к получению = всего добыто − уже получено.
    pub fn claimable(&self) -> f64 {
        (self.max_claimable_total - self.claimed_total).max(0.0)
    }
}

/// Бенчмарк/железо и число приложений ноды (stats.runonflux.io/fluxinfo).
/// Один batch-запрос на всю сеть отдаёт это для ~7000 нод (§5.4 — не N запросов).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct NodeStats {
    /// Бенчмарк пройден: error пуст и статус валиден.
    pub bench_passed: bool,
    pub cores: f64,
    pub ram: f64,
    pub ssd: f64,
    /// EPS (events per second) — производительность CPU.
    pub eps: f64,
    pub ping: f64,
    pub flux_os_version: String,
    /// Число запущенных приложений на ноде.
    pub apps_count: u32,
    /// Геолокация ноды (репортит сам Flux-бенчмарк, projection=geolocation).
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub country_code: String,
    /// Регион/область (regionName) — у источника нет города, показываем регион.
    #[serde(default)]
    pub region: String,
}

// --- Внутренние формы ответа fluxinfo (один комбинированный projection). ---
#[derive(Debug, Deserialize)]
struct FluxInfoRec {
    #[serde(default)]
    ip: String,
    benchmark: Option<BenchWrap>,
    apps: Option<AppsWrap>,
    geolocation: Option<GeoRec>,
}
#[derive(Debug, Default, Deserialize)]
struct GeoRec {
    #[serde(default)]
    country: String,
    #[serde(rename = "countryCode", default)]
    country_code: String,
    #[serde(rename = "regionName", default)]
    region_name: String,
}
#[derive(Debug, Deserialize)]
struct BenchWrap {
    info: Option<BenchInfo>,
    bench: Option<BenchData>,
}
#[derive(Debug, Deserialize)]
struct BenchInfo {
    #[serde(default)]
    version: String,
}
#[derive(Debug, Deserialize)]
struct BenchData {
    #[serde(default)]
    status: String,
    #[serde(default)]
    error: String,
    #[serde(default)]
    cores: f64,
    #[serde(default)]
    ram: f64,
    #[serde(default)]
    ssd: f64,
    #[serde(default)]
    eps: f64,
    #[serde(default)]
    ping: f64,
}
#[derive(Debug, Deserialize)]
struct AppsWrap {
    #[serde(default)]
    runningapps: Vec<serde_json::Value>,
}

/// Нормализовать IP к host без порта (для сопоставления нод между источниками).
pub fn ip_host(ip: &str) -> &str {
    ip.split(':').next().unwrap_or(ip)
}

/// Клиент. Дёшев для клонирования (внутри `reqwest::Client`).
#[derive(Clone)]
pub struct FluxClient {
    http: reqwest::Client,
    base_url: String,
}

impl FluxClient {
    pub fn new() -> Result<Self, FluxError> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("FluxScope/0.1")
            .build()?;
        Ok(Self {
            http,
            base_url: DEFAULT_BASE_URL.to_owned(),
        })
    }

    /// Полный детерминированный список нод сети — один запрос вместо N (§5.4 ТЗ).
    pub async fn deterministic_node_list(&self) -> Result<Vec<DeterministicNode>, FluxError> {
        let url = format!("{}/daemon/viewdeterministicfluxnodelist", self.base_url);
        let env: FluxEnvelope<Vec<DeterministicNode>> = self.get_json(&url).await?;
        env.into_data()
    }

    /// Ноды конкретного владельца — фильтр общего списка по `payment_address` (§5.4).
    pub async fn nodes_for_wallet(
        &self,
        wallet: &str,
    ) -> Result<Vec<DeterministicNode>, FluxError> {
        let all = self.deterministic_node_list().await?;
        Ok(all
            .into_iter()
            .filter(|n| n.payment_address == wallet)
            .collect())
    }

    /// Баланс адреса в сатоши Flux (`/explorer/balance`).
    pub async fn balance_satoshi(&self, address: &str) -> Result<u64, FluxError> {
        let url = format!("{}/explorer/balance?address={}", self.base_url, address);
        let env: FluxEnvelope<u64> = self.get_json(&url).await?;
        env.into_data()
    }

    /// Текущая цена FLUX в USD с изменением за 24ч (CoinGecko, §14.2).
    pub async fn flux_price(&self) -> Result<PriceInfo, FluxError> {
        let resp = self.http.get(COINGECKO_PRICE_URL).send().await?;
        if !resp.status().is_success() {
            return Err(FluxError::Status(resp.status()));
        }
        let body: CoinGeckoPrice = resp.json().await?;
        Ok(PriceInfo {
            usd: body.zelcash.usd,
            change_24h: body.zelcash.usd_24h_change.unwrap_or(0.0),
        })
    }

    /// Только цена в USD (для readiness-probe — обратная совместимость).
    pub async fn flux_price_usd(&self) -> Result<f64, FluxError> {
        Ok(self.flux_price().await?.usd)
    }

    /// История цены FLUX за год (CoinGecko market_chart, daily) — для тултипа-графика.
    /// Возвращает точки (unix_secs, usd).
    pub async fn flux_price_history(&self) -> Result<Vec<(i64, f64)>, FluxError> {
        let url = "https://api.coingecko.com/api/v3/coins/zelcash/market_chart?vs_currency=usd&days=365&interval=daily";
        let resp = self.http.get(url).send().await?;
        if !resp.status().is_success() {
            return Err(FluxError::Status(resp.status()));
        }
        let body: CoinGeckoChart = resp.json().await?;
        Ok(body
            .prices
            .into_iter()
            .filter_map(|p| match p.as_slice() {
                [ts_ms, usd] => Some(((*ts_ms / 1000.0) as i64, *usd)),
                _ => None,
            })
            .collect())
    }

    /// Сводка по сети: всего нод и счётчики по тирам (`getfluxnodecount`).
    /// Один дешёвый запрос — нужен делителем в оценке earnings.
    pub async fn network_count(&self) -> Result<NetworkCount, FluxError> {
        let url = format!("{}/daemon/getfluxnodecount", self.base_url);
        let env: FluxEnvelope<NetworkCount> = self.get_json(&url).await?;
        env.into_data()
    }

    /// Текущая высота блока сети (`daemon/getinfo` → blocks) — для окна обслуживания.
    pub async fn block_height(&self) -> Result<i64, FluxError> {
        let url = format!("{}/daemon/getinfo", self.base_url);
        let env: FluxEnvelope<GetInfo> = self.get_json(&url).await?;
        Ok(env.into_data()?.blocks)
    }

    /// Статус конкретной ноды (`getfluxnodestatus?ip=`) — для детали ноды (ленивый).
    pub async fn node_status(&self, ip: &str) -> Result<NodeStatusInfo, FluxError> {
        let url = format!("{}/daemon/getfluxnodestatus?ip={}", self.base_url, ip);
        let env: FluxEnvelope<NodeStatusInfo> = self.get_json(&url).await?;
        env.into_data()
    }

    /// Имена приложений, установленных на ноде (`apps/installedapps?ip=`) — ленивый.
    /// Возвращает имена; пустой список — норма (на ноде нет пользовательских apps).
    pub async fn node_apps(&self, ip: &str) -> Result<Vec<String>, FluxError> {
        let url = format!("{}/apps/installedapps?ip={}", self.base_url, ip);
        let env: FluxEnvelope<Vec<serde_json::Value>> = self.get_json(&url).await?;
        let apps = env.into_data()?;
        Ok(apps
            .into_iter()
            .filter_map(|a| a.get("name").and_then(|n| n.as_str()).map(|s| s.to_owned()))
            .collect())
    }

    /// Сводка Parallel Assets кошелька (fusion.runonflux.io/coinbase/summary).
    /// Реальные mined/claimed/claimable по всем PA-чейнам (не оценка).
    pub async fn wallet_pa_summary(&self, address: &str) -> Result<PaSummary, FluxError> {
        let url = format!("https://fusion.runonflux.io/coinbase/summary?address={address}");
        let resp = self.http.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(FluxError::Status(resp.status()));
        }
        // fusion оборачивает в { data: {...} }.
        let env: FluxFusionEnvelope = resp.json().await?;
        Ok(env.data)
    }

    /// Бенчмарк/железо и число приложений по всем нодам сети одним batch-запросом
    /// (stats.runonflux.io/fluxinfo). Ключ результата — host IP (без порта).
    /// Кэшируется выше по стеку (данные меняются медленно).
    pub async fn network_node_stats(
        &self,
    ) -> Result<std::collections::HashMap<String, NodeStats>, FluxError> {
        let url = "https://stats.runonflux.io/fluxinfo?projection=ip,apps.runningapps,benchmark.info.version,benchmark.bench.status,benchmark.bench.error,benchmark.bench.cores,benchmark.bench.ram,benchmark.bench.ssd,benchmark.bench.eps,benchmark.bench.ping,geolocation.country,geolocation.countryCode,geolocation.regionName";
        let resp = self.http.get(url).send().await?;
        if !resp.status().is_success() {
            return Err(FluxError::Status(resp.status()));
        }
        let env: FluxEnvelope<Vec<FluxInfoRec>> = resp.json().await?;
        let recs = env.into_data()?;
        let mut map = std::collections::HashMap::with_capacity(recs.len());
        for r in recs {
            if r.ip.is_empty() {
                continue;
            }
            let host = ip_host(&r.ip).to_owned();
            let (bench, version) = match &r.benchmark {
                Some(b) => (
                    b.bench.as_ref(),
                    b.info
                        .as_ref()
                        .map(|i| i.version.clone())
                        .unwrap_or_default(),
                ),
                None => (None, String::new()),
            };
            // Бенчмарк пройден: запись бенча есть, error пуст, статус валиден (не пуст).
            let bench_passed = bench
                .map(|b| b.error.is_empty() && !b.status.is_empty())
                .unwrap_or(false);
            let geo = r.geolocation.unwrap_or_default();
            let stats = NodeStats {
                bench_passed,
                cores: bench.map(|b| b.cores).unwrap_or(0.0),
                ram: bench.map(|b| b.ram).unwrap_or(0.0),
                ssd: bench.map(|b| b.ssd).unwrap_or(0.0),
                eps: bench.map(|b| b.eps).unwrap_or(0.0),
                ping: bench.map(|b| b.ping).unwrap_or(0.0),
                flux_os_version: version,
                apps_count: r.apps.map(|a| a.runningapps.len() as u32).unwrap_or(0),
                country: geo.country,
                country_code: geo.country_code,
                region: geo.region_name,
            };
            map.insert(host, stats);
        }
        Ok(map)
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T, FluxError> {
        let resp = self.http.get(url).send().await?;
        if !resp.status().is_success() {
            return Err(FluxError::Status(resp.status()));
        }
        Ok(resp.json().await?)
    }
}

#[derive(Debug, Deserialize)]
struct CoinGeckoPrice {
    zelcash: UsdPrice,
}

#[derive(Debug, Deserialize)]
struct UsdPrice {
    usd: f64,
    #[serde(default)]
    usd_24h_change: Option<f64>,
}

/// Ответ market_chart: prices = [[ts_ms, usd], ...].
#[derive(Debug, Deserialize)]
struct CoinGeckoChart {
    prices: Vec<Vec<f64>>,
}

/// Обёртка ответа fusion: `{ "data": {...} }` (без поля status, в отличие от Flux API).
#[derive(Debug, Deserialize)]
struct FluxFusionEnvelope {
    data: PaSummary,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_unwraps_success() {
        let env = FluxEnvelope {
            status: "success".to_owned(),
            data: 42u64,
        };
        assert_eq!(env.into_data().unwrap(), 42);
    }

    #[test]
    fn envelope_rejects_non_success() {
        let env = FluxEnvelope {
            status: "error".to_owned(),
            data: 0u64,
        };
        assert!(env.into_data().is_err());
    }

    #[test]
    fn deserializes_real_node_shape() {
        // Форма из реального ответа api.runonflux.io.
        let json = r#"{
            "ip": "82.64.11.18:16137",
            "tier": "CUMULUS",
            "payment_address": "t1Whn4HFFRYPoQqUVYNK2fLoHadBkFzM1Sh",
            "rank": 0,
            "activesince": "1771638874",
            "amount": "1000.00"
        }"#;
        let node: DeterministicNode = serde_json::from_str(json).unwrap();
        assert_eq!(node.tier, "CUMULUS");
        assert_eq!(node.payment_address, "t1Whn4HFFRYPoQqUVYNK2fLoHadBkFzM1Sh");
    }
}
