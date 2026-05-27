//! HTTP API FluxScope — REST-прокси перед Flux API + бизнес-логика.
//!
//! Стек: Axum 0.8 (см. docs/ADR-001-stack.md). Эндпоинты — §8 ТЗ.

use std::str::FromStr;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use domain::{node_age_secs, NetworkSummary, SummaryInputs, Tier};
use flux_client::{DeterministicNode, FluxClient};
use serde_json::json;
use storage::cache::{self, RedisPool};
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

/// Общее состояние приложения, разделяемое между хендлерами.
#[derive(Clone)]
struct AppState {
    flux: FluxClient,
    redis: RedisPool,
}

/// TTL кэша ответов wallet/* (§8: max-age=30).
const CACHE_TTL_SECS: u64 = 30;
/// TTL кэша детали ноды (статус/apps) — дороже, живёт дольше.
const DETAIL_TTL_SECS: u64 = 300;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt().json().init();

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());
    let state = Arc::new(AppState {
        flux: FluxClient::new()?,
        redis: cache::connect(&redis_url).await?,
    });

    let app = Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/ready", get(ready))
        .route("/api/v1/network/price", get(network_price))
        .route("/api/v1/network/price/history", get(price_history))
        .route("/api/v1/network/nodes", get(network_nodes))
        .route("/api/v1/stats/visitors", get(visitor_stats))
        .route("/api/v1/wallet/{address}/summary", get(wallet_summary))
        .route("/api/v1/wallet/{address}/nodes", get(wallet_nodes))
        .route("/api/v1/wallet/{address}/apps", get(wallet_apps))
        .route("/api/v1/node/{ip}/detail", get(node_detail))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = std::env::var("API_BIND").unwrap_or_else(|_| "0.0.0.0:5049".to_owned());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(%addr, "FluxScope API слушает");
    axum::serve(listener, app).await?;
    Ok(())
}

/// Liveness probe — процесс жив (§6).
async fn health() -> &'static str {
    "ok"
}

/// Readiness probe — проверяет Redis и доступность Flux API (§6).
async fn ready(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let redis_ok = cache::ping(&state.redis).await.is_ok();
    let flux_ok = state.flux.flux_price_usd().await.is_ok();
    if redis_ok && flux_ok {
        (
            StatusCode::OK,
            Json(json!({"redis": true, "flux_api": true})),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"redis": redis_ok, "flux_api": flux_ok})),
        )
    }
}

/// Текущая цена FLUX в USD + изменение за 24ч (§8).
async fn network_price(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.flux.flux_price().await {
        Ok(p) => (
            StatusCode::OK,
            Json(json!({"flux_usd": p.usd, "change_24h": p.change_24h})),
        )
            .into_response(),
        Err(err) => upstream_error(err),
    }
}

/// Счётчики активных нод сети по тирам (для калькулятора доходности). Кэш в Redis 60с.
async fn network_nodes(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    const KEY: &str = "network:nodes";
    const TTL: u64 = 60;
    if let Ok(Some(cached)) = cache::get(&state.redis, KEY).await {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&cached) {
            return (StatusCode::OK, Json(val)).into_response();
        }
    }
    match state.flux.network_count().await {
        Ok(n) => {
            let body = json!({
                "total": n.total,
                "cumulus": n.cumulus,
                "nimbus": n.nimbus,
                "stratus": n.stratus,
            });
            if let Ok(s) = serde_json::to_string(&body) {
                if let Err(e) = cache::set_ex(&state.redis, KEY, &s, TTL).await {
                    warn!(?e, "не удалось записать кэш network:nodes");
                }
            }
            (StatusCode::OK, Json(body)).into_response()
        }
        Err(err) => upstream_error(err),
    }
}

