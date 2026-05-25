//! Локализация бота FluxRadar: ru / en / zh.
//!
//! Язык определяется из `language_code` Telegram при первом контакте и затем
//! персистится в `chat_settings.lang` (storage::ChatSettingsRepo). Тексты заданы
//! таблицей `Strings` на каждый язык — `match (lang, key)` был бы шумным, три
//! `const`-структуры читаются и правятся проще.

/// Язык интерфейса бота.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Ru,
    Zh,
}

impl Lang {
    /// Код языка для хранения в БД ('en'/'ru'/'zh').
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ru => "ru",
            Lang::Zh => "zh",
        }
    }

    /// Разобрать сохранённый код языка из БД. Неизвестное → En.
    pub fn from_code(code: &str) -> Lang {
        match code {
            "ru" => Lang::Ru,
            "zh" => Lang::Zh,
            _ => Lang::En,
        }
    }

    /// Определить язык из Telegram `language_code` (напр. "ru", "zh-hans",
    /// "en-US"): префикс "ru" → Ru, "zh" → Zh, иначе En.
    pub fn from_telegram(code: Option<&str>) -> Lang {
        match code {
            Some(c) if c.starts_with("ru") => Lang::Ru,
            Some(c) if c.starts_with("zh") => Lang::Zh,
            _ => Lang::En,
        }
    }

    /// Таблица строк для этого языка.
    fn strings(self) -> &'static Strings {
        match self {
            Lang::En => &EN,
            Lang::Ru => &RU,
            Lang::Zh => &ZH,
        }
    }
}

/// Все видимые строки бота на одном языке.
pub struct Strings {
    pub welcome: &'static str,
    pub help: &'static str,

    // Кнопки главного меню.
    pub btn_link: &'static str,
    pub btn_wallets: &'static str,
    pub btn_status: &'static str,
    pub btn_notifications: &'static str,
    pub btn_language: &'static str,
    pub btn_help: &'static str,
    pub btn_back: &'static str,

    // Привязка / отвязка.
    pub link_prompt: &'static str,
    pub link_usage: &'static str,
    pub link_invalid: &'static str,
    pub link_added: &'static str,
    pub link_exists: &'static str,
    pub unlink_usage: &'static str,
    pub unlink_removed: &'static str,
    pub unlink_absent: &'static str,

    // Кошельки / статус.
    pub wallets_empty: &'static str,
    pub wallets_title: &'static str,
    pub status_empty: &'static str,
    pub status_wallet_header: &'static str,
    pub status_no_nodes: &'static str,
    pub status_fetching: &'static str,
    pub status_flux_error: &'static str,

    // Меню уведомлений.
    pub notif_title: &'static str,
    pub notif_offline: &'static str,
    pub notif_online: &'static str,
    pub notif_status: &'static str,

    // Меню языка.
    pub lang_title: &'static str,

    // Ошибки / общее.
    pub rate_limited: &'static str,
    pub internal_error: &'static str,

    // Подсказка на обычное сообщение, когда бот не ждёт ввода адреса.
    pub not_awaiting_hint: &'static str,
}

/// Получить таблицу строк по языку (для прямого доступа к полям).
pub fn s(lang: Lang) -> &'static Strings {
    lang.strings()
}

