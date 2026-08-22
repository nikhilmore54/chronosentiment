// Temporary test to capture instrumentation logs for generation 98
use coralys_moga::engine::EvolutionEngine;
use coralys_moga::config::EvolutionConfig;
use coralys_moga::traits::{Genome, GenomeFactory, FitnessEvaluator, MutationOperator, CrossoverOperator, Evaluated};
use rand::rngs::StdRng;
use rand::SeedableRng;

#[derive(Clone, Debug, PartialEq)]
struct DummyGenome {
    data: u32,
}
impl Genome for DummyGenome {}

struct DummyFactory;
impl GenomeFactory<DummyGenome> for DummyFactory {
    fn create(&self, _rng: &mut StdRng) -> DummyGenome {
        DummyGenome { data: 0 }
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
        DummyEval { fitness: rand::random::<f64>() * 10000.0, valid: true, genome: candidate.clone() }
    }
}

struct DummyMutator;
impl MutationOperator<DummyGenome> for DummyMutator {
    fn mutate(&self, _candidate: &mut DummyGenome, _rng: &mut StdRng) {}
}

struct DummyCrossover;
impl CrossoverOperator<DummyGenome> for DummyCrossover {
    fn crossover(&self, p1: &DummyGenome, p2: &DummyGenome, _rng: &mut StdRng) -> (DummyGenome, DummyGenome) {
        (p1.clone(), p2.clone())
    }
}

#[test]
fn debug_instrument() {
    let engine = EvolutionEngine::new(DummyEvaluator, DummyMutator, DummyCrossover, DummyFactory);
    let config = EvolutionConfig {
        population_size: 20,
        generation_limit: 120,
        elite_count: 2,
        tournament_size: Some(3),
        mutation_rate: 0.1,
        crossover_rate: 0.8,
        seed: Some(42),
        ..Default::default()
    };
    let _ = engine.run_ga_evolution(config).expect("run failed");
}
