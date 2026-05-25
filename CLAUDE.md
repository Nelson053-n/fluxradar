# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Статус проекта

**Рабочий вертикальный срез MVP готов** (2026-05-25). Бэкенд собирается и проходит проверки, фронт-дашборд по мокапу тянет реальные данные кошелька через наш API, всё поднимается локально в прод-режиме (`./scripts/prod-up.sh` → http://localhost:8080).

Источники истины:
- `docs/TZ_Fluxnode_Service_v1.1.md` — полное техническое задание (требования).
- `docs/mockup_dashboard.html` — HTML-мокап главного экрана (дизайн).

### Что сделано
- Бэкенд workspace (api/worker/bot/domain/flux-client/storage) собирается; `cargo clippy -- -D warnings` чисто; тесты domain/flux-client проходят.
- `crates/api` — рабочий REST-прокси: `/health`, `/ready`, `/network/price`, `/wallet/{addr}/summary`, `/wallet/{addr}/nodes` с кэшем Redis. Сверено с Flux API (§13.1): 443 ноды тестового кошелька совпадают 1-в-1.
- `web/` — React+Vite+TS+Tailwind дашборд по мокапу (токены `:root` 1-в-1), реальные данные через `/api`.
- Прод локально: `infra/docker-compose.prod.yml` + `infra/nginx.prod.conf` + `scripts/prod-up.sh|prod-down.sh`.

### Что НЕ доделано (осознанно, для следующих этапов)
- `worker` — polling-тик пустой (заглушка); `bot` — команды-заглушки; репозитории `storage` (PG) — TODO.
- Формула earnings отложена (нужны golden-тесты, §13.10). Данные мокапа без покрытия API (uptime/earnings/benchmark) показаны как «—».
- TLS/HSTS/certbot, prod-Dockerfile'ы бинарей, rate-limit бота.

**Перед началом любой работы прочитай ТЗ целиком** (`docs/TZ_Fluxnode_Service_v1.1.md`), включая §0 «Инструкция для Claude» — оно задаёт обязательный порядок первых шагов и развилки, требующие согласования с пользователем.

### Обязательные первые артефакты (из §0 и §11 Этап 1)

Порядок важен — выполнять сверху вниз:

1. `docs/LICENSE_NOTES.md` — ✅ создан 2026-05-25. **Решение по лицензии принято.**
2. `docs/ADR-001-stack.md` — обоснование выбора **Axum vs actix-web** (Rust зафиксирован, выбирается только web-фреймворк).
3. Дальше — по §11 Этап 1 (MVP) сверху вниз.

### Решение по лицензии оригинала (важно)

Оригинал `2ndtlmining/Fluxnode` **не имеет лицензии** (режим «All Rights Reserved»). Финальное решение заказчика (2026-05-25): **пишем весь код с нуля**, оригинал — только «референс по фактам». Детали и точная граница — в `docs/LICENSE_NOTES.md`. Практические следствия:

- **Можно** черпать из оригинала факты/идеи (какие Flux-эндпоинты, смысл формул earnings) — idea–expression dichotomy.
- **Нельзя** копировать или построчно портировать их `.rs`/`.tsx` код, «переводить» исходник в свой, повторять структуру модулей 1-в-1.
- План §3.4 ТЗ (взять прокси `fluxnode_api_mask` за основу) — **НЕ активируется**.
- Метод чистоты: выписываем спецификацию словами → реализуем по докам Flux API → сверяем с публичным rich-list-кошельком (§13.1), а не с чужим кодом.
- Лицензия нашего репозитория **свободна** (не производное от чужого кода) — выбор за заказчиком, не блокирует.
- Брендинг оригинала не копируем (§2 ТЗ) — дизайн свой, из мокапа.
- Выбор Axum (ADR-001) стоит на технических основаниях, не на совместимости с их кодом.

## Что строим

Публичный self-hosted веб-сервис мониторинга Flux-нод (аналог fluxnode.app.runonflux.io) + Telegram-бот для алертов. Пользователь вводит публичный адрес кошелька (ZelID / ETH / BTC P2PKH) → видит дашборд флота нод. Telegram-привязка делается через бота командой `/link <wallet>`, **без регистрации на сайте**.

В мокапе сервис назван **FluxScope**, бот — **@FluxScopeBot**.

## Архитектура (зафиксирована в §3 ТЗ)

Cargo workspace из трёх бинарей + трёх библиотек. Разделение на 3 бинаря — намеренное, чтобы масштабировать их независимо:

```
flux-monitor/
├── Cargo.toml          # workspace
├── crates/
│   ├── api/            # бинарь: REST-прокси перед Flux API + бизнес-логика (Axum/actix)
│   ├── worker/         # бинарь: polling Flux API по расписанию + расчёт алертов (Tokio)
│   ├── bot/            # бинарь: Telegram-бот, команды + отправка алертов (teloxide)
│   ├── flux-client/    # библиотека: типизированный клиент Flux API
│   ├── domain/         # библиотека: модели, формулы (earnings, tier, uptime), общие типы
│   └── storage/        # библиотека: репозитории PostgreSQL + Redis (sqlx)
├── web/                # React 18 + Vite + TS + Tailwind + shadcn/ui
├── migrations/         # SQL-миграции (sqlx-cli)
├── infra/              # docker-compose, nginx.conf, prometheus.yml
└── docs/               # ТЗ, ADR, мокап
```

Стек зафиксирован — **не предлагать альтернативы**: Rust (бэкенд), PostgreSQL 16 + sqlx, Redis 7, React+Vite+Tailwind+shadcn/ui (фронт), Nginx (reverse-proxy/TLS), Docker Compose (деплой).

### Поток данных

Браузер → собственный REST API (`api/`) → кэш в Redis → Flux API (api.runonflux.io). Прямых запросов из браузера к Flux API **нет** — свой прокси нужен ради CORS, кэша (у владельца 100+ нод), rate-limit защиты, бизнес-логики и сокрытия токена бота (§3.3).

Параллельно `worker/` опрашивает Flux API по расписанию, сравнивает со снапшотами в PG (`node_status_snapshots`) и через `bot/` шлёт алерты. Ключевая оптимизация polling: один запрос `/daemon/viewdeterministicfluxnodelist` отдаёт весь список нод сети (~8000), который затем фильтруется по адресам подписчиков локально — **не N запросов на N нод** (§5.4).

### Схема БД

Определена в §5.5 ТЗ: `telegram_subscriptions`, `node_status_snapshots`, `alerts_history`, `wallet_tx_cursor`. Сайт **не имеет** доступа к таблице подписок — она используется только воркером и ботом.

## Дизайн-система (из мокапа)

Источник истины — `docs/mockup_dashboard.html` (ТЗ §7.4 называет его `docs/design-mockup.html` — это расхождение в имени, файл один). Извлекать CSS-переменные из `:root` 1-в-1 в `web/src/styles/tokens.css` как Tailwind theme extension.

- Шрифты: **Manrope** (текст) + **JetBrains Mono** (все числа, IP, адреса).
- Палитра: `--flux-primary #2B61D1`, `--flux-cyan #4FD7E8`, `--flux-glow #5B8DEF`, `--flux-purple #7B5BFF`. Тёмная тема по умолчанию.
- Стиль: glassmorphism, радиальные свечения на фоне, `rounded-2xl`, staggered `fadeUp`-анимации. Иконки — Lucide.
- Структура дашборда: header → wallet header → 2 ряда стат-карточек → Spotlight (3 карточки) → таблица нод → Telegram CTA → footer.

**Не менять без согласования:** палитру, пару шрифтов, общую структуру дашборда (§7.4).

## Команды разработки

> Структура проекта ещё не создана — команды ниже станут актуальны после каркаса workspace (§11 Этап 1). Это целевые команды из ТЗ §10.3 / §13.

Rust workspace:
```bash
cargo build                          # сборка всего workspace
cargo run -p api                     # запустить бинарь api (аналогично: -p worker, -p bot)
cargo test                           # все тесты
cargo test -p domain <test_name>     # один тест в конкретном crate
cargo fmt --check                    # проверка форматирования (требование CI)
cargo clippy -- -D warnings          # линт без предупреждений (критерий приёмки §13.9)
```

Миграции (sqlx-cli):
```bash
sqlx migrate add <name>              # новая миграция в migrations/
sqlx migrate run                     # применить
```

Фронтенд (в `web/`):
```bash
npm run dev                          # dev-сервер Vite
npm run build                        # production-сборка
npm run lint                         # ESLint
```

Инфраструктура:
```bash
docker compose -f infra/docker-compose.yml up    # PG + Redis + сервисы локально
```

## Критерии качества (из §13 ТЗ)

- `cargo clippy -- -D warnings` проходит чисто по всему workspace.
- Test coverage ≥ 60% на crates `domain` и `flux-client`.
- Формулы earnings портируются из `client/src/utils/` оригинала и **фиксируются тестами** (golden-тесты на известных значениях) — это доменная логика, которую нельзя сломать молча.
- Lighthouse Performance ≥ 85 на главной.
- Дашборд ≤ 3 сек при 100 нодах (P95); cached API ≤ 500 мс.

## Безопасность (из §9 ТЗ)

- **Никогда не запрашивать приватные ключи / seed-фразы** — только публичные адреса.
- Telegram Bot Token — только в `.env` сервера, в репозитории `.env.example` с плейсхолдером.
- Валидация адреса кошелька regex'ом (ZelID / ETH / BTC P2PKH) **до** обращения к Flux API: невалидный → 400 без похода наружу.
- Логи (`tracing` + JSON) не должны содержать `tg_user_id` и полные адреса на уровне ERROR.
- Rate-limit: API 60 req/min + 600/час с IP; бот 30 команд/мин с `tg_user_id`.

## Ограничения объёма (из §0 «Не делай»)

- Не предлагать альтернативный стек (Rust зафиксирован).
- Не добавлять регистрацию пользователей на сайте (сервис публичный).
- Не реализовывать геймификацию/ачивки в MVP (это backlog, Этап 3).
- Не менять палитру и шрифты из мокапа без согласования.
- На каждой развилке, не описанной в ТЗ, **задавать вопрос пользователю** перед принятием решения (§0).

## Открытые вопросы

§14 ТЗ содержит нерешённые вопросы (источник цены FLUX, геолокация, период хранения истории, parallel assets в MVP, webhook vs long polling). Согласовывать с пользователем, не решать молча.