/// История цены FLUX за год (для тултипа-графика). Кэш в Redis на 1 час.
async fn price_history(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    const KEY: &str = "price_history:365d";
    const TTL: u64 = 3600;
    if let Ok(Some(cached)) = cache::get(&state.redis, KEY).await {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&cached) {
            return (StatusCode::OK, Json(val)).into_response();
        }
    }
    match state.flux.flux_price_history().await {
        Ok(points) => {
            let series: Vec<_> = points
                .into_iter()
                .map(|(ts, usd)| json!({"t": ts, "usd": usd}))
                .collect();
            let body = json!({ "points": series });
            if let Ok(s) = serde_json::to_string(&body) {
                if let Err(e) = cache::set_ex(&state.redis, KEY, &s, TTL).await {
                    warn!(?e, "не удалось записать кэш price_history");
                }
            }
            (StatusCode::OK, Json(body)).into_response()
        }
        Err(err) => upstream_error(err),
    }
}

/// Счётчик уникальных посетителей сайта: `{total, today}`.
///
/// Фронт дёргает при загрузке — IP учитывается в HLL (Redis). За nginx-прокси
/// реальный адрес берём из `X-Forwarded-For` (первый) / `X-Real-IP`.
async fn visitor_stats(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let ip = client_ip(&headers);
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    match cache::track_visitor(&state.redis, &ip, &date).await {
        Ok((total, today)) => (StatusCode::OK, Json(json!({"total": total, "today": today}))),
        Err(e) => {
            warn!(?e, "не удалось учесть посетителя");
            (StatusCode::OK, Json(json!({"total": 0, "today": 0})))
        }
    }
}

/// Реальный IP клиента за прокси: первый в `X-Forwarded-For`, иначе `X-Real-IP`,
/// иначе "unknown" (все unknown схлопнутся в одного — приемлемо для счётчика).
fn client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.trim().to_owned())
        })
        .unwrap_or_else(|| "unknown".to_owned())
}

