// tests/regression/issue_003_seed.rs

use coralys_moga::engine::{EvolutionEngine, EvolutionConfig};
use coralys_moga::traits::{Genome, GenomeFactory, Evaluated, FitnessEvaluator, MutationOperator, CrossoverOperator};
use rand::rngs::StdRng;
use rand::SeedableRng;

#[derive(Clone, Debug, PartialEq)]
struct DummyGenome {
    value: usize,
}
impl Genome for DummyGenome {}

struct DummyFactory;
impl GenomeFactory<DummyGenome> for DummyFactory {
    fn create(&self, _rng: &mut StdRng) -> DummyGenome {
        DummyGenome { value: 0 }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct DummyEval {
    fitness: f64,
    valid: bool,
    genome: DummyGenome,
}
impl Evaluated for DummyEval {
    type Genome = DummyGenome;
    fn fitness(&self) -> f64 { self.fitness }
    fn is_valid(&self) -> bool { self.valid }
    fn genome(&self) -> &Self::Genome { &self.genome }
}

struct DummyEvaluator;
impl FitnessEvaluator<DummyGenome> for DummyEvaluator {
    type Evaluation = DummyEval;
    fn evaluate(&self, candidate: &DummyGenome) -> Self::Evaluation {
        // fitness is the current value; deterministic based on mutations
        DummyEval { fitness: candidate.value as f64, valid: true, genome: candidate.clone() }
    }
}

struct DummyMutator;
impl MutationOperator<DummyGenome> for DummyMutator {
    fn mutate(&self, candidate: &mut DummyGenome, _rng: &mut StdRng) {
        // deterministic increment
        candidate.value += 1;
    }
}

struct DummyCrossover;
impl CrossoverOperator<DummyGenome> for DummyCrossover {
    fn crossover(&self, p1: &DummyGenome, _p2: &DummyGenome, _rng: &mut StdRng) -> (DummyGenome, DummyGenome) {
        (p1.clone(), p1.clone())
    }
}

#[test]
fn test_seed_determinism() {
    let engine = EvolutionEngine::new(DummyEvaluator, DummyMutator, DummyCrossover, DummyFactory);
    let seed = Some(7777);
    let config = EvolutionConfig {
        population_size: 5,
        generation_limit: 3,
        elite_count: 1,
        mutation_rate: 1.0, // ensure mutation occurs
        crossover_rate: 0.0,
        seed,
        tournament_size: Some(2),
    };

    // First run
    let ga1 = engine.run_ga_evolution(config.clone()).expect("run1 failed");

    // Second run with same seed
    let ga2 = engine.run_ga_evolution(config).expect("run2 failed");

    // Compare deterministic outputs
    assert_eq!(ga1.global_best.fitness(), ga2.global_best.fitness(), "Best fitness differs");
    assert_eq!(ga1.generation_history.len(), ga2.generation_history.len(), "Generation history length differs");
    for (e1, e2) in ga1.generation_history.iter().zip(ga2.generation_history.iter()) {
        assert_eq!(e1.fitness(), e2.fitness(), "Generation fitness differs");
    }
}
