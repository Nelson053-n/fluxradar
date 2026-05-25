//! Пул соединений PostgreSQL и репозитории (§5.5 ТЗ).
//!
//! Используются `sqlx::query*` (runtime-проверка), а не макросы `query!`,
//! чтобы сборка не требовала живой БД / offline-кэша sqlx.

use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Row;

use crate::StorageError;

/// Создать пул PG по строке подключения (`DATABASE_URL`).
pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}

// --- Подписки TG ↔ кошелёк. Доступ только у bot и worker (§5.1). ---

/// Одна подписка из `telegram_subscriptions`.
#[derive(Debug, Clone)]
pub struct Subscription {
    pub tg_chat_id: i64,
    pub tg_user_id: i64,
    pub wallet_address: String,
}

/// Репозиторий подписок.
#[derive(Clone)]
pub struct SubscriptionsRepo {
    pool: PgPool,
}

impl SubscriptionsRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Привязать кошелёк к чату. Идемпотентно: повтор той же пары (chat, wallet)
    /// не создаёт дубль (ON CONFLICT по UNIQUE). Возвращает true, если добавлено.
    pub async fn add(
        &self,
        tg_chat_id: i64,
        tg_user_id: i64,
        wallet: &str,
    ) -> Result<bool, StorageError> {
        let res = sqlx::query(
            "INSERT INTO telegram_subscriptions (tg_chat_id, tg_user_id, wallet_address)
             VALUES ($1, $2, $3)
             ON CONFLICT (tg_chat_id, wallet_address) DO NOTHING",
        )
        .bind(tg_chat_id)
        .bind(tg_user_id)
        .bind(wallet)
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    /// Отвязать кошелёк от чата. Возвращает true, если что-то удалено.
    pub async fn remove(&self, tg_chat_id: i64, wallet: &str) -> Result<bool, StorageError> {
        let res = sqlx::query(
            "DELETE FROM telegram_subscriptions WHERE tg_chat_id = $1 AND wallet_address = $2",
        )
        .bind(tg_chat_id)
        .bind(wallet)
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    /// Кошельки, привязанные к конкретному чату (для команды /wallets).
    pub async fn wallets_for_chat(&self, tg_chat_id: i64) -> Result<Vec<String>, StorageError> {
        let rows = sqlx::query(
            "SELECT wallet_address FROM telegram_subscriptions
             WHERE tg_chat_id = $1 ORDER BY created_at",
        )
        .bind(tg_chat_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.get("wallet_address")).collect())
    }

    /// Все подписки сети (для воркера): по каждому кошельку — кому слать алерты.
    pub async fn all(&self) -> Result<Vec<Subscription>, StorageError> {
        let rows = sqlx::query(
            "SELECT tg_chat_id, tg_user_id, wallet_address FROM telegram_subscriptions",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| Subscription {
                tg_chat_id: r.get("tg_chat_id"),
                tg_user_id: r.get("tg_user_id"),
                wallet_address: r.get("wallet_address"),
            })
            .collect())
    }

    /// Уникальные адреса кошельков среди всех подписок (фильтр для §5.4).
    pub async fn distinct_wallets(&self) -> Result<Vec<String>, StorageError> {
        let rows = sqlx::query("SELECT DISTINCT wallet_address FROM telegram_subscriptions")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(|r| r.get("wallet_address")).collect())
    }
}

// --- Настройки чата: язык бота + тогглы уведомлений. ---

/// Тогглы типов уведомлений на чат (какие алерты слать).
#[derive(Debug, Clone, Copy)]
pub struct AlertPrefs {
    pub offline: bool,
    pub online: bool,
    pub status: bool,
}

impl Default for AlertPrefs {
    fn default() -> Self {
        // По умолчанию все уведомления включены.
        Self {
            offline: true,
            online: true,
            status: true,
        }
    }
}

/// Репозиторий настроек чата (язык + уведомления) — таблица chat_settings.
#[derive(Clone)]
pub struct ChatSettingsRepo {
    pool: PgPool,
}