/// Сводка по кошельку (§4.1, §8) — с кэшем в Redis.
async fn wallet_summary(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    if !domain::is_valid_address(&address) {
        return bad_address();
    }

    let cache_key = format!("summary:{address}");
    if let Ok(Some(cached)) = cache::get(&state.redis, &cache_key).await {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&cached) {
            return (StatusCode::OK, Json(val)).into_response();
        }
    }

    // Cold path: все внешние запросы независимы — запускаем параллельно (§5.4),
    // общее время ≈ самый медленный запрос, а не сумма. Бенчмарк/apps по всей сети
    // (node_stats) кэшируется в Redis на 5 мин — тяжёлый запрос (~7000 нод).
    let (nodes_res, balance_res, price_res, network_res, pa_res, node_stats) = tokio::join!(
        state.flux.nodes_for_wallet(&address),
        state.flux.balance_satoshi(&address),
        state.flux.flux_price(),
        state.flux.network_count(),
        state.flux.wallet_pa_summary(&address),
        cached_node_stats(&state),
    );

    let nodes = match nodes_res {
        Ok(n) => n,
        Err(err) => return upstream_error(err),
    };
    let balance = balance_res.unwrap_or(0);
    let price = price_res.ok();
    let network = network_res.ok();
    let pa = pa_res.ok();
    let (bench_passed, hosted_apps) = match &node_stats {
        Some(stats) => {
            let mut bench = 0u32;
            let mut apps = 0u32;
            for n in &nodes {
                if let Some(s) = stats.get(flux_client::ip_host(&n.ip)) {
                    if s.bench_passed {
                        bench += 1;
                    }
                    apps += s.apps_count;
                }
            }
            (bench, apps)
        }
        None => (0, 0),
    };

    let tiers: Vec<Tier> = nodes
        .iter()
        .filter_map(|n| Tier::from_str(&n.tier).ok())
        .collect();

    // Возраст старейшей ноды флота из activesince (без доп. запросов).
    let now = chrono::Utc::now().timestamp();
    let oldest_age = nodes
        .iter()
        .filter_map(|n| n.activesince.parse::<i64>().ok())
        .map(|a| node_age_secs(a, now))
        .max()
        .unwrap_or(0);

    // Per-node (тир, возраст, секунды с последней выплаты) — для mined/claimable.
    let nodes_age_paid: Vec<(Tier, i64, i64)> = nodes
        .iter()
        .filter_map(|n| {
            let tier = Tier::from_str(&n.tier).ok()?;
            let age = n
                .activesince
                .parse::<i64>()
                .ok()
                .map(|a| node_age_secs(a, now))
                .unwrap_or(0);
            let since_paid = n
                .lastpaid
                .parse::<i64>()
                .ok()
                .map(|lp| node_age_secs(lp, now))
                .unwrap_or(0);
            Some((tier, age, since_paid))
        })
        .collect();

    let inputs = SummaryInputs {
        balance_satoshi: balance,
        flux_price_usd: price.map(|p| p.usd).unwrap_or(0.0),
        price_change_24h: price.map(|p| p.change_24h).unwrap_or(0.0),
        network: network
            .map(|n| NetworkSummary {
                total: n.total,
                cumulus: n.cumulus,
                nimbus: n.nimbus,
                stratus: n.stratus,
            })
            .unwrap_or(NetworkSummary {
                total: 0,
                cumulus: 0,
                nimbus: 0,
                stratus: 0,
            }),
        oldest_node_age_secs: oldest_age,
        // Нода в детерминированном списке = подтверждённая и активная (§5.4),
        // поэтому все ноды флота считаем CONFIRMED — без N запросов статуса.
        confirmed_nodes: tiers.len() as u32,
        nodes_age_paid,
        real_pa: pa.map(|p| domain::RealPaTotals {
            mined_flux: p.max_claimable_total,
            claimed_flux: p.claimed_total,
            claimable_flux: p.claimable(),
            chains: p
                .chain_statistics
                .into_iter()
                .map(|c| domain::PaChain {
                    chain: c.chain,
                    claimable: c.possible_to_claim,
                    claimed: c.claimed_amount,
                    received: c.received_amount,
                    fees: c.fees_paid,
                })
                .collect(),
        }),
        bench_passed,
        hosted_apps,
    };

    let summary = domain::build_summary(&tiers, &inputs);
    let body = serde_json::to_value(&summary).unwrap_or_else(|_| json!({}));

    if let Ok(s) = serde_json::to_string(&body) {
        if let Err(e) = cache::set_ex(&state.redis, &cache_key, &s, CACHE_TTL_SECS).await {
            warn!(?e, "не удалось записать кэш summary");
        }
    }
    (StatusCode::OK, Json(body)).into_response()
}

/// Список нод владельца (§4.2, §8) — с возрастом, выплатой, apps/FluxOS и гео.
async fn wallet_nodes(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    if !domain::is_valid_address(&address) {
        return bad_address();
    }
    let nodes = match state.flux.nodes_for_wallet(&address).await {
        Ok(n) => n,
        Err(err) => return upstream_error(err),
    };

    // Параллельно: бенчмарк/apps/FluxOS/гео (тёплый кэш node_stats) + высота блока.
    let (node_stats, height_res) =
        tokio::join!(cached_node_stats(&state), state.flux.block_height());
    let block_height = height_res.unwrap_or(0);

    let now = chrono::Utc::now().timestamp();
    let items: Vec<_> = nodes
        .into_iter()
        .map(|n| {
            let host = flux_client::ip_host(&n.ip);
            let stats = node_stats.as_ref().and_then(|m| m.get(host));
            node_json(&n, now, stats, block_height)
        })
        .collect();
    (StatusCode::OK, Json(json!({"nodes": items}))).into_response()
}

