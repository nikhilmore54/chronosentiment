//! Strategy identity outcome model for V-001.
//!
//! This module is compiled Rust scaffolding for replay-aware identity
//! consolidation. It defines the outcome vocabulary and a non-routing canonical
//! parser prototype, but it does not route or replace legacy parsers yet.

use crate::domain::*;
use serde::{Deserialize, Serialize};

pub const CANONICAL_PARSER_SOURCE: &str = "core/src/strategy_id.rs::parse_strategy_id";
pub const COMPATIBILITY_PARSER_SOURCE: &str =
    "core/src/strategy_id.rs::parse_strategy_id_with_compatibility";

/// Whether a serialized identity entered replay scope for a parser lineage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdmissibilityResult {
    Accepted,
    Rejected,
}

/// What identity meaning was assigned after admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterpretationResult {
    SameMeaning,
    NormalizedMeaning,
    DivergentMeaning,
    NotInterpreted,
}

/// Cross-lineage replay comparison classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayClassification {
    BitEquivalent,
    SemanticallyEquivalent,
    DivergentSemantics,
    DivergentAcceptance,
    AllRejected,
}

/// Canonical outcome categories expected from future strategy identity parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrategyIdentityOutcome {
    AcceptedSameMeaning,
    AcceptedNormalized,
    AcceptedDivergentSemantics,
    RejectedHistoricallyAdmitted,
    RejectedUniversally,
}

/// Provenance-aware resolution path for one serialized identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrategyIdentityResolution {
    Canonical,
    CompatibilityTranslated,
    HistoricallyVisibleButNonCanonical,
    RejectedHistoricallyAdmitted,
    RejectedUniversally,
}

/// Parse failure categories for the canonical prototype.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum StrategyIdError {
    UnsupportedLegacyFormat,
    TooFewFields {
        expected_min: usize,
        actual: usize,
    },
    InvalidField {
        field: &'static str,
        index: usize,
        value: String,
    },
    LegacyCompatibilityTooFewNumericFields {
        actual: usize,
    },
}

impl ReplayClassification {
    /// Map differential evidence classifications onto the replay outcome model.
    pub fn outcome(self) -> StrategyIdentityOutcome {
        match self {
            Self::BitEquivalent => StrategyIdentityOutcome::AcceptedSameMeaning,
            Self::SemanticallyEquivalent => StrategyIdentityOutcome::AcceptedNormalized,
            Self::DivergentSemantics => StrategyIdentityOutcome::AcceptedDivergentSemantics,
            Self::DivergentAcceptance => StrategyIdentityOutcome::RejectedHistoricallyAdmitted,
            Self::AllRejected => StrategyIdentityOutcome::RejectedUniversally,
        }
    }
}

impl StrategyIdentityOutcome {
    pub fn admissibility(self) -> AdmissibilityResult {
        match self {
            Self::AcceptedSameMeaning
            | Self::AcceptedNormalized
            | Self::AcceptedDivergentSemantics => AdmissibilityResult::Accepted,
            Self::RejectedHistoricallyAdmitted | Self::RejectedUniversally => {
                AdmissibilityResult::Rejected
            }
        }
    }

    pub fn interpretation(self) -> InterpretationResult {
        match self {
            Self::AcceptedSameMeaning => InterpretationResult::SameMeaning,
            Self::AcceptedNormalized => InterpretationResult::NormalizedMeaning,
            Self::AcceptedDivergentSemantics => InterpretationResult::DivergentMeaning,
            Self::RejectedHistoricallyAdmitted | Self::RejectedUniversally => {
                InterpretationResult::NotInterpreted
            }
        }
    }
}

/// Observed result from one parser lineage for one serialized identity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyIdentityObservation {
    pub serialized_id: String,
    pub parser_source: String,
    pub admissibility: AdmissibilityResult,
    pub resolution: StrategyIdentityResolution,
    pub parsed_strategy: Option<Strategy>,
    pub round_trip_serialization: Option<String>,
    pub source_identity: Option<String>,
    pub error: Option<String>,
}

/// Cross-lineage comparison record for one serialized identity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyIdentityComparison {
    pub serialized_id: String,
    pub classification: ReplayClassification,
    pub outcome: StrategyIdentityOutcome,
    pub observations: Vec<StrategyIdentityObservation>,
}

