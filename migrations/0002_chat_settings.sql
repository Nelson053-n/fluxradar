-- Настройки чата Telegram: язык интерфейса бота и тогглы типов уведомлений.
-- На уровне чата (не per-wallet): один язык и один набор тогглов на чат.
CREATE TABLE chat_settings (
    tg_chat_id      BIGINT PRIMARY KEY,
    lang            TEXT NOT NULL DEFAULT 'en',     -- 'en' / 'ru' / 'zh'
    alert_offline   BOOLEAN NOT NULL DEFAULT TRUE,  -- нода ушла офлайн
    alert_online    BOOLEAN NOT NULL DEFAULT TRUE,  -- нода вернулась онлайн
    alert_status    BOOLEAN NOT NULL DEFAULT TRUE,  -- смена статуса (CONFIRMED/DOS/...)
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
