use coralys_moga::engine::EvolutionEngine;
use coralys_moga::EvolutionConfig;
use coralys_moga::traits::{Genome, GenomeFactory, FitnessEvaluator, Evaluated, MutationOperator, CrossoverOperator};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::time::Instant;

#[derive(Clone, Debug, PartialEq)]
struct DummyGenome { value: usize }
impl Genome for DummyGenome {}

struct DummyFactory;
impl GenomeFactory<DummyGenome> for DummyFactory {
    fn create(&self, _rng: &mut StdRng) -> DummyGenome { DummyGenome { value: 0 } }
}

#[derive(Clone, Debug, PartialEq)]
struct DummyEval { fitness: f64, valid: bool, genome: DummyGenome }
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
        DummyEval { fitness: candidate.value as f64, valid: true, genome: candidate.clone() }
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
    fn crossover(&self, p1: &DummyGenome, _p2: &DummyGenome, _rng: &mut StdRng) -> (DummyGenome, DummyGenome) {
        (p1.clone(), p1.clone())
    }
}

fn run_experiment(config: EvolutionConfig) -> (f64, usize, u128) {
    let engine = EvolutionEngine::new(DummyEvaluator, DummyMutator, DummyCrossover, DummyFactory);
    let start = Instant::now();
    let result = engine.run_ga_evolution(config).expect("evolution failed");
    let duration = start.elapsed().as_millis();
    let best = result.global_best.fitness();
    let generations = result.generation_history.len();
    (best, generations, duration)
}

fn main() {
    // Deterministic seed test
    let seed = Some(12345);
    let base_config = EvolutionConfig {
        population_size: 50,
        generation_limit: 10,
        elite_count: 2,
        mutation_rate: 0.1,
        crossover_rate: 0.9,
        seed,
        tournament_size: Some(3),
        termination_policy: None,
    };
    println!("=== Deterministic Seed Verification ===");
    for i in 0..3 {
        let (best, gens, dur) = run_experiment(base_config.clone());
        println!("run {}: best={}, generations={}, time_ms={}", i+1, best, gens, dur);
    }
    // Elite count variations
    println!("=== Elite Count Variation ===");
    for elite in [1usize,2,5,10].iter() {
        let mut cfg = base_config.clone();
        cfg.elite_count = *elite;
        let (best, gens, dur) = run_experiment(cfg);
        println!("elite={}, best={}, gens={}, time_ms={}", elite, best, gens, dur);
    }
    // Mutation rate variations
    println!("=== Mutation Rate Variation ===");
    for mr in [0.01f64,0.05,0.10,0.20].iter() {
        let mut cfg = base_config.clone();
        cfg.mutation_rate = *mr;
        let (best, gens, dur) = run_experiment(cfg);
        println!("mutation_rate={:.2}, best={}, gens={}, time_ms={}", mr, best, gens, dur);
    }
    // Crossover rate variations
    println!("=== Crossover Rate Variation ===");
    for cr in [0.40f64,0.60,0.80].iter() {
        let mut cfg = base_config.clone();
        cfg.crossover_rate = *cr;
        let (best, gens, dur) = run_experiment(cfg);
        println!("crossover_rate={:.2}, best={}, gens={}, time_ms={}", cr, best, gens, dur);
    }
    // Population size variations
    println!("=== Population Size Variation ===");
    for pop in [50usize,100,250,500].iter() {
        let mut cfg = base_config.clone();
        cfg.population_size = *pop;
        // reduce generations for large populations to keep runtime reasonable
        cfg.generation_limit = 10;
        let (best, gens, dur) = run_experiment(cfg);
        println!("pop={}, best={}, gens={}, time_ms={}", pop, best, gens, dur);
    }
    // Multiple seed robustness
    println!("=== Multiple Seed Robustness ===");
    let mut results = Vec::new();
    for s in 1..=20u64 {
        let mut cfg = base_config.clone();
        cfg.seed = Some(s);
        let (best, _, _) = run_experiment(cfg);
        results.push(best);
    }
    let sum: f64 = results.iter().sum();
    let mean = sum / results.len() as f64;
    let mut sorted = results.clone();
    sorted.sort_by(|a,b| a.partial_cmp(b).unwrap());
    let median = sorted[sorted.len()/2];
    let min = sorted[0];
    let max = sorted[sorted.len()-1];
    let stddev = (results.iter().map(|v| (v-mean)*(v-mean)).sum::<f64>() / results.len() as f64).sqrt();
    println!("seed_stats: mean={:.2}, median={:.2}, min={:.2}, max={:.2}, stddev={:.2}", mean, median, min, max, stddev);
}
