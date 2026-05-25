# План доработки FluxScope (мониторинг Flux-нод)

Статус: проектный план (read-only). Реализация — отдельным проходом.
Источники истины: `docs/TZ_Fluxnode_Service_v1.1.md`, `docs/mockup_dashboard.html`, `CLAUDE.md`.
Дизайн-инвариант (CLAUDE.md §7.4): палитра/шрифты/структура дашборда не меняются. Тёмная тема — дефолт, светлая — опциональный тоггл. Бренд-цвета (`--flux-*`) одинаковы в обеих темах.

---

## 0. Разрешённые развилки (обоснование решений)

§0 ТЗ требует согласовывать развилки. Инструмент вопросов недоступен — фиксирую решения с обоснованием; при несогласии правим до реализации.

- **Персист адреса:** URL query `?wallet=<addr>` как ведущий источник + `localStorage` как fallback «последний просмотренный». Причина: SPA-дашборд по чужому публичному кошельку — shareable-ссылка важнее всего (можно кинуть ссылку «вот мой флот»), а history/F5 чинятся бесплатно. localStorage хранит последний адрес для случая входа без query. Приоритет при загрузке: `?wallet=` → localStorage → `DEFAULT_ADDRESS`.
- **Навигация Header:** НЕ вводим react-router (стек без роутера, MVP — один экран). Делаем вкладки-якоря к секциям одной страницы: Dashboard→верх, All Nodes→секция таблицы, Guide→секция-гайд (новая статичная), Demo→загрузка `DEFAULT_ADDRESS` через onSubmit. Активная вкладка подсвечивается по scroll-spy (IntersectionObserver) или по клику. Это закрывает «вкладки не работают» без архитектурного расширения и сохраняет структуру мокапа.
- **i18n:** собственный лёгкий провайдер на Context + словари (без react-i18next). 3 языка, плоские ключи, без плюрализации/ICU — этого хватает для дашборда, ноль рантайм-зависимостей, проще держать Lighthouse ≥85. Персист выбора в localStorage, дефолт — по `navigator.language` с fallback en.
- **Темизация:** `data-theme="dark|light"` на `<html>`. Тёмные токены остаются в `:root`, светлые — в `:root[data-theme="light"]`. Тоггл пишет в localStorage; первичная установка — инлайн-скрипт в `index.html` до рендера (без FOUC). Бренд-цвета не дублируются по-разному.
- **Динамический фон:** усиление существующего `body::before/after` — медленно дрейфующие радиальные свечения (CSS `@keyframes` на `background-position`/`transform`, 30-60s, `prefers-reduced-motion: reduce` отключает). Без canvas/JS, чтобы не ронять Performance и не трогать палитру.
- **Earnings:** оценка из tier-reward × частота выплат, помечается в UI как «est.» (честное приближение). Точные значения reward фиксируются golden-тестами в `domain/earnings.rs`.
- **Геолокация:** ip-api.com batch (бесплатно, ≤100 IP/POST), агрессивный Redis-кэш (TTL ~24ч). Лениво, только для деталей/видимых строк, не для всего флота на summary.

---

## 1. Бэкенд (Rust)

### 1.1. flux-client: расширить разбор `viewdeterministicfluxnodelist`
Файл: `crates/flux-client/src/lib.rs`
- В `DeterministicNode` добавить уже-приходящие поля (deserialize, без новых запросов): `lastpaid: String`, `last_paid_height: i64`, `added_height: i64`, `collateral: String` (оставить `#[serde(default)]`). Тест `deserializes_real_node_shape` дополнить новыми полями.
- Добавить `flux_price_usd_with_change()` → `(usd, usd_24h_change)`: URL CoinGecko + `&include_24hr_change=true`, расширить `UsdPrice { usd, usd_24h_change: Option<f64> }`. Старый `flux_price_usd()` оставить как обёртку (совместимость).
- Сетевая сводка: `getfluxnodecount()` → `daemon/getfluxnodecount` → структура `{ total, cumulus_enabled, nimbus_enabled, stratus_enabled, ipv4 }`. Один дешёвый запрос.
- Ленивые дорогие методы (для эндпоинта детали ноды, см. 1.4): `node_status(ip)` → `daemon/getfluxnodestatus?ip=`, `benchmarks(ip)` (через ноду), `installed_apps(ip)` → `apps/installedapps/<ip>`. Каждый — отдельный метод, вызывается только из detail-эндпоинта.
- Геолокация: `geolocate_batch(ips: &[String])` → POST на `http://ip-api.com/batch` (массив до 100), парс `country, countryCode, city, query(ip)`.