/// Non-routing canonical parser prototype for V-001.
///
/// This function observes canonical behavior without replacing legacy call sites.
/// It accepts `STRAT_` identities in the same field order emitted by
/// `ga::strategy_to_id`, expands 13-field legacy `STRAT_` forms with defaults,
/// and rejects underscore legacy IDs as unsupported historical formats.
pub fn parse_strategy_id(serialized_id: &str) -> StrategyIdentityObservation {
    match parse_strategy_id_inner(serialized_id) {
        Ok(strategy) => StrategyIdentityObservation {
            serialized_id: serialized_id.to_string(),
            parser_source: CANONICAL_PARSER_SOURCE.to_string(),
            admissibility: AdmissibilityResult::Accepted,
            resolution: StrategyIdentityResolution::Canonical,
            round_trip_serialization: Some(strategy_to_id(&strategy)),
            parsed_strategy: Some(strategy),
            source_identity: None,
            error: None,
        },
        Err(error) => StrategyIdentityObservation {
            serialized_id: serialized_id.to_string(),
            parser_source: CANONICAL_PARSER_SOURCE.to_string(),
            admissibility: AdmissibilityResult::Rejected,
            resolution: StrategyIdentityResolution::RejectedUniversally,
            parsed_strategy: None,
            round_trip_serialization: None,
            source_identity: Some(serialized_id.to_string()),
            error: Some(format!("{error:?}")),
        },
    }
}

/// Provenance-aware parser for compatibility entry points.
///
/// This accepts native canonical `STRAT_...` identities directly and translates
/// legacy underscore IDs through an explicit compatibility path. The underscore
/// syntax remains non-canonical; successful compatibility parsing emits canonical
/// `ga::strategy_to_id` serialization and preserves the source identity.
pub fn parse_strategy_id_with_compatibility(serialized_id: &str) -> StrategyIdentityObservation {
    if serialized_id.starts_with("STRAT_") {
        return parse_strategy_id(serialized_id);
    }

    match parse_legacy_underscore_strategy(serialized_id) {
        Ok(strategy) => StrategyIdentityObservation {
            serialized_id: serialized_id.to_string(),
            parser_source: COMPATIBILITY_PARSER_SOURCE.to_string(),
            admissibility: AdmissibilityResult::Accepted,
            resolution: StrategyIdentityResolution::CompatibilityTranslated,
            round_trip_serialization: Some(strategy_to_id(&strategy)),
            parsed_strategy: Some(strategy),
            source_identity: Some(serialized_id.to_string()),
            error: None,
        },
        Err(error) => StrategyIdentityObservation {
            serialized_id: serialized_id.to_string(),
            parser_source: COMPATIBILITY_PARSER_SOURCE.to_string(),
            admissibility: AdmissibilityResult::Rejected,
            resolution: StrategyIdentityResolution::RejectedUniversally,
            parsed_strategy: None,
            round_trip_serialization: None,
            source_identity: Some(serialized_id.to_string()),
            error: Some(format!("{error:?}")),
        },
    }
}

fn parse_legacy_underscore_strategy(serialized_id: &str) -> Result<Strategy, StrategyIdError> {
    let nums: Vec<u64> = serialized_id
        .split('_')
        .rev()
        .filter_map(|part| part.parse::<u64>().ok())
        .collect();

    if nums.len() < 4 {
        return Err(StrategyIdError::LegacyCompatibilityTooFewNumericFields { actual: nums.len() });
    }

    Ok(Strategy {
        queue_threshold: nums.get(3).copied().unwrap_or(100),
        base_edge: nums.get(2).copied().unwrap_or(2),
        take_profit: nums.get(1).copied().unwrap_or(20),
        stop_loss: nums.first().copied().unwrap_or(10),
        holding_period: 0,
        w_conviction: 50,
        w_momentum: 30,
        w_volatility: 20,
        exp_conviction: 100,
        exp_momentum: 100,
        exp_volatility: 100,
        selectivity: 75,
        archetype: 0,
        entry_offset: 0,
        direction_bias: 50,
        vol_floor: 20,
        mom_floor: 20,
        edge_ratio: 150,
        participation_threshold: 30,
        exec_aggression: 50,
        latency_bias: 10,
        fill_threshold: 50,
    })
}

