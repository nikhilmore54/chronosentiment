//! CS-P-006-M.1 decision-value fitness for a future C.3 search.
//!
//! Search #1 continues to use `score_genome` (traded-only mean).
//! This module is not wired into `evolve_on_development`.
//! Evaluation cannot be scored. Regret and unique-best cannot enter fitness.

use coralys_moga::runtime::optimization::metric::MetricReport;
use coralys_moga::traits::FitnessEvaluator;

use super::csp006_protocol::RESEARCH_UNIVERSE;
use super::dataset_partition::PartitionKind;
use super::decision_value_harness::ProtocolValue;
use super::decision_value_landscape::action_value;
use super::observation_value::{GenomeEvaluation, ObservationSlice, SliceScore};
use super::policy_artifact::first_match_action;
use super::policy_genome::RuleListGenome;
use super::DecisionAction;

use std::collections::BTreeMap;

/// M.1 protocol value of a genome on a development or selection slice.
pub fn score_decision_value(
    genome: &RuleListGenome,
    slice: &ObservationSlice,
) -> Result<SliceScore, String> {
    if slice.kind == PartitionKind::Evaluation {
        return Err("decision-value fitness must not score the evaluation slice".into());
    }
    let mut n_traded = 0u32;
    let mut n_stood_aside = 0u32;
    let mut n_unavailable = 0u32;
    let mut per_instrument: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for ticker in RESEARCH_UNIVERSE {
        per_instrument.insert((*ticker).to_string(), Vec::new());
    }
    for ticker in RESEARCH_UNIVERSE {
        for row in slice.rows.iter().filter(|r| r.instrument == *ticker) {
            let action = first_match_action(&genome.rules, genome.unmatched_action, &row.profile);
            match row.instrument_return {
                None => n_unavailable += 1,
                Some(raw) => {
                    let v = action_value(action, raw);
                    per_instrument.get_mut(ticker).unwrap().push(v);
                    match action {
                        DecisionAction::NoTrade => n_stood_aside += 1,
                        DecisionAction::Long | DecisionAction::Short => n_traded += 1,
                    }
                }
            }
        }
    }
    let protocol = ProtocolValue::from_per_instrument_v(&per_instrument)?;
    Ok(SliceScore {
        fitness: protocol.value,
        n_rows: slice.rows.len(),
        n_traded,
        n_stood_aside,
        n_unavailable,
    })
}

/// Development-only M.1 evaluator. Not used by Search #1.
pub struct DevelopmentValue {
    slice: ObservationSlice,
}

impl DevelopmentValue {
    pub fn new(slice: ObservationSlice) -> Result<Self, String> {
        if slice.kind != PartitionKind::Development {
            return Err("development_value may only use the development slice".into());
        }
        Ok(Self { slice })
    }
}

impl FitnessEvaluator<RuleListGenome> for DevelopmentValue {
    type Evaluation = GenomeEvaluation;

    fn evaluate(&self, genome: &RuleListGenome, _metrics: &MetricReport) -> Self::Evaluation {
        let score =
            score_decision_value(genome, &self.slice).expect("development slice is permitted");
        GenomeEvaluation {
            genome: genome.clone(),
            fitness: score.fitness,
            valid: true,
        }
    }
}
