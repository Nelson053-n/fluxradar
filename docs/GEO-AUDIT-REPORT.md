# GEO Audit Report: FluxRadar

**Audit Date:** 2026-05-28
**URL:** https://fluxradar.ru
**Business Type:** SaaS / Web tool (публичный self-hosted монитор Flux-нод; одностраничный React SPA)
**Pages Analyzed:** 1 (сайт — одностраничное приложение; уникальных серверных HTML-страниц нет)

---

## Executive Summary

**Overall GEO Score: 5/100 (Critical)**

FluxRadar практически невидим для AI-систем. Сайт — это клиентский React SPA (Vite) без
server-side rendering и без prerendering: сырой HTML главной = 1605 байт и содержит только
`<title>` и пустой `<div id="root">`. Весь контент (дашборд, гайд, калькулятор, описания)
рендерится JavaScript-ом в браузере, поэтому AI-краулеры (GPTBot, ClaudeBot, PerplexityBot)
и поисковые AI-обзоры видят **только заголовок страницы**. Дополнительно отсутствуют
`robots.txt`, `sitemap.xml`, `llms.txt`, meta description, Open Graph и любая schema.org-разметка,
а у домена нулевое присутствие во внешних источниках. Главная задача — сделать контент видимым
без JS (prerender/SSG) и добавить базовые мета-сигналы.

### Score Breakdown

| Category | Score | Weight | Weighted Score |
|---|---|---|---|
| AI Citability | 5/100 | 25% | 1.25 |
| Brand Authority | 3/100 | 20% | 0.60 |
| Content E-E-A-T | 5/100 | 20% | 1.00 |
| Technical GEO | 15/100 | 15% | 2.25 |
| Schema & Structured Data | 0/100 | 10% | 0.00 |
| Platform Optimization | 2/100 | 10% | 0.20 |
| **Overall GEO Score** | | | **5.3/100** |

---

## Critical Issues (Fix Immediately)

1. **Контент не индексируется — SPA без SSR/prerender.**
   Сырой HTML (`curl https://fluxradar.ru/`) = 1605 байт, только `<title>` + `<div id="root">`.
   Проверка с `User-Agent: GPTBot` → те же 1606 байт пустого shell. AI-системы и поисковые
   AI-обзоры не видят ни описания продукта, ни гайда, ни калькулятора, ни данных дашборда.
   **Фикс:** добавить prerendering статической главной (минимум) — `vite-plugin-prerender` /
   `vite-plugin-ssg`, либо отдавать ботам предрендеренный HTML на nginx. Как минимум — вписать
   осмысленный текстовый контент (что за продукт, для кого, фичи) прямо в `index.html`.

2. **Полное отсутствие structured data (schema.org).**
   0 JSON-LD блоков. Нет `Organization`, `SoftwareApplication`, `FAQPage`, `WebSite`.
   AI не распознаёт FluxRadar как сущность (entity).
   **Фикс:** добавить JSON-LD `Organization` + `SoftwareApplication` в `index.html`.

3. **Бренд не распознаётся как сущность ни одной AI-системой.**
   Веб-поиск по «FluxRadar / fluxradar.ru / flux node monitoring» не возвращает наш домен —
   выдаются FluxCD, FluxCloud, fluxnode.app. Нулевые внешние упоминания.

---

## High Priority Issues (Fix Within 1 Week)

1. **Нет `robots.txt`.** `/robots.txt` → 200, но `content_type: text/html` (SPA-fallback
   отдаёт index.html). Реального файла нет → нет директив для AI-краулеров и нет ссылки на sitemap.
2. **Нет `llms.txt`.** `/llms.txt` → отдаёт тот же HTML-shell. Файл отсутствует.
3. **Нет `sitemap.xml`.** `/sitemap.xml` → HTML-shell, не XML.
4. **Нет meta description.** 0 в `index.html` — AI и поисковики не имеют сниппета о продукте.
5. **Нет Open Graph / Twitter Card.** 0 OG-тегов → плохие превью при шеринге, слабый сигнал сущности.
6. **Нет question-answering / FAQ контента в HTML.** Гайд есть в JS, но недоступен краулеру.

---

## Medium Priority Issues (Fix Within 1 Month)

1. **SPA-fallback отдаёт `index.html` со статусом 200 на ЛЮБОЙ путь** (включая `/robots.txt`,
   `/sitemap.xml`, несуществующие URL). Это маскирует 404 и мешает корректной индексации.
   Реальные статические файлы (robots/sitemap/llms) должны отдаваться nginx до SPA-fallback.
2. **Нет canonical URL** в `<head>`.
3. **Нет страницы/секции «About» с авторством и контекстом проекта** в индексируемом виде.
4. **Нулевое присутствие на Wikipedia / Reddit / YouTube** под брендом FluxRadar.

---

## Low Priority Issues (Optimize When Possible)

1. `lang="ru"` зашит в HTML, хотя интерфейс ru/en — для en-аудитории сигнал языка неточен.
2. Favicon/иконки на месте (это плюс) — но нет `og:image` для соц-превью.
3. Нет `<noscript>`-фоллбэка с кратким описанием продукта.

---

## Category Deep Dives