fn parse_strategy_id_inner(serialized_id: &str) -> Result<Strategy, StrategyIdError> {
    if !serialized_id.starts_with("STRAT_") {
        return Err(StrategyIdError::UnsupportedLegacyFormat);
    }

    let parts: Vec<&str> = serialized_id.split('v').collect();
    if parts.len() < 13 {
        return Err(StrategyIdError::TooFewFields {
            expected_min: 13,
            actual: parts.len(),
        });
    }

    Ok(Strategy {
        queue_threshold: parse_u64_field(&parts, 0, "queue_threshold", true)?,
        base_edge: parse_u64_field(&parts, 1, "base_edge", false)?,
        take_profit: parse_u64_field(&parts, 2, "take_profit", false)?,
        stop_loss: parse_u64_field(&parts, 3, "stop_loss", false)?,
        holding_period: parse_u64_field(&parts, 4, "holding_period", false)?,
        w_conviction: parse_u64_field(&parts, 5, "w_conviction", false)?,
        w_momentum: parse_u64_field(&parts, 6, "w_momentum", false)?,
        w_volatility: parse_optional_u64_field(&parts, 7, "w_volatility", 20)?,
        exp_conviction: parse_optional_u64_field(&parts, 8, "exp_conviction", 100)?,
        exp_momentum: parse_optional_u64_field(&parts, 9, "exp_momentum", 100)?,
        exp_volatility: parse_optional_u64_field(&parts, 10, "exp_volatility", 100)?,
        selectivity: parse_optional_u8_field(&parts, 11, "selectivity", 75)?,
        archetype: parse_optional_u8_field(&parts, 12, "archetype", 0)?,
        entry_offset: parse_optional_i32_field(&parts, 13, "entry_offset", 0)?,
        direction_bias: parse_optional_u8_field(&parts, 14, "direction_bias", 50)?,
        vol_floor: parse_optional_u8_field(&parts, 15, "vol_floor", 20)?,
        mom_floor: parse_optional_u8_field(&parts, 16, "mom_floor", 20)?,
        edge_ratio: parse_optional_u8_field(&parts, 17, "edge_ratio", 150)?,
        participation_threshold: parse_optional_u8_field(
            &parts,
            18,
            "participation_threshold",
            30,
        )?,
        exec_aggression: 50,
        latency_bias: 10,
        fill_threshold: 50,
    })
}

fn field_value<'a>(
    parts: &'a [&str],
    index: usize,
    field: &'static str,
    strip_prefix: bool,
) -> Result<&'a str, StrategyIdError> {
    let value = parts
        .get(index)
        .copied()
        .ok_or(StrategyIdError::TooFewFields {
            expected_min: index + 1,
            actual: parts.len(),
        })?;

    Ok(if strip_prefix {
        value.trim_start_matches("STRAT_")
    } else {
        value
    })
    .and_then(|value| {
        if value.is_empty() {
            Err(StrategyIdError::InvalidField {
                field,
                index,
                value: value.to_string(),
            })
        } else {
            Ok(value)
        }
    })
}

fn parse_u64_field(
    parts: &[&str],
    index: usize,
    field: &'static str,
    strip_prefix: bool,
) -> Result<u64, StrategyIdError> {
    let value = field_value(parts, index, field, strip_prefix)?;
    value.parse().map_err(|_| StrategyIdError::InvalidField {
        field,
        index,
        value: value.to_string(),
    })
}

fn parse_optional_u64_field(
    parts: &[&str],
    index: usize,
    field: &'static str,
    default: u64,
) -> Result<u64, StrategyIdError> {
    match parts.get(index).copied() {
        Some(value) => value.parse().map_err(|_| StrategyIdError::InvalidField {
            field,
            index,
            value: value.to_string(),
        }),
        None => Ok(default),
    }
}

fn parse_optional_u8_field(
    parts: &[&str],
    index: usize,
    field: &'static str,
    default: u8,
) -> Result<u8, StrategyIdError> {
    match parts.get(index).copied() {
        Some(value) => value.parse().map_err(|_| StrategyIdError::InvalidField {
            field,
            index,
            value: value.to_string(),
        }),
        None => Ok(default),
    }
}

