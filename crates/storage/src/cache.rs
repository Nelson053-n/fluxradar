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

fn map_pool_err(e: bb8::RunError<redis::RedisError>) -> crate::StorageError {
    match e {
        bb8::RunError::User(err) => crate::StorageError::Redis(err),
        bb8::RunError::TimedOut => crate::StorageError::Redis(redis::RedisError::from((
            redis::ErrorKind::IoError,
            "timeout получения соединения из пула Redis",
        ))),
    }
}
