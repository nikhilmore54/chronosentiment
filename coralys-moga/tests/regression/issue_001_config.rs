// tests/regression/issue_001_config.rs

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
        DummyEval { fitness: 1.0, valid: true, genome: candidate.clone() }
    }
}

struct DummyMutator;
impl MutationOperator<DummyGenome> for DummyMutator {
    fn mutate(&self, _candidate: &mut DummyGenome, _rng: &mut StdRng) {}
}

struct DummyCrossover;
impl CrossoverOperator<DummyGenome> for DummyCrossover {
    fn crossover(&self, _p1: &DummyGenome, _p2: &DummyGenome, _rng: &mut StdRng) -> (DummyGenome, DummyGenome) {
        (DummyGenome { value: 0 }, DummyGenome { value: 0 })
    }
}

#[test]
fn test_config_wiring() {
    let engine = EvolutionEngine::new(DummyEvaluator, DummyMutator, DummyCrossover, DummyFactory);
    let config = EvolutionConfig {
        population_size: 20,
        generation_limit: 3,
        elite_count: 5,
        mutation_rate: 0.2,
        crossover_rate: 0.7,
        seed: Some(12345),
        tournament_size: Some(4),
    };
    let ga_result = engine.run_ga_evolution(config).expect("evolution failed");
    // Verify that the expected number of generations were recorded
    assert_eq!(ga_result.generation_history.len(), 3);
    // Verify that a global best was produced
    assert!(ga_result.global_best.fitness() > 0.0);
}