### 1.2. domain/earnings.rs: формула + golden-тесты (§13.10)
Файл: `crates/domain/src/earnings.rs`
- Константы reward на выплату по tier (ФАКТ из требований; зафиксировать как источник в doc-комментарии): `CUMULUS ≈ 2.8125`, `NIMBUS ≈ 4.6875`, `STRATUS ≈ 11.25` FLUX.
- Частота выплат: вывести из сетевого размера tier-набора (нод в tier) и интервала блока — формализовать как функцию `expected_payouts_per_month(tier, tier_node_count)`. Если точную частоту не зафиксировать без доп. данных — взять консервативную модель «N выплат/месяц на ноду» и пометить как оценку; именно её замораживаем golden-тестом.
- API:
  - `monthly_estimate(tier, tier_node_count) -> f64 (FLUX)`
  - `fleet_monthly_estimate(tiers: &[Tier], network: &NetworkCounts) -> Money` (сумма по нодам, ×price)
  - `yearly = monthly * 12`
- Golden-тесты: зафиксировать выход на конкретных входах (1 cumulus при известном tier_count → ожидаемое FLUX/мес, с допуском). Тесты — гарантия §13.10, что формула не поедет молча. Покрытие domain должно остаться ≥60%.
- Возраст ноды: `node_age_secs(activesince, now) -> i64`, хелпер `format_age` оставить фронту (см. 2.x). В domain — только число.

### 1.3. domain/summary.rs: новые агрегаты
Файл: `crates/domain/src/summary.rs`
- Расширить `WalletSummary`: добавить `earnings: Earnings { monthly: Money, yearly: Money, estimated: true }`, `network: NetworkCounts { total, cumulus, nimbus, stratus }` (из getfluxnodecount), `oldest_node_age_secs: Option<i64>` (для Spotlight «Best Uptime»→«Longest-running»), опц. `price_change_24h`.
- Сигнатуру `build(...)` расширить: принять `&[DeterministicNode]`-производные (tier + activesince), `NetworkCounts`, `price`, `change_24h`. Обновить существующие тесты `aggregates_tiers_and_balance`, `empty_fleet`.
- Fleet Uptime / Bench Passed / Hosted Apps на summary-уровне: НЕ тянуть per-node (§5.4). Варианты:
  - Fleet Uptime — заменить на честную метрику флота из дешёвых данных: % нод в статусе CONFIRMED по списку (status есть в deterministic-списке как производное активности) ИЛИ «средний возраст флота». Помечается «est.».
  - Bench Passed / Hosted Apps — дорогие (per-node). На summary показывать «—» с честной подписью «per-node, open details», заполнять только в detail-эндпоинте. Это соответствует §5.4 и не врёт пользователю.

### 1.4. api: обогащение ответов + новый detail-эндпоинт
Файл: `crates/api/src/main.rs`
- `wallet_summary`: дёргать дополнительно `getfluxnodecount` (кэш отдельным ключом `network:count`, TTL 60), `flux_price_usd_with_change`, прокидывать `activesince` для возраста. Собирать расширенный `WalletSummary` из 1.3. Кэш-ключ summary не менять (TTL 30).
- `wallet_nodes`: в каждую ноду добавить дешёвые поля из deterministic-списка: `age_secs` (now-activesince), `last_paid` (lastpaid), `last_paid_height`, `collateral`. НЕ добавлять status/apps/location (дорогие) — они приходят из detail.
- `network_price`: вернуть `{ flux_usd, change_24h }`.
- НОВЫЙ: `GET /api/v1/node/{ip}/detail` — ленивая дорогая агрегация одной ноды: `getfluxnodestatus` (status, FluxOS version, bench version), `getbenchmarks` (cores/ram/ssd/eps/dws), `installedapps` (кол-во и список), геолокация по IP. Кэш-ключ `node:detail:{ip}` TTL 300; гео — `geo:{ip}` TTL 86400. Валидация формата IP перед запросом.
- НОВЫЙ (опц., для таблицы): `POST /api/v1/nodes/geo` с массивом IP → batch-гео из кэша/ip-api. Позволяет таблице заполнять Location без N запросов. Альтернатива — заполнять Location лениво при раскрытии строки.
- `upstream_error`/`bad_address` — переиспользовать. Per-node таймауты строже (детальные апстримы медленные).

