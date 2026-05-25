//! Telegram-бот FluxRadar — привязка кошельков, статус нод, настройки (§5.1 ТЗ).
//!
//! Dev: long polling. Токен только из `TELEGRAM_BOT_TOKEN` (§9), не из репозитория.
//! Подписки и настройки пишутся в PG (storage), к которой имеют доступ только bot
//! и worker (§5.1).
//!
//! Управление — текстовыми командами (совместимость) и inline-меню под сообщением:
//! главное меню → привязка / кошельки / статус / уведомления / язык / помощь.
//! Язык определяется из Telegram `language_code` при первом контакте и персистится.

mod i18n;
mod ratelimit;

use std::sync::Arc;

use teloxide::payloads::{EditMessageTextSetters, SendMessageSetters};
use teloxide::prelude::*;
use teloxide::types::{
    BotCommand, InlineKeyboardButton, InlineKeyboardMarkup, MaybeInaccessibleMessage, ParseMode,
};
use teloxide::utils::command::BotCommands;
use tokio::sync::Mutex;

use flux_client::FluxClient;
use i18n::{s, Lang};
use ratelimit::RateLimiter;
use storage::pg::{ChatSettingsRepo, SubscriptionsRepo};

/// Команды бота (§5.1). Описания для синей кнопки меню задаются через
/// `set_my_commands` отдельно (см. `register_commands`).
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    Start,
    Menu,
    Link(String),
    Unlink(String),
    Wallets,
    Status,
    Help,
}

/// Callback-данные inline-кнопок. Сериализуются в строку (≤64 байт у Telegram).
mod cb {
    pub const MENU: &str = "menu";
    pub const LINK: &str = "link";
    pub const WALLETS: &str = "wallets";
    pub const STATUS: &str = "status";
    pub const NOTIF: &str = "notif";
    pub const LANG: &str = "lang";
    pub const HELP: &str = "help";
    /// Тоггл уведомления: `notif:set:<kind>` где kind ∈ offline/online/status.
    pub const NOTIF_SET_PREFIX: &str = "notif:set:";
    /// Выбор языка: `lang:set:<code>`.
    pub const LANG_SET_PREFIX: &str = "lang:set:";
}

/// Разделяемое состояние обработчиков. Все поля дёшевы для клонирования.
#[derive(Clone)]
struct BotState {
    subs: SubscriptionsRepo,
    settings: ChatSettingsRepo,
    flux: FluxClient,
    limiter: Arc<Mutex<RateLimiter>>,
    /// chat_id, для которых бот ждёт следующего обычного сообщения как адреса
    /// (после кнопки/команды «Привязать» без аргумента). In-memory, как limiter.
    awaiting: Arc<Mutex<std::collections::HashSet<i64>>>,
}

