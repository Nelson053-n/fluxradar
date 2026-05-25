//! Простой in-memory rate-limit команд на `tg_user_id` (§9 ТЗ: 30 команд/мин).
//!
//! Скользящего окна не нужно — фиксированное окно достаточно для защиты от флуда
//! и тривиально тестируется. Состояние живёт в памяти процесса бота.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Счётчик команд в фиксированном окне на пользователя.
pub struct RateLimiter {
    limit: u32,
    window: Duration,
    hits: HashMap<i64, (Instant, u32)>,
}

impl RateLimiter {
    /// `limit` команд за `window`.
    pub fn new(limit: u32, window: Duration) -> Self {
        Self {
            limit,
            window,
            hits: HashMap::new(),
        }
    }

    /// Лимит по умолчанию из ТЗ: 30 команд в минуту.
    pub fn default_per_minute() -> Self {
        Self::new(30, Duration::from_secs(60))
    }

    /// Зарегистрировать команду пользователя. `true` — разрешено, `false` — превышен лимит.
    /// `now` параметризован для детерминированных тестов.
    pub fn check_at(&mut self, user_id: i64, now: Instant) -> bool {
        let entry = self.hits.entry(user_id).or_insert((now, 0));
        if now.duration_since(entry.0) >= self.window {
            *entry = (now, 0); // окно истекло — сброс
        }
        if entry.1 < self.limit {
            entry.1 += 1;
            true
        } else {
            false
        }
    }

    /// Удобный вызов с текущим временем.
    pub fn check(&mut self, user_id: i64) -> bool {
        self.check_at(user_id, Instant::now())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_up_to_limit_then_blocks() {
        let mut rl = RateLimiter::new(3, Duration::from_secs(60));
        let t0 = Instant::now();
        assert!(rl.check_at(1, t0));
        assert!(rl.check_at(1, t0));
        assert!(rl.check_at(1, t0));
        assert!(!rl.check_at(1, t0)); // 4-я — заблокирована
    }

    #[test]
    fn separate_users_independent() {
        let mut rl = RateLimiter::new(1, Duration::from_secs(60));
        let t0 = Instant::now();
        assert!(rl.check_at(1, t0));
        assert!(rl.check_at(2, t0)); // другой пользователь — свой лимит
        assert!(!rl.check_at(1, t0));
    }

    #[test]
    fn window_resets() {
        let mut rl = RateLimiter::new(1, Duration::from_secs(60));
        let t0 = Instant::now();
        assert!(rl.check_at(1, t0));
        assert!(!rl.check_at(1, t0));
        let later = t0 + Duration::from_secs(61);
        assert!(rl.check_at(1, later)); // окно истекло — снова разрешено
    }
}