/// Подсчёт приложений на нодах флота (§4) — дозагрузка после основной страницы.
/// installedapps дёргается на каждую ноду параллельно пачками; результат в Redis.
/// Возвращает: total (всего apps по флоту) и max_on_node (макс. на одной ноде).
async fn wallet_apps(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    if !domain::is_valid_address(&address) {
        return bad_address();
    }
    let cache_key = format!("apps:{address}");
    if let Ok(Some(cached)) = cache::get(&state.redis, &cache_key).await {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&cached) {
            return (StatusCode::OK, Json(val)).into_response();
        }
    }

    let nodes = match state.flux.nodes_for_wallet(&address).await {
        Ok(n) => n,
        Err(err) => return upstream_error(err),
    };
    // (ip, tier) для каждой ноды — чтобы вернуть ноду-лидера по числу приложений.
    let node_meta: Vec<(String, String)> = nodes
        .iter()
        .map(|n| (n.ip.clone(), n.tier.clone()))
        .collect();

    // Параллельный fan-out installedapps пачками по CONCURRENCY одновременно.
    const CONCURRENCY: usize = 25;
    let mut total: u32 = 0;
    let mut max_on_node: u32 = 0;
    // Нода-лидер по числу приложений (для подписи «Apps on a single node»).
    let mut top: Option<(String, String, u32)> = None;
    for chunk in node_meta.chunks(CONCURRENCY) {
        let mut set = tokio::task::JoinSet::new();
        for (ip, tier) in chunk {
            let flux = state.flux.clone();
            let ip = ip.clone();
            let tier = tier.clone();
            set.spawn(async move {
                let count = flux.node_apps(&ip).await.map(|a| a.len() as u32)?;
                Ok::<_, flux_client::FluxError>((ip, tier, count))
            });
        }
        while let Some(res) = set.join_next().await {
            if let Ok(Ok((ip, tier, count))) = res {
                total += count;
                if count > max_on_node {
                    max_on_node = count;
                }
                if count > 0 && top.as_ref().map(|(_, _, c)| count > *c).unwrap_or(true) {
                    top = Some((ip, tier, count));
                }
            }
        }
    }

    let top_node = top.map(|(ip, tier, count)| json!({"ip": ip, "tier": tier, "count": count}));
    let body = json!({ "total": total, "max_on_node": max_on_node, "top_node": top_node });
    if let Ok(s) = serde_json::to_string(&body) {
        // Кэш на 10 мин — apps меняются редко, повтор мгновенный.
        if let Err(e) = cache::set_ex(&state.redis, &cache_key, &s, 600).await {
            warn!(?e, "не удалось записать кэш apps");
        }
    }
    (StatusCode::OK, Json(body)).into_response()
}

/// JSON одной ноды для списка: дешёвые поля + apps/FluxOS/гео (node_stats) + обслуживание.
fn node_json(
    n: &DeterministicNode,
    now: i64,
    stats: Option<&flux_client::NodeStats>,
    block_height: i64,
) -> serde_json::Value {
    let age = n
        .activesince
        .parse::<i64>()
        .ok()
        .map(|a| node_age_secs(a, now))
        .unwrap_or(0);
    let last_paid = n.lastpaid.parse::<i64>().ok();
    json!({
        "ip": n.ip,
        "tier": n.tier,
        "rank": n.rank,
        "payment_address": n.payment_address,
        "age_secs": age,
        "last_paid": last_paid,
        // Оценка времени до выплаты по рангу (позиция в очереди round-robin).
        "payout_eta_secs": domain::payout_eta_secs(n.rank),
        // Окно обслуживания: сек до переподтверждения ноды (null = закрыто/нет данных).
        "maintenance_window_secs": domain::maintenance_window_secs(n.last_confirmed_height, block_height),
        // Нода в списке детерминированных = активна.
        "status": "CONFIRMED",
        // Версия FluxOS из node_stats (тёплый кэш) по IP. Apps в таблице не показываем
        // (running != installed — расхождение; число приложений см. в деталях ноды).
        "flux_os_version": stats.map(|s| s.flux_os_version.clone()).filter(|v| !v.is_empty()),
        "geo": geo_json(stats),
    })
}

/// Геолокация ноды из NodeStats (официальный Flux-источник, projection=geolocation).
/// Форма `{country, country_code, city}` сохранена для фронта (city = region). null — нет страны.
fn geo_json(stats: Option<&flux_client::NodeStats>) -> serde_json::Value {
    match stats {
        Some(s) if !s.country.is_empty() => json!({
            "country": s.country,
            "country_code": s.country_code,
            "city": s.region,
        }),
        _ => serde_json::Value::Null,
    }
}