const EN: Strings = Strings {
    welcome: "FluxRadar — a Flux node monitoring service.\n\nTo receive alerts about node status changes (going offline, coming back online, state change) you need to link a Flux address (format t1… or t3…).\n\nOnce linked, you'll automatically get notifications about critical events.\n\nUse the menu below.",
    help: "FluxRadar commands:\n\n/start, /menu — open the menu\n/link <address> — track a wallet's nodes\n/unlink <address> — stop tracking a wallet\n/wallets — list linked wallets\n/status — active nodes summary\n/help — this message\n\nTip: tap “🔗 Link wallet” and just send the Flux address (t1… or t3…) in your next message — no command needed.",

    btn_link: "🔗 Link wallet",
    btn_wallets: "💼 My wallets",
    btn_status: "📊 Nodes status",
    btn_notifications: "🔔 Notifications",
    btn_language: "🌐 Language",
    btn_help: "❓ Help",
    btn_back: "⬅️ Back",

    link_prompt: "Send me a Flux wallet address (t1… or t3…) in one message. For example:\n\nt1Whn4HFFRYPoQqUVYNK2fLoHadBkFzM1Sh",
    link_usage: "Specify an address: /link <address>",
    link_invalid: "Address {wallet} doesn't look like a Flux address (it should start with t1 or t3). Please check it.",
    link_added: "✅ Wallet {wallet} linked. I'll send alerts about its nodes.",
    link_exists: "Wallet {wallet} is already linked to this chat.",
    unlink_usage: "Specify an address: /unlink <wallet address>",
    unlink_removed: "Wallet {wallet} unlinked.",
    unlink_absent: "Wallet {wallet} was not linked to this chat.",

    wallets_empty: "You have no linked wallets yet. Use /link <address>.",
    wallets_title: "Linked wallets:",
    status_empty: "No linked wallets. Use /link <address>.",
    status_wallet_header: "📊 Wallet",
    status_no_nodes: "No active nodes.",
    status_fetching: "Fetching node status…",
    status_flux_error: "couldn't reach Flux API, try again later",

    notif_title: "Notifications — tap to toggle:",
    notif_offline: "Node offline",
    notif_online: "Node online",
    notif_status: "Status change",

    lang_title: "Choose your language:",

    rate_limited: "Too many commands. Please wait a minute.",
    internal_error: "Internal error. Please try again later.",

    not_awaiting_hint: "Open the menu with /start, then tap “🔗 Link wallet” to add an address.",
};

const RU: Strings = Strings {
    welcome: "FluxRadar — сервис мониторинга Flux-нод.\n\nДля получения уведомлений об изменении статуса ноды (уход в офлайн, возвращение в сеть, смена состояния) требуется привязать Flux-адрес (формат t1… или t3…).\n\nПосле привязки будут автоматически направляться оповещения о критических событиях.\n\nИспользуйте меню ниже.",
    help: "Команды FluxRadar:\n\n/start, /menu — открыть меню\n/link <адрес> — следить за нодами кошелька\n/unlink <адрес> — перестать следить\n/wallets — список привязанных кошельков\n/status — сводка по активным нодам\n/help — это сообщение\n\nСовет: нажмите «🔗 Привязать кошелёк» и просто пришлите Flux-адрес (t1… или t3…) следующим сообщением — команда не нужна.",

    btn_link: "🔗 Привязать кошелёк",
    btn_wallets: "💼 Мои кошельки",
    btn_status: "📊 Статус нод",
    btn_notifications: "🔔 Уведомления",
    btn_language: "🌐 Язык",
    btn_help: "❓ Помощь",
    btn_back: "⬅️ Назад",

    link_prompt: "Пришлите Flux-адрес кошелька (t1… или t3…) одним сообщением. Например:\n\nt1Whn4HFFRYPoQqUVYNK2fLoHadBkFzM1Sh",
    link_usage: "Укажите адрес: /link <адрес>",
    link_invalid: "Адрес {wallet} не похож на Flux-адрес (должен начинаться с t1 или t3). Проверьте.",
    link_added: "✅ Кошелёк {wallet} привязан. Буду слать алерты по его нодам.",
    link_exists: "Кошелёк {wallet} уже привязан к этому чату.",
    unlink_usage: "Укажите адрес: /unlink <адрес кошелька>",
    unlink_removed: "Кошелёк {wallet} отвязан.",
    unlink_absent: "Кошелёк {wallet} не был привязан к этому чату.",

    wallets_empty: "У вас пока нет привязанных кошельков. Используйте /link <адрес>.",
    wallets_title: "Привязанные кошельки:",
    status_empty: "Нет привязанных кошельков. Используйте /link <адрес>.",
    status_wallet_header: "📊 Кошелёк",
    status_no_nodes: "Активных нод нет.",
    status_fetching: "Запрашиваю статус нод…",
    status_flux_error: "не удалось получить данные Flux API, попробуйте позже",

    notif_title: "Уведомления — нажмите, чтобы переключить:",
    notif_offline: "Нода офлайн",
    notif_online: "Нода онлайн",
    notif_status: "Смена статуса",

    lang_title: "Выберите язык:",

    rate_limited: "Слишком много команд. Подождите минуту.",
    internal_error: "Внутренняя ошибка. Попробуйте позже.",

    not_awaiting_hint: "Откройте меню командой /start, затем нажмите «🔗 Привязать кошелёк», чтобы добавить адрес.",
};

