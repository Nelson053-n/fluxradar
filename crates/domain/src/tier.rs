//! Tier ноды — три уровня, отличаются требуемым collateral и наградой (§15 ТЗ).
//!
//! Flux API отдаёт tier готовой строкой в поле `tier` ответа
//! `viewdeterministicfluxnodelist` (факт, проверено на api.runonflux.io),
//! поэтому парсим из строки, а не вычисляем из collateral.

use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Cumulus,
    Nimbus,
    Stratus,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("неизвестный tier: {0}")]
pub struct UnknownTier(String);

impl FromStr for Tier {
    type Err = UnknownTier;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_str() {
            "CUMULUS" => Ok(Tier::Cumulus),
            "NIMBUS" => Ok(Tier::Nimbus),
            "STRATUS" => Ok(Tier::Stratus),
            other => Err(UnknownTier(other.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_uppercase_from_flux_api() {
        assert_eq!("CUMULUS".parse(), Ok(Tier::Cumulus));
        assert_eq!("NIMBUS".parse(), Ok(Tier::Nimbus));
        assert_eq!("STRATUS".parse(), Ok(Tier::Stratus));
    }

    #[test]
    fn parsing_is_case_insensitive() {
        assert_eq!("cumulus".parse(), Ok(Tier::Cumulus));
    }

    #[test]
    fn rejects_unknown() {
        assert!("ROGUE".parse::<Tier>().is_err());
    }
}
