-- Начальная схема FluxScope (§5.5 ТЗ).
-- Применяется через `sqlx migrate run`.

-- Привязки TG ↔ кошелёк. Сайт к этой таблице доступа НЕ имеет —
-- только worker и bot (§5.1 ТЗ).
CREATE TABLE telegram_subscriptions (
    id              BIGSERIAL PRIMARY KEY,
    tg_chat_id      BIGINT NOT NULL,
    tg_user_id      BIGINT NOT NULL,
    wallet_address  TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    muted_until     TIMESTAMPTZ,
    settings_json   JSONB NOT NULL DEFAULT '{}'::jsonb,  -- какие типы алертов включены
    UNIQUE (tg_chat_id, wallet_address)
);
CREATE INDEX idx_telegram_subscriptions_wallet ON telegram_subscriptions(wallet_address);

-- Снапшоты статусов нод для дельта-вычисления (§5.4).
CREATE TABLE node_status_snapshots (
    node_ip         TEXT PRIMARY KEY,
    wallet_address  TEXT NOT NULL,
    status          TEXT NOT NULL,         -- CONFIRMED / EXPIRED / DOS / OFFLINE
    benchmark_ok    BOOLEAN NOT NULL,
    flux_os_version TEXT,
    bench_version   TEXT,
    last_seen_at    TIMESTAMPTZ NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_node_status_wallet ON node_status_snapshots(wallet_address);

-- История отправленных алертов (аудит + дедупликация).
CREATE TABLE alerts_history (
    id              BIGSERIAL PRIMARY KEY,
    tg_chat_id      BIGINT NOT NULL,
    wallet_address  TEXT NOT NULL,
    alert_type      TEXT NOT NULL,         -- 'node_offline' / 'benchmark_failed' / 'incoming_tx' / ...
    subject         TEXT,                  -- IP ноды / txid
    payload_json    JSONB,
    sent_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_alerts_history_subject ON alerts_history(alert_type, subject, sent_at DESC);

-- Последняя обработанная транзакция на кошелёк (для алертов группы B, §5.2).
CREATE TABLE wallet_tx_cursor (
    wallet_address  TEXT PRIMARY KEY,
    last_txid       TEXT,
    last_block      BIGINT,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
