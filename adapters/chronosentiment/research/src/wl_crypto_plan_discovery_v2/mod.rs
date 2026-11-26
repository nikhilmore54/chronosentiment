pub mod audit;
pub mod compile;
pub mod data;
pub mod discover;
pub mod gate2;
pub mod lineage;
pub mod report;
pub mod stability;
pub mod types;

use self::audit::{feature_availability_matrix, integrity_check, session_composition};
use self::compile::{compile_blocked_research_records, compile_hypotheses};
use self::data::{filter_days, load_trades, DISCOVERY_DAYS, HOLDOUT_EX_ANOMALY};
use self::discover::discover_associations;
use self::gate2::{evaluate_gate2, overall_disposition};
use self::lineage::v1_lineage_registry;
use self::stability::assess_stability;
use self::types::PipelineReport;
use std::path::Path;

pub fn run_pipeline(journal: &Path) -> Result<PipelineReport, String> {
    let all = load_trades(journal)?;
    let (integrity_pass, integrity_notes) = integrity_check(&all);
    if !integrity_pass {
        return Ok(PipelineReport {
            protocol: "RP-608-WL-IMPROVE-001-DISCOVERY-V2".into(),
            version: "v2".into(),
            authority: "0.0%".into(),
            baseline_frozen: true,
            gate2_criteria_changed: false,
            integrity_pass: false,
            integrity_notes,
            feature_availability: vec![],
            session_composition: vec![],
            lineage_registry: v1_lineage_registry(),
            candidates: vec![],
            stability: vec![],
            hypotheses: vec![],
            research_only_blocked_records: vec![],
            gate2_assessments: vec![],
            gate2_disposition: "RESEARCH_ONLY_DATA_INTEGRITY_UNRESOLVED".into(),
            research_conclusion: "Data integrity unresolved — no Gate 2 candidate.".into(),
        });
    }

    let discovery = filter_days(&all, DISCOVERY_DAYS);
    let holdout = filter_days(&all, HOLDOUT_EX_ANOMALY);
    let availability = feature_availability_matrix(&discovery, &holdout);
    let composition = session_composition(&all);
    let lineage = v1_lineage_registry();
    let candidates = discover_associations(&discovery);
    let stability = assess_stability(&candidates);
    let hypotheses = compile_hypotheses(&candidates, &stability, &holdout);
    let blocked = compile_blocked_research_records(&candidates, &stability, &holdout);
    let assessments: Vec<_> = hypotheses.iter().map(evaluate_gate2).collect();
    let gate2_disposition = overall_disposition(&assessments);

    let promoted = stability.iter().filter(|s| s.promote_to_compiler).count();
    let replicates = hypotheses
        .iter()
        .filter(|h| h.replication_verdict == "REPLICATES")
        .count();
    let advanced = assessments.iter().filter(|a| a.all_satisfied).count();

    let research_conclusion = if advanced > 0 {
        format!(
            "Discovery v2 produced {advanced} hypothesis(es) satisfying unchanged Gate 2 criteria."
        )
    } else if promoted == 0 {
        "No Gate 2 hypothesis is supported by the available decision-time features under the v2 \
         discovery algorithm. Stability review promoted 0 candidates to the hypothesis compiler. \
         Available recorded features remain insufficient for a falsifiable, replicable entry-quality \
         claim — without changing Gate 2 criteria or remine-slicing failed v1 lineage."
            .into()
    } else {
        format!(
            "Compiler emitted {promoted} stability-promoted hypothesis(es); REPLICATES={replicates}; \
             none satisfied all unchanged Gate 2 criteria. Gate 2 remains closed."
        )
    };

    Ok(PipelineReport {
        protocol: "RP-608-WL-IMPROVE-001-DISCOVERY-V2".into(),
        version: "v2".into(),
        authority: "0.0%".into(),
        baseline_frozen: true,
        gate2_criteria_changed: false,
        integrity_pass: true,
        integrity_notes,
        feature_availability: availability,
        session_composition: composition,
        lineage_registry: lineage,
        candidates,
        stability,
        hypotheses,
        research_only_blocked_records: blocked,
        gate2_assessments: assessments,
        gate2_disposition,
        research_conclusion,
    })
}
