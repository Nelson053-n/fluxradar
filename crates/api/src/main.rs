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
    // RUST_LOG задаёт уровень (по умолчанию info); без EnvFilter `fmt()`
    // пишет только ERROR и переменная окружения молча игнорируется.
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());
    let state = Arc::new(AppState {
        flux: FluxClient::new()?,
        redis: cache::connect(&redis_url).await?,
    });

    let app = router(state);

    let addr = std::env::var("API_BIND").unwrap_or_else(|_| "0.0.0.0:5049".to_owned());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(%addr, "FluxScope API слушает");
    axum::serve(listener, app).await?;
    Ok(())
}

/// Сборка роутера. Вынесена из `main`, чтобы тесты могли проверить контракт
/// путей (`/api/v1/...`) без Redis и сети: nginx проксирует на бэкенд полный
/// путь, поэтому потеря префикса `/api/v1` ломает прод целиком.
fn router(state: Arc<AppState>) -> Router {
    Router::new()
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
        .with_state(state)
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
/// реальный адрес берём из `X-Real-IP` / хвоста `X-Forwarded-For` (см. `client_ip`).
async fn visitor_stats(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let ip = client_ip(&headers);
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    match cache::track_visitor(&state.redis, &ip, &date).await {
        Ok((total, today)) => (
            StatusCode::OK,
            Json(json!({"total": total, "today": today})),
        ),
        Err(e) => {
            warn!(?e, "не удалось учесть посетителя");
            (StatusCode::OK, Json(json!({"total": 0, "today": 0})))
        }
    }
}

/// Реальный IP клиента за прокси: `X-Real-IP`, иначе ПОСЛЕДНИЙ в `X-Forwarded-For`,
/// иначе "unknown" (все unknown схлопнутся в одного — приемлемо для счётчика).
///
/// Оба заголовка клиент может прислать сам, поэтому доверяем только тому, что
/// дописал наш nginx: `X-Real-IP` он ставит из `$remote_addr`, а в `X-Forwarded-For`
/// (`proxy_add_x_forwarded_for`) дописывает реальный адрес в КОНЕЦ цепочки.
/// Брать первый элемент нельзя — его полностью контролирует клиент и счётчик
/// уникальных посетителей накручивался бы заголовком.
fn client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            headers
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.rsplit(',').next())
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
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
    Path(raw_ip): Path<String>,
) -> impl IntoResponse {
    // Валидация host И port целиком: всё, что после первого ':', раньше уходило
    // в запрос к Flux API и в ключ Redis без проверки (инъекция query-параметров
    // и неограниченный рост ключей кэша). Ниже используется ТОЛЬКО нормализованное
    // значение `ip`, а не исходная строка из пути.
    let Some((ip, host)) = normalize_node_ip(&raw_ip) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "невалидный IP"})),
        )
            .into_response();
    };
    let host = host.as_str();

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

