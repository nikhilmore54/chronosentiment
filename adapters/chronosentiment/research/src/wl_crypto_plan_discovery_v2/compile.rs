//! Hypothesis compiler — falsifiable records only; never invents missing evidence.

use super::discover::{holdout_contrast, spec_meta};
use super::types::{
    CandidateAssociation, HypothesisRecord, StabilityAssessment, TradeRow,
};

pub fn compile_hypotheses(
    candidates: &[CandidateAssociation],
    stability: &[StabilityAssessment],
    holdout: &[&TradeRow],
) -> Vec<HypothesisRecord> {
    let mut out = Vec::new();
    for (c, s) in candidates.iter().zip(stability.iter()) {
        if !s.promote_to_compiler {
            continue;
        }
        let (of_era, excl) = spec_meta(&c.feature_id).unwrap_or((false, true));
        let rep = holdout_contrast(
            holdout,
            &c.feature_id,
            &c.high_key,
            &c.low_key,
            of_era,
            excl,
        );
        let verdict = replication_verdict(&c.discovery.wr_gap_pp, &rep, 15);
        let mut limitations = s.specificity_notes.clone();
        limitations.extend(s.block_reasons.iter().cloned());
        if let Some(id) = &c.v1_lineage_id {
            limitations.push(format!("v1_lineage={id}"));
        }
        if let Some(drop) = s.large_coverage_drop_if_selected_pct {
            limitations.push(format!("coverage_drop_if_selected_pct={drop:.1}"));
        }

        let expected = format!(
            "WR({}) > WR({}) by meaningful margin on eligible population",
            c.high_key, c.low_key
        );
        let falsify = format!(
            "On holdout (or predeclared eval set), WR({}) − WR({}) ≤ 0 with both sides n≥15, \
             or sign flips vs discovery, or either side underpowered",
            c.high_key, c.low_key
        );

        let gate_status = if verdict == "REPLICATES" {
            "Gate2_CANDIDATE_PENDING_CRITERIA".to_string()
        } else {
            "research-only".to_string()
        };

        out.push(HypothesisRecord {
            hypothesis_id: format!("H-V2-{}", c.candidate_id),
            book: "crypto_recommendation_plan".into(),
            population: format!(
                "Closed Crypto plan trades; feature={} high={} low={}; of_era_only={of_era}",
                c.feature_id, c.high_key, c.low_key
            ),
            feature: format!("{} (decision-time)", c.feature_id),
            comparison: format!("{} vs {}", c.high_key, c.low_key),
            outcome: "win := net_pnl_usdt > 0 (existing W/L definition)".into(),
            expected_direction: expected,
            mechanism: mechanism_text(&c.feature_id),
            discovery_evidence: c.discovery.clone(),
            replication_evidence: Some(rep),
            replication_verdict: verdict,
            known_limitations: limitations,
            falsification_condition: falsify,
            gate_status,
            v1_lineage_id: c.v1_lineage_id.clone(),
        });
    }
    out
}

fn replication_verdict(
    disc_gap: &Option<f64>,
    rep: &super::types::Contrast,
    min_n: usize,
) -> String {
    if rep.high.n < min_n || rep.low.n < min_n {
        return "INCONCLUSIVE_UNDERPOWERED".into();
    }
    match (*disc_gap, rep.wr_gap_pp) {
        (Some(d), Some(h)) if d > 0.0 && h > 0.0 => "REPLICATES".into(),
        (Some(d), Some(h)) if d > 0.0 && h <= 0.0 => "FAILS_REPLICATION".into(),
        _ => "INCONCLUSIVE".into(),
    }
}

fn mechanism_text(feature_id: &str) -> String {
    match feature_id {
        "direction" => "Directional alignment of plan side with latent market drift at entry.".into(),
        "of_supported" => {
            "OF latch quality as a contemporaneous confirmation of aggression/liquidity state.".into()
        }
        "dir_x_of" => "Joint directional and OF-support confirmation at decision time.".into(),
        "imb_bucket" => "Stronger absolute book imbalance as entry aggression intensity.".into(),
        "atr_bucket" => "Volatility regime affecting stop/target reachability under fixed geometry.".into(),
        "net_rr_bucket" => "Plan net RR geometry quality at entry.".into(),
        "hour_ist" => "Intraday temporal regime (diagnostic; high confound risk).".into(),
        _ => "Decision-time observable hypothesized to relate to entry win probability.".into(),
    }
}

/// Research-only falsifiable statements for candidates that failed stability
/// (transparency; not Gate 2 candidates).
pub fn compile_blocked_research_records(
    candidates: &[CandidateAssociation],
    stability: &[StabilityAssessment],
    holdout: &[&TradeRow],
) -> Vec<HypothesisRecord> {
    let mut out = Vec::new();
    for (c, s) in candidates.iter().zip(stability.iter()) {
        if s.promote_to_compiler {
            continue;
        }
        if !s.association_ok && c.discovery.wr_gap_pp.unwrap_or(0.0) <= 0.0 {
            continue;
        }
        let (of_era, excl) = spec_meta(&c.feature_id).unwrap_or((false, true));
        let rep = holdout_contrast(
            holdout,
            &c.feature_id,
            &c.high_key,
            &c.low_key,
            of_era,
            excl,
        );
        let verdict = replication_verdict(&c.discovery.wr_gap_pp, &rep, 15);
        let mut limitations = s.block_reasons.clone();
        limitations.extend(s.specificity_notes.clone());
        limitations.push("NOT_PROMOTED_TO_GATE2_COMPILER_INPUT".into());
        out.push(HypothesisRecord {
            hypothesis_id: format!("H-V2-BLOCKED-{}", c.candidate_id),
            book: "crypto_recommendation_plan".into(),
            population: format!(
                "Closed Crypto plan; blocked candidate {} vs {}",
                c.high_key, c.low_key
            ),
            feature: c.feature_id.clone(),
            comparison: format!("{} vs {}", c.high_key, c.low_key),
            outcome: "win := net_pnl_usdt > 0".into(),
            expected_direction: format!("WR({}) > WR({})", c.high_key, c.low_key),
            mechanism: mechanism_text(&c.feature_id),
            discovery_evidence: c.discovery.clone(),
            replication_evidence: Some(rep),
            replication_verdict: verdict,
            known_limitations: limitations,
            falsification_condition: "Stability and/or lineage rules already block promotion."
                .into(),
            gate_status: "research-only".into(),
            v1_lineage_id: c.v1_lineage_id.clone(),
        });
    }
    out
}
