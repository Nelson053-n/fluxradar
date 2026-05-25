//! Вычисление дельты статусов нод между снапшотами для алертов (§5.2, §5.4 ТЗ).
//!
//! Чистая функция без I/O: вход — прошлое состояние (из `node_status_snapshots`)
//! и текущий срез нод владельца (из `viewdeterministicfluxnodelist`), выход —
//! события для отправки. Воркер записывает события в `alerts_history` и шлёт ботом.

use std::collections::{HashMap, HashSet};

/// Текущее наблюдение по одной ноде (срез из детерминированного списка сети).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeObservation {
    pub ip: String,
    /// Статус ноды: CONFIRMED / DOS / EXPIRED и т.п. (как отдаёт Flux API).
    pub status: String,
}

/// Событие изменения, заслуживающее алерта.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeEvent {
    /// Нода исчезла из списка сети (была, теперь нет) — офлайн.
    Offline { ip: String },
    /// Нода появилась в списке (не было в прошлом снапшоте) — онлайн.
    Online { ip: String },
    /// Нода осталась в списке, но сменила статус (напр. CONFIRMED → DOS).
    StatusChanged {
        ip: String,
        from: String,
        to: String,
    },
}

/// Сравнить прошлые снапшоты (ip → статус) с текущими наблюдениями и вернуть события.
///
/// - ip есть в `previous`, нет в `current` → `Offline`.
/// - ip нет в `previous`, есть в `current` → `Online` (новая или вернувшаяся нода).
/// - ip в обоих, но статус отличается → `StatusChanged`.
/// - ip в обоих с тем же статусом → нет события.
///
/// `previous` пустой (первый запуск для кошелька) → онлайн-события НЕ генерируются,
/// чтобы не спамить алертами «online» при первичной инициализации снапшотов.
pub fn compute_events(
    previous: &HashMap<String, String>,
    current: &[NodeObservation],
) -> Vec<NodeEvent> {
    let mut events = Vec::new();
    let first_run = previous.is_empty();

    let current_ips: HashSet<&str> = current.iter().map(|o| o.ip.as_str()).collect();

    // Online / StatusChanged — проход по текущим наблюдениям.
    for obs in current {
        match previous.get(&obs.ip) {
            None => {
                if !first_run {
                    events.push(NodeEvent::Online { ip: obs.ip.clone() });
                }
            }
            Some(prev_status) if *prev_status != obs.status => {
                events.push(NodeEvent::StatusChanged {
                    ip: obs.ip.clone(),
                    from: prev_status.clone(),
                    to: obs.status.clone(),
                });
            }
            Some(_) => {}
        }
    }

    // Offline — ноды из прошлого, которых больше нет в текущем срезе.
    for ip in previous.keys() {
        if !current_ips.contains(ip.as_str()) {
            events.push(NodeEvent::Offline { ip: ip.clone() });
        }
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prev(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(ip, s)| (ip.to_string(), s.to_string()))
            .collect()
    }

    fn obs(ip: &str, status: &str) -> NodeObservation {
        NodeObservation {
            ip: ip.to_string(),
            status: status.to_string(),
        }
    }

    #[test]
    fn node_goes_offline() {
        let previous = prev(&[("1.1.1.1", "CONFIRMED")]);
        let events = compute_events(&previous, &[]);
        assert_eq!(
            events,
            vec![NodeEvent::Offline {
                ip: "1.1.1.1".into()
            }]
        );
    }

    #[test]
    fn node_comes_online() {
        let previous = prev(&[("1.1.1.1", "CONFIRMED")]);
        let events = compute_events(
            &previous,
            &[obs("1.1.1.1", "CONFIRMED"), obs("2.2.2.2", "CONFIRMED")],
        );
        assert_eq!(
            events,
            vec![NodeEvent::Online {
                ip: "2.2.2.2".into()
            }]
        );
    }

    #[test]
    fn status_change_detected() {
        let previous = prev(&[("1.1.1.1", "CONFIRMED")]);
        let events = compute_events(&previous, &[obs("1.1.1.1", "DOS")]);
        assert_eq!(
            events,
            vec![NodeEvent::StatusChanged {
                ip: "1.1.1.1".into(),
                from: "CONFIRMED".into(),
                to: "DOS".into(),
            }]
        );
    }

    #[test]
    fn no_event_when_unchanged() {
        let previous = prev(&[("1.1.1.1", "CONFIRMED")]);
        let events = compute_events(&previous, &[obs("1.1.1.1", "CONFIRMED")]);
        assert!(events.is_empty());
    }

    #[test]
    fn first_run_does_not_spam_online() {
        // Пустой previous (первый запуск) → ноды просто фиксируются, без online-алертов.
        let events = compute_events(&HashMap::new(), &[obs("1.1.1.1", "CONFIRMED")]);
        assert!(events.is_empty());
    }

    #[test]
    fn mixed_changes() {
        let previous = prev(&[
            ("1.1.1.1", "CONFIRMED"), // останется CONFIRMED
            ("2.2.2.2", "CONFIRMED"), // уйдёт офлайн
            ("3.3.3.3", "CONFIRMED"), // сменит на DOS
        ]);
        let current = [
            obs("1.1.1.1", "CONFIRMED"),
            obs("3.3.3.3", "DOS"),
            obs("4.4.4.4", "CONFIRMED"), // новая → online
        ];
        let mut events = compute_events(&previous, &current);
        // Порядок offline-событий зависит от обхода HashMap — сортируем для детерминизма.
        events.sort_by_key(|e| format!("{e:?}"));
        let mut expected = vec![
            NodeEvent::Online {
                ip: "4.4.4.4".into(),
            },
            NodeEvent::StatusChanged {
                ip: "3.3.3.3".into(),
                from: "CONFIRMED".into(),
                to: "DOS".into(),
            },
            NodeEvent::Offline {
                ip: "2.2.2.2".into(),
            },
        ];
        expected.sort_by_key(|e| format!("{e:?}"));
        assert_eq!(events, expected);
    }
}