type HandlerResult = ResponseResult<()>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt().json().init();

    let database_url =
        std::env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL не задан"))?;
    let pool = storage::pg::connect(&database_url).await?;

    let state = BotState {
        subs: SubscriptionsRepo::new(pool.clone()),
        settings: ChatSettingsRepo::new(pool),
        flux: FluxClient::new()?,
        limiter: Arc::new(Mutex::new(RateLimiter::default_per_minute())),
        awaiting: Arc::new(Mutex::new(std::collections::HashSet::new())),
    };

    // ТЗ §9 фиксирует имя TELEGRAM_BOT_TOKEN (teloxide по умолчанию ждёт TELOXIDE_TOKEN).
    let token = std::env::var("TELEGRAM_BOT_TOKEN")
        .map_err(|_| anyhow::anyhow!("TELEGRAM_BOT_TOKEN не задан"))?;
    let bot = Bot::new(token);

    register_commands(&bot).await;
    tracing::info!("FluxRadar bot запущен (long polling)");

    // Порядок веток важен: сначала команды (filter_command перехватывает /start и
    // прочее), затем обычный текст (адрес после кнопки «Привязать»), затем callback.
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(command_handler),
        )
        .branch(Update::filter_message().endpoint(text_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}

/// Зарегистрировать список команд (синяя кнопка меню в клиенте Telegram).
/// Дефолтные описания — на en; для ru/zh ставим локализованные через scope языка.
async fn register_commands(bot: &Bot) {
    let en = [
        BotCommand::new("start", "Open the menu"),
        BotCommand::new("menu", "Open the menu"),
        BotCommand::new("link", "Track a wallet's nodes"),
        BotCommand::new("unlink", "Stop tracking a wallet"),
        BotCommand::new("wallets", "List linked wallets"),
        BotCommand::new("status", "Active nodes summary"),
        BotCommand::new("help", "Show help"),
    ];
    if let Err(e) = bot.set_my_commands(en).await {
        tracing::warn!(?e, "set_my_commands (en) не удалось");
    }

    let ru = [
        BotCommand::new("start", "Открыть меню"),
        BotCommand::new("menu", "Открыть меню"),
        BotCommand::new("link", "Следить за нодами кошелька"),
        BotCommand::new("unlink", "Перестать следить"),
        BotCommand::new("wallets", "Список кошельков"),
        BotCommand::new("status", "Сводка по нодам"),
        BotCommand::new("help", "Помощь"),
    ];
    if let Err(e) = bot.set_my_commands(ru).language_code("ru").await {
        tracing::warn!(?e, "set_my_commands (ru) не удалось");
    }

    let zh = [
        BotCommand::new("start", "打开菜单"),
        BotCommand::new("menu", "打开菜单"),
        BotCommand::new("link", "跟踪钱包的节点"),
        BotCommand::new("unlink", "停止跟踪"),
        BotCommand::new("wallets", "钱包列表"),
        BotCommand::new("status", "节点概况"),
        BotCommand::new("help", "帮助"),
    ];
    if let Err(e) = bot.set_my_commands(zh).language_code("zh").await {
        tracing::warn!(?e, "set_my_commands (zh) не удалось");
    }
}

// --- Определение и персист языка чата. ---

/// Язык чата: из БД, иначе из Telegram `language_code` с сохранением в БД.
/// `tg_code` — `msg.from.language_code` (None для callback без него).
async fn resolve_lang(state: &BotState, chat_id: i64, tg_code: Option<&str>) -> Lang {
    match state.settings.get_lang(chat_id).await {
        Ok(Some(code)) => Lang::from_code(&code),
        Ok(None) => {
            let lang = Lang::from_telegram(tg_code);
            if let Err(e) = state.settings.set_lang(chat_id, lang.code()).await {
                tracing::warn!(?e, "не удалось сохранить определённый язык");
            }
            lang
        }
        Err(e) => {
            tracing::error!(?e, "ошибка get_lang — fallback на язык Telegram");
            Lang::from_telegram(tg_code)
        }
    }
}

// --- Inline-меню. ---

/// Главное меню под приветствием.
fn main_menu(lang: Lang) -> InlineKeyboardMarkup {
    let t = s(lang);
    InlineKeyboardMarkup::new([
        // Основная кнопка — статус нод (отдельной строкой сверху).
        vec![InlineKeyboardButton::callback(t.btn_status, cb::STATUS)],
        vec![
            InlineKeyboardButton::callback(t.btn_wallets, cb::WALLETS),
            InlineKeyboardButton::callback(t.btn_link, cb::LINK),
        ],
        vec![
            InlineKeyboardButton::callback(t.btn_notifications, cb::NOTIF),
            InlineKeyboardButton::callback(t.btn_language, cb::LANG),
            InlineKeyboardButton::callback(t.btn_help, cb::HELP),
        ],
    ])
}

/// Подменю уведомлений с галочками текущего состояния.
fn notif_menu(lang: Lang, prefs: storage::pg::AlertPrefs) -> InlineKeyboardMarkup {
    let t = s(lang);
    let mark = |on: bool| if on { "✅" } else { "❌" };
    let row = |label: &str, on: bool, kind: &str| {
        vec![InlineKeyboardButton::callback(
            format!("{} {}", mark(on), label),
            format!("{}{}", cb::NOTIF_SET_PREFIX, kind),
        )]
    };
    InlineKeyboardMarkup::new([
        row(t.notif_offline, prefs.offline, "offline"),
        row(t.notif_online, prefs.online, "online"),
        row(t.notif_status, prefs.status, "status"),
        vec![InlineKeyboardButton::callback(t.btn_back, cb::MENU)],
    ])
}

/// Подменю выбора языка.
fn lang_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([
        vec![InlineKeyboardButton::callback(
            "🇬🇧 English",
            format!("{}en", cb::LANG_SET_PREFIX),
        )],
        vec![InlineKeyboardButton::callback(
            "🇷🇺 Русский",
            format!("{}ru", cb::LANG_SET_PREFIX),
        )],
        vec![InlineKeyboardButton::callback(
            "🇨🇳 中文",
            format!("{}zh", cb::LANG_SET_PREFIX),
        )],
    ])
}

