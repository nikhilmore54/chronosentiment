use chrono::Utc;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use coralys_moga::config::EvolutionConfig;
use coralys_moga::engine::{EvolutionEngine, GaResult as CoralysGaResult};
use coralys_moga::runtime::optimization::metric::MetricReport;
use coralys_moga::traits::{
    CrossoverOperator, Evaluated, FitnessEvaluator as CoralysFitnessEvaluator, Genome,
    GenomeFactory, MutationOperator,
};

pub trait FitnessEvaluator<T> {
    type Evaluation;
    fn evaluate(&self, candidate: &T) -> Self::Evaluation;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GaDiversityMode {
    Attract,
    Repel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateAnnotation {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct GaConfig {
    pub population_size: usize,
    pub generations: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub seed: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Candidate {
    pub queue_threshold: u64,
    pub base_edge: u64,
    pub take_profit: u64,
    pub stop_loss: u64,
    pub holding_period: u64,
    pub w_conviction: u64,
    pub w_momentum: u64,
    pub w_volatility: u64,
    pub exp_conviction: u64,
    pub exp_momentum: u64,
    pub exp_volatility: u64,
    pub selectivity: u8,
    pub archetype: u8,
    pub entry_offset: i32,
    pub direction_bias: u8,
    pub vol_floor: u8,
    pub mom_floor: u8,
    pub edge_ratio: u8,
    pub participation_threshold: u8,
    pub exec_aggression: u8,
    pub latency_bias: u8,
    pub fill_threshold: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CandidateEvaluation {
    pub candidate_edges: Vec<f64>,
    pub winner_idx: usize,
    pub strategy_id: String,
    pub candidate: Candidate,
    pub evaluation_valid: bool,
    pub real_dom: f64,
    pub had_organic_signals: bool,
    pub std_dev: f64,
    pub downside_std_dev: f64,
    pub worst: f64,
    pub robustness: f64,
    pub max_signature_credibility: f64,
    pub forced_win_ratio: f64,
    pub fitness: f64,
    pub trade_count: usize,
    pub max_drawdown: f64,
    pub participation_rate: f64,
    pub profitable_trades: usize,
    pub zero_pnl_trades: usize,
    pub quality_trades: f64,
    pub total_pnl: f64,
    pub avg_pnl: f64,
    pub win_rate: f64,
    pub payoff: f64,
    pub payoff_ratio: f64,
    pub direction_ratio: f64,
    pub baseline_pnl: f64,
    pub execution_friction: f64,
    pub scenario_signature: Vec<f64>,
    #[serde(default)]
    pub pnl_fingerprint: Vec<f32>,
    #[serde(default)]
    pub behavioral_signature: Vec<f64>,
    pub evaluation_flag: Option<String>,
    pub avg_conviction: f64,
    pub avg_efficiency: f64,
    pub avg_edge_quality: f64,
    pub directional_accuracy: f64,
    pub decisiveness: f64,
    pub short_term_capture_eff: f64,
    pub long_term_capture_eff: f64,
    pub trade_density: f64,
    pub queue_blocked_count: usize,
    pub liquidity_starved_count: usize,
    pub total_attempts: usize,
    pub exec_opportunity_rate: f64,
    #[serde(default)]
    pub failure_profile: Vec<f64>,
    #[serde(default)]
    pub realized_pnl_rolling: f64,
    #[serde(default)]
    pub predicted_pnl_rolling: f64,
    #[serde(default)]
    pub trade_qualities: Vec<f64>,
    #[serde(default)]
    pub outcome_consistency: f64,
    #[serde(default)]
    pub avg_trade_quality: f64,
    #[serde(default)]
    pub std_trade_quality: f64,
    pub exit_tp_count: usize,
    pub exit_sl_count: usize,
    pub exit_ts_count: usize,
    #[serde(default)]
    pub avg_hold_time: f64,
    pub annotations: Vec<CandidateAnnotation>,
    pub score_history: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GaResult {
    pub global_best: CandidateEvaluation,
    pub generation_history: Vec<CandidateEvaluation>,
    pub run_id: String,
    pub timestamp: i64,
    pub top_10: Vec<CandidateEvaluation>,
}

// -----------------------------------------------------------------------------
// LEGACY IMPLEMENTATION
// -----------------------------------------------------------------------------

pub fn random_strategy(_config: &GaConfig, rng: &mut StdRng) -> Candidate {
    Candidate {
        queue_threshold: rng.gen_range(50..5000),
        base_edge: rng.gen_range(5..200),
        take_profit: rng.gen_range(10..500),
        stop_loss: rng.gen_range(5..300),
        holding_period: rng.gen_range(5..200),
        w_conviction: rng.gen_range(5..150),
        w_momentum: rng.gen_range(5..150),
        w_volatility: rng.gen_range(5..150),
        exp_conviction: rng.gen_range(50..300),
        exp_momentum: rng.gen_range(50..300),
        exp_volatility: rng.gen_range(50..300),
        selectivity: rng.gen_range(20..100),
        archetype: rng.gen_range(0..4),
        entry_offset: rng.gen_range(-10..11),
        direction_bias: rng.gen_range(0..101),
        vol_floor: rng.gen_range(5..80),
        mom_floor: rng.gen_range(5..80),
        edge_ratio: rng.gen_range(50..250),
        participation_threshold: rng.gen_range(5..80),
        exec_aggression: rng.gen_range(20..100),
        latency_bias: rng.gen_range(0..100),
        fill_threshold: rng.gen_range(10..90),
    }
}

pub fn initialize_population(config: &GaConfig, rng: &mut StdRng) -> Vec<Candidate> {
    (0..config.population_size)
        .map(|_| random_strategy(config, rng))
        .collect()
}

pub fn crossover(parent1: &Candidate, parent2: &Candidate, rng: &mut StdRng) -> Candidate {
    let mut child = parent1.clone();

    if rng.gen_bool(0.5) {
        child.queue_threshold = parent2.queue_threshold;
    }
    if rng.gen_bool(0.5) {
        child.base_edge = parent2.base_edge;
    }
    if rng.gen_bool(0.5) {
        child.take_profit = parent2.take_profit;
    }
    if rng.gen_bool(0.5) {
        child.stop_loss = parent2.stop_loss;
    }
    if rng.gen_bool(0.5) {
        child.holding_period = parent2.holding_period;
    }
    if rng.gen_bool(0.5) {
        child.w_conviction = parent2.w_conviction;
    }
    if rng.gen_bool(0.5) {
        child.w_momentum = parent2.w_momentum;
    }
    if rng.gen_bool(0.5) {
        child.w_volatility = parent2.w_volatility;
    }
    if rng.gen_bool(0.5) {
        child.selectivity = parent2.selectivity;
    }
    if rng.gen_bool(0.5) {
        child.archetype = parent2.archetype;
    }

    child
}

pub fn mutate_candidate(candidate: &mut Candidate, rng: &mut StdRng, mutation_scale: f64) {
    if rng.gen_bool(0.05 * mutation_scale) {
        candidate.holding_period = candidate.holding_period.saturating_add(rng.gen_range(1..5));
    }
    if rng.gen_bool(0.05 * mutation_scale) {
        candidate.take_profit = candidate.take_profit.saturating_add(rng.gen_range(1..5));
    }
}

pub fn tournament_selection<'a>(
    evaluations: &'a [CandidateEvaluation],
    k: usize,
    rng: &mut StdRng,
) -> &'a CandidateEvaluation {
    let mut best: Option<&'a CandidateEvaluation> = None;
    for _ in 0..k {
        let idx = rng.gen_range(0..evaluations.len());
        let candidate = &evaluations[idx];
        if best.is_none() || candidate.fitness > best.unwrap().fitness {
            best = Some(candidate);
        }
    }
    best.unwrap()
}

pub fn legacy_run_ga_evolution(
    config: GaConfig,
    evaluator: &dyn FitnessEvaluator<Candidate, Evaluation = CandidateEvaluation>,
) -> GaResult {
    let mut rng = StdRng::seed_from_u64(config.seed);

    let mut population = initialize_population(&config, &mut rng);
    let mut global_best: Option<CandidateEvaluation> = None;
    let mut history = Vec::new();

    for _gen in 0..config.generations {
        let mut evals: Vec<CandidateEvaluation> = population
            .iter()
            .map(|c| evaluator.evaluate(c))
            .filter(|e| e.evaluation_valid)
            .collect();

        if evals.is_empty() {
            population = initialize_population(&config, &mut rng);
            continue;
        }

        evals.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(Ordering::Equal));

        let gen_best = evals[0].clone();
        if global_best.is_none() || gen_best.fitness > global_best.as_ref().unwrap().fitness {
            global_best = Some(gen_best.clone());
        }
        history.push(gen_best.clone());

        let mut next_gen = Vec::with_capacity(config.population_size);
        next_gen.extend(evals.iter().take(2).map(|e| e.candidate.clone()));

        while next_gen.len() < config.population_size {
            let parent1 = tournament_selection(&evals, 3, &mut rng);
            let parent2 = tournament_selection(&evals, 3, &mut rng);

            let mut child = crossover(&parent1.candidate, &parent2.candidate, &mut rng);
            mutate_candidate(&mut child, &mut rng, 1.0);
            next_gen.push(child);
        }

        population = next_gen;
    }

    GaResult {
        global_best: global_best.unwrap_or_else(|| {
            let dummy = initialize_population(&config, &mut rng)[0].clone();
            evaluator.evaluate(&dummy)
        }),
        generation_history: history,
        run_id: "generic-run".to_string(),
        timestamp: Utc::now().timestamp(),
        top_10: Vec::new(),
    }
}

// -----------------------------------------------------------------------------
// CORALYS ADAPTER LAYER
// -----------------------------------------------------------------------------

impl Genome for Candidate {}

impl Evaluated for CandidateEvaluation {
    type Genome = Candidate;
    fn fitness(&self) -> f64 {
        self.fitness
    }
    fn is_valid(&self) -> bool {
        self.evaluation_valid
    }
    fn genome(&self) -> &Self::Genome {
        &self.candidate
    }
}

pub struct ChronoFactory {
    config: GaConfig,
}
impl GenomeFactory<Candidate> for ChronoFactory {
    fn create(&self, rng: &mut StdRng) -> Candidate {
        random_strategy(&self.config, rng)
    }
}

pub struct ChronoMutator;
impl MutationOperator<Candidate> for ChronoMutator {
    fn mutate(&self, candidate: &mut Candidate, rng: &mut StdRng) {
        mutate_candidate(candidate, rng, 1.0);
    }
}

pub struct ChronoCrossover;
impl CrossoverOperator<Candidate> for ChronoCrossover {
    fn crossover(
        &self,
        parent_a: &Candidate,
        parent_b: &Candidate,
        rng: &mut StdRng,
    ) -> (Candidate, Candidate) {
        let child = crossover(parent_a, parent_b, rng);
        // The Coralys trait returns 2 children, but ChronoSentiment logic only returned 1.
        // We simply return the child for both, and let the caller throw one away or just use it.
        (child.clone(), child)
    }
}

pub struct ChronoFitnessEvaluator<'a> {
    legacy_evaluator: &'a dyn FitnessEvaluator<Candidate, Evaluation = CandidateEvaluation>,
}
impl<'a> CoralysFitnessEvaluator<Candidate> for ChronoFitnessEvaluator<'a> {
    type Evaluation = CandidateEvaluation;
    fn evaluate(&self, genome: &Candidate, _metrics: &MetricReport) -> CandidateEvaluation {
        self.legacy_evaluator.evaluate(genome)
    }
}

pub fn coralys_run_ga_evolution(
    config: GaConfig,
    evaluator: &dyn FitnessEvaluator<Candidate, Evaluation = CandidateEvaluation>,
) -> GaResult {
    let factory = ChronoFactory {
        config: config.clone(),
    };
    let engine = EvolutionEngine::new(
        ChronoFitnessEvaluator {
            legacy_evaluator: evaluator,
        },
        ChronoMutator,
        ChronoCrossover,
        factory,
    );

    let coralys_config = EvolutionConfig {
        population_size: config.population_size,
        mutation_rate: config.mutation_rate,
        crossover_rate: config.crossover_rate,
        elite_count: 10,
        generation_limit: config.generations,
        seed: Some(config.seed),
        tournament_size: None,
        ..Default::default()
    };

    let result = engine
        .run_ga_evolution(coralys_config)
        .expect("EvolutionEngine failed");

    GaResult {
        global_best: result.global_best,
        generation_history: result.generation_history,
        run_id: "coralys-run".to_string(),
        timestamp: Utc::now().timestamp(),
        top_10: result.top_10,
    }
}

pub fn run_ga_evolution(
    config: GaConfig,
    evaluator: &dyn FitnessEvaluator<Candidate, Evaluation = CandidateEvaluation>,
) -> GaResult {
    coralys_run_ga_evolution(config, evaluator)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyLegacyEvaluator;
    impl FitnessEvaluator<Candidate> for DummyLegacyEvaluator {
        type Evaluation = CandidateEvaluation;
        fn evaluate(&self, candidate: &Candidate) -> Self::Evaluation {
            let mut eval = CandidateEvaluation::default();
            eval.candidate = candidate.clone();
            // A deterministic but non-trivial fitness function
            eval.fitness = (candidate.queue_threshold as f64) + (candidate.take_profit as f64);
            eval.evaluation_valid = true;
            eval
        }
    }

    #[test]
    fn test_evolution_trajectory_equivalence() {
        let config = GaConfig {
            population_size: 20,
            generations: 5,
            mutation_rate: 0.1,
            crossover_rate: 0.8,
            seed: 42,
        };

        let evaluator = DummyLegacyEvaluator;

        let legacy_result = legacy_run_ga_evolution(config.clone(), &evaluator);
        let coralys_result = coralys_run_ga_evolution(config.clone(), &evaluator);

        assert_eq!(
            legacy_result.generation_history.len(),
            coralys_result.generation_history.len()
        );

        for (gen, (legacy, coralys)) in legacy_result
            .generation_history
            .iter()
            .zip(coralys_result.generation_history.iter())
            .enumerate()
        {
            assert_eq!(
                legacy.fitness, coralys.fitness,
                "Fitness mismatch at generation {}",
                gen
            );
            assert_eq!(
                legacy.candidate, coralys.candidate,
                "Candidate mismatch at generation {}",
                gen
            );
        }

        assert_eq!(
            legacy_result.global_best.fitness,
            coralys_result.global_best.fitness
        );
        assert_eq!(
            legacy_result.global_best.candidate,
            coralys_result.global_best.candidate
        );
    }
}