### 1.5. Кэш
Файл: `crates/storage/src/cache.rs` — без структурных изменений (get/set_ex/ping достаточно). Только новые ключи и TTL (выше). При желании добавить `mget` для batch-гео — опционально.

### Критерии готовности бэка
- `cargo fmt --check`, `cargo clippy -- -D warnings` чисто по workspace.
- `cargo test -p domain` (включая новые golden-тесты earnings) и `-p flux-client` зелёные; покрытие domain/flux-client ≥60%.
- Ручная сверка: `/wallet/<DEFAULT>/summary` отдаёт earnings/network/age; `/node/<ip>/detail` отдаёт гео+bench+apps; повторный запрос — из кэша (≤500мс).

---

## 2. Фронт (React/Vite/TS/Tailwind)

### 2.1. Персист адреса + фикс F5 (баг #3)
Файлы: `web/src/App.tsx`, новый `web/src/lib/walletParam.ts`
- `walletParam.ts`: `readInitialAddress()` (читает `?wallet=`, иначе localStorage `fluxscope:wallet`, иначе DEFAULT), `writeAddress(addr)` (пишет в history `replaceState` + localStorage).
- В `App.tsx`: `useState(() => readInitialAddress())` вместо `useState(DEFAULT_ADDRESS)`. Слить два эффекта первичной загрузки в один, грузящий начальный адрес (не хардкод DEFAULT). `handleSubmit` дополнительно вызывает `writeAddress`. Слушать `popstate` (назад/вперёд браузера) → перезагрузка адреса. Это устраняет сброс на дефолт при F5.

### 2.2. Навигация Header (баг #2)
Файлы: `web/src/components/Header.tsx`, `web/src/App.tsx`, новая секция Guide.
- Заменить `NAV` строки на объекты `{ key, labelKey, target }`. Dashboard/All Nodes/Guide → `onClick` со `scrollIntoView` к `id` секций (`#dashboard-top`, `#nodes`, `#guide`). Demo → `onSubmit(DEFAULT_ADDRESS)`.
- Убрать `href="#"`; для доступности использовать `<button>` или `<a href="#nodes">` с onClick+preventDefault. Активная вкладка — состояние, обновляемое scroll-spy (IntersectionObserver на секциях) либо по клику.
- В `App.tsx` навесить `id` на контейнеры секций; добавить статичную секцию Guide (как короткий how-to: ввести адрес → смотреть флот → привязать ТГ). Контент — через i18n-ключи.

### 2.3. Темизация (требование #4)
Файлы: `web/src/styles/tokens.css`, `web/src/index.css`, `web/index.html`, новый `web/src/lib/theme.ts`, новый `web/src/components/ThemeToggle.tsx`
- `tokens.css`: оставить тёмные токены в `:root`; добавить блок `:root[data-theme="light"] { ... }` со светлыми bg/text/border значениями. Бренд `--flux-*` и `--success/--warning/--danger` оставить одинаковыми (инвариант). Светлый фон — мягкий (например `#F4F7FF`), карточки — полупрозрачное светлое стекло, текст — тёмный; подобрать так, чтобы glassmorphism читался.
- `theme.ts`: `getTheme()/setTheme()/toggleTheme()` (localStorage `fluxscope:theme`, дефолт `dark`).
- `index.html`: инлайн-скрипт в `<head>` ставит `document.documentElement.dataset.theme` из localStorage ДО рендера (анти-FOUC).
- `ThemeToggle.tsx`: иконка солнце/луна, в Header рядом с языковым переключателем; стиль — как `toggle-pill` в мокапе. Тёмная по умолчанию.
- `index.css`: места с хардкодом тёмных rgba (например `bg-[rgba(20,28,48,...)]` в Header/WalletHeader, `body` background) — заменить на токены/перекрыть для light через `[data-theme=light]` правила, чтобы светлая тема не выглядела сломанной. Tailwind-классы с literal rgba придётся либо токенизировать, либо добавить data-theme override в CSS.

