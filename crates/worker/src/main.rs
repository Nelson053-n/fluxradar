//! Worker FluxScope — polling Flux API по расписанию и расчёт алертов (§5.4 ТЗ).
//!
//! Цикл (60с): ОДИН запрос `viewdeterministicfluxnodelist` на весь список нод сети,
//! затем фильтр по адресам подписчиков локально (не N запросов на N нод!), сравнение
//! со снапшотами предыдущего тика (domain::compute_events) → события офлайн/онлайн/
//! смена статуса → запись в alerts_history (с дедупом) и отправка ботом.

use std::collections::HashMap;
use std::time::Duration;

use domain::{compute_events, NodeEvent, NodeObservation};
use storage::pg::{AlertsRepo, ChatSettingsRepo, SnapshotsRepo, SubscriptionsRepo};
use teloxide::prelude::*;
use tracing::{error, info, warn};

const POLL_INTERVAL: Duration = Duration::from_secs(60);
/// Окно дедупликации алертов: не повторяем один и тот же алерт чаще, чем раз в N сек.
const DEDUP_SECS: i64 = 3600;

/// Опциональный отправитель в Telegram — None, если токен не задан (worker всё равно
/// крутит polling и пишет алерты в лог/историю, не падая из-за отсутствия токена).
struct Notifier {
    bot: Option<Bot>,
}

impl Notifier {
    fn from_env() -> Self {
        match std::env::var("TELEGRAM_BOT_TOKEN") {
            Ok(t) if !t.is_empty() && !t.contains("placeholder") => Self {
                bot: Some(Bot::new(t)),
            },
            _ => {
                warn!("TELEGRAM_BOT_TOKEN не задан — алерты только в лог, без отправки в Telegram");
                Self { bot: None }
            }
        }
    }

