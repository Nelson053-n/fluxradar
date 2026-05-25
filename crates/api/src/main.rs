//! HTTP API FluxScope — REST-прокси перед Flux API + бизнес-логика.
//!
//! Стек: Axum 0.8 (см. docs/ADR-001-stack.md). Эндпоинты — §8 ТЗ.

use std::str::FromStr;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
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
/// TTL кэша геолокации — почти статична.
const GEO_TTL_SECS: u64 = 86_400;

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

    // Параллельно: бенчмарк/apps/FluxOS (тёплый кэш), гео IP флота, высота блока.
    let host_ips: Vec<String> = nodes
        .iter()
        .map(|n| flux_client::ip_host(&n.ip).to_owned())
        .collect();
    let (node_stats, geo, height_res) = tokio::join!(
        cached_node_stats(&state),
        fleet_geo(&state, &host_ips),
        state.flux.block_height(),
    );
    let block_height = height_res.unwrap_or(0);

    let now = chrono::Utc::now().timestamp();
    let items: Vec<_> = nodes
        .into_iter()
        .map(|n| {
            let host = flux_client::ip_host(&n.ip);
            let stats = node_stats.as_ref().and_then(|m| m.get(host));
            let geo_val = geo.get(host).cloned().unwrap_or(serde_json::Value::Null);
            node_json(&n, now, stats, geo_val, block_height)
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

/// Геолокация IP флота одним batch-запросом с кэшем в Redis (host → {country,...}).
async fn fleet_geo(
    state: &AppState,
    host_ips: &[String],
) -> std::collections::HashMap<String, serde_json::Value> {
    let mut result = std::collections::HashMap::new();
    let mut to_fetch = Vec::new();
    // Сначала из кэша (гео почти статична, TTL сутки).
    for ip in host_ips {
        let key = format!("geo:{ip}");
        if let Ok(Some(c)) = cache::get(&state.redis, &key).await {
            if let Ok(v) = serde_json::from_str(&c) {
                result.insert(ip.clone(), v);
                continue;
            }
        }
        to_fetch.push(ip.clone());
    }
    // Остаток — одним batch к ip-api.
    if !to_fetch.is_empty() {
        if let Ok(geos) = state.flux.geo_batch(&to_fetch).await {
            for g in geos {
                let host = flux_client::ip_host(&g.query).to_owned();
                let v =
                    json!({"country": g.country, "country_code": g.country_code, "city": g.city});
                if let Ok(s) = serde_json::to_string(&v) {
                    let _ =
                        cache::set_ex(&state.redis, &format!("geo:{host}"), &s, GEO_TTL_SECS).await;
                }
                result.insert(host, v);
            }
        }
    }
    result
}

/// JSON одной ноды для списка: дешёвые поля + apps/FluxOS (node_stats) + гео + обслуживание.
fn node_json(
    n: &DeterministicNode,
    now: i64,
    stats: Option<&flux_client::NodeStats>,
    geo: serde_json::Value,
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
        "geo": geo,
    })
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
    let geo = node_geo(&state, host).await;

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

/// Геолокация одного host с отдельным длинным кэшем (гео почти статична).
async fn node_geo(state: &AppState, host: &str) -> serde_json::Value {
    let geo_key = format!("geo:{host}");
    if let Ok(Some(cached)) = cache::get(&state.redis, &geo_key).await {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&cached) {
            return val;
        }
    }
    let geo = match state.flux.geo_batch(&[host.to_owned()]).await {
        Ok(mut v) if !v.is_empty() => {
            let g = v.remove(0);
            json!({"country": g.country, "country_code": g.country_code, "city": g.city})
        }
        _ => json!(null),
    };
    if let Ok(s) = serde_json::to_string(&geo) {
        let _ = cache::set_ex(&state.redis, &geo_key, &s, GEO_TTL_SECS).await;
    }
    geo
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
