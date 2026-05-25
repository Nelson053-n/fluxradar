// Русский словарь. Полнота ключей проверяется компилятором (Record<Keys, string>).
import type { Keys } from './store'

const ru: Record<Keys, string> = {
  // Nav
  'nav.dashboard': 'Обзор',
  'nav.allNodes': 'Ноды',
  'nav.guide': 'Гайд',
  'nav.bot': 'Бот',

  // Header / search
  'search.placeholder': 'Flux-адрес (t1… / t3…)',
  'search.aria': 'Адрес кошелька',
  'search.track': 'Отслеживать',
  'header.flux': 'FLUX',

  // Theme / lang
  'theme.toggleToLight': 'Переключить на светлую тему',
  'theme.toggleToDark': 'Переключить на тёмную тему',
  'lang.label': 'Язык',

  // Wallet header
  'wallet.label': 'Кошелёк',
  'wallet.autoRefresh': 'Автообновление · 60с',
  'wallet.copy': 'Копировать адрес',
  'wallet.copied': 'Скопировано!',
  'wallet.refresh': 'Обновить',

  // Stat cards
  'stat.totalNodes': 'Всего нод',
  'stat.totalNodes.sub': 'активны в сети',
  'stat.walletBalance': 'Баланс кошелька',
  'stat.walletBalance.sub': 'FLUX · ≈ {usd} USD',
  'stat.fleetUptime': 'Аптайм нод',
  'stat.benchPassed': 'Бенчмарк пройден',
  'stat.benchPassed.sub': 'нод прошли бенчмарк',
  'stat.hostedApps': 'Хостинг приложений',
  'stat.hostedApps.sub': 'приложений на нодах',
  'stat.notAvailable': 'доступно в деталях ноды',
  'stat.counting': 'Подсчёт приложений…',

  // Tier card
  'tier.title': 'Разбивка по тирам',

  // Earnings
  'earnings.title': 'Прогноз дохода',
  'earnings.daily': 'В день',
  'earnings.monthly': 'В месяц',
  'earnings.yearly': 'В год',
  'earnings.period.day': 'День',
  'earnings.period.month': 'Месяц',
  'earnings.period.year': 'Год',
  'earnings.apy': 'APY от залога',
  'earnings.notAvailable': 'пока недоступно',
  'earnings.estimate': 'оценка',

  // Parallel Assets
  'pa.title': 'Parallel Assets',
  'pa.summary': '{pct}% от общего дохода по {count} чейнам',
  'pa.perChainYear': '≈ FLUX / год на чейн',
  'pa.share': 'каждый ≈ {pct}% доли',
  'pa.realSubtitle': 'Реальный доход по чейнам (FLUX)',
  'pa.claimable': 'К получению',
  'pa.claimed': 'Получено',
  'pa.received': 'Зачислено',
  'pa.fees': 'Комиссия',

  // Totals
  'totals.mined': 'Всего намайнено',
  'totals.claimed': 'Получено',
  'totals.claimable': 'Доступно к получению',
  'totals.note': 'оценка по возрасту нод',
  'totals.noteReal': 'по данным Parallel Assets',

  // Spotlight
  'spotlight.title': 'В центре внимания',
  'spotlight.subtitle': 'Ваши выдающиеся ноды',
  'spotlight.oldest': 'Старейшая нода',
  'spotlight.oldest.title': 'Дольше всех в работе',
  'spotlight.highestRank': 'Высший ранг',
  'spotlight.highestRank.title': 'Следующая в очереди на выплату',
  'spotlight.mostHosted': 'Больше всего приложений',
  'spotlight.mostHosted.title': 'Приложений на одной ноде',
  'spotlight.payoutEta': 'До выплаты',
  'spotlight.noData': 'нет данных',

  // Nodes table
  'nodes.title': 'Обзор нод',
  'nodes.showing': 'Показано {shown} из {total} · клик по строке для деталей',
  'nodes.filter.all': 'Все',
  'nodes.filter.cumulus': 'Cumulus',
  'nodes.filter.nimbus': 'Nimbus',
  'nodes.filter.stratus': 'Stratus',
  'nodes.searchPlaceholder': 'поиск по IP…',
  'nodes.col.ip': 'IP',
  'nodes.col.tier': 'Тир',
  'nodes.col.rank': 'Ранг',
  'nodes.col.status': 'Статус',
  'nodes.col.age': 'Возраст',
  'nodes.col.lastPaid': 'Оплачено',
  'nodes.col.apps': 'Приложения',
  'nodes.col.location': 'Локация',
  'nodes.col.fluxos': 'FluxOS',
  'nodes.col.maintenance': 'Сервис',
  'nodes.col.payout': 'До выплаты',
  'nodes.maintClosed': 'Закрыто',
  'nodes.payoutSoon': 'скоро',
  'nodes.payoutIn': '{dur}',
  'nodes.noMatch': 'Нет нод по этому фильтру.',
  'nodes.page': 'Стр.',
  'nodes.pageOf': 'из',
  'nodes.prev': 'Назад',
  'nodes.next': 'Вперёд',
  'nodes.sortAsc': 'по возрастанию',
  'nodes.sortDesc': 'по убыванию',
  'nodes.sortBy': 'сортировать по «{col}»',

  // Tiers (display labels)
  'tierName.cumulus': 'Cumulus',
  'tierName.nimbus': 'Nimbus',
  'tierName.stratus': 'Stratus',

  // Price tooltip (FLUX year chart)
  'price.history.title': 'FLUX · за год',
  'price.history.low': 'Мин.',
  'price.history.high': 'Макс.',
  'price.history.now': 'Сейчас',
  'price.history.loading': 'Загрузка истории цены…',
  'price.history.error': 'История цены недоступна',

  // Telegram CTA
  'telegram.title': 'Получайте алерты в Telegram',
  'telegram.body':
    'Узнавайте первыми, когда нода уходит в офлайн, проваливает бенчмарк или когда на кошелёк приходит выплата. Без регистрации на сайте — привязка прямо через бота.',
  'telegram.open': 'Открыть @FluxRadar_bot',

  // Footer
  'footer.copyright': '© 2026 FluxRadar · Self-hosted мониторинг Flux-нод',
  'footer.github': 'GitHub',
  'footer.apiDocs': 'API-доки',
  'footer.privacy': 'Приватность',
  'footer.fluxNetwork': 'Сеть Flux',

  // Донаты
  'donate.title': 'Поддержать FluxRadar',
  'donate.text': 'FluxRadar бесплатен и self-hosted. Если сервис вам полезен — поддержите развитие донатом в FLUX.',
  'donate.address': 'FLUX-адрес',
  'donate.copy': 'Скопировать адрес',
  'donate.copied': 'Скопировано!',
  'donate.visitorsTotal': 'Всего посетителей',
  'donate.visitorsToday': 'За сутки',

  // Privacy modal
  'privacy.title': 'Приватность',
  'privacy.body':
    'FluxRadar никогда не запрашивает и не хранит приватные ключи или seed-фразы — только публичные адреса кошельков. Адреса используются исключительно для запроса публичных данных сети Flux и не передаются третьим лицам.',
  'privacy.close': 'Закрыть',

  // States
  'state.loadingDashboard': 'Загрузка дашборда',
  'error.title': 'Не удалось загрузить',
  'error.retry': 'Повторить',
  'error.unknown': 'Неизвестная ошибка при загрузке данных',
  'empty.title': 'Ноды не найдены',
  'empty.body': 'С этим кошельком пока не связано ни одной Flux-ноды.',

  // Guide
  'guide.title': 'Как это работает',
  'guide.subtitle': 'Три шага к мониторингу нод',
  'guide.step1.title': 'Введите адрес кошелька',
  'guide.step1.body':
    'Вставьте публичный Flux-адрес кошелька (t1… или t3…) в строку поиска. Без регистрации и приватных ключей — только публичные адреса.',
  'guide.step2.title': 'Следите за нодами',
  'guide.step2.body':
    'Видите весь флот сразу: разбивку по тирам, ранг в очереди на выплату, баланс и детали по каждой ноде.',
  'guide.step3.title': 'Привяжите алерты Telegram',
  'guide.step3.body':
    'Откройте @FluxRadar_bot и отправьте /link <кошелёк>, чтобы мгновенно получать алерты, когда нода уходит в офлайн или приходит выплата.',

  // Node detail drawer
  'drawer.title': 'Детали ноды',
  'drawer.close': 'Закрыть',
  'drawer.status': 'Статус',
  'drawer.tier': 'Тир',
  'drawer.location': 'Локация',
  'drawer.apps': 'Запущенные приложения',
  'drawer.appsCount': 'Число приложений',
  'drawer.fluxos': 'FluxOS',
  'drawer.payout': 'Следующая выплата',
  'drawer.maintenance': 'Обслуживание',
  'drawer.noApps': 'Нет приложений',
  'drawer.noLocation': 'Локация недоступна',
  'drawer.loading': 'Загрузка деталей ноды…',
  'drawer.error': 'Не удалось загрузить детали ноды.',
  'drawer.retry': 'Повторить',
}

export default ru
