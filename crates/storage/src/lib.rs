//! Слой хранения: репозитории PostgreSQL (sqlx) и кэш Redis (bb8).
//!
//! PG хранит привязки TG↔кошелёк, снапшоты статусов нод, историю алертов,
//! курсоры транзакций (схема — migrations/0001_init.sql, §5.5 ТЗ).
//! Redis — кэш ответов Flux API, rate-limit, дедупликация алертов (§5.3, §8).

pub mod cache;
pub mod pg;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("ошибка PostgreSQL: {0}")]
    Pg(#[from] sqlx::Error),
    #[error("ошибка Redis: {0}")]
    Redis(#[from] redis::RedisError),
}