### AI Citability (5/100)
AI-системы извлекают текстовые блоки из HTML. Здесь извлекать нечего: единственный
текст — `<title>FluxRadar — Node Dashboard</title>`. Весь ценный контент (объяснение
механики Flux-нод, гайд, FAQ-подобные блоки, калькулятор доходности) живёт в React-бандле
и появляется только после исполнения JS, которое AI-краулеры обычно не выполняют.
Потенциал высокий (контент содержательный), но он недостижим без prerender/SSR.

### Brand Authority (3/100)
Веб-поиск не находит `fluxradar.ru`. Конкуренты-аналоги (fluxnode.app.runonflux.io,
cloud.runonflux.com) индексируются и упоминаются. Нет обратных ссылок, нет упоминаний на
Reddit/форумах Flux, нет Wikipedia. Для новой сущности это ожидаемо, но это основной
долгосрочный рычаг.

### Content E-E-A-T (5/100)
В приложении есть полезный экспертный контент (README на GitHub хороший, гайд в UI), но для
краулера E-E-A-T-сигналов нет: ни автора, ни описания, ни дат, ни источников в индексируемом
HTML. GitHub-репозиторий (github.com/Nelson053-n/fluxradar) — единственный текстовый актив,
видимый машинам.

### Technical GEO (15/100)
Плюсы: HTTPS работает, валидный сертификат, быстрый отклик, корректный `<title>`, есть favicon,
preconnect к шрифтам. Минусы (критичные): нет SSR/prerender, нет robots.txt/sitemap.xml/llms.txt
(SPA-fallback отдаёт HTML вместо них), пустой `#root`. Балл держится только на базовой
тех-исправности транспорта.

### Schema & Structured Data (0/100)
Разметки нет вообще. Ни одного JSON-LD. Для SaaS-инструмента критичны как минимум
`Organization`, `SoftwareApplication`, `WebSite` (+ `SearchAction`), `FAQPage` для гайда.

### Platform Optimization (2/100)
Нет присутствия на платформах, на которых обучаются и которые цитируют AI-модели
(Wikipedia, Reddit, YouTube, Stack Overflow, профильные Flux-сообщества). Есть только
публичный GitHub-репозиторий — это единственный машиночитаемый источник о продукте.

---

## Quick Wins (Implement This Week)

1. **Вписать осмысленный контент прямо в `web/index.html`**: `<meta name="description">`,
   видимый текст в `<noscript>` (что такое FluxRadar, для кого, ключевые фичи). Мгновенно даёт
   AI хоть какой-то индексируемый контент. Ожидаемый эффект: из «невидим» → «есть базовое описание».
2. **Добавить JSON-LD `Organization` + `SoftwareApplication`** в `index.html` (статический блок).
3. **Создать реальные `public/robots.txt`** (Allow всем + AI-краулерам, ссылка на sitemap) и
   **`public/sitemap.xml`** — настроить nginx отдавать их ДО SPA-fallback.
4. **Добавить Open Graph + Twitter Card** (`og:title/description/image/url`) в `index.html`.
5. **Добавить `public/llms.txt`** с кратким описанием продукта, ключевыми разделами и ссылкой
   на GitHub/гайд — формат, который специально читают AI-системы.

## 30-Day Action Plan

### Week 1: Базовая видимость без JS
- [ ] Meta description + `<noscript>`-описание в `web/index.html`
- [ ] JSON-LD `Organization` + `SoftwareApplication`
- [ ] Open Graph / Twitter Card + `og:image`
- [ ] Реальные `robots.txt`, `sitemap.xml`, `llms.txt` (отдача через nginx до SPA-fallback)

### Week 2: Prerender контента
- [ ] Подключить prerendering главной (`vite-plugin-ssg` / `vite-plugin-prerender` или nginx-prerender для ботов)
- [ ] Вынести описание продукта, FAQ/гайд в предрендеренный HTML
- [ ] Добавить `FAQPage` schema на основе гайда
- [ ] Canonical URL, уточнить `lang`

### Week 3: E-E-A-T и контент
- [ ] Индексируемая секция «About»: что за проект, автор, открытый код (ссылка на GitHub)
- [ ] Текстовые объяснения механики (tier-награды, Parallel Assets, расчёт доходности) в HTML
- [ ] Даты обновления, ссылки на офиц. источники Flux

### Week 4: Brand authority (внешние сигналы)
- [ ] Пост/тред в сообществах Flux (Discord/Reddit/форум) с ссылкой на инструмент
- [ ] Описание в README + topics на GitHub (уже частично есть)
- [ ] Заявка в каталоги Flux-экосистемы / списки community-инструментов

---

## Appendix: Pages Analyzed

| URL | Title | GEO Issues |
|---|---|---|
| https://fluxradar.ru/ | FluxRadar — Node Dashboard | SPA-shell (нет SSR), 0 schema, 0 meta desc, 0 OG |
| https://fluxradar.ru/robots.txt | (SPA-fallback HTML) | файл отсутствует — отдаётся index.html |
| https://fluxradar.ru/sitemap.xml | (SPA-fallback HTML) | файл отсутствует |
| https://fluxradar.ru/llms.txt | (SPA-fallback HTML) | файл отсутствует |

**Метод:** прямой `curl` (raw HTML, точнее JS-рендера), проверка с `User-Agent: GPTBot`,
веб-поиск бренда. Сайт — одностраничное приложение, поэтому отдельных серверных страниц нет;
все «маршруты» (`/`, `?wallet=`) отдают один и тот же `index.html`.