### 2.4. Динамический фон (требование #5)
Файл: `web/src/index.css`
- Добавить `@keyframes drift` (медленный сдвиг позиций/масштаба радиальных градиентов) на `body::before`; лёгкая параллакс-пульсация прозрачности. Длительность 40-60s, `ease-in-out infinite alternate`.
- Обернуть в `@media (prefers-reduced-motion: no-preference)`; при reduce — статика. Без JS, без новой палитры.

### 2.5. i18n ru/en/zh (требование #6)
Файлы: новые `web/src/i18n/index.tsx` (Context+provider+`useT()`), `web/src/i18n/{en,ru,zh}.ts`, новый `web/src/components/LangSwitch.tsx`; правки во всех компонентах с текстом.
- Провайдер: `I18nProvider` хранит lang (localStorage `fluxscope:lang`, дефолт из `navigator.language`), отдаёт `t(key)`. Тип ключей — единый union из en-словаря (компайл-тайм проверка полноты ru/zh).
- Словари: вынести все строки UI (Header nav, лейблы карточек «Total Nodes/Wallet Balance/Fleet Uptime/Bench Passed/Hosted Apps/Estimated Earnings/Monthly/Yearly», Spotlight ribbons/titles, таблица заголовки/тосты, Telegram CTA, Footer, состояния Skeleton/Error/Empty, Guide). zh — китайский (Simplified).
- `LangSwitch.tsx`: компактный селектор RU/EN/中文 в Header.
- `main.tsx`: обернуть `<App/>` в `<I18nProvider>`.
- Числа/деньги форматировать через `Intl` с учётом locale там, где уместно (но моно-числа FLUX/IP оставить как есть для дизайна).

### 2.6. Заполнение данных (требования #1, #7)
Файлы: `web/src/api/client.ts`, `App.tsx`, `EarningsCard.tsx`, `Spotlight.tsx`, `NodesTable.tsx`, `WalletHeader.tsx`, новый `web/src/components/NodeDetailDrawer.tsx`, `lib/format.ts`.
- `client.ts`: расширить типы под новый бэк: `WalletSummary` += `earnings{monthly{flux,usd},yearly{...},estimated}`, `network{...}`, `oldest_node_age_secs`, `price_change_24h`. `FluxNode` += `age_secs`, `last_paid`, `last_paid_height`, `collateral`. Добавить `fetchNodeDetail(ip)` и (опц.) `fetchNodesGeo(ips)`. `PriceResponse` += `change_24h`.
- `lib/format.ts`: добавить `formatAge(secs)` → «427d 12h», `formatChange(pct)` → «+3.2%» с цветом, country→flag-emoji map (или из бэка countryCode).
- `EarningsCard.tsx`: принять `earnings`, показать Monthly/Yearly FLUX + ≈USD, бейдж «est.»; убрать хардкод `—`.
- `Spotlight.tsx`: «Best Uptime»→заполнить из `oldest_node_age_secs` (или из nodes по min activesince) как «Longest-running». «Highest Rank» уже реальный. «Most Hosted» — оставить ленивым (заполняется только если открыли детали; иначе честное «open details»).
- `App.tsx` карточки ряда 2: Wallet Balance (реальный, с usd по актуальной цене). Fleet Uptime → метрика флота из summary (% CONFIRMED или средний возраст, бейдж est.). Bench Passed / Hosted Apps → честная подпись «per-node · open details» вместо голого `—`.
- `NodesTable.tsx`: колонки Status/Uptime(age)/Apps/Location/FluxOS. Age — из `age_secs` сразу (дёшево). Status/Apps/Location/FluxOS — лениво: клик по строке открывает `NodeDetailDrawer` (fetchNodeDetail), либо подгрузка гео батчем для видимой страницы. Заголовок мокапа «search by IP or location» — поиск по IP уже есть; добавить по city после загрузки гео. Фильтр-табы оставить (tiers); можно добавить Online/Offline когда статус доступен.
- `NodeDetailDrawer.tsx` (новый): боковая панель/модал с гео, FluxOS/bench версии, железо (cores/ram/ssd/eps/dws), список приложений. Дизайн — те же glass-токены. Это «ленивые дорогие данные» из 1.4.
- `WalletHeader.tsx`: «Auto-refresh · 60s» — либо реализовать реальный интервал-рефреш (setInterval 60s, пауза при скрытой вкладке), либо честно подписать. Цена в Header — добавить ±24ч change рядом (`formatChange`).

