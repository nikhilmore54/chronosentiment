use crate::domain::StrategyEvaluation;
use crate::pipeline::SignalAction;

/// Represents the directional strength of a signal.
/// Positive values indicate BUY pressure, negative indicates SELL.
#[derive(Debug, Clone, Copy)]
pub struct SignalStrength {
    pub value: f64, // signed: +BUY, -SELL (edge * confidence)
}

/// A cluster-representative strategy with dynamic weighting factors.
#[derive(Debug, Clone)]
pub struct EnsembleMember<'a> {
    pub strategy_id: &'a str,
    pub weight: f64,
}

/// Binds a member to its evaluated signal for a specific scenario.
pub struct EnsembleInput<'a> {
    pub member: &'a EnsembleMember<'a>,
    pub evaluation: &'a StrategyEvaluation,
    pub signal: SignalStrength,
}

/// The result of an ensemble decision, ready for the pipeline's execution gate.
#[derive(Debug, Clone)]
pub struct ConsensusDecision {
    pub combined_action: SignalAction,
    pub consensus_score: f64,    // Normalized Σ(w_i * signal_i) / Σ|w_i|
    pub disagreement_score: f64, // std(signals) / (mean|signals| + ε)
    pub is_conflict_hold: bool,  // true if disagreement > threshold
    pub effective_edge: f64,
    pub contributing_members: Vec<String>,
}

/// Computes the dynamic weighting for an ensemble member based on
/// normalized long-term fitness, consistency, and current regime alignment.
/// Allows for negative weights (penalizing historically bad strategies in this regime).
pub fn calculate_member_weight(
    eval: &StrategyEvaluation,
    regime_alignment: f64,
    pop_fitness_mu: f64,
    pop_fitness_sigma: f64,
    pop_consistency_mu: f64,
    pop_consistency_sigma: f64,
    pop_recent_mu: f64,
    pop_recent_sigma: f64,
) -> f64 {
    let f_norm = (eval.fitness - pop_fitness_mu) / pop_fitness_sigma.max(1e-6);
    // TODO: Restore domain metrics (consistency, recent_performance) calculation using new Evaluator
    let c_norm = (eval.fitness - pop_consistency_mu) / pop_consistency_sigma.max(1e-6);
    let r_norm = (eval.fitness - pop_recent_mu) / pop_recent_sigma.max(1e-6);

    // Final multi-factor weight (Phase 11.2) - ALLOWING SIGNED WEIGHTS
    let w = 0.4 * f_norm + 0.2 * c_norm + 0.2 * r_norm + 0.2 * regime_alignment;

    // Soft magnitude floor to prevent total collapse, but allowing sign preservation
    if w.abs() < 0.01 {
        0.0
    } else {
        w
    }
}

/// Computes the relative disagreement across a set of signals using
/// Welford's algorithm for numerical stability.
pub fn calculate_relative_disagreement(inputs: &[EnsembleInput]) -> f64 {
    let mut mean = 0.0;
    let mut m2 = 0.0;
    let mut count = 0.0;
    let mut abs_sum = 0.0;

    for input in inputs {
        // Apply Clipping here as well for consistency in variance
        let x = input.signal.value.clamp(-1.0, 1.0);
        count += 1.0;

        let delta = x - mean;
        mean += delta / count;
        let delta2 = x - mean;
        m2 += delta * delta2;

        abs_sum += x.abs();
    }

    if count < 2.0 {
        return 0.0;
    }

    // Population variance
    let variance = m2 / count;
    let std_dev = variance.sqrt();
    let mean_abs = abs_sum / count;

    if mean_abs > 1e-9 {
        std_dev / mean_abs
    } else {
        0.0
    }
}

/// The core decision logic. Pipeline calls evaluate_all_strategies() to get signals,
/// then hands them off here to synthesize the final action.
pub fn compute_consensus(
    inputs: &[EnsembleInput],
    decision_threshold: f64,
    disagreement_limit: f64,
) -> ConsensusDecision {
    if inputs.is_empty() {
        return ConsensusDecision {
            combined_action: SignalAction::HOLD,
            consensus_score: 0.0,
            disagreement_score: 0.0,
            is_conflict_hold: false,
            effective_edge: 0.0,
            contributing_members: Vec::new(),
        };
    }

    let mut weighted_signal_sum = 0.0;
    let mut weight_abs_sum = 0.0;
    let mut contributing_members = Vec::with_capacity(inputs.len());

    for input in inputs {
        // 0. SIGNAL CLIPPING (Phase 11.2 Safeguard: Outlier suppression)
        let x = input.signal.value.clamp(-1.0, 1.0);

        weighted_signal_sum += input.member.weight * x;
        weight_abs_sum += input.member.weight.abs();
        contributing_members.push(input.member.strategy_id.to_string());
    }

    // 1. Normalized Consensus Score (Phase 11.2)
    let consensus_score = if weight_abs_sum > 1e-12 {
        weighted_signal_sum / weight_abs_sum
    } else {
        0.0
    };

    // 2. DISAGREEMENT CALCULATION (Welford-based)
    let disagreement_score = calculate_relative_disagreement(inputs);

    // 3. Resolve Conflict
    let is_conflict_hold = disagreement_score > disagreement_limit;

    // 4. Final Action Determination
    let combined_action = if is_conflict_hold {
        SignalAction::HOLD
    } else if consensus_score > decision_threshold {
        SignalAction::BUY
    } else if consensus_score < -decision_threshold {
        SignalAction::SELL
    } else {
        SignalAction::HOLD
    };

    // 5. ATTENUATED EFFECTIVE EDGE (Phase 11.2 Sophistication)
    // Edge is attenuated by disagreement: High internal conflict -> Lower conviction
    let effective_edge = if is_conflict_hold {
        0.0
    } else {
        consensus_score.abs() * (1.0 - disagreement_score.min(1.0))
    };

    ConsensusDecision {
        combined_action,
        consensus_score,
        disagreement_score,
        is_conflict_hold,
        effective_edge,
        contributing_members,
    }
}
