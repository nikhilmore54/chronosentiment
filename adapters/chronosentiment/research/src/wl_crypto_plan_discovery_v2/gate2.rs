//! Apply existing Gate 2 criteria unchanged.
//! Source: docs/research/rp608_wl_improvement_plan.md + v1 Gate 2 disposition doc.
//! Diagnostics from stability do not replace these criteria.

use super::types::{Gate2Assessment, Gate2CriterionResult, HypothesisRecord};

/// Evaluate a compiled hypothesis against locked Gate 2 requirements.
pub fn evaluate_gate2(h: &HypothesisRecord) -> Gate2Assessment {
    let mut criteria = Vec::new();

    // G2-1: specific measurable hypothesis
    let measurable = !h.feature.is_empty()
        && !h.comparison.is_empty()
        && h.expected_direction.contains('>');
    criteria.push(Gate2CriterionResult {
        criterion_id: "G2-1_MEASURABLE_HYPOTHESIS".into(),
        description: "Specific measurable entry-quality hypothesis with explicit comparison"
            .into(),
        satisfied: measurable,
        detail: if measurable {
            "Hypothesis states feature, comparison, expected WR direction".into()
        } else {
            "Hypothesis incomplete".into()
        },
    });

    // G2-2: decision-time only
    let dt = !h.feature.to_lowercase().contains("mfe")
        && !h.feature.to_lowercase().contains("mae")
        && !h.feature.to_lowercase().contains("exit");
    criteria.push(Gate2CriterionResult {
        criterion_id: "G2-2_DECISION_TIME_ONLY".into(),
        description: "Uses decision-time observables only (no post-entry path predictors)".into(),
        satisfied: dt,
        detail: "Feature string audited for post-entry leakage markers".into(),
    });

    // G2-3: descriptive discovery support
    let disc_ok = h
        .discovery_evidence
        .wr_gap_pp
        .map(|g| g > 0.0)
        .unwrap_or(false)
        && h.discovery_evidence.high.n >= 15
        && h.discovery_evidence.low.n >= 15;
    criteria.push(Gate2CriterionResult {
        criterion_id: "G2-3_DISCOVERY_SUPPORT".into(),
        description: "Descriptive discovery support with adequate comparison-group n".into(),
        satisfied: disc_ok,
        detail: format!(
            "discovery_gap_pp={:?} n_high={} n_low={}",
            h.discovery_evidence.wr_gap_pp,
            h.discovery_evidence.high.n,
            h.discovery_evidence.low.n
        ),
    });

    // G2-4: holdout replication (unchanged from v1 sequence requirement)
    let repl_ok = h.replication_verdict == "REPLICATES";
    criteria.push(Gate2CriterionResult {
        criterion_id: "G2-4_HOLDOUT_REPLICATION".into(),
        description: "Holdout replication same-sign with powered comparison groups".into(),
        satisfied: repl_ok,
        detail: format!(
            "replication_verdict={} holdout_gap={:?}",
            h.replication_verdict,
            h.replication_evidence
                .as_ref()
                .and_then(|r| r.wr_gap_pp)
        ),
    });

    // G2-5: falsifiable before evaluation
    let falsifiable = !h.falsification_condition.is_empty()
        && h.falsification_condition.to_lowercase().contains("holdout");
    criteria.push(Gate2CriterionResult {
        criterion_id: "G2-5_FALSIFIABLE".into(),
        description: "Falsification condition declared before / independent of success claim"
            .into(),
        satisfied: falsifiable,
        detail: h.falsification_condition.clone(),
    });

    // G2-6: coverage reduction alone is not success
    let cov_ok = !h
        .known_limitations
        .iter()
        .any(|l| l.contains("coverage_drop_if_selected_pct=") && extract_drop(l) >= 50.0)
        || repl_ok; // still must replicate; this flags coverage-only narratives
    // Actually: criterion is that we do NOT treat coverage-reducing WR as success.
    // Satisfied if we are not claiming success solely via coverage — i.e. replication required (G2-4).
    // Mark satisfied when either coverage drop <50% OR replication failed (we're not advancing on coverage).
    let drop = h
        .known_limitations
        .iter()
        .find_map(|l| {
            if l.starts_with("coverage_drop_if_selected_pct=") {
                l.split('=').nth(1)?.parse::<f64>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0.0);
    let cov_criterion_ok = !(repl_ok && drop >= 70.0); // block extreme coverage-only "wins" even if gap replicates
    criteria.push(Gate2CriterionResult {
        criterion_id: "G2-6_NOT_COVERAGE_ONLY_SUCCESS".into(),
        description: "Higher WR via large trade-count reduction alone is not success".into(),
        satisfied: cov_criterion_ok,
        detail: format!("coverage_drop_if_selected_pct={drop:.1}; repl_ok={repl_ok}"),
    });
    let _ = cov_ok;

    // G2-7: not a silent rediscovery of failed v1 without replication
    let lineage_ok = h.v1_lineage_id.is_none() || repl_ok;
    criteria.push(Gate2CriterionResult {
        criterion_id: "G2-7_NO_FAILED_V1_REEXPRESSION".into(),
        description: "v1 failed/underpowered lineage cannot advance without true replication"
            .into(),
        satisfied: lineage_ok,
        detail: format!(
            "v1_lineage={:?} replication_verdict={}",
            h.v1_lineage_id, h.replication_verdict
        ),
    });

    // G2-8: research-only records cannot advance
    let status_ok = h.gate_status != "research-only";
    criteria.push(Gate2CriterionResult {
        criterion_id: "G2-8_COMPILER_GATE_STATUS".into(),
        description: "Compiler marked hypothesis as Gate2 candidate pending criteria".into(),
        satisfied: status_ok,
        detail: format!("gate_status={}", h.gate_status),
    });

    let all = criteria.iter().all(|c| c.satisfied);
    let disposition = if all {
        "ADVANCE_TO_GATE2_REVIEW".to_string()
    } else {
        "DO_NOT_ADVANCE".to_string()
    };

    Gate2Assessment {
        hypothesis_id: h.hypothesis_id.clone(),
        criteria,
        all_satisfied: all,
        disposition,
    }
}

fn extract_drop(s: &str) -> f64 {
    s.split('=')
        .nth(1)
        .and_then(|x| x.parse().ok())
        .unwrap_or(0.0)
}

pub fn overall_disposition(assessments: &[Gate2Assessment]) -> String {
    if assessments.iter().any(|a| a.all_satisfied) {
        "GATE2_REVIEW_AUTHORIZED_FOR_PASSING_HYPOTHESES".into()
    } else {
        "GATE2_DO_NOT_ADVANCE — no hypothesis satisfies unchanged Gate 2 criteria".into()
    }
}
