//! ChronoSentiment evaluation handoff after a sealed candidate exists.
//!
//! Coralys does not call this during evolution or selection.

use serde::Serialize;

use super::dataset_partition::PartitionKind;
use super::observation_value::{ObservationSlice, SliceScore};
use super::policy_artifact::PolicyArtifact;
use super::policy_genome::RuleListGenome;

#[derive(Debug, Clone, Serialize)]
pub struct EvaluationHandoff {
    pub artifact_hash: String,
    pub policy_name: String,
    pub evaluation: SliceScore,
}

pub fn evaluate_sealed_candidate(
    artifact: &PolicyArtifact,
    evaluation: &ObservationSlice,
) -> Result<EvaluationHandoff, String> {
    if evaluation.kind != PartitionKind::Evaluation {
        return Err("handoff requires the evaluation slice".into());
    }
    if artifact.artifact_hash.is_empty() {
        return Err("candidate is not sealed".into());
    }
    let genome = RuleListGenome {
        rules: artifact.rules.clone(),
        unmatched_action: artifact.unmatched_action,
    };
    Ok(EvaluationHandoff {
        artifact_hash: artifact.artifact_hash.clone(),
        policy_name: format!("{}@{}", artifact.policy_id, artifact.policy_version),
        evaluation: score_holdout(&genome, evaluation),
    })
}

fn score_holdout(genome: &RuleListGenome, slice: &ObservationSlice) -> SliceScore {
    use super::csp006_protocol::RESEARCH_UNIVERSE;
    use super::policy_artifact::first_match_action;
    use super::DecisionAction;
    let mut n_traded = 0u32;
    let mut n_stood_aside = 0u32;
    let mut n_unavailable = 0u32;
    let mut per_instrument: Vec<f64> = Vec::new();
    for ticker in RESEARCH_UNIVERSE {
        let mut traded = Vec::new();
        for row in slice.rows.iter().filter(|r| r.instrument == *ticker) {
            let action = first_match_action(&genome.rules, genome.unmatched_action, &row.profile);
            match action {
                DecisionAction::NoTrade => n_stood_aside += 1,
                DecisionAction::Long | DecisionAction::Short => match row.instrument_return {
                    Some(raw) => {
                        n_traded += 1;
                        traded.push(if action == DecisionAction::Long {
                            raw
                        } else {
                            -raw
                        });
                    }
                    None => n_unavailable += 1,
                },
            }
        }
        per_instrument.push(if traded.is_empty() {
            0.0
        } else {
            traded.iter().sum::<f64>() / traded.len() as f64
        });
    }
    SliceScore {
        fitness: per_instrument.iter().sum::<f64>() / per_instrument.len() as f64,
        n_rows: slice.rows.len(),
        n_traded,
        n_stood_aside,
        n_unavailable,
    }
}

pub fn render_handoff(handoff: &EvaluationHandoff) -> String {
    let mut md = String::from("# ChronoSentiment evaluation handoff\n\n");
    md.push_str("Independent evaluation of a **sealed** PolicyArtifact. Coralys receives no feedback from this file.\n\n");
    md.push_str(&format!("- policy: `{}`\n", handoff.policy_name));
    md.push_str(&format!("- artifact_hash: `{}`\n", handoff.artifact_hash));
    md.push_str(&format!(
        "- evaluation mean signed traded return: {:.6}\n",
        handoff.evaluation.fitness
    ));
    md.push_str(&format!("- traded: {}\n", handoff.evaluation.n_traded));
    md.push_str(&format!(
        "- stood aside: {}\n",
        handoff.evaluation.n_stood_aside
    ));
    md.push_str("\nThis number is evidence, not a reason to retune the genome.\n");
    md
}