/// Кнопка «Назад в меню» — для экранов без своих кнопок (кошельки, статус, помощь).
fn back_only(lang: Lang) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(s(lang).btn_back, cb::MENU)]])
}

// --- Обработчик текстовых команд. ---

async fn command_handler(bot: Bot, msg: Message, cmd: Command, state: BotState) -> HandlerResult {
    let chat_id = msg.chat.id;
    let user = msg.from.as_ref();
    let user_id = user.map(|u| u.id.0 as i64).unwrap_or(0);
    let tg_code = user.and_then(|u| u.language_code.as_deref());

    let lang = resolve_lang(&state, chat_id.0, tg_code).await;

    // Rate-limit на пользователя (§9): тихо отбиваем флуд.
    {
        let mut limiter = state.limiter.lock().await;
        if !limiter.check(user_id) {
            bot.send_message(chat_id, s(lang).rate_limited).await?;
            return Ok(());
        }
    }

    match cmd {
        Command::Start | Command::Menu => {
            bot.send_message(chat_id, s(lang).welcome)
                .reply_markup(main_menu(lang))
                .await?;
        }
        Command::Help => {
            bot.send_message(chat_id, s(lang).help)
                .reply_markup(back_only(lang))
                .await?;
        }
        Command::Link(wallet) => {
            let wallet = wallet.trim();
            if wallet.is_empty() {
                // Без аргумента — переходим в режим ожидания адреса следующим сообщением.
                state.awaiting.lock().await.insert(chat_id.0);
                bot.send_message(chat_id, s(lang).link_prompt).await?;
            } else {
                let text = handle_link(&state, chat_id.0, user_id, wallet, lang).await;
                bot.send_message(chat_id, text).await?;
            }
        }
        Command::Unlink(wallet) => {
            let text = handle_unlink(&state, chat_id.0, wallet.trim(), lang).await;
            bot.send_message(chat_id, text).await?;
        }
        Command::Wallets => {
            let text = handle_wallets(&state, chat_id.0, lang).await;
            bot.send_message(chat_id, text).await?;
        }
        Command::Status => {
            // Статус может быть медленным (запрос к Flux) — подсказка, затем результат.
            // handle_status сам шлёт таблицу (возможно несколькими сообщениями).
            bot.send_message(chat_id, s(lang).status_fetching).await?;
            handle_status(&bot, &state, chat_id, lang).await?;
        }
    }
    Ok(())
}

// --- Обработчик обычных текстовых сообщений (не команд). ---
//
// Ветка идёт ПОСЛЕ command_handler в dptree, поэтому /start и прочие команды сюда
// не попадают. Назначение: если чат в режиме ожидания (после кнопки/команды
// «Привязать»), трактовать следующее сообщение как адрес кошелька.

async fn text_handler(bot: Bot, msg: Message, state: BotState) -> HandlerResult {
    let Some(text) = msg.text() else {
        return Ok(()); // не текст (фото/стикер/…) — игнорируем
    };
    let text = text.trim();

    let chat_id = msg.chat.id;
    let user = msg.from.as_ref();
    let user_id = user.map(|u| u.id.0 as i64).unwrap_or(0);
    let tg_code = user.and_then(|u| u.language_code.as_deref());

    let lang = resolve_lang(&state, chat_id.0, tg_code).await;

    // Rate-limit на пользователя (§9), как и для команд.
    {
        let mut limiter = state.limiter.lock().await;
        if !limiter.check(user_id) {
            bot.send_message(chat_id, s(lang).rate_limited).await?;
            return Ok(());
        }
    }

    // Ждём ли адрес от этого чата? Снимаем флаг сразу, чтобы повторный ввод требовал
    // нового нажатия кнопки.
    let awaited = state.awaiting.lock().await.remove(&chat_id.0);

    if awaited && !text.starts_with('/') {
        let reply = handle_link(&state, chat_id.0, user_id, text, lang).await;
        bot.send_message(chat_id, reply).await?;
    } else {
        // Не ждали ввода — мягкая подсказка открыть меню.
        bot.send_message(chat_id, s(lang).not_awaiting_hint).await?;
    }
    Ok(())
}

// --- Обработчик callback-кнопок. ---

