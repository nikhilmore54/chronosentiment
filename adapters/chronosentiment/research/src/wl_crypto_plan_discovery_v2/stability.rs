//! Stability / redundancy diagnostics before hypothesis promotion.
//! These inform the compiler; they do NOT replace Gate 2 criteria.

use super::types::{CandidateAssociation, StabilityAssessment};

pub fn assess_stability(cands: &[CandidateAssociation]) -> Vec<StabilityAssessment> {
    cands.iter().map(assess_one).collect()
}

fn assess_one(c: &CandidateAssociation) -> StabilityAssessment {
    let mut block = Vec::new();
    let mut notes = Vec::new();

    let gap = c.discovery.wr_gap_pp.unwrap_or(0.0);
    let association_ok = gap > 5.0 && c.discovery.high.n >= 15 && c.discovery.low.n >= 15;
    if !association_ok {
        block.push("ASSOCIATION_WEAK_OR_UNDERPOWERED".into());
    }

    let evaluable: Vec<_> = c
        .session_effects
        .iter()
        .filter(|s| s.sign == "POSITIVE" || s.sign == "NEGATIVE" || s.sign == "NEAR_ZERO")
        .collect();
    let sign_flip = evaluable
        .iter()
        .filter(|s| s.sign == "NEGATIVE")
        .count();
    let positive = evaluable
        .iter()
        .filter(|s| s.sign == "POSITIVE")
        .count();

    let simpson = gap > 5.0 && positive > 0 && sign_flip > positive;
    if simpson {
        notes.push("Simpson-like risk: pooled positive but more negative session signs".into());
    }

    // Single-session domination of the high slice
    let high_n = c.discovery.high.n.max(1);
    let mut max_day_share = 0.0;
    for s in &c.session_effects {
        let share = s.high_n as f64 / high_n as f64;
        if share > max_day_share {
            max_day_share = share;
        }
    }
    let single_dom = max_day_share >= 0.70;
    if single_dom {
        block.push("SINGLE_SESSION_DOMINANT_HIGH_SLICE".into());
        notes.push(format!(
            "One session contributes ≥70% of high-slice n (max_share={:.1}%)",
            100.0 * max_day_share
        ));
    }

    let low_count = c.discovery.high.n < 15 || c.discovery.low.n < 15;
    if low_count {
        block.push("LOW_COUNT_COMPARISON".into());
    }

    let coverage_drop = c.discovery.high.coverage_pct.map(|cov| 100.0 - cov);
    if let Some(drop) = coverage_drop {
        if drop >= 50.0 {
            notes.push(format!(
                "Selecting high slice alone would drop ~{drop:.1}% of discovery trades"
            ));
        }
    }

    // Stability: need ≥2 evaluable sessions with positive sign and not majority flips
    let stability_ok = evaluable.len() >= 2 && positive >= 2 && sign_flip == 0 && !single_dom;
    if !stability_ok {
        block.push("STABILITY_FAIL".into());
        notes.push(format!(
            "sessions_evaluable={} positive_sign={} negative_sign={}",
            evaluable.len(),
            positive,
            sign_flip
        ));
    }

    if c.flags.iter().any(|f| f == "TEMPORAL_DIAGNOSTIC_ONLY") {
        block.push("TEMPORAL_DIAGNOSTIC_ONLY".into());
    }

    // Redundancy / lineage: v1 failed features cannot promote solely from rediscovery
    if c.v1_lineage_id.is_some() {
        notes.push(format!(
            "Overlaps v1 lineage {}; compiler will require holdout REPLICATES, not rediscovery",
            c.v1_lineage_id.as_deref().unwrap_or("")
        ));
        // Do not auto-block association reporting — block promotion to Gate2 candidate without replication
    }

    let promote = association_ok && stability_ok && !c.flags.iter().any(|f| f == "TEMPORAL_DIAGNOSTIC_ONLY");
    if !promote && block.is_empty() {
        block.push("PROMOTION_BLOCKED".into());
    }

    StabilityAssessment {
        candidate_id: c.candidate_id.clone(),
        association_ok,
        stability_ok,
        specificity_notes: notes,
        pooled_vs_session_simpson_risk: simpson,
        single_session_dominant: single_dom,
        low_count_comparison: low_count,
        large_coverage_drop_if_selected_pct: coverage_drop,
        sign_flip_sessions: sign_flip,
        sessions_evaluable: evaluable.len(),
        promote_to_compiler: promote,
        block_reasons: block,
    }
}
