// src/benchmark.rs

use crate::config::EvolutionConfig;
use crate::engine::EvolutionEngine;
use crate::traits::{
    CrossoverOperator, Evaluated, FitnessEvaluator, Genome, GenomeFactory, MutationOperator,
};
use rand::rngs::StdRng;
// SeedableRng not needed in benchmark

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
    fn fitness(&self) -> f64 {
        self.fitness
    }
    fn is_valid(&self) -> bool {
        self.valid
    }
    fn genome(&self) -> &Self::Genome {
        &self.genome
    }
}

struct DummyEvaluator;
impl FitnessEvaluator<DummyGenome> for DummyEvaluator {
    type Evaluation = DummyEval;
    fn evaluate(
        &self,
        candidate: &DummyGenome,
        _metrics: &crate::runtime::optimization::metric::MetricReport,
    ) -> Self::Evaluation {
        DummyEval {
            fitness: candidate.value as f64,
            valid: true,
            genome: candidate.clone(),
        }
    }
}

struct DummyMutator;
impl MutationOperator<DummyGenome> for DummyMutator {
    fn mutate(&self, candidate: &mut DummyGenome, _rng: &mut StdRng) {
        candidate.value += 1;
    }
}

struct DummyCrossover;
impl CrossoverOperator<DummyGenome> for DummyCrossover {
    fn crossover(
        &self,
        p1: &DummyGenome,
        _p2: &DummyGenome,
        _rng: &mut StdRng,
    ) -> (DummyGenome, DummyGenome) {
        (p1.clone(), p1.clone())
    }
}

/// Minimal benchmark harness – runs a single GA evolution with a deterministic seed.
/// This function is intended for internal verification and does not expose any UI.
pub fn run_dummy_benchmark() {
    let engine = EvolutionEngine::new(DummyEvaluator, DummyMutator, DummyCrossover, DummyFactory);
    let config = EvolutionConfig {
        population_size: 20,
        generation_limit: 5,
        elite_count: 2,
        mutation_rate: 0.1,
        crossover_rate: 0.9,
        seed: Some(12345),
        tournament_size: Some(3),
        termination_policy: None,
    };
    match engine.run_ga_evolution(config) {
        Ok(result) => {
            println!(
                "Benchmark completed: best fitness = {}",
                result.global_best.fitness()
            );
        }
        Err(e) => {
            eprintln!("Benchmark failed: {}", e);
        }
    }
}
