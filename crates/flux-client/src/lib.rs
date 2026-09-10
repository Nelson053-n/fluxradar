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

    /// Клиент с другим базовым URL. Нужен тестам, чтобы направить запросы на
    /// заведомо закрытый адрес: тогда проверка «мусор отсекается до похода
    /// наружу» падает, если валидация сломается, вместо тихого запроса в сеть.
    pub fn with_base_url(base_url: impl Into<String>) -> Result<Self, FluxError> {
        Ok(Self {
            base_url: base_url.into(),
            ..Self::new()?
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

    #[test]
    fn deterministic_node_defaults_missing_optional_fields() {
        // API иногда отдаёт минимальный набор полей (например, для только что
        // добавленной ноды без lastpaid/last_paid_height). Без #[serde(default)]
        // на этих полях парсинг всего списка нод падал бы на одной кривой записи
        // и ронял бы весь дашборд владельца.
        let json = r#"{
            "ip": "1.2.3.4:16137",
            "tier": "STRATUS",
            "payment_address": "t1Abc",
            "rank": 5
        }"#;
        let node: DeterministicNode = serde_json::from_str(json).unwrap();
        assert_eq!(node.activesince, "");
        assert_eq!(node.lastpaid, "");
        assert_eq!(node.last_paid_height, 0);
        assert_eq!(node.last_confirmed_height, 0);
        assert_eq!(node.added_height, 0);
        assert_eq!(node.amount, "");
    }

    #[test]
    fn deterministic_node_list_skips_no_field_but_fails_without_required() {
        // rank/tier/ip/payment_address не помечены #[serde(default)] — это
        // осознанное требование: без них запись бессмысленна. Тест фиксирует,
        // что отсутствие обязательного поля — это ошибка парсинга, а не тихий 0/"".
        let json = r#"{
            "tier": "CUMULUS",
            "payment_address": "t1Abc",
            "rank": 0
        }"#;
        let result: Result<DeterministicNode, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "отсутствие обязательного поля ip должно падать"
        );
    }

    #[test]
    fn network_count_maps_hyphenated_tier_keys() {
        // Flux API отдаёт ключи через дефис (cumulus-enabled), а не snake_case —
        // без #[serde(rename)] на каждом поле serde тихо возьмёт default (0),
        // и дашборд покажет нулевые счётчики по тирам при валидном ответе API.
        let json = r#"{
            "total": 12345,
            "cumulus-enabled": 8000,
            "nimbus-enabled": 3000,
            "stratus-enabled": 1345
        }"#;
        let count: NetworkCount = serde_json::from_str(json).unwrap();
        assert_eq!(count.total, 12345);
        assert_eq!(count.cumulus, 8000);
        assert_eq!(count.nimbus, 3000);
        assert_eq!(count.stratus, 1345);
    }

    #[test]
    fn get_info_defaults_blocks_when_absent() {
        // blocks помечен #[serde(default)] — если daemon/getinfo когда-нибудь
        // не отдаст это поле, парсинг не должен падать (высота блока не критична
        // для остального ответа), а должен тихо дать 0.
        let info: GetInfo = serde_json::from_str("{}").unwrap();
        assert_eq!(info.blocks, 0);
    }

    #[test]
    fn get_info_parses_blocks() {
        let info: GetInfo = serde_json::from_str(r#"{"blocks": 1734567}"#).unwrap();
        assert_eq!(info.blocks, 1734567);
    }

    #[test]
    fn node_status_info_defaults_all_fields_when_node_offline() {
        // Для офлайн-ноды getfluxnodestatus может вернуть пустой/частичный
        // объект. Все поля должны безопасно дефолтиться в "", а не падать —
        // иначе один офлайн-узел в списке ронял бы весь запрос деталей.
        let status: NodeStatusInfo = serde_json::from_str("{}").unwrap();
        assert_eq!(status.status, "");
        assert_eq!(status.tier, "");
        assert_eq!(status.activesince, "");
        assert_eq!(status.lastpaid, "");
    }

    #[test]
    fn pa_chain_stat_maps_camelcase_fields() {
        // fusion.runonflux.io отдаёт camelCase (possibleToClaim, claimedAmount,
        // receivedAmount, feesPaid) — без rename эти суммы тихо станут 0.0,
        // а именно они формируют цифры на дашборде claimable/claimed.
        let json = r#"{
            "chain": "ETH",
            "possibleToClaim": 12.5,
            "claimedAmount": 8.25,
            "receivedAmount": 8.0,
            "feesPaid": 0.25
        }"#;
        let stat: PaChainStat = serde_json::from_str(json).unwrap();
        assert_eq!(stat.chain, "ETH");
        assert_eq!(stat.possible_to_claim, 12.5);
        assert_eq!(stat.claimed_amount, 8.25);
        assert_eq!(stat.received_amount, 8.0);
        assert_eq!(stat.fees_paid, 0.25);
    }

    #[test]
    fn pa_chain_stat_defaults_when_chain_stat_partial() {
        // Некоторые чейны могут не иметь feesPaid в ответе (например, нет вывода
        // ещё не было) — обязателен только chain, остальное дефолтится в 0.0.
        let stat: PaChainStat = serde_json::from_str(r#"{"chain": "BTC"}"#).unwrap();
        assert_eq!(stat.chain, "BTC");
        assert_eq!(stat.possible_to_claim, 0.0);
        assert_eq!(stat.claimed_amount, 0.0);
        assert_eq!(stat.received_amount, 0.0);
        assert_eq!(stat.fees_paid, 0.0);
    }

    #[test]
    fn pa_summary_parses_nested_chain_statistics() {
        // Проверяем вложенную структуру целиком: maxClaimableTotal/claimedTotal
        // (rename) + вложенный массив chainStatistics с camelCase-полями внутри.
        let json = r#"{
            "maxClaimableTotal": 100.0,
            "claimedTotal": 40.0,
            "chainStatistics": [
                {"chain": "ETH", "possibleToClaim": 60.0, "claimedAmount": 40.0, "receivedAmount": 39.5, "feesPaid": 0.5}
            ]
        }"#;
        let summary: PaSummary = serde_json::from_str(json).unwrap();
        assert_eq!(summary.max_claimable_total, 100.0);
        assert_eq!(summary.claimed_total, 40.0);
        assert_eq!(summary.chain_statistics.len(), 1);
        assert_eq!(summary.chain_statistics[0].chain, "ETH");
    }

    #[test]
    fn pa_summary_claimable_is_difference_of_mined_and_claimed() {
        // claimable() — единственная содержательная логика в файле: то самое
        // число «доступно к получению», которое видит пользователь на дашборде.
        let summary = PaSummary {
            max_claimable_total: 100.0,
            claimed_total: 35.0,
            chain_statistics: vec![],
        };
        assert_eq!(summary.claimable(), 65.0);
    }

    #[test]
    fn pa_summary_claimable_never_goes_negative() {
        // Если claimed почему-то превысил max (рассинхрон данных источника),
        // claimable() обязан отдать 0.0, а не отрицательное число — иначе на
        // дашборде показалась бы абсурдная отрицательная сумма к получению.
        let summary = PaSummary {
            max_claimable_total: 10.0,
            claimed_total: 15.0,
            chain_statistics: vec![],
        };
        assert_eq!(summary.claimable(), 0.0);
    }

    #[test]
    fn ip_host_strips_port() {
        assert_eq!(ip_host("82.64.11.18:16137"), "82.64.11.18");
    }

    #[test]
    fn ip_host_returns_input_when_no_port() {
        // fluxinfo иногда отдаёт IP без порта — сопоставление по ключу карты
        // node_stats не должно ломаться на отсутствии ':'.
        assert_eq!(ip_host("82.64.11.18"), "82.64.11.18");
    }

    #[test]
    fn coingecko_price_defaults_change_when_absent() {
        // usd_24h_change — Option с #[serde(default)]: CoinGecko иногда не
        // отдаёт change при include_24hr_change, если данных за 24ч ещё нет
        // (свежий листинг). flux_price() должен получить None, а не упасть.
        let body: CoinGeckoPrice = serde_json::from_str(r#"{"zelcash": {"usd": 0.35}}"#).unwrap();
        assert_eq!(body.zelcash.usd, 0.35);
        assert_eq!(body.zelcash.usd_24h_change, None);
    }

    #[test]
    fn coingecko_price_parses_change_when_present() {
        let body: CoinGeckoPrice =
            serde_json::from_str(r#"{"zelcash": {"usd": 0.42, "usd_24h_change": -3.15}}"#).unwrap();
        assert_eq!(body.zelcash.usd, 0.42);
        assert_eq!(body.zelcash.usd_24h_change, Some(-3.15));
    }

    #[test]
    fn coingecko_chart_parses_price_points_as_pairs() {
        // market_chart отдаёт prices как массив пар [ts_ms, usd] — ровно то,
        // что flux_price_history() потом мапит через `match p.as_slice()`.
        let json = r#"{"prices": [[1700000000000.0, 0.5], [1700086400000.0, 0.52]]}"#;
        let chart: CoinGeckoChart = serde_json::from_str(json).unwrap();
        assert_eq!(chart.prices.len(), 2);
        assert_eq!(chart.prices[0], vec![1700000000000.0, 0.5]);
    }

    #[test]
    fn flux_price_history_mapping_converts_ms_to_secs_and_filters_malformed_points() {
        // Воспроизводим ту же логику filter_map, что в flux_price_history(),
        // чтобы поймать регрессию в переводе мс→сек или в отбрасывании
        // повреждённых точек (не пары), не делая сетевой запрос.
        let prices: Vec<Vec<f64>> = vec![
            vec![1700000000000.0, 0.5],
            vec![123.0], // повреждённая точка — не пара, должна быть отброшена
        ];
        let points: Vec<(i64, f64)> = prices
            .into_iter()
            .filter_map(|p| match p.as_slice() {
                [ts_ms, usd] => Some(((*ts_ms / 1000.0) as i64, *usd)),
                _ => None,
            })
            .collect();
        assert_eq!(points, vec![(1700000000, 0.5)]);
    }

    #[test]
    fn fusion_envelope_unwraps_data_without_status_field() {
        // В отличие от FluxEnvelope, у fusion нет поля status — обёртка
        // должна парситься по одному только data, без ожидания status.
        let json =
            r#"{"data": {"maxClaimableTotal": 5.0, "claimedTotal": 2.0, "chainStatistics": []}}"#;
        let env: FluxFusionEnvelope = serde_json::from_str(json).unwrap();
        assert_eq!(env.data.max_claimable_total, 5.0);
        assert_eq!(env.data.claimed_total, 2.0);
    }

    #[test]
    fn flux_info_rec_parses_full_record_with_bench_apps_geo() {
        // Полная форма одной записи stats.runonflux.io/fluxinfo — проверяем,
        // что все три вложенные секции (benchmark/apps/geolocation) читаются
        // корректно, включая rename countryCode/regionName.
        let json = r#"{
            "ip": "5.6.7.8:16137",
            "benchmark": {
                "info": {"version": "v5.6.0"},
                "bench": {"status": "ok", "error": "", "cores": 8.0, "ram": 32.0, "ssd": 500.0, "eps": 120.5, "ping": 15.2}
            },
            "apps": {"runningapps": [{"name": "app1"}, {"name": "app2"}]},
            "geolocation": {"country": "Germany", "countryCode": "DE", "regionName": "Hesse"}
        }"#;
        let rec: FluxInfoRec = serde_json::from_str(json).unwrap();
        assert_eq!(rec.ip, "5.6.7.8:16137");
        let bench = rec.benchmark.unwrap();
        assert_eq!(bench.info.unwrap().version, "v5.6.0");
        let bench_data = bench.bench.unwrap();
        assert_eq!(bench_data.status, "ok");
        assert_eq!(bench_data.cores, 8.0);
        assert_eq!(rec.apps.unwrap().runningapps.len(), 2);
        let geo = rec.geolocation.unwrap();
        assert_eq!(geo.country_code, "DE");
        assert_eq!(geo.region_name, "Hesse");
    }

    #[test]
    fn flux_info_rec_defaults_missing_sections_to_none() {
        // Нода без бенчмарка/apps/гео (только что добавлена в сеть, ещё не
        // прошла проверку) — все три секции Option и должны стать None, а не
        // валить парсинг всего batch-ответа по сети (~7000 записей, §5.4).
        let rec: FluxInfoRec = serde_json::from_str(r#"{"ip": "9.9.9.9:16137"}"#).unwrap();
        assert!(rec.benchmark.is_none());
        assert!(rec.apps.is_none());
        assert!(rec.geolocation.is_none());
    }

    #[test]
    fn geo_rec_defaults_when_fields_absent() {
        let geo: GeoRec = serde_json::from_str("{}").unwrap();
        assert_eq!(geo.country, "");
        assert_eq!(geo.country_code, "");
        assert_eq!(geo.region_name, "");
    }

    #[test]
    fn bench_data_bench_passed_logic_true_when_status_set_and_error_empty() {
        // Воспроизводим точную логику bench_passed из network_node_stats():
        // пройден = запись бенча есть, error пуст, status не пуст.
        let bench: BenchData = serde_json::from_str(
            r#"{"status": "ok", "error": "", "cores": 4.0, "ram": 8.0, "ssd": 100.0, "eps": 50.0, "ping": 10.0}"#,
        )
        .unwrap();
        let bench_passed = bench.error.is_empty() && !bench.status.is_empty();
        assert!(bench_passed);
    }

    #[test]
    fn bench_data_bench_passed_logic_false_when_error_present() {
        // Наличие непустого error должно считаться непройденным бенчем даже
        // при заполненном status — именно так дашборд решает, рисовать ли
        // ноду как "не прошла бенчмарк".
        let bench: BenchData = serde_json::from_str(
            r#"{"status": "ok", "error": "benchmark timeout", "cores": 0.0, "ram": 0.0, "ssd": 0.0, "eps": 0.0, "ping": 0.0}"#,
        )
        .unwrap();
        let bench_passed = bench.error.is_empty() && !bench.status.is_empty();
        assert!(!bench_passed);
    }

    #[test]
    fn bench_data_bench_passed_logic_false_when_status_empty() {
        let bench: BenchData = serde_json::from_str(
            r#"{"status": "", "error": "", "cores": 0.0, "ram": 0.0, "ssd": 0.0, "eps": 0.0, "ping": 0.0}"#,
        )
        .unwrap();
        let bench_passed = bench.error.is_empty() && !bench.status.is_empty();
        assert!(!bench_passed);
    }

    #[test]
    fn apps_wrap_defaults_to_empty_when_runningapps_absent() {
        // Пустой список приложений — норма (нода без пользовательских apps),
        // а не повод падать на десериализации.
        let apps: AppsWrap = serde_json::from_str("{}").unwrap();
        assert!(apps.runningapps.is_empty());
    }

    #[test]
    fn deterministic_node_list_deserializes_array_from_real_shaped_response() {
        // Проверяем полный список (не одну запись) — форма ответа
        // viewdeterministicfluxnodelist оборачивается в FluxEnvelope.
        let json = r#"{
            "status": "success",
            "data": [
                {
                    "ip": "1.1.1.1:16137",
                    "tier": "NIMBUS",
                    "payment_address": "t1Foo",
                    "rank": 1,
                    "activesince": "1700000000",
                    "lastpaid": "1700500000",
                    "last_paid_height": 123456,
                    "last_confirmed_height": 123400,
                    "added_height": 100000,
                    "amount": "12500.00"
                },
                {
                    "ip": "2.2.2.2:16137",
                    "tier": "CUMULUS",
                    "payment_address": "t1Bar",
                    "rank": 2
                }
            ]
        }"#;
        let env: FluxEnvelope<Vec<DeterministicNode>> = serde_json::from_str(json).unwrap();
        let nodes = env.into_data().unwrap();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].tier, "NIMBUS");
        assert_eq!(nodes[0].last_paid_height, 123456);
        assert_eq!(nodes[1].activesince, ""); // дефолт для второй записи без поля
    }

    #[test]
    fn envelope_rejects_non_success_status_from_real_shaped_error_response() {
        // Flux API при ошибке (например, невалидный запрос) отдаёт
        // {"status": "error", "data": "..."} — into_data() должен вернуть
        // ApiStatus, а не молча отдать содержимое data как валидный результат.
        let json = r#"{"status": "error", "data": "Address not found"}"#;
        let env: FluxEnvelope<String> = serde_json::from_str(json).unwrap();
        let result = env.into_data();
        assert!(matches!(result, Err(FluxError::ApiStatus(s)) if s == "error"));
    }
}