    async fn send(&self, chat_id: i64, text: &str) {
        match &self.bot {
            Some(bot) => {
                if let Err(e) = bot.send_message(ChatId(chat_id), text).await {
                    error!(?e, chat_id, "не удалось отправить алерт");
                }
            }
            None => info!(chat_id, text, "ALERT (без отправки: нет токена)"),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt().json().init();

    let database_url =
        std::env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL не задан"))?;
    let pool = storage::pg::connect(&database_url).await?;

    // Redis — для прогрева кэша node_stats (его читает api при cold-summary).
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());
    let redis = storage::cache::connect(&redis_url).await?;

    let ctx = Context {
        flux: flux_client::FluxClient::new()?,
        subs: SubscriptionsRepo::new(pool.clone()),
        snapshots: SnapshotsRepo::new(pool.clone()),
        alerts: AlertsRepo::new(pool.clone()),
        settings: ChatSettingsRepo::new(pool),
        notifier: Notifier::from_env(),
        redis,
    };

    info!(
        interval_secs = POLL_INTERVAL.as_secs(),
        "FluxScope worker запущен"
    );

    let mut ticker = tokio::time::interval(POLL_INTERVAL);
    loop {
        ticker.tick().await;
        // Прогрев кэша node_stats — независимо от наличия подписок, чтобы первый
        // заход на любой кошелёк в api читал тёплый кэш (не ждал тяжёлый fluxinfo).
        warm_node_stats_cache(&ctx).await;
        if let Err(err) = poll_tick(&ctx).await {
            error!(?err, "ошибка в polling-тике");
        }
    }
}

/// Контекст воркера: клиент Flux + репозитории + отправитель + кэш.
struct Context {
    flux: flux_client::FluxClient,
    subs: SubscriptionsRepo,
    snapshots: SnapshotsRepo,
    alerts: AlertsRepo,
    settings: ChatSettingsRepo,
    notifier: Notifier,
    redis: storage::cache::RedisPool,
}

/// Прогреть кэш бенчмарк/apps по всем нодам сети (тот же ключ, что читает api).
/// Тяжёлый запрос (~7000 нод) выполняется здесь раз в тик, а не в горячем пути api.
async fn warm_node_stats_cache(ctx: &Context) {
    let map = match ctx.flux.network_node_stats().await {
        Ok(m) => m,
        Err(e) => {
            warn!(?e, "прогрев node_stats: запрос fluxinfo не удался");
            return;
        }
    };
    match serde_json::to_string(&map) {
        Ok(s) => {
            if let Err(e) = storage::cache::set_ex(
                &ctx.redis,
                storage::cache::NODE_STATS_KEY,
                &s,
                storage::cache::NODE_STATS_TTL_SECS,
            )
            .await
            {
                warn!(?e, "прогрев node_stats: запись в Redis не удалась");
            } else {
                info!(nodes = map.len(), "node_stats прогрет в кэш");
            }
        }
        Err(e) => warn!(?e, "прогрев node_stats: сериализация не удалась"),
    }
}

/// Один проход polling-цикла (§5.4).
async fn poll_tick(ctx: &Context) -> anyhow::Result<()> {
    // Кому слать: уникальные кошельки подписчиков. Нет подписок — нечего делать.
    let wallets = ctx.subs.distinct_wallets().await?;
    if wallets.is_empty() {
        return Ok(());
    }

    // ОДИН запрос на весь список сети (§5.4), фильтрация локально.
    let all_nodes = ctx.flux.deterministic_node_list().await?;

    // Подписки по кошельку: wallet → список (chat_id) для адресной рассылки.
    let subs = ctx.subs.all().await?;
    let mut chats_by_wallet: HashMap<String, Vec<i64>> = HashMap::new();
    for s in subs {
        chats_by_wallet
            .entry(s.wallet_address)
            .or_default()
            .push(s.tg_chat_id);
    }

    for wallet in &wallets {
        // Текущий срез нод этого кошелька.
        let current: Vec<NodeObservation> = all_nodes
            .iter()
            .filter(|n| &n.payment_address == wallet)
            .map(|n| NodeObservation {
                ip: n.ip.clone(),
                // Нода в детерминированном списке = активна/подтверждена.
                status: "CONFIRMED".to_owned(),
            })
            .collect();

        // Прошлые снапшоты этого кошелька.
        let previous = ctx
            .snapshots
            .load_for_wallets(std::slice::from_ref(wallet))
            .await?;

        // Дельта → события.
        let events = compute_events(&previous, &current);

        // Разослать события подписчикам кошелька (с дедупом).
        if let Some(chats) = chats_by_wallet.get(wallet) {
            for ev in &events {
                dispatch_event(ctx, wallet, chats, ev).await;
            }
        }

        // Обновить снапшоты: upsert текущих, удалить ушедшие.
        let present_ips: Vec<String> = current.iter().map(|o| o.ip.clone()).collect();
        for obs in &current {
            if let Err(e) = ctx.snapshots.upsert(&obs.ip, wallet, &obs.status).await {
                warn!(?e, ip = %obs.ip, "не удалось upsert снапшот");
            }
        }
        if let Err(e) = ctx.snapshots.remove_missing(wallet, &present_ips).await {
            warn!(?e, "не удалось удалить ушедшие снапшоты");
        }
    }

    Ok(())
}

/// Отправить одно событие всем чатам кошелька, с дедупом и записью в историю.
async fn dispatch_event(ctx: &Context, wallet: &str, chats: &[i64], ev: &NodeEvent) {
    let (alert_type, subject, text) = render_event(ev);
    for &chat_id in chats {
        // Настройки уведомлений чата: пропускаем выключенные типы.
        let prefs = ctx
            .settings
            .get_alert_prefs(chat_id)
            .await
            .unwrap_or_default();
        let enabled = match ev {
            NodeEvent::Offline { .. } => prefs.offline,
            NodeEvent::Online { .. } => prefs.online,
            NodeEvent::StatusChanged { .. } => prefs.status,
        };
        if !enabled {
            continue;
        }
        // Дедуп: не повторяем тот же алерт в окне DEDUP_SECS.
        match ctx
            .alerts
            .recently_sent(chat_id, alert_type, &subject, DEDUP_SECS)
            .await
        {
            Ok(true) => continue,
            Ok(false) => {}
            Err(e) => {
                warn!(?e, "ошибка проверки дедупа — шлём без дедупа");
            }
        }
        ctx.notifier.send(chat_id, &text).await;
        if let Err(e) = ctx
            .alerts
            .record(chat_id, wallet, alert_type, &subject)
            .await
        {
            warn!(?e, "не удалось записать alerts_history");
        }
    }
}

/// Текст и метаданные алерта по событию.
fn render_event(ev: &NodeEvent) -> (&'static str, String, String) {
    match ev {
        NodeEvent::Offline { ip } => (
            "node_offline",
            ip.clone(),
            format!("🔴 Нода {ip} ушла офлайн (пропала из списка сети)."),
        ),
        NodeEvent::Online { ip } => (
            "node_online",
            ip.clone(),
            format!("🟢 Нода {ip} снова онлайн."),
        ),
        NodeEvent::StatusChanged { ip, from, to } => (
            "node_status",
            ip.clone(),
            format!("⚠️ Нода {ip} сменила статус: {from} → {to}."),
        ),
    }
}