async fn callback_handler(bot: Bot, q: CallbackQuery, state: BotState) -> HandlerResult {
    // Подтверждаем приём (убираем «часики» у кнопки).
    bot.answer_callback_query(q.id.clone()).await?;

    let Some(data) = q.data.clone() else {
        return Ok(());
    };
    let Some(message) = q.message.as_ref() else {
        return Ok(());
    };
    let chat_id = message.chat().id;
    let user_id = q.from.id.0 as i64;
    let tg_code = q.from.language_code.as_deref();

    // Rate-limit и на нажатия кнопок (§9).
    {
        let mut limiter = state.limiter.lock().await;
        if !limiter.check(user_id) {
            return Ok(());
        }
    }

    let lang = resolve_lang(&state, chat_id.0, tg_code).await;

    // Разбор callback-данных и действие.
    if data == cb::MENU {
        edit(&bot, message, s(lang).welcome.to_owned(), main_menu(lang)).await?;
    } else if data == cb::LINK {
        // Переходим в режим ожидания: следующее обычное сообщение — это адрес.
        state.awaiting.lock().await.insert(chat_id.0);
        edit(
            &bot,
            message,
            s(lang).link_prompt.to_owned(),
            back_only(lang),
        )
        .await?;
    } else if data == cb::WALLETS {
        let text = handle_wallets(&state, chat_id.0, lang).await;
        edit(&bot, message, text, back_only(lang)).await?;
    } else if data == cb::STATUS {
        // Таблица может занять несколько сообщений и требует моноширинного <pre>,
        // поэтому исходное меню оставляем как «загружаю» + кнопку «Назад», а саму
        // таблицу шлём отдельными сообщениями ниже.
        edit(
            &bot,
            message,
            s(lang).status_fetching.to_owned(),
            back_only(lang),
        )
        .await?;
        handle_status(&bot, &state, chat_id, lang).await?;
    } else if data == cb::NOTIF {
        let prefs = state
            .settings
            .get_alert_prefs(chat_id.0)
            .await
            .unwrap_or_default();
        edit(
            &bot,
            message,
            s(lang).notif_title.to_owned(),
            notif_menu(lang, prefs),
        )
        .await?;
    } else if let Some(kind) = data.strip_prefix(cb::NOTIF_SET_PREFIX) {
        // Тоггл: читаем текущее, инвертируем нужный, пишем, перерисовываем.
        let mut prefs = state
            .settings
            .get_alert_prefs(chat_id.0)
            .await
            .unwrap_or_default();
        let new_val = match kind {
            "offline" => {
                prefs.offline = !prefs.offline;
                prefs.offline
            }
            "online" => {
                prefs.online = !prefs.online;
                prefs.online
            }
            "status" => {
                prefs.status = !prefs.status;
                prefs.status
            }
            _ => return Ok(()),
        };
        if let Err(e) = state
            .settings
            .set_alert_pref(chat_id.0, kind, new_val)
            .await
        {
            tracing::error!(?e, kind, "не удалось сохранить настройку уведомления");
        }
        edit(
            &bot,
            message,
            s(lang).notif_title.to_owned(),
            notif_menu(lang, prefs),
        )
        .await?;
    } else if data == cb::LANG {
        edit(&bot, message, s(lang).lang_title.to_owned(), lang_menu()).await?;
    } else if let Some(code) = data.strip_prefix(cb::LANG_SET_PREFIX) {
        let new_lang = Lang::from_code(code);
        if let Err(e) = state.settings.set_lang(chat_id.0, new_lang.code()).await {
            tracing::error!(?e, "не удалось сохранить выбранный язык");
        }
        // Возвращаемся в ГЛАВНОЕ меню на новом языке — чтобы сразу было видно,
        // что весь интерфейс бота переключился (а не только подменю языка).
        edit(
            &bot,
            message,
            s(new_lang).welcome.to_owned(),
            main_menu(new_lang),
        )
        .await?;
    } else if data == cb::HELP {
        edit(&bot, message, s(lang).help.to_owned(), back_only(lang)).await?;
    }

    Ok(())
}

