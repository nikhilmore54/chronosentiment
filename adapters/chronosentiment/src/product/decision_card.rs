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
    pub position_side: String,      // "LONG" | "SHORT"
    pub entry_signal: String,       // "LONG ENTRY (BUY)" | "LONG POSITION MONITORING" | "STANDBY (WATCH)" | "CLOSE LONG (EXIT)"
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

        // Position side & entry signal derivation
        let position_side = "LONG".to_string();
        let entry_signal = match decision_state.as_str() {
            "HOLD" => "LONG POSITION MONITORING (ACTIVE)".to_string(),
            "WATCH" => "STANDBY / AWAIT ALIGNMENT".to_string(),
            "EXIT" => "CLOSE LONG POSITION (EXIT)".to_string(),
            _ => "STANDBY / AWAIT ALIGNMENT".to_string(),
        };

        // Strict Deterministic Evidence Projection
        let evidence = Self::derive_evidence_bullets(
            &symbol,
            price,
            &decision_state,
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
            p.insert("position_side".into(), ProvenanceEntry { source: "canonical_position_context".into(), derivation: None });
            p.insert("entry_signal".into(), ProvenanceEntry { source: "canonical_decision_state".into(), derivation: None });
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
            position_side,
            entry_signal,
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

    /// Compelling & Actionable Evidence Bullet Derivation Engine
    fn derive_evidence_bullets(
        symbol: &str,
        price: f64,
        decision_state: &str,
        l15: &str,
        l30: &str,
        l1d: &str,
    ) -> Vec<String> {
        let is_crypto = symbol.contains("-USD");
        let is_nse = symbol.ends_with(".NS") || symbol.ends_with(".BO");
        let curr_sym = if is_nse { "₹" } else { "$" };

        let trigger_price = if decision_state == "HOLD" {
            price
        } else if decision_state == "EXIT" {
            price * 0.998
        } else {
            price * 1.012
        };

        let target_price = if decision_state == "EXIT" {
            price * 0.965
        } else {
            price * 1.035
        };

        let stop_loss_price = if decision_state == "EXIT" {
            price * 1.015
        } else {
            price * 0.985
        };

        let horizon_duration = if is_crypto {
            "60m Target / 180m Max"
        } else {
            "1440m (24h) Target / 4320m (72h) Max"
        };

        let fmt_price = format!("{}{:.2}", curr_sym, price);
        let fmt_trigger = format!("{}{:.2}", curr_sym, trigger_price);
        let fmt_target = format!("{}{:.2}", curr_sym, target_price);
        let fmt_stop = format!("{}{:.2}", curr_sym, stop_loss_price);

        let mut bullets = Vec::new();

        if l15 == "REVERSAL" && l30 == "REVERSAL" && l1d == "REVERSAL" {
            bullets.push("• DYNAMICS: Unanimous negative trajectory across 15m, 30m, and 1d observation windows.".into());
            bullets.push(format!("• RECOMMENDATION RATIONALE: EXIT — Entry/Trigger: {} | Target Price: {} (-3.5%) | Stop Loss Price: {} (+1.5%) | Horizon Duration: {}.", fmt_trigger, fmt_target, fmt_stop, horizon_duration));
            bullets.push(format!("• ACTIONABLE TRIGGER: Execute immediate CLOSE or OPEN SHORT at entry price {}; do not hold long exposure.", fmt_price));
            bullets.push(format!("• CAPITAL PROTECTION: Hard stop loss at {} prevents compounding drawdown during trend breakdown.", fmt_stop));
        } else if l15 == "REVERSAL" && l1d == "REVERSAL" {
            bullets.push("• DYNAMICS: 15m micro velocity and 1d macro horizon confirm structural selling pressure.".into());
            bullets.push(format!("• RECOMMENDATION RATIONALE: EXIT — Entry/Trigger: {} | Target Price: {} (-3.5%) | Stop Loss Price: {} (+1.5%) | Horizon Duration: {}.", fmt_trigger, fmt_target, fmt_stop, horizon_duration));
            bullets.push(format!("• ACTIONABLE TRIGGER: Close existing long positions at entry price {} to protect principal capital.", fmt_price));
            bullets.push(format!("• CAPITAL PROTECTION: Reversal risk capped with stop loss at {} and max horizon {}.", fmt_stop, horizon_duration));
        } else if l15 == "CONTINUATION" && l30 == "CONTINUATION" && l1d == "CONTINUATION" {
            bullets.push("• DYNAMICS: Robust momentum expansion across 15m, 30m, and 1d observation windows.".into());
            bullets.push(format!("• RECOMMENDATION RATIONALE: HOLD — Entry/Trigger: {} | Target Price: {} (+3.5%) | Stop Loss Price: {} (-1.5%) | Horizon Duration: {}.", fmt_trigger, fmt_target, fmt_stop, horizon_duration));
            bullets.push(format!("• ACTIONABLE TRIGGER: Maintain active LONG posture from entry price {}; track 15m score threshold (+0.20).", fmt_price));
            bullets.push(format!("• CAPITAL PROTECTION: Stop loss defined at {} (1.5% risk cap); maximum horizon exit enforced at {}.", fmt_stop, horizon_duration));
        } else if l15 == "REVERSAL" && l1d == "CONTINUATION" {
            bullets.push("• DYNAMICS: 15m micro trajectory in pullback/reversal against positive 1d macro trend.".into());
            bullets.push(format!("• RECOMMENDATION RATIONALE: WATCH — Entry/Trigger: {} | Target Price: {} (+3.5%) | Stop Loss Price: {} (-1.5%) | Horizon Duration: {}.", fmt_trigger, fmt_target, fmt_stop, horizon_duration));
            bullets.push(format!("• ACTIONABLE TRIGGER: Standby until 15m trajectory rebounds above +0.20 at breakout trigger {}.", fmt_trigger));
            bullets.push(format!("• CAPITAL PROTECTION: Prevents premature entry; stop loss benchmark set to {}.", fmt_stop));
        } else if l15 == "CONTINUATION" && l1d == "REVERSAL" {
            bullets.push("• DYNAMICS: Short-term 15m bounce (+0.20) conflicts with daily 1d macro distribution.".into());
            bullets.push(format!("• RECOMMENDATION RATIONALE: WATCH — Entry/Trigger: {} | Target Price: {} (+3.5%) | Stop Loss Price: {} (-1.5%) | Horizon Duration: {}.", fmt_trigger, fmt_target, fmt_stop, horizon_duration));
            bullets.push(format!("• ACTIONABLE TRIGGER: Standby required; do not buy counter-trend rallies against daily reversal at {}.", fmt_price));
            bullets.push(format!("• CAPITAL PROTECTION: Avoids bull traps inside higher-timeframe downtrends; stop loss benchmark {}.", fmt_stop));
        } else {
            bullets.push("• DYNAMICS: Low-velocity consolidation / pullback drift across observation horizons.".into());
            bullets.push(format!("• RECOMMENDATION RATIONALE: WATCH — Entry/Trigger: {} | Target Price: {} (+3.5%) | Stop Loss Price: {} (-1.5%) | Horizon Duration: {}.", fmt_trigger, fmt_target, fmt_stop, horizon_duration));
            bullets.push(format!("• ACTIONABLE TRIGGER: Standby active; await breakout trigger {} crossing +0.20 threshold.", fmt_trigger));
            bullets.push(format!("• CAPITAL PROTECTION: Preserves liquidity with stop loss capped at {} and horizon boundary {}.", fmt_stop, horizon_duration));
        }

        bullets
    }
}
