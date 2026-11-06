//! Создание клиента Telegram с опциональным SOCKS5-прокси.
//!
//! Зачем: с прод-сервера `api.telegram.org` недоступен (блокировка), но там есть
//! `telegram-egress` — SOCKS5, который пускает только на `api.telegram.org:443`
//! через отдельный туннель. Адрес прокси задаётся в `TELEGRAM_PROXY`, например
//! `socks5h://172.20.0.1:1083`. Без переменной клиент работает напрямую (dev).
//!
//! Схема `socks5h` обязательна: прокси принимает только доменное имя и сам его
//! резолвит (он намеренно не принимает IP-адрес в запросе CONNECT), поэтому при
//! `socks5://` клиент резолвил бы адрес локально и получал отказ.

use teloxide::Bot;

/// Переменная окружения с URL прокси. Пустое значение = без прокси.
pub const PROXY_ENV: &str = "TELEGRAM_PROXY";

/// Значение прокси из окружения: обрезаем пробелы, пустое считаем отсутствием.
pub fn normalize_proxy(value: Option<String>) -> Option<String> {
    value.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
}

/// Прокси из окружения процесса.
pub fn proxy_from_env() -> Option<String> {
    normalize_proxy(std::env::var(PROXY_ENV).ok())
}

/// Клиент Telegram: через прокси, если он задан, иначе напрямую.
///
/// Ошибка — только если прокси задан и не собирается (опечатка в URL), чтобы
/// сервис не работал молча «напрямую» там, где прямой путь заблокирован.
pub fn bot(token: impl Into<String>) -> anyhow::Result<Bot> {
    let token = token.into();
    match proxy_from_env() {
        None => Ok(Bot::new(token)),
        Some(url) => {
            let proxy = tg_http::Proxy::all(url.as_str())
                .map_err(|e| anyhow::anyhow!("{PROXY_ENV}={url}: {e}"))?;
            let client = tg_http::Client::builder()
                .proxy(proxy)
                .build()
                .map_err(|e| anyhow::anyhow!("клиент Telegram: {e}"))?;
            Ok(Bot::with_client(token, client))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_proxy;

    #[test]
    fn proxy_value_is_trimmed_and_blank_means_disabled() {
        assert_eq!(normalize_proxy(None), None);
        assert_eq!(normalize_proxy(Some("   ".to_owned())), None);
        assert_eq!(
            normalize_proxy(Some(" socks5h://172.20.0.1:1083 ".to_owned())).as_deref(),
            Some("socks5h://172.20.0.1:1083")
        );
    }
}