/// Перерисовать сообщение меню новым текстом и клавиатурой.
async fn edit(
    bot: &Bot,
    message: &MaybeInaccessibleMessage,
    text: String,
    keyboard: InlineKeyboardMarkup,
) -> HandlerResult {
    bot.edit_message_text(message.chat().id, message.id(), text)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

// --- Доменные действия (общие для команд и кнопок). ---

async fn handle_link(
    state: &BotState,
    chat_id: i64,
    user_id: i64,
    wallet: &str,
    lang: Lang,
) -> String {
    let t = s(lang);
    if wallet.is_empty() {
        return t.link_usage.to_owned();
    }
    // Валидация адреса ДО записи (§9.4): невалидный — не пишем в БД.
    if !domain::is_valid_address(wallet) {
        return t.link_invalid.replace("{wallet}", wallet);
    }
    match state.subs.add(chat_id, user_id, wallet).await {
        Ok(true) => t.link_added.replace("{wallet}", wallet),
        Ok(false) => t.link_exists.replace("{wallet}", wallet),
        Err(e) => {
            tracing::error!(?e, "ошибка add подписки");
            t.internal_error.to_owned()
        }
    }
}

async fn handle_unlink(state: &BotState, chat_id: i64, wallet: &str, lang: Lang) -> String {
    let t = s(lang);
    if wallet.is_empty() {
        return t.unlink_usage.to_owned();
    }
    match state.subs.remove(chat_id, wallet).await {
        Ok(true) => t.unlink_removed.replace("{wallet}", wallet),
        Ok(false) => t.unlink_absent.replace("{wallet}", wallet),
        Err(e) => {
            tracing::error!(?e, "ошибка remove подписки");
            t.internal_error.to_owned()
        }
    }
}

async fn handle_wallets(state: &BotState, chat_id: i64, lang: Lang) -> String {
    let t = s(lang);
    match state.subs.wallets_for_chat(chat_id).await {
        Ok(ws) if ws.is_empty() => t.wallets_empty.to_owned(),
        Ok(ws) => {
            let list = ws
                .iter()
                .map(|w| format!("• {w}"))
                .collect::<Vec<_>>()
                .join("\n");
            format!("{}\n{list}", t.wallets_title)
        }
        Err(e) => {
            tracing::error!(?e, "ошибка wallets_for_chat");
            t.internal_error.to_owned()
        }
    }
}

/// Максимальная длина одного Telegram-сообщения ~4096 символов; режем с запасом,
/// чтобы оставить место под обёртку `<pre>…</pre>` и HTML-экранирование.
const MSG_CHUNK_LIMIT: usize = 3500;

/// Сводка по активным нодам кошельков в табличном виде (как fluxstats): на каждый
/// кошелёк — таблица rank / IP / время до выплаты, затем разбивка по тирам и Total.
/// §5.4: один запрос к Flux на список сети внутри `nodes_for_wallet`, фильтр локально.
///
/// Шлёт результат сам (возможно несколькими сообщениями) — таблица не помещается в
/// одно сообщение при большом флоте. Таблицы — в HTML `<pre>` для моноширинного вида.
async fn handle_status(bot: &Bot, state: &BotState, chat_id: ChatId, lang: Lang) -> HandlerResult {
    let t = s(lang);
    let wallets = match state.subs.wallets_for_chat(chat_id.0).await {
        Ok(ws) if ws.is_empty() => {
            bot.send_message(chat_id, t.status_empty).await?;
            return Ok(());
        }
        Ok(ws) => ws,
        Err(e) => {
            tracing::error!(?e, "ошибка status: wallets_for_chat");
            bot.send_message(chat_id, t.internal_error).await?;
            return Ok(());
        }
    };

    for wallet in &wallets {
        // Заголовок-адрес кошелька — обычным текстом, без backticks.
        let header = format!("{} {wallet}", t.status_wallet_header);
        bot.send_message(chat_id, header).await?;

        match state.flux.nodes_for_wallet(wallet).await {
            Ok(nodes) if nodes.is_empty() => {
                bot.send_message(chat_id, t.status_no_nodes).await?;
            }
            Ok(mut nodes) => {
                // Ближайшие к выплате — сверху.
                nodes.sort_by_key(|n| n.rank);
                for chunk in status_table_chunks(&nodes) {
                    bot.send_message(chat_id, chunk)
                        .parse_mode(ParseMode::Html)
                        .await?;
                }
            }
            Err(e) => {
                tracing::warn!(?e, %wallet, "ошибка nodes_for_wallet");
                bot.send_message(chat_id, t.status_flux_error).await?;
            }
        }
    }
    Ok(())
}

/// Форматирует секунды в Ч:ММ (часы без ведущего нуля, минуты с ведущим нулём).
/// Время до выплаты бывает много часов (rank × 30с), потому часы, а не минуты.
/// Напр. 8340с → "2:19", 36900с → "10:15", 300с → "0:05".
fn fmt_hhmm(secs: i64) -> String {
    let m = secs.max(0) / 60;
    format!("{}:{:02}", m / 60, m % 60)
}

/// Экранирование под HTML `<pre>` (Telegram требует &lt; &gt; &amp;).
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Собирает табличный вывод по нодам и режет на куски ≤ MSG_CHUNK_LIMIT, не разрывая
/// строки. Каждый кусок — самостоятельный `<pre>…</pre>` блок. Ноды должны быть уже
/// отсортированы вызывающим (по rank). Подвал (Total + тиры) идёт последним блоком.
fn status_table_chunks(nodes: &[flux_client::DeterministicNode]) -> Vec<String> {
    // Ширины колонок: rank и IP выравниваем по самому длинному значению.
    let rank_w = nodes
        .iter()
        .map(|n| n.rank.to_string().len())
        .max()
        .unwrap_or(0)
        .max("rank".len());
    let ip_w = nodes
        .iter()
        .map(|n| n.ip.len())
        .max()
        .unwrap_or(0)
        .max("IP".len());

    // Строки таблицы (заголовок + по ноде).
    let header = format!("{:>rank_w$}  {:>ip_w$}  {}", "rank", "IP", "Payout(h:m)");
    let mut rows: Vec<String> = Vec::with_capacity(nodes.len() + 1);
    rows.push(header);
    for n in nodes {
        rows.push(format!(
            "{:>rank_w$}  {:>ip_w$}  {}",
            n.rank,
            n.ip,
            fmt_hhmm(domain::payout_eta_secs(n.rank)),
        ));
    }

    // Подвал: непустые тиры + Total.
    let mut footer_lines: Vec<String> = Vec::new();
    footer_lines.push(format!("{:>rank_w$}  Total", ""));
    for (tier, count) in tier_counts(nodes) {
        footer_lines.push(format!("{tier:<8}{count}"));
    }

    // Режем на куски: каждый кусок оборачиваем в <pre>. Подвал держим в одном куске.
    let footer = footer_lines.join("\n");
    let mut chunks: Vec<String> = Vec::new();
    let mut buf = String::new();
    for row in &rows {
        // +1 на перевод строки между строками внутри буфера.
        if !buf.is_empty() && buf.len() + 1 + row.len() > MSG_CHUNK_LIMIT {
            chunks.push(wrap_pre(&buf));
            buf.clear();
        }
        if !buf.is_empty() {
            buf.push('\n');
        }
        buf.push_str(row);
    }
    // Подвал: приклеиваем к текущему буферу, если влезает, иначе отдельным куском.
    if !buf.is_empty() && buf.len() + 1 + footer.len() <= MSG_CHUNK_LIMIT {
        buf.push('\n');
        buf.push_str(&footer);
        chunks.push(wrap_pre(&buf));
    } else {
        if !buf.is_empty() {
            chunks.push(wrap_pre(&buf));
        }
        chunks.push(wrap_pre(&footer));
    }
    chunks
}

/// Обернуть моноширинный текст в HTML `<pre>` с экранированием содержимого.
fn wrap_pre(body: &str) -> String {
    format!("<pre>{}</pre>", html_escape(body))
}

/// Счётчики по тирам в фиксированном порядке (CUMULUS, NIMBUS, STRATUS, затем прочие),
/// только непустые. Прочие тиры группируются под меткой "OTHER".
fn tier_counts(nodes: &[flux_client::DeterministicNode]) -> Vec<(&'static str, u32)> {
    let mut cumulus = 0u32;
    let mut nimbus = 0u32;
    let mut stratus = 0u32;
    let mut other = 0u32;
    for n in nodes {
        match n.tier.to_ascii_uppercase().as_str() {
            "CUMULUS" => cumulus += 1,
            "NIMBUS" => nimbus += 1,
            "STRATUS" => stratus += 1,
            _ => other += 1,
        }
    }
    let mut out = Vec::new();
    if cumulus > 0 {
        out.push(("CUMULUS", cumulus));
    }
    if nimbus > 0 {
        out.push(("NIMBUS", nimbus));
    }
    if stratus > 0 {
        out.push(("STRATUS", stratus));
    }
    if other > 0 {
        out.push(("OTHER", other));
    }
    out
}