fn parse_optional_i32_field(
    parts: &[&str],
    index: usize,
    field: &'static str,
    default: i32,
) -> Result<i32, StrategyIdError> {
    match parts.get(index).copied() {
        Some(value) => value.parse().map_err(|_| StrategyIdError::InvalidField {
            field,
            index,
            value: value.to_string(),
        }),
        None => Ok(default),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_classification_maps_to_outcome_model() {
        assert_eq!(
            ReplayClassification::BitEquivalent.outcome(),
            StrategyIdentityOutcome::AcceptedSameMeaning
        );
        assert_eq!(
            ReplayClassification::SemanticallyEquivalent.outcome(),
            StrategyIdentityOutcome::AcceptedNormalized
        );
        assert_eq!(
            ReplayClassification::DivergentSemantics.outcome(),
            StrategyIdentityOutcome::AcceptedDivergentSemantics
        );
        assert_eq!(
            ReplayClassification::DivergentAcceptance.outcome(),
            StrategyIdentityOutcome::RejectedHistoricallyAdmitted
        );
        assert_eq!(
            ReplayClassification::AllRejected.outcome(),
            StrategyIdentityOutcome::RejectedUniversally
        );
    }

    #[test]
    fn outcome_preserves_admissibility_and_interpretation_axes() {
        assert_eq!(
            StrategyIdentityOutcome::AcceptedDivergentSemantics.admissibility(),
            AdmissibilityResult::Accepted
        );
        assert_eq!(
            StrategyIdentityOutcome::AcceptedDivergentSemantics.interpretation(),
            InterpretationResult::DivergentMeaning
        );
        assert_eq!(
            StrategyIdentityOutcome::RejectedHistoricallyAdmitted.admissibility(),
            AdmissibilityResult::Rejected
        );
        assert_eq!(
            StrategyIdentityOutcome::RejectedHistoricallyAdmitted.interpretation(),
            InterpretationResult::NotInterpreted
        );
    }

    #[test]
    fn canonical_prototype_parses_full_strategy_id_order() {
        let observed =
            parse_strategy_id("STRAT_100v2v20v10v50v50v30v20v100v100v100v75v0v3v55v22v23v160v35");

        assert_eq!(observed.admissibility, AdmissibilityResult::Accepted);
        assert_eq!(observed.resolution, StrategyIdentityResolution::Canonical);
        let strategy = observed.parsed_strategy.expect("expected parsed strategy");
        assert_eq!(strategy.entry_offset, 3);
        assert_eq!(strategy.direction_bias, 55);
        assert_eq!(strategy.participation_threshold, 35);
        assert_eq!(
            observed.round_trip_serialization.as_deref(),
            Some("STRAT_100v2v20v10v50v50v30v20v100v100v100v75v0v3v55v22v23v160v35")
        );
    }

    #[test]
    fn canonical_prototype_rejects_underscore_legacy_ids() {
        let observed = parse_strategy_id("strat_BTCUSDT_jsonl_window_1_201_2_31_10");

        assert_eq!(observed.admissibility, AdmissibilityResult::Rejected);
        assert_eq!(
            observed.resolution,
            StrategyIdentityResolution::RejectedUniversally
        );
        assert!(observed.parsed_strategy.is_none());
        assert!(observed
            .error
            .as_deref()
            .is_some_and(|error| error.contains("UnsupportedLegacyFormat")));
    }

    #[test]
    fn compatibility_parser_translates_underscore_legacy_ids() {
        let observed =
            parse_strategy_id_with_compatibility("strat_BTCUSDT_jsonl_window_1_201_2_31_10");

        assert_eq!(observed.admissibility, AdmissibilityResult::Accepted);
        assert_eq!(
            observed.resolution,
            StrategyIdentityResolution::CompatibilityTranslated
        );
        assert_eq!(
            observed.source_identity.as_deref(),
            Some("strat_BTCUSDT_jsonl_window_1_201_2_31_10")
        );
        assert_eq!(
            observed.round_trip_serialization.as_deref(),
            Some("STRAT_201v2v31v10v0v50v30v20v100v100v100v75v0v0v50v20v20v150v30")
        );
    }
}
