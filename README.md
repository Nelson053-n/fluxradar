# FluxScope

Self-hosted веб-сервис мониторинга Flux-нод с Telegram-уведомлениями.
Вводишь публичный адрес кошелька → видишь дашборд флота нод. Алерты — через бота, без регистрации.

Спецификация: [`docs/TZ_Fluxnode_Service_v1.1.md`](docs/TZ_Fluxnode_Service_v1.1.md).
Дизайн-референс: [`docs/mockup_dashboard.html`](docs/mockup_dashboard.html).

## Архитектура

Cargo workspace (§3.2 ТЗ):

| Крейт | Тип | Назначение |
|---|---|---|
| `crates/api` | бинарь | REST-прокси перед Flux API + бизнес-логика (Axum, см. [ADR-001](docs/ADR-001-stack.md)) |
| `crates/worker` | бинарь | polling Flux API + расчёт алертов (Tokio) |
| `crates/bot` | бинарь | Telegram-бот: команды + отправка алертов (teloxide) |
| `crates/flux-client` | lib | типизированный клиент Flux API |
| `crates/domain` | lib | модели, формулы (earnings, tier), общие типы |
| `crates/storage` | lib | репозитории PostgreSQL (sqlx) + кэш Redis (bb8) |

Данные: PostgreSQL 16 (привязки TG↔кошелёк, снапшоты, история алертов), Redis 7 (кэш Flux API, rate-limit, дедуп).

## Локальный запуск

Нужно: Rust toolchain (cargo/rustc), Docker, [sqlx-cli](https://crates.io/crates/sqlx-cli).

```bash
# 1. Поднять PG + Redis
cp .env.example .env          # заполнить TELEGRAM_BOT_TOKEN
docker compose -f infra/docker-compose.yml up -d

# 2. Применить миграции
sqlx migrate run

# 3. Запустить сервисы (в разных терминалах)
cargo run -p api
cargo run -p worker
cargo run -p bot
```

ENV-переменные описаны в [`.env.example`](.env.example).

## Прод-режим локально

Полный вертикальный срез (Nginx отдаёт собранный фронт + проксирует `/api` на release-бинарь, тот ходит в PG/Redis/Flux API):

```bash
./scripts/prod-up.sh      # PG+Redis → web/dist → release-api(:5049) → Nginx(:8080)
# дашборд: http://localhost:8080
./scripts/prod-down.sh    # остановить Nginx + api (PG/Redis оставить)
./scripts/prod-down.sh --all   # остановить всё (данные в volume сохраняются)
```

Конфиги прод-режима: [`infra/docker-compose.prod.yml`](infra/docker-compose.prod.yml), [`infra/nginx.prod.conf`](infra/nginx.prod.conf).

## Разработка

```bash
cargo build                    # сборка workspace
cargo test                     # тесты
cargo fmt --check              # форматирование (CI)
cargo clippy -- -D warnings    # линт без предупреждений (§13.9)
```

## Лицензия

Код пишется с нуля; оригинал Fluxnode используется только как референс по фактам.
Выбор лицензии — открытый вопрос (см. [`docs/LICENSE_NOTES.md`](docs/LICENSE_NOTES.md)).
