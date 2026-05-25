//! Пул соединений Redis (bb8) — кэш Flux API, rate-limit, дедуп алертов.

use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;

pub type RedisPool = bb8::Pool<RedisConnectionManager>;

/// Ключ кэша бенчмарк/apps по всем нодам сети (fluxinfo). Пишут оба: `api`
/// (lazy при cold-summary) и `worker` (прогрев каждый тик) — ключ общий, чтобы
/// первый заход на любой кошелёк читал уже тёплый кэш.
pub const NODE_STATS_KEY: &str = "node_stats:all";
/// TTL кэша node_stats. Worker прогревает каждые 60с — TTL с запасом, чтобы кэш
/// не успевал протухнуть между тиками даже при пропуске одного.
pub const NODE_STATS_TTL_SECS: u64 = 300;

/// Создать пул Redis по URL (`REDIS_URL`).
pub async fn connect(redis_url: &str) -> Result<RedisPool, redis::RedisError> {
    let manager = RedisConnectionManager::new(redis_url)?;
    // build() с валидным менеджером не падает; реальное соединение проверяется при первом get().
    Ok(bb8::Pool::builder()
        .build(manager)
        .await
        .expect("RedisConnectionManager уже провалидирован выше"))
}

/// Прочитать строковое значение по ключу (если есть).
pub async fn get(pool: &RedisPool, key: &str) -> Result<Option<String>, crate::StorageError> {
    let mut conn = pool.get().await.map_err(map_pool_err)?;
    let val: Option<String> = conn.get(key).await?;
    Ok(val)
}

/// Записать значение с TTL в секундах (§8: TTL 30–60с).
pub async fn set_ex(
    pool: &RedisPool,
    key: &str,
    value: &str,
    ttl_secs: u64,
) -> Result<(), crate::StorageError> {
    let mut conn = pool.get().await.map_err(map_pool_err)?;
    let _: () = conn.set_ex(key, value, ttl_secs).await?;
    Ok(())
}

/// Проверка доступности Redis для readiness-probe (§6).
pub async fn ping(pool: &RedisPool) -> Result<(), crate::StorageError> {
    let mut conn = pool.get().await.map_err(map_pool_err)?;
    let _: String = redis::cmd("PING").query_async(&mut *conn).await?;
    Ok(())
}

/// Ключ HLL уникальных посетителей за всё время.
const VISITORS_TOTAL_KEY: &str = "visitors:total";
/// За сутки HLL хранится 48ч — на стыке дней «вчера» ещё доступен для отладки.
const VISITORS_DAY_TTL_SECS: i64 = 172_800;

/// Учесть посетителя (по IP) и вернуть `(total, today)` уникальных.
///
/// Считаем через HyperLogLog (PFADD/PFCOUNT): компактно и достаточно точно для
/// наших объёмов. Ключ суток — `visitors:day:<UTC-дата>` с TTL 48ч.
pub async fn track_visitor(
    pool: &RedisPool,
    ip: &str,
    utc_date: &str,
) -> Result<(u64, u64), crate::StorageError> {
    let day_key = format!("visitors:day:{utc_date}");
    let mut conn = pool.get().await.map_err(map_pool_err)?;

    let _: () = redis::cmd("PFADD")
        .arg(VISITORS_TOTAL_KEY)
        .arg(ip)
        .query_async(&mut *conn)
        .await?;
    let _: () = redis::cmd("PFADD")
        .arg(&day_key)
        .arg(ip)
        .query_async(&mut *conn)
        .await?;
    // TTL обновляем каждый раз — день «живёт» пока на него заходят + 48ч.
    let _: () = redis::cmd("EXPIRE")
        .arg(&day_key)
        .arg(VISITORS_DAY_TTL_SECS)
        .query_async(&mut *conn)
        .await?;

    let total: u64 = redis::cmd("PFCOUNT")
        .arg(VISITORS_TOTAL_KEY)
        .query_async(&mut *conn)
        .await?;
    let today: u64 = redis::cmd("PFCOUNT")
        .arg(&day_key)
        .query_async(&mut *conn)
        .await?;
    Ok((total, today))
}

fn map_pool_err(e: bb8::RunError<redis::RedisError>) -> crate::StorageError {
    match e {
        bb8::RunError::User(err) => crate::StorageError::Redis(err),
        bb8::RunError::TimedOut => crate::StorageError::Redis(redis::RedisError::from((
            redis::ErrorKind::IoError,
            "timeout получения соединения из пула Redis",
        ))),
    }
}