/// Разобрать и нормализовать IP ноды из пути: строго `host` или `host:port`.
///
/// Возвращает `(нормализованный ip, host)` — обе части уже проверены, так что их
/// безопасно подставлять в запрос к Flux API и в ключ Redis. `None` — мусор (400).
///
/// Проверяется ВСЯ строка: host как Ipv4Addr, порт как u16 > 0. Иначе всё после
/// первого ':' утекало бы в апстрим-запрос (`?ip=1.1.1.1:16137&foo=bar`) и плодило
/// бы отдельный ключ кэша на каждый вариант.
fn normalize_node_ip(raw: &str) -> Option<(String, String)> {
    let (host, port) = match raw.split_once(':') {
        Some((h, p)) => (h, Some(p)),
        None => (raw, None),
    };
    // Строгий разбор: отсекает и лишние октеты, и пустые, и любые не-цифры.
    let addr: std::net::Ipv4Addr = host.parse().ok()?;
    let host = addr.to_string();
    match port {
        Some(p) => {
            let port: u16 = p.parse().ok()?;
            if port == 0 {
                return None;
            }
            Some((format!("{host}:{port}"), host))
        }
        None => Some((host.clone(), host)),
    }
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

#[cfg(test)]
mod tests {
    use super::{client_ip, normalize_node_ip, router, AppState};
    use axum::body::Body;
    use axum::http::{HeaderMap, Request, StatusCode};
    use std::sync::Arc;
    use tower::ServiceExt;

    /// Роутер поверх состояния, не требующего живых бэкендов: `cache::connect`
    /// только валидирует URL, соединение открывается при первом обращении.
    /// Поэтому маршрутизацию и валидацию адреса можно проверить без Redis и сети.
    async fn test_router() -> axum::Router {
        let state = Arc::new(AppState {
            // Оба бэкенда намеренно указывают в никуда: тесты проверяют
            // маршрутизацию и раннюю валидацию, поэтому любой реальный поход
            // наружу — это дефект, который должен уронить тест, а не тихо
            // сходить в сеть и записать мусор в живой кэш.
            flux: flux_client::FluxClient::with_base_url("http://127.0.0.1:1")
                .expect("клиент собирается без сети"),
            redis: storage::cache::connect("redis://127.0.0.1:1")
                .await
                .expect("пул строится без соединения"),
        });
        router(state)
    }

    async fn status_of(uri: &str) -> StatusCode {
        let resp = test_router()
            .await
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        resp.status()
    }

    /// Проверка, что путь зарегистрирован, без выполнения хендлера: на известный
    /// маршрут Axum отвечает 405 (метод не разрешён), на неизвестный — 404.
    ///
    /// GET здесь применять нельзя: хендлеры ходят в Flux API и Redis, поэтому на
    /// холодном кэше такой тест висит минутами и зависит от внешней сети.
    async fn route_probe(uri: &str) -> StatusCode {
        method_probe("PROPFIND", uri).await
    }

    /// Тот же зонд конкретным методом. Применять его к остальным маршрутам
    /// нельзя: GET выполняет хендлер, а тот ходит в Flux API и Redis.
    async fn method_probe(method: &str, uri: &str) -> StatusCode {
        let resp = test_router()
            .await
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        resp.status()
    }

    /// Прод-инцидент 2026-09-09: диагностика шла по путям без `v1` и получала
    /// 404, из-за чего рабочий сервис выглядел упавшим. Пути — публичный
    /// контракт с nginx (`proxy_pass` без слэша передаёт путь целиком),
    /// поэтому фиксируем их тестом.
    #[tokio::test]
    async fn all_v1_routes_are_registered() {
        for uri in [
            "/api/v1/health",
            "/api/v1/ready",
            "/api/v1/network/price",
            "/api/v1/network/price/history",
            "/api/v1/network/nodes",
            "/api/v1/stats/visitors",
            "/api/v1/wallet/t1cuMLs3MUkMUH8tnzrkGHQJvxvQqrfuQAf/summary",
            "/api/v1/wallet/t1cuMLs3MUkMUH8tnzrkGHQJvxvQqrfuQAf/nodes",
            "/api/v1/wallet/t1cuMLs3MUkMUH8tnzrkGHQJvxvQqrfuQAf/apps",
            "/api/v1/node/1.2.3.4:16137/detail",
        ] {
            assert_eq!(
                route_probe(uri).await,
                StatusCode::METHOD_NOT_ALLOWED,
                "маршрут {uri} не зарегистрирован"
            );
        }
    }

    #[tokio::test]
    async fn health_is_ok_without_backends() {
        // Liveness не должен зависеть от Redis/Flux API, иначе внешний
        // мониторинг покажет падение при живом процессе.
        assert_eq!(status_of("/api/v1/health").await, StatusCode::OK);
        // Маршрут должен отвечать именно на GET: 405 в предыдущем тесте
        // подтверждает лишь существование пути, но не метод, а фронт и внешний
        // мониторинг ходят сюда GET'ом. Проверяем на health — единственном
        // хендлере, который не обращается ни к Flux API, ни к Redis.
        assert_eq!(
            method_probe("POST", "/api/v1/health").await,
            StatusCode::METHOD_NOT_ALLOWED
        );
    }

    #[tokio::test]
    async fn routes_without_v1_prefix_are_not_served() {
        // Обратная сторона контракта: старые пути без версии отвечать не должны.
        for uri in ["/api/health", "/health", "/api/network/price"] {
            assert_eq!(route_probe(uri).await, StatusCode::NOT_FOUND, "путь {uri}");
        }
    }

    /// Невалидный адрес отсекается до похода в сеть (§9.4): ответ 400, а не 502.
    #[tokio::test]
    async fn invalid_wallet_address_is_rejected_before_upstream() {
        for bad in [
            "not-an-address",
            "0x52908400098527886E0F7030069857D2E4169EE7",
        ] {
            let uri = format!("/api/v1/wallet/{bad}/summary");
            assert_eq!(
                status_of(&uri).await,
                StatusCode::BAD_REQUEST,
                "адрес {bad}"
            );
        }
    }

    /// IP с инъекцией параметров тоже отсекается до обращения к Flux API.
    #[tokio::test]
    async fn malformed_node_ip_is_rejected_before_upstream() {
        let uri = "/api/v1/node/1.1.1.1:16137&foo=bar/detail";
        assert_eq!(status_of(uri).await, StatusCode::BAD_REQUEST);
    }

    fn headers(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut h = HeaderMap::new();
        for (k, v) in pairs {
            let name: axum::http::HeaderName = k.parse().unwrap();
            h.insert(name, v.parse().unwrap());
        }
        h
    }

    #[test]
    fn client_ip_prefers_x_real_ip() {
        // X-Real-IP ставит наш nginx из $remote_addr — ему доверяем в первую очередь.
        let h = headers(&[("x-real-ip", "9.9.9.9"), ("x-forwarded-for", "1.2.3.4")]);
        assert_eq!(client_ip(&h), "9.9.9.9");
    }

    #[test]
    fn client_ip_ignores_spoofed_prefix_of_forwarded_for() {
        // Клиент прислал свой XFF, nginx дописал реальный адрес в конец —
        // берём последний, иначе счётчик посетителей накручивается заголовком.
        let h = headers(&[("x-forwarded-for", "1.2.3.4, 5.6.7.8, 203.0.113.9")]);
        assert_eq!(client_ip(&h), "203.0.113.9");
    }

    #[test]
    fn client_ip_falls_back_to_unknown() {
        assert_eq!(client_ip(&HeaderMap::new()), "unknown");
        assert_eq!(client_ip(&headers(&[("x-real-ip", "  ")])), "unknown");
    }

    #[test]
    fn accepts_plain_ip_and_ip_with_port() {
        assert_eq!(
            normalize_node_ip("1.2.3.4:16137"),
            Some(("1.2.3.4:16137".to_owned(), "1.2.3.4".to_owned()))
        );
        assert_eq!(
            normalize_node_ip("1.2.3.4"),
            Some(("1.2.3.4".to_owned(), "1.2.3.4".to_owned()))
        );
    }

    #[test]
    fn rejects_query_param_injection() {
        // Раньше эти payload'ы проходили: валидировался только host до ':',
        // а хвост уходил в `?ip=` запроса к Flux API и в ключ Redis.
        assert!(normalize_node_ip("1.1.1.1:16137&foo=bar").is_none());
        assert!(normalize_node_ip("1.1.1.1:16137/../../../etc").is_none());
        assert!(normalize_node_ip("1.1.1.1:@evil.com/").is_none());
        assert!(normalize_node_ip("1.1.1.1:16137#frag").is_none());
        assert!(normalize_node_ip("1.1.1.1:1 HTTP/1.1\r\nX: y").is_none());
    }

    #[test]
    fn rejects_malformed_host_and_port() {
        assert!(normalize_node_ip("").is_none());
        assert!(normalize_node_ip("evil.com").is_none());
        assert!(normalize_node_ip("1.1.1.1.1").is_none()); // лишний октет
        assert!(normalize_node_ip("1.1.1").is_none()); // неполный
        assert!(normalize_node_ip("999.1.1.1").is_none()); // октет > 255
        assert!(normalize_node_ip("1.1.1.1:0").is_none()); // порт 0
        assert!(normalize_node_ip("1.1.1.1:99999").is_none()); // порт > u16
        assert!(normalize_node_ip("1.1.1.1:").is_none()); // пустой порт
    }

    #[test]
    fn normalizes_cache_key_variants_to_same_value() {
        // Ключ кэша строится из нормализованного значения — «01.1.1.1» и «1.1.1.1»
        // не должны плодить два разных ключа для одной ноды.
        let a = normalize_node_ip("1.1.1.1:16137").unwrap().0;
        let b = normalize_node_ip("1.1.1.1:016137").unwrap().0;
        assert_eq!(a, b);
    }
}
