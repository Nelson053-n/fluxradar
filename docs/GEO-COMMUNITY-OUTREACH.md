# FluxRadar — где анонсировать в community (GEO brand authority)

**Дата:** 2026-05-29
**Цель:** внешние упоминания fluxradar.ru — единственный оставшийся слабый сигнал GEO
(техническая часть закрыта: dynamic rendering, schema, robots/sitemap/llms — всё есть).
**Метрика:** Ahrefs (дек 2025) — упоминания бренда коррелируют с AI-цитированием в 3× сильнее, чем бэклинки.

## ⚠️ Важно: не перепутать экосистему
Есть ДВА разных «Flux»:
- ❌ **FluxCD** (`fluxcd.io`) — Kubernetes GitOps, CNCF. **НЕ наш.** Их Slack/PR/ecosystem не подходят.
- ✅ **RunOnFlux** (`runonflux.com`) — криптовалюта FLUX, FluxNodes. **Наша экосистема.**

Прецедент: аналог fluxnode.app сделан community-членом «2ndTL Mining», есть и другие
community-tools (Flux View, JKTUNING Flux-Node-Tools). Community-инструменты тут приветствуются.

---

## Приоритет 1 — официальные каналы RunOnFlux (максимум аудитории операторов нод)

| Площадка | Ссылка | Как заходить |
|---|---|---|
| **Discord RunOnFlux** | https://discord.gg/runonflux | Главный хаб. Искать канал тип `#community-projects` / `#tools` / `#general`; спросить модераторов, куда постить community-tool. Самый тёплый канал для операторов нод. |
| **Telegram (community)** | https://t.me/runonflux | Официальный чат сообщества — короткий анонс со ссылкой + скрин. |
| **Twitter/X @RunOnFlux** | https://x.com/runonflux | Запостить свой твит с тегом @RunOnFlux + #Flux #FluxNode — есть шанс на ретвит официального аккаунта. |
| **LinkedIn Flux Official** | https://www.linkedin.com/company/flux-official | Тэг компании в посте (для b2b-видимости). |

## Приоритет 2 — каталоги / форумы (живут долго, индексируются AI)

| Площадка | Ссылка | Что делать |
|---|---|---|
| **Bitcointalk ANN-тред Flux** | https://bitcointalk.org/index.php?topic=2853688 | Пост в официальном треде Flux — индексируется поисковиками и AI, живёт годами. Хороший вечный сигнал. |
| **Reddit r/ZelOfficial** | https://www.reddit.com/r/ZelOfficial/ | Историческое имя проекта (Zel→Flux). Проверить активность; пост «community tool for node monitoring» с ссылкой. |
| **Medium (гостевой/свой)** | https://fluxofficial.medium.com | Свой пост-обзор «как мониторить флот Flux-нод» со ссылкой на FluxRadar — отличный citable-контент для AI. |

## Приоритет 3 — GitHub-видимость (машиночитаемо для AI)
- Репозиторий уже публичный: https://github.com/Nelson053-n/fluxradar
- Добавить **GitHub topics**: `flux`, `runonflux`, `fluxnode`, `node-monitoring`, `cryptocurrency`, `rust`, `react`.
- Заполнить **About/description** + ссылку на https://fluxradar.ru.
- Опционально: PR/issue в community-списки инструментов (искать «awesome-flux» / списки в Discord-пинах).

---

## Шаблон анонса (готов к постингу)

> **FluxRadar — мониторинг ваших Flux-нод по адресу кошелька**
> Сделал бесплатный self-hosted дашборд для операторов FluxNode: вводите публичный адрес
> кошелька (t1…/t3…) — видите весь флот (тиры, ранг в очереди выплат, доходность с Parallel
> Assets, гео, FluxOS), калькулятор доходности и Telegram-алерты о падении нод. Без регистрации,
> открытый код.
> 🔗 https://fluxradar.ru · код: https://github.com/Nelson053-n/fluxradar · бот: @FluxRadar_bot

(en-версия — взять из noscript/llms.txt сайта.)

## Замечания
- Перед постингом в Discord/Reddit — прочитать правила канала (многие требуют постить
  community-tools только в отведённый канал, иначе бан за «промо»).
- НЕ слать в fluxcd.io (другой проект).
- Эффект на GEO виден не сразу — AI-индексация упоминаний занимает недели. Через ~2-4 недели
  можно перепроверить `/geo quick` (балл Brand Authority должен вырасти).