const ZH: Strings = Strings {
    welcome: "FluxRadar — Flux 节点监控服务。\n\n要接收节点状态变化的通知（离线、恢复在线、状态变更），需要绑定一个 Flux 地址（格式 t1… 或 t3…）。\n\n绑定后将自动发送关键事件的提醒。\n\n请使用下方菜单。",
    help: "FluxRadar 命令：\n\n/start、/menu — 打开菜单\n/link <地址> — 跟踪钱包的节点\n/unlink <地址> — 停止跟踪\n/wallets — 已绑定钱包列表\n/status — 活跃节点概况\n/help — 显示本说明\n\n提示：点击「🔗 绑定钱包」，然后直接把 Flux 地址（t1… 或 t3…）作为下一条消息发给我即可，无需命令。",

    btn_link: "🔗 绑定钱包",
    btn_wallets: "💼 我的钱包",
    btn_status: "📊 节点状态",
    btn_notifications: "🔔 通知",
    btn_language: "🌐 语言",
    btn_help: "❓ 帮助",
    btn_back: "⬅️ 返回",

    link_prompt: "请用一条消息发送 Flux 钱包地址（t1… 或 t3…）。例如：\n\nt1Whn4HFFRYPoQqUVYNK2fLoHadBkFzM1Sh",
    link_usage: "请提供地址：/link <地址>",
    link_invalid: "地址 {wallet} 看起来不是 Flux 地址（应以 t1 或 t3 开头），请检查。",
    link_added: "✅ 钱包 {wallet} 已绑定。我会发送其节点的提醒。",
    link_exists: "钱包 {wallet} 已绑定到此聊天。",
    unlink_usage: "请提供地址：/unlink <钱包地址>",
    unlink_removed: "钱包 {wallet} 已解绑。",
    unlink_absent: "钱包 {wallet} 未绑定到此聊天。",

    wallets_empty: "你还没有绑定任何钱包。请使用 /link <地址>。",
    wallets_title: "已绑定钱包：",
    status_empty: "没有已绑定的钱包。请使用 /link <地址>。",
    status_wallet_header: "📊 钱包",
    status_no_nodes: "没有活跃节点。",
    status_fetching: "正在查询节点状态…",
    status_flux_error: "无法连接 Flux API，请稍后再试",

    notif_title: "通知 — 点击切换：",
    notif_offline: "节点离线",
    notif_online: "节点上线",
    notif_status: "状态变化",

    lang_title: "选择语言：",

    rate_limited: "命令过多，请稍候一分钟。",
    internal_error: "内部错误，请稍后再试。",

    not_awaiting_hint: "请用 /start 打开菜单，然后点击「🔗 绑定钱包」来添加地址。",
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telegram_language_detection() {
        assert_eq!(Lang::from_telegram(Some("ru")), Lang::Ru);
        assert_eq!(Lang::from_telegram(Some("ru-RU")), Lang::Ru);
        assert_eq!(Lang::from_telegram(Some("zh-hans")), Lang::Zh);
        assert_eq!(Lang::from_telegram(Some("en-US")), Lang::En);
        assert_eq!(Lang::from_telegram(Some("fr")), Lang::En);
        assert_eq!(Lang::from_telegram(None), Lang::En);
    }

    #[test]
    fn code_roundtrip() {
        for lang in [Lang::En, Lang::Ru, Lang::Zh] {
            assert_eq!(Lang::from_code(lang.code()), lang);
        }
        assert_eq!(Lang::from_code("xx"), Lang::En);
    }
}
