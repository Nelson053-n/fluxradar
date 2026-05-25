# План: доведение FluxScope до рабочего состояния

## Context

После вертикального среза остались проблемы, которые пользователь хочет устранить:
половина карточек дашборда показывает «—», навигация в Header — мёртвые `<a href="#">`,
при F5 теряется введённый адрес (всегда грузится `DEFAULT_ADDRESS`), нет светлой темы,
нет анимированного фона, нет переводов. Цель — рабочий, заполненный данными дашборд
с темой, динамическим фоном и i18n (ru/en/zh), по лучшим практикам.

Корневые причины (из исследования):
- Адрес хранится только в `useState(DEFAULT_ADDRESS)` без персиста → баг F5.
- `Header.tsx:11` NAV = массив ссылок `<a href="#">` без onClick → «вкладки не работают».
- Поля uptime/earnings/status/apps/location/fluxos захардкожены как `NA` — API их не отдаёт.
- `tokens.css` — только тёмная тема, нет механизма темизации; i18n отсутствует полностью.

## Согласованные решения
- **Персист адреса:** URL `?wallet=<addr>` (ведущий, shareable) + `localStorage` (fallback). Приоритет: `?wallet=` → localStorage → `DEFAULT_ADDRESS`.
- **Язык по умолчанию:** автоопределение из `navigator.language` (ru→ru, zh→zh, иначе en), выбор пользователя персистится в localStorage.
- **Детали ноды:** лениво по клику (drawer) — дорогие данные (железо/apps/гео/точный статус) тянутся по запросу с Redis-кэшем. Соблюдает §5.4.
- **Навигация:** без роутера — вкладки-якоря к секциям (scrollIntoView) + scroll-spy; «Demo» = дефолтный кошелёк; «Guide» = новая секция-инструкция.
- **i18n:** свой лёгкий Context-провайдер + плоские словари ru/en/zh (без react-i18next — ноль рантайм-зависимостей).
- **Темизация:** `data-theme` на `<html>`, тёмные токены в `:root`, светлые в `:root[data-theme=light]`. Бренд `--flux-*` неизменны (§7.4). Анти-FOUC инлайн-скрипт в `index.html`.
- **Фон:** CSS-анимация существующих радиальных свечений (медленный дрейф 40–60s), отключается при `prefers-reduced-motion`. Без canvas/JS, без новой палитры.

## Бэкенд (Rust)

1. **`crates/flux-client/src/lib.rs`**
   - Расширить `DeterministicNode` уже-приходящими полями: `lastpaid`, `last_paid_height`, `added_height`, `collateral` (БЕЗ доп. запросов).
   - `flux_price_usd` → вернуть и `usd_24h_change` (CoinGecko `include_24hr_change=true`).
   - Новый дешёвый метод `network_count()` → `getfluxnodecount` (сводка сети).
   - Ленивые методы для детали ноды: `node_status(ip)`, `node_benchmark(ip)`, `node_apps(ip)`; гео — `geo_batch(ips)` через ip-api batch (кэш отдельно).

2. **`crates/domain/src/earnings.rs`** — формула оценки earnings из tier-reward
   (CUMULUS≈2.8125 / NIMBUS≈4.6875 / STRATUS≈11.25 FLUX за выплату) × частота выплат;
   `node_age_secs(activesince)`. **Golden-тесты** замораживают значения (§13.10). Поле помечается «est.».

3. **`crates/domain/src/summary.rs`** — `WalletSummary` += `earnings {monthly, yearly}`,
   `network {total, cumulus, nimbus, stratus}`, `oldest_node_age_secs`, `price_change_24h`,
   `fleet_uptime_pct` (честная метрика: доля нод в норме / нормализованный возраст).

4. **`crates/api/src/main.rs`**
   - Обогатить `/wallet/{addr}/summary` и `/wallet/{addr}/nodes` (age, last_paid из deterministic-списка, status).
   - `/network/price` += `change_24h`.
   - **Новый `GET /node/{ip}/detail`** — гео+bench+apps+status, Redis-кэш (detail TTL 300с, гео TTL 86400с).

## Фронт (`web/`)

- **`web/src/lib/walletParam.ts`** (новый) — чтение/запись `?wallet=` + localStorage, нормализация.
- **`web/src/App.tsx`** — инициализация адреса из walletParam (фикс F5), `id`-секции для якорей,
  проводка новых полей, провайдеры Theme + i18n, секция Guide.
- **`web/src/components/Header.tsx`** — рабочие вкладки (scrollIntoView/Demo), `ThemeToggle`,
  `LangSwitch`, бейдж ±24ч у цены.
- **`web/src/i18n/{index.tsx, en.ts, ru.ts, zh.ts}`** (новые) — провайдер + словари; тип ключей из `en` (контроль полноты).
- **`web/src/styles/tokens.css` + `web/src/index.css`** — светлый набор токенов через `data-theme`; анимация фона.
- **`web/index.html`** — анти-FOUC инлайн-скрипт (тема+язык до рендера).
- **Карточки/данные:** `EarningsCard` (Monthly/Yearly est.), `Spotlight` (Best Uptime ← oldest age,
  Most Hosted — из detail при наличии), `NodesTable` (+колонка Age сразу; Status/Apps/Location/FluxOS — лениво),
  **`NodeDetailDrawer.tsx`** (новый) для дорогих данных, `api/client.ts` под новый контракт.

## Порядок работ (A и B параллельны; C после A)
- **A (бэк-данные):** golden-тесты earnings → поля flux-client → summary/network → api + `/node/{ip}/detail`.
- **B (фронт-инфра):** персист адреса/F5, навигация, темы+фон, i18n — независимы от бэка.
- **C (фронт-данные):** контракт `client.ts` согласовать заранее; заполнение карточек, drawer.

## Критерии готовности
- Бэк: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings` чисто; `cargo test` зелёный (вкл. golden earnings).
- Фронт: `npm run build` + `npm run lint` чисто.
- Ручная на `:8080` (`./scripts/prod-up.sh`): F5 хранит адрес и `?wallet=` шарится; все вкладки реагируют;
  тоггл темы без FOUC, бренд-цвета сохранены; ru/en/zh переключаются и персистятся; карточки заполнены
  (без «голых —»); фон анимируется; клик по ноде открывает drawer с гео/железом/apps.

## Критические файлы
- `crates/flux-client/src/lib.rs`
- `crates/domain/src/earnings.rs`, `crates/domain/src/summary.rs`
- `crates/api/src/main.rs`
- `web/src/App.tsx`, `web/src/components/Header.tsx`, `web/src/components/NodesTable.tsx`
- `web/src/i18n/*` (новое), `web/src/styles/tokens.css`, `web/src/index.css`, `web/index.html`
