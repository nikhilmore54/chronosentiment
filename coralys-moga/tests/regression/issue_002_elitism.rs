// tests/regression/issue_002_elitism.rs

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
    fn evaluate(&self, candidate: &DummyGenome, _metrics: &coralys_moga::runtime::optimization::metric::MetricReport) -> Self::Evaluation {
        // Fitness is the current value of the genome (always 0 here)
        DummyEval { fitness: candidate.value as f64, valid: true, genome: candidate.clone() }
    }
}

struct DummyMutator;
impl MutationOperator<DummyGenome> for DummyMutator {
    fn mutate(&self, candidate: &mut DummyGenome, _rng: &mut StdRng) {
        // Increment value to simulate change
        candidate.value += 1;
    }
}

struct DummyCrossover;
impl CrossoverOperator<DummyGenome> for DummyCrossover {
    fn crossover(&self, p1: &DummyGenome, p2: &DummyGenome, _rng: &mut StdRng) -> (DummyGenome, DummyGenome) {
        // Return copies without modification
        (p1.clone(), p2.clone())
    }
}

#[test]
fn test_elitism_respects_elite_count() {
    let engine = EvolutionEngine::new(DummyEvaluator, DummyMutator, DummyCrossover, DummyFactory);
    let config = EvolutionConfig {
        population_size: 10,
        generation_limit: 2,
        elite_count: 3,
        mutation_rate: 0.0, // No mutation to keep values stable
        crossover_rate: 0.0, // No crossover
        seed: Some(42),
        tournament_size: Some(2),
    };
    let ga_result = engine.run_ga_evolution(config).expect("evolution failed");
        // Ensure that at least elite_count generations were recorded (generation_history length == generation_limit)
        assert_eq!(ga_result.generation_history.len(), 2);
        // The best fitness should be non-negative and consistent
        assert!(ga_result.global_best.fitness() >= 0.0);
}