### Критерии готовности фронта
- `npm run build` (tsc -b + vite) без ошибок типов; `npm run lint` чисто.
- Ручная проверка через прод :8080 (`./scripts/prod-up.sh`):
  1. F5 на `?wallet=<addr>` сохраняет адрес (баг #3 закрыт).
  2. Все 4 вкладки Header реагируют (скролл/Demo) (баг #2 закрыт).
  3. Тоггл темы переключает dark/light без FOUC, бренд-цвета на месте.
  4. Языки ru/en/zh переключаются и персистятся.
  5. Карточки заполнены (earnings est., age, network), нет «голых —» без объяснения; дорогие поля — в drawer.
  6. Фон анимируется, при prefers-reduced-motion статичен.
- Lighthouse Performance ≥85 на главной (§13.7).

---

## 3. Порядок работ и параллелизм

Параллельные дорожки (независимы):
- **Дорожка A (бэк, данные):** 1.1 → 1.2 → 1.3 → 1.4/1.5. Earnings golden-тесты (1.2) можно делать первыми независимо.
- **Дорожка B (фронт, инфра-фичи):** 2.1 (персист/F5), 2.2 (навигация), 2.3 (темы), 2.4 (фон), 2.5 (i18n) — не зависят от бэка, делаются параллельно с A.
- **Дорожка C (фронт, данные):** 2.6 — зависит от A (новые поля API). Контракт типов в `client.ts` согласовать с 1.3/1.4 заранее, тогда C стартует на моках и сходится с A в конце.

Рекомендуемая последовательность мерджа: golden-тесты earnings (1.2) → расширение API-контракта (1.1/1.3/1.4) ↔ обновление `client.ts` → фронт-инфра B → фронт-данные C → drawer/ленивые → финальная сверка :8080 + Lighthouse.

## 4. Инварианты и риски
- Не менять палитру/шрифты/структуру (CLAUDE.md §7.4); светлая тема перекрывает только bg/text/border, бренд `--flux-*` неизменны.
- §5.4: никаких N запросов на N нод для summary/таблицы; дорогое — лениво + Redis-кэш.
- Earnings — приближение; в UI всегда «est.», точность зафиксирована golden-тестами (§13.10).
- ip-api бесплатный (rate-limit) — батч + суточный кэш; деградация при недоступности (Location пустой, не ломает таблицу).
- Светлая тема: literal-rgba в Tailwind-классах (Header/WalletHeader) потребуют CSS-override через `[data-theme=light]` либо токенизации — заложить время.

## 5. Критические файлы
- crates/api/src/main.rs — обогащение summary/nodes, новый detail-эндпоинт, кэш-ключи.
- crates/flux-client/src/lib.rs — новые поля deterministic-списка, getfluxnodecount, detail/гео/price-change.
- crates/domain/src/earnings.rs + summary.rs — формула earnings + golden-тесты, расширенная сводка.
- web/src/App.tsx — персист адреса/F5, секции/навигация, проводка новых данных, провайдеры.
- web/src/styles/tokens.css + web/src/index.css — светлая тема (data-theme) + динамический фон.
