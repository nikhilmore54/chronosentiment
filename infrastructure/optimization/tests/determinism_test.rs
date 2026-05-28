use chronosentiment_optimization::*;
use rand::rngs::StdRng;
use rand::SeedableRng;

struct MockEvaluator;

impl FitnessEvaluator<Candidate> for MockEvaluator {
    type Evaluation = CandidateEvaluation;

    fn evaluate(&self, candidate: &Candidate) -> Self::Evaluation {
        // A pure mathematical evaluation (e.g. sum of fields)
        let sum = candidate.queue_threshold as f64 + candidate.base_edge as f64;
        let fitness = sum / 5200.0;
        
        CandidateEvaluation {
            candidate_edges: vec![],
            winner_idx: 0,
            strategy_id: "mock".to_string(),
            candidate: candidate.clone(),
            evaluation_valid: true,
            real_dom: 0.0,
            had_organic_signals: false,
            std_dev: 0.0,
            downside_std_dev: 0.0,
            worst: 0.0,
            robustness: 0.0,
            max_signature_credibility: 0.0,
            forced_win_ratio: 0.0,
            fitness,
            trade_count: 100,
            max_drawdown: 0.0,
            participation_rate: 1.0,
            profitable_trades: 50,
            zero_pnl_trades: 0,
            quality_trades: 0.0,
            total_pnl: sum,
            avg_pnl: sum / 100.0,
            win_rate: 0.5,
            payoff: 1.0,
            payoff_ratio: 1.0,
            direction_ratio: 1.0,
            baseline_pnl: 0.0,
            execution_friction: 0.0,
            scenario_signature: vec![],
            pnl_fingerprint: vec![],
            behavioral_signature: vec![],
            evaluation_flag: None,
            avg_conviction: 0.0,
            avg_efficiency: 0.0,
            avg_edge_quality: 0.0,
            directional_accuracy: 0.0,
            decisiveness: 0.0,
            short_term_capture_eff: 0.0,
            long_term_capture_eff: 0.0,
            trade_density: 0.0,
            queue_blocked_count: 0,
            liquidity_starved_count: 0,
            total_attempts: 100,
            exec_opportunity_rate: 0.0,
            failure_profile: vec![],
            realized_pnl_rolling: 0.0,
            predicted_pnl_rolling: 0.0,
            trade_qualities: vec![],
            outcome_consistency: 0.0,
            avg_trade_quality: 0.0,
            std_trade_quality: 0.0,
            exit_tp_count: 0,
            exit_sl_count: 0,
            exit_ts_count: 0,
            avg_hold_time: 0.0,
            annotations: vec![],
            score_history: vec![],
        }
    }
}

#[test]
fn test_candidate_generation_determinism() {
    let mut config = GaConfig::default();
    config.population_size = 10;
    
    let mut rng1 = StdRng::seed_from_u64(42);
    let mut rng2 = StdRng::seed_from_u64(42);
    let mut rng3 = StdRng::seed_from_u64(43);

    let pop1 = initialize_population(&config, &mut rng1);
    let pop2 = initialize_population(&config, &mut rng2);
    let pop3 = initialize_population(&config, &mut rng3);

    assert_eq!(pop1, pop2, "Population generation from same seed must be identical");
    assert_ne!(pop1, pop3, "Population generation from different seeds must differ");
}

#[test]
fn test_crossover_determinism() {
    let mut rng1 = StdRng::seed_from_u64(100);
    let mut rng2 = StdRng::seed_from_u64(100);

    let parent1 = Candidate::default();
    let mut parent2 = Candidate::default();
    parent2.queue_threshold = 999;
    parent2.base_edge = 999;

    let child1 = crossover(&parent1, &parent2, &mut rng1);
    let child2 = crossover(&parent1, &parent2, &mut rng2);

    assert_eq!(child1, child2, "Crossover must be deterministic for the same RNG sequence");
}

#[test]
fn evolution_is_deterministic_for_fixed_seed() {
    let seed = 42;
    
    let config = GaConfig {
        population_size: 20,
        generations: 5,
        mutation_rate: 0.1,
        crossover_rate: 0.8,
        seed,
    };

    let evaluator = MockEvaluator;

    let result1 = run_ga_evolution(config.clone(), &evaluator);
    let result2 = run_ga_evolution(config, &evaluator);

    assert_eq!(
        result1.global_best.candidate, 
        result2.global_best.candidate, 
        "Best candidate divergence"
    );
    assert_eq!(
        result1.global_best.fitness,
        result2.global_best.fitness,
        "Best fitness divergence"
    );
    
    for (idx, (h1, h2)) in result1.generation_history.iter().zip(result2.generation_history.iter()).enumerate() {
        assert_eq!(
            h1.candidate, h2.candidate,
            "History divergence at generation {}", idx
        );
    }
}
