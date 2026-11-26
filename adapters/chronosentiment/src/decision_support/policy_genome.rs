//! Rule-list genome Coralys evolves. Not a handwritten mapping.

use coralys_moga::traits::{CrossoverOperator, Genome, GenomeFactory, MutationOperator};
use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::csp006_protocol::MAX_RULES_FIRST_DISCOVERY;
use super::policy_artifact::{DecisionRule, FactorPredicate};
use super::DecisionAction;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleListGenome {
    pub rules: Vec<DecisionRule>,
    pub unmatched_action: DecisionAction,
}

impl Genome for RuleListGenome {}

impl RuleListGenome {
    pub fn identity_hash(&self) -> String {
        let bytes = serde_json::to_vec(self).expect("genome serializes");
        format!("{:x}", Sha256::digest(&bytes))
    }

    pub fn canonicalize(&mut self) {
        if self.rules.len() > MAX_RULES_FIRST_DISCOVERY {
            self.rules.truncate(MAX_RULES_FIRST_DISCOVERY);
        }
        for rule in &mut self.rules {
            rule.when.sort_by(|a, b| a.concept.cmp(&b.concept));
            rule.when.dedup();
            rule.when.retain(|p| {
                if p.concept == "Volatility" {
                    p.direction.is_none()
                } else {
                    true
                }
            });
        }
    }
}

pub struct RuleListFactory;

impl GenomeFactory<RuleListGenome> for RuleListFactory {
    fn create(&self, rng: &mut StdRng) -> RuleListGenome {
        let n = rng.gen_range(0..=6);
        let mut genome = RuleListGenome {
            rules: (0..n).map(|_| random_rule(rng)).collect(),
            unmatched_action: random_action(rng),
        };
        genome.canonicalize();
        genome
    }
}

pub struct RuleListMutation;

impl MutationOperator<RuleListGenome> for RuleListMutation {
    fn mutate(&self, genome: &mut RuleListGenome, rng: &mut StdRng) {
        match rng.gen_range(0..5) {
            0 if !genome.rules.is_empty() => {
                let i = rng.gen_range(0..genome.rules.len());
                genome.rules[i].action = random_action(rng);
            }
            1 if !genome.rules.is_empty() => {
                let i = rng.gen_range(0..genome.rules.len());
                genome.rules[i] = random_rule(rng);
            }
            2 if genome.rules.len() < MAX_RULES_FIRST_DISCOVERY => {
                genome.rules.push(random_rule(rng));
            }
            3 if genome.rules.len() > 1 => {
                let i = rng.gen_range(0..genome.rules.len());
                genome.rules.remove(i);
            }
            _ => genome.unmatched_action = random_action(rng),
        }
        genome.canonicalize();
    }
}

pub struct RuleListCrossover;

impl CrossoverOperator<RuleListGenome> for RuleListCrossover {
    fn crossover(
        &self,
        parent_a: &RuleListGenome,
        parent_b: &RuleListGenome,
        rng: &mut StdRng,
    ) -> (RuleListGenome, RuleListGenome) {
        let cut_a = if parent_a.rules.is_empty() {
            0
        } else {
            rng.gen_range(0..=parent_a.rules.len())
        };
        let cut_b = if parent_b.rules.is_empty() {
            0
        } else {
            rng.gen_range(0..=parent_b.rules.len())
        };
        let mut child_a = RuleListGenome {
            rules: [
                parent_a.rules[..cut_a].to_vec(),
                parent_b.rules[cut_b..].to_vec(),
            ]
            .concat(),
            unmatched_action: if rng.gen_bool(0.5) {
                parent_a.unmatched_action
            } else {
                parent_b.unmatched_action
            },
        };
        let mut child_b = RuleListGenome {
            rules: [
                parent_b.rules[..cut_b].to_vec(),
                parent_a.rules[cut_a..].to_vec(),
            ]
            .concat(),
            unmatched_action: if rng.gen_bool(0.5) {
                parent_a.unmatched_action
            } else {
                parent_b.unmatched_action
            },
        };
        child_a.canonicalize();
        child_b.canonicalize();
        (child_a, child_b)
    }
}

fn random_action(rng: &mut StdRng) -> DecisionAction {
    match rng.gen_range(0..3) {
        0 => DecisionAction::Long,
        1 => DecisionAction::Short,
        _ => DecisionAction::NoTrade,
    }
}

fn random_rule(rng: &mut StdRng) -> DecisionRule {
    let n = rng.gen_range(1..=3);
    let mut when = Vec::with_capacity(n);
    for _ in 0..n {
        when.push(random_predicate(rng));
    }
    DecisionRule {
        when,
        action: random_action(rng),
    }
}

fn random_predicate(rng: &mut StdRng) -> FactorPredicate {
    match rng.gen_range(0..3) {
        0 => FactorPredicate {
            concept: "Trend".to_string(),
            present: Some(true),
            direction: Some(["Bullish", "Bearish", "Neutral"][rng.gen_range(0..3)].to_string()),
        },
        1 => FactorPredicate {
            concept: "Momentum".to_string(),
            present: Some(true),
            direction: Some(["Positive", "Negative", "Neutral"][rng.gen_range(0..3)].to_string()),
        },
        _ => FactorPredicate {
            concept: "Volatility".to_string(),
            present: Some(rng.gen_bool(0.8)),
            direction: None,
        },
    }
}
