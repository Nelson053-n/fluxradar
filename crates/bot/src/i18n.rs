//! Локализация бота FluxRadar: ru / en.
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
}

impl Lang {
    /// Код языка для хранения в БД ('en'/'ru').
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ru => "ru",
        }
    }

    /// Разобрать сохранённый код языка из БД. Неизвестное → En.
    pub fn from_code(code: &str) -> Lang {
        match code {
            "ru" => Lang::Ru,
            _ => Lang::En,
        }
    }

    /// Определить язык из Telegram `language_code` (напр. "ru", "ru-RU",
    /// "en-US"): префикс "ru" → Ru, иначе En.
    pub fn from_telegram(code: Option<&str>) -> Lang {
        match code {
            Some(c) if c.starts_with("ru") => Lang::Ru,
            _ => Lang::En,
        }
    }

    /// Таблица строк для этого языка.
    fn strings(self) -> &'static Strings {
        match self {
            Lang::En => &EN,
            Lang::Ru => &RU,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telegram_language_detection() {
        assert_eq!(Lang::from_telegram(Some("ru")), Lang::Ru);
        assert_eq!(Lang::from_telegram(Some("ru-RU")), Lang::Ru);
        assert_eq!(Lang::from_telegram(Some("zh-hans")), Lang::En);
        assert_eq!(Lang::from_telegram(Some("en-US")), Lang::En);
        assert_eq!(Lang::from_telegram(Some("fr")), Lang::En);
        assert_eq!(Lang::from_telegram(None), Lang::En);
    }

    #[test]
    fn code_roundtrip() {
        for lang in [Lang::En, Lang::Ru] {
            assert_eq!(Lang::from_code(lang.code()), lang);
        }
        assert_eq!(Lang::from_code("xx"), Lang::En);
    }
}
