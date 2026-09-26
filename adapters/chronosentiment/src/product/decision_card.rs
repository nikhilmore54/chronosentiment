//! DecisionCard View Model (RP-600 / RP-601 Canonical Presentation Layer)
//!
//! Provides the read-only presentation schema for decision states across multi-horizon streams (15m, 30m, 1d).
//! Invariant: Every field must either come directly from canonical DecisionState/evidence or be a deterministic
//! presentation projection. No field may introduce new semantic inference.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Individual Horizon Resolution State (15m, 30m, 1d)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResolutionState {
    pub label: String,
    pub score: f64,
}

/// Multi-Horizon Map for 15m, 30m, and 1d
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResolutionsMap {
    #[serde(rename = "15m")]
    pub res_15m: ResolutionState,
    #[serde(rename = "30m")]
    pub res_30m: ResolutionState,
    #[serde(rename = "1d")]
    pub res_1d: ResolutionState,
}

/// Provenance Entry for Development Mode Audit
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProvenanceEntry {
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub derivation: Option<String>,
}

/// Read-Only DecisionCard View Model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecisionCardViewModel {
    pub card_id: String,
    pub instrument: String,
    pub timestamp: i64,
    pub datetime: String,
    pub current_price: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_status: Option<String>,
    pub resolutions: ResolutionsMap,
    pub cross_resolution_relationship: String,
    pub risk_state: String,
    pub decision_state: String,
    pub evidence: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _provenance: Option<HashMap<String, ProvenanceEntry>>,
}

impl DecisionCardViewModel {
    /// Creates a new DecisionCard View Model enforcing strict deterministic evidence derivation and canonical risk states.
    pub fn create_canonical(
        symbol: String,
        timestamp: i64,
        price: f64,
        res_15m: ResolutionState,
        res_30m: ResolutionState,
        res_1d: ResolutionState,
        domain_status: Option<String>,
        include_provenance: bool,
    ) -> Self {
        let iso_dt = chrono::DateTime::from_timestamp(timestamp, 0)
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
            .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string());

        let resolutions = ResolutionsMap {
            res_15m: res_15m.clone(),
            res_30m: res_30m.clone(),
            res_1d: res_1d.clone(),
        };

        // Canonical Derivation of Cross-Resolution Relationship, Risk State, and Decision State
        let (cross_res, risk_state, decision_state) = Self::derive_canonical_states(
            &res_15m.label,
            &res_30m.label,
            &res_1d.label,
        );

        // Strict Deterministic Evidence Projection
        let evidence = Self::derive_evidence_bullets(
            &res_15m.label,
            &res_30m.label,
            &res_1d.label,
        );

        let provenance = if include_provenance {
            let mut p = HashMap::new();
            p.insert("instrument".into(), ProvenanceEntry { source: "canonical_decision_state".into(), derivation: None });
            p.insert("timestamp".into(), ProvenanceEntry { source: "canonical_temporal_identity".into(), derivation: None });
            p.insert("datetime".into(), ProvenanceEntry { source: "presentation_projection".into(), derivation: Some("ISO-8601-UTC(timestamp)".into()) });
            p.insert("current_price".into(), ProvenanceEntry { source: "canonical_observation".into(), derivation: None });
            p.insert("resolutions".into(), ProvenanceEntry { source: "canonical_multi_horizon_evidence".into(), derivation: None });
            p.insert("cross_resolution_relationship".into(), ProvenanceEntry { source: "canonical_decision_state".into(), derivation: None });
            p.insert("risk_state".into(), ProvenanceEntry { source: "canonical_decision_state".into(), derivation: None });
            p.insert("decision_state".into(), ProvenanceEntry { source: "canonical_decision_state".into(), derivation: None });
            p.insert("evidence".into(), ProvenanceEntry { source: "canonical_evidence_justification".into(), derivation: None });
            Some(p)
        } else {
            None
        };

        Self {
            card_id: format!("card_{}_{}", symbol, timestamp),
            instrument: symbol,
            timestamp,
            datetime: iso_dt,
            current_price: price,
            domain_status,
            resolutions,
            cross_resolution_relationship: cross_res,
            risk_state,
            decision_state,
            evidence,
            _provenance: provenance,
        }
    }

    /// Canonical State Derivation Engine
    fn derive_canonical_states(l15: &str, l30: &str, l1d: &str) -> (String, String, String) {
        if l15 == "REVERSAL" && l30 == "REVERSAL" && l1d == "REVERSAL" {
            ("UNANIMOUS REVERSAL".into(), "REVERSAL DEVELOPING".into(), "EXIT".into())
        } else if l15 == "REVERSAL" && l1d == "REVERSAL" {
            ("PARTIAL AGREEMENT".into(), "REVERSAL DEVELOPING".into(), "EXIT".into())
        } else if l15 == "REVERSAL" && l1d == "CONTINUATION" {
            ("TIME-SCALE CONFLICT".into(), "EVIDENCE CONFLICTED".into(), "WATCH".into())
        } else if l15 == "CONTINUATION" && l1d == "CONTINUATION" {
            ("UNANIMOUS CONTINUATION".into(), "TRAJECTORY STABLE".into(), "HOLD".into())
        } else if l15 == "PULLBACK" {
            ("PARTIAL AGREEMENT".into(), "EVIDENCE CONFLICTED".into(), "WATCH".into())
        } else {
            ("PARTIAL AGREEMENT".into(), "EVIDENCE CONFLICTED".into(), "WATCH".into())
        }
    }

    /// Strict Deterministic Evidence Bullet Projection
    fn derive_evidence_bullets(l15: &str, l30: &str, l1d: &str) -> Vec<String> {
        let mut bullets = Vec::new();

        // 15m Horizon
        match l15 {
            "PULLBACK" => bullets.push("• 15m trajectory is in a PULLBACK state".into()),
            "CONTINUATION" => bullets.push("• 15m trajectory maintains positive momentum".into()),
            "REVERSAL" => bullets.push("• 15m trajectory crossed reversal boundary".into()),
            _ => bullets.push("• 15m observation stream insufficient".into()),
        }

        // 30m Horizon
        match l30 {
            "PULLBACK" => bullets.push("• 30m trajectory is also in a PULLBACK state".into()),
            "CONTINUATION" => bullets.push("• 30m intermediate horizon confirms continuation".into()),
            "REVERSAL" => bullets.push("• 30m intermediate horizon indicates reversal risk".into()),
            _ => bullets.push("• 30m intermediate horizon observation insufficient".into()),
        }

        // 1d Horizon
        match l1d {
            "REVERSAL" => bullets.push("• 1d trajectory indicates REVERSAL".into()),
            "CONTINUATION" => bullets.push("• 1d macro horizon remains solidly in trend continuation".into()),
            "PULLBACK" => bullets.push("• 1d macro horizon in pullback state".into()),
            _ => bullets.push("• 1d macro horizon observation insufficient".into()),
        }

        // Alignment Bullet
        if l15 == "REVERSAL" && l1d == "REVERSAL" {
            bullets.push("• 15m and 1d structural reversal confirmation".into());
        } else if l15 != l1d {
            bullets.push("• Intraday and daily trajectories are not aligned".into());
        } else {
            bullets.push("• Multi-horizon trajectories in alignment".into());
        }

        bullets
    }
}