impl ChatSettingsRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Язык чата ('en'/'ru'/'zh'). None, если запись ещё не создавалась.
    pub async fn get_lang(&self, chat_id: i64) -> Result<Option<String>, StorageError> {
        let row = sqlx::query("SELECT lang FROM chat_settings WHERE tg_chat_id = $1")
            .bind(chat_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.get("lang")))
    }

    /// Установить язык чата (upsert).
    pub async fn set_lang(&self, chat_id: i64, lang: &str) -> Result<(), StorageError> {
        sqlx::query(
            "INSERT INTO chat_settings (tg_chat_id, lang) VALUES ($1, $2)
             ON CONFLICT (tg_chat_id) DO UPDATE SET lang = EXCLUDED.lang, updated_at = NOW()",
        )
        .bind(chat_id)
        .bind(lang)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Тогглы уведомлений чата. Если записи нет — дефолт (всё включено).
    pub async fn get_alert_prefs(&self, chat_id: i64) -> Result<AlertPrefs, StorageError> {
        let row = sqlx::query(
            "SELECT alert_offline, alert_online, alert_status FROM chat_settings WHERE tg_chat_id = $1",
        )
        .bind(chat_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(match row {
            Some(r) => AlertPrefs {
                offline: r.get("alert_offline"),
                online: r.get("alert_online"),
                status: r.get("alert_status"),
            },
            None => AlertPrefs::default(),
        })
    }

    /// Переключить один тогл уведомлений (upsert). `kind` ∈ offline/online/status.
    pub async fn set_alert_pref(
        &self,
        chat_id: i64,
        kind: &str,
        value: bool,
    ) -> Result<(), StorageError> {
        // Колонку выбираем из белого списка — не подставляем извне напрямую.
        let col = match kind {
            "offline" => "alert_offline",
            "online" => "alert_online",
            "status" => "alert_status",
            _ => return Ok(()),
        };
        // Сначала гарантируем строку, затем обновляем нужную колонку.
        let sql = format!(
            "INSERT INTO chat_settings (tg_chat_id, {col}) VALUES ($1, $2)
             ON CONFLICT (tg_chat_id) DO UPDATE SET {col} = EXCLUDED.{col}, updated_at = NOW()"
        );
        sqlx::query(&sql)
            .bind(chat_id)
            .bind(value)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

// --- Снапшоты статусов нод (дельта-вычисление, §5.4). ---

/// Репозиторий снапшотов статусов нод.
#[derive(Clone)]
pub struct SnapshotsRepo {
    pool: PgPool,
}

impl SnapshotsRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Загрузить прошлые статусы нод для набора кошельков: ip → status.
    pub async fn load_for_wallets(
        &self,
        wallets: &[String],
    ) -> Result<std::collections::HashMap<String, String>, StorageError> {
        if wallets.is_empty() {
            return Ok(Default::default());
        }
        let rows = sqlx::query(
            "SELECT node_ip, status FROM node_status_snapshots WHERE wallet_address = ANY($1)",
        )
        .bind(wallets)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| (r.get::<String, _>("node_ip"), r.get::<String, _>("status")))
            .collect())
    }

    /// Upsert одного снапшота (после тика). `last_seen_at` = NOW().
    pub async fn upsert(
        &self,
        node_ip: &str,
        wallet: &str,
        status: &str,
    ) -> Result<(), StorageError> {
        sqlx::query(
            "INSERT INTO node_status_snapshots
               (node_ip, wallet_address, status, benchmark_ok, last_seen_at)
             VALUES ($1, $2, $3, TRUE, NOW())
             ON CONFLICT (node_ip) DO UPDATE
               SET wallet_address = EXCLUDED.wallet_address,
                   status = EXCLUDED.status,
                   last_seen_at = NOW(),
                   updated_at = NOW()",
        )
        .bind(node_ip)
        .bind(wallet)
        .bind(status)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Удалить снапшоты нод, которых больше нет в сети (ушли офлайн), для кошелька.
    pub async fn remove_missing(
        &self,
        wallet: &str,
        present_ips: &[String],
    ) -> Result<(), StorageError> {
        sqlx::query(
            "DELETE FROM node_status_snapshots
             WHERE wallet_address = $1 AND NOT (node_ip = ANY($2))",
        )
        .bind(wallet)
        .bind(present_ips)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

// --- История алертов (аудит + дедупликация). ---

/// Репозиторий истории алертов.
#[derive(Clone)]
pub struct AlertsRepo {
    pool: PgPool,
}

impl AlertsRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Записать отправленный алерт.
    pub async fn record(
        &self,
        tg_chat_id: i64,
        wallet: &str,
        alert_type: &str,
        subject: &str,
    ) -> Result<(), StorageError> {
        sqlx::query(
            "INSERT INTO alerts_history (tg_chat_id, wallet_address, alert_type, subject)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(tg_chat_id)
        .bind(wallet)
        .bind(alert_type)
        .bind(subject)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Был ли уже такой алерт за последние `within_secs` секунд (дедуп).
    pub async fn recently_sent(
        &self,
        tg_chat_id: i64,
        alert_type: &str,
        subject: &str,
        within_secs: i64,
    ) -> Result<bool, StorageError> {
        let row = sqlx::query(
            "SELECT 1 AS hit FROM alerts_history
             WHERE tg_chat_id = $1 AND alert_type = $2 AND subject = $3
               AND sent_at > NOW() - make_interval(secs => $4)
             LIMIT 1",
        )
        .bind(tg_chat_id)
        .bind(alert_type)
        .bind(subject)
        .bind(within_secs as f64)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.is_some())
    }
}
