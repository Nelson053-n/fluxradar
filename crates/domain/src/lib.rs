//! Доменные модели, формулы и общие типы FluxScope.
//!
//! Крейт не зависит от инфраструктуры (БД, HTTP, Flux API) — только чистые типы
//! и бизнес-логика. Формулы и агрегации реализованы с нуля по публичной
//! документации Flux API (см. `docs/LICENSE_NOTES.md`).

pub mod delta;
pub mod earnings;
pub mod node;
pub mod summary;
pub mod tier;
pub mod wallet;

pub use delta::{compute_events, NodeEvent, NodeObservation};
pub use earnings::{maintenance_window_secs, node_age_secs, payout_eta_secs};
pub use node::NodeStatus;
pub use summary::{
    build as build_summary, NetworkSummary, PaChain, RealPaTotals, SummaryInputs, WalletSummary,
};
pub use tier::Tier;
pub use wallet::{classify as classify_address, is_valid as is_valid_address, AddressKind};
