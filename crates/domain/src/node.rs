//! Статус ноды для дельта-вычислений и алертов (§5.2, §5.5 ТЗ).

use serde::{Deserialize, Serialize};

/// Статус ноды, как его трактует worker при сравнении снапшотов.
///
/// Значения соответствуют колонке `node_status_snapshots.status`
/// (см. migrations/0001_init.sql).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum NodeStatus {
    Confirmed,
    Expired,
    Dos,
    Offline,
}