/// Деталь конкретной ноды (§8) — дорогие данные лениво: статус, apps, гео.
/// Кэшируется в Redis (DETAIL_TTL для статуса/apps, GEO_TTL для гео).
async fn node_detail(
    State(state): State<Arc<AppState>>,
    Path(ip): Path<String>,
) -> impl IntoResponse {
    // Грубая валидация IP (host[:port]) — отсечь мусор до запроса наружу.
    let host = ip.split(':').next().unwrap_or("");
    if host.is_empty() || !host.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "невалидный IP"})),
        )
            .into_response();
    }

    let cache_key = format!("detail:{ip}");
    if let Ok(Some(cached)) = cache::get(&state.redis, &cache_key).await {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&cached) {
            return (StatusCode::OK, Json(val)).into_response();
        }
    }

    // Тир/статус берём из детерминированного списка по ТОЧНОМУ ip (с портом):
    // getfluxnodestatus?ip= ненадёжен (возвращает чужую ноду при дублях host).
    // На одном host может быть несколько нод разных кошельков — ищем точное совпадение.
    let det = state.flux.deterministic_node_list().await.ok();
    let node = det
        .as_ref()
        .and_then(|list| list.iter().find(|n| n.ip == ip));
    let (status, tier) = match node {
        // Нода найдена в детерминированном списке = подтверждена и активна.
        Some(n) => (Some("CONFIRMED".to_owned()), Some(n.tier.clone())),
        None => (None, None),
    };

    let apps = state.flux.node_apps(&ip).await.unwrap_or_default();
    // Гео — из общего node_stats-кэша (официальный Flux-источник) по host.
    let node_stats = cached_node_stats(&state).await;
    let geo = geo_json(node_stats.as_ref().and_then(|m| m.get(host)));

    let body = json!({
        "ip": ip,
        "status": status,
        "tier": tier,
        // Число приложений = длина списка installedapps (согласовано с hosted apps).
        "apps_count": apps.len(),
        "apps": apps,
        "geo": geo,
    });

    if let Ok(s) = serde_json::to_string(&body) {
        if let Err(e) = cache::set_ex(&state.redis, &cache_key, &s, DETAIL_TTL_SECS).await {
            warn!(?e, "не удалось записать кэш detail");
        }
    }
    (StatusCode::OK, Json(body)).into_response()
}

/// Бенчмарк/apps по всем нодам сети с кэшем в Redis. None при ошибке.
/// Кэш прогревается воркером каждые 60с — обычно читается из тёплого кэша;
/// fallback тянет fluxinfo сам, если кэш пуст (ключ/TTL общие со storage).
async fn cached_node_stats(
    state: &AppState,
) -> Option<std::collections::HashMap<String, flux_client::NodeStats>> {
    use cache::{NODE_STATS_KEY as KEY, NODE_STATS_TTL_SECS as TTL};
    if let Ok(Some(cached)) = cache::get(&state.redis, KEY).await {
        if let Ok(map) = serde_json::from_str(&cached) {
            return Some(map);
        }
    }
    let map = state.flux.network_node_stats().await.ok()?;
    if let Ok(s) = serde_json::to_string(&map) {
        if let Err(e) = cache::set_ex(&state.redis, KEY, &s, TTL).await {
            warn!(?e, "не удалось записать кэш node_stats");
        }
    }
    Some(map)
}

fn bad_address() -> axum::response::Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({"error": "невалидный адрес кошелька"})),
    )
        .into_response()
}

fn upstream_error(err: flux_client::FluxError) -> axum::response::Response {
    warn!(%err, "ошибка обращения к Flux API");
    (
        StatusCode::BAD_GATEWAY,
        Json(json!({"error": "Flux API недоступен"})),
    )
        .into_response()
}
