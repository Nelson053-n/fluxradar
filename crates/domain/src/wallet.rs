//! Валидация адреса кошелька до обращения к Flux API (§9.4 ТЗ).
//!
//! Невалидный адрес → ошибка → API отвечает 400 без похода наружу.
//!
//! Принимаются ТОЛЬКО Flux-адреса: `t1...` (transparent P2PKH) и `t3...` (P2SH).
//! Это единственные форматы `payment_address` в `viewdeterministicfluxnodelist`
//! (проверено на сети: 7123 ноды — все t1/t3; ETH/BTC-адресов у нод НЕТ, поэтому
//! отслеживать ноды по ним невозможно — такие форматы не принимаем).

/// Тип распознанного Flux-адреса.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressKind {
    /// Flux transparent P2PKH: `t1` + base58, 35 символов.
    Transparent,
    /// Flux P2SH: `t3` + base58, 35 символов.
    P2sh,
}

/// Проверить адрес и определить его тип. Принимает только Flux t1/t3.
///
/// Лёгкая структурная проверка (префикс, длина, алфавит) — не криптографическая
/// валидация контрольной суммы. Цель — отсечь очевидный мусор до запроса к Flux API.
pub fn classify(address: &str) -> Option<AddressKind> {
    if !is_flux_shape(address) {
        return None;
    }
    if address.starts_with("t1") {
        Some(AddressKind::Transparent)
    } else if address.starts_with("t3") {
        Some(AddressKind::P2sh)
    } else {
        None
    }
}

pub fn is_valid(address: &str) -> bool {
    classify(address).is_some()
}

/// Общая структурная проверка Flux-адреса: длина 35, base58, префикс t1/t3.
fn is_flux_shape(s: &str) -> bool {
    s.len() == 35 && (s.starts_with("t1") || s.starts_with("t3")) && s.bytes().all(is_base58)
}

fn is_base58(b: u8) -> bool {
    // Алфавит base58: без 0, O, I, l.
    matches!(b,
        b'1'..=b'9'
        | b'A'..=b'H' | b'J'..=b'N' | b'P'..=b'Z'
        | b'a'..=b'k' | b'm'..=b'z')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_real_t1_address() {
        // Реальный payment_address из ответа Flux API (t1, 35 симв.).
        assert_eq!(
            classify("t1cuMLs3MUkMUH8tnzrkGHQJvxvQqrfuQAf"),
            Some(AddressKind::Transparent)
        );
    }

    #[test]
    fn accepts_real_t3_address() {
        // Реальный t3 (P2SH) payment_address из сети — 1290 таких нод.
        assert_eq!(
            classify("t3SZMe4fVEgR844osB43Go1SgFZpYF7epDt"),
            Some(AddressKind::P2sh)
        );
    }

    #[test]
    fn rejects_eth_and_btc() {
        // По ETH/BTC-адресам нод в сети нет — не принимаем.
        assert!(!is_valid("0x52908400098527886E0F7030069857D2E4169EE7")); // Ethereum
        assert!(!is_valid("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2")); // BTC P2PKH
    }

    #[test]
    fn rejects_garbage() {
        assert!(!is_valid(""));
        assert!(!is_valid("not-an-address"));
        assert!(!is_valid("t1short")); // короткий
        assert!(!is_valid("t10OIlxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")); // запрещённые base58-символы
        assert!(!is_valid("t2cuMLs3MUkMUH8tnzrkGHQJvxvQqrfuQAf")); // t2 — не Flux-формат
    }
}
