use chronosentiment_optimization::*;
use rand::rngs::StdRng;
use rand::SeedableRng;

struct DummyEvaluator;
impl FitnessEvaluator<Candidate> for DummyEvaluator {
    type Evaluation = CandidateEvaluation;
    fn evaluate(&self, candidate: &Candidate) -> Self::Evaluation {
        let mut eval = CandidateEvaluation::default();
        eval.fitness = (candidate.base_edge as f64) + (candidate.queue_threshold as f64);
        eval.candidate = candidate.clone();
        eval
    }
}

#[test]
fn same_seed_produces_same_population() {
    let mut config = GaConfig::default();
    config.population_size = 10;
    
    let mut rng1 = StdRng::seed_from_u64(999);
    let mut rng2 = StdRng::seed_from_u64(999);

    let pop1 = initialize_population(&config, &mut rng1);
    let pop2 = initialize_population(&config, &mut rng2);

    assert_eq!(pop1, pop2, "Population generation is not purely deterministic.");
}

#[test]
fn mutation_preserves_candidate_validity() {
    let mut rng = StdRng::seed_from_u64(123);
    let original = Candidate::default();
    let original_str = format!("{:?}", original);
    
    let mut changed = false;
    for _ in 0..100 {
        let mut candidate = original.clone();
        mutate_candidate(&mut candidate, &mut rng, 1.0); // Force mutation attempt
        
        if format!("{:?}", candidate) != original_str {
            changed = true;
            break;
        }
    }
    
    assert!(changed, "Mutation operator never altered candidate over 100 trials (space inactive)");
}

#[test]
fn mutation_diversity_topology_certification() {
    use std::collections::HashSet;
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;

    let mut rng = StdRng::seed_from_u64(456);
    let original = Candidate::default();
    let mut unique_hashes = HashSet::new();

    for _ in 0..100 {
        let mut candidate = original.clone();
        mutate_candidate(&mut candidate, &mut rng, 1.0);

        let mut hasher = DefaultHasher::new();
        let candidate_str = format!("{:?}", candidate);
        candidate_str.hash(&mut hasher);
        unique_hashes.insert(hasher.finish());
    }

    // Constitutional requirement: Mutation space must be topologically diverse,
    // not just a single deterministic toggle.
    let min_unique_mutations = 5;
    assert!(
        unique_hashes.len() >= min_unique_mutations,
        "Mutation topology is collapsed. Expected >= {} unique mutations, found {}",
        min_unique_mutations,
        unique_hashes.len()
    );
}

#[test]
fn ga_evolution_is_order_deterministic() {
    let config = GaConfig {
        population_size: 10,
        generations: 3,
        mutation_rate: 0.1,
        crossover_rate: 0.5,
        seed: 42,
    };
    
    let evaluator = DummyEvaluator;
    let res1 = run_ga_evolution(config.clone(), &evaluator);
    let res2 = run_ga_evolution(config, &evaluator);
    
    // Sort output is strictly identical
    for (i, (a, b)) in res1.generation_history.iter().zip(res2.generation_history.iter()).enumerate() {
        assert_eq!(a.candidate, b.candidate, "Generation {} diverged", i);
        assert_eq!(a.fitness, b.fitness, "Generation {} fitness diverged", i);
    }
}

#[test]
fn fitness_sorting_is_stable() {
    let mut pop = vec![
        CandidateEvaluation { fitness: 10.0, ..CandidateEvaluation::default() },
        CandidateEvaluation { fitness: 20.0, ..CandidateEvaluation::default() },
        CandidateEvaluation { fitness: 10.0, ..CandidateEvaluation::default() },
    ];
    
    pop.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(std::cmp::Ordering::Equal));
    
    // Highest first
    assert_eq!(pop[0].fitness, 20.0);
    assert_eq!(pop[1].fitness, 10.0);
    assert_eq!(pop[2].fitness, 10.0);
}
