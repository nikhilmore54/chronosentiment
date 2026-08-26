// Benchmark for CVRP performance using the Coralys MOGA engine

use coralys_core::DecisionPlugin;
use coralys_moga::EvolutionConfig;
use coralys_moga::engine::EvolutionEngine;
use coralys_moga::engine::PluginFitnessEvaluator;
use criterion::{Criterion, criterion_group, criterion_main};
use cvrp::moga_impl::{CvrpCrossover, CvrpMutator};
use cvrp::{CvrpDecisionPlugin, CvrpGenomeFactory, CvrpInstance};
use rand::SeedableRng;
use rand::rngs::StdRng;

fn cvrp_benchmark(c: &mut Criterion) {
    // Setup a CVRP instance and the decision plugin
    let instance = CvrpInstance::a_n32_k5();
    let plugin = CvrpDecisionPlugin::new(instance.clone());

    // Obtain the current state for the evaluator
    let state = plugin.current_state();
    let evaluator = PluginFitnessEvaluator {
        plugin: &plugin,
        state: &state,
        _marker: std::marker::PhantomData,
    };

    // Mutator, crossover and factory required by EvolutionEngine
    let mutator = CvrpMutator::new(instance.clone(), cvrp::RadiusPolicy::Control);
    let crossover = CvrpCrossover; // basic OX1 crossover
    let factory = CvrpGenomeFactory {
        num_customers: instance.customers.len(),
    };

    // Configure the MOGA engine
    let evo_config = EvolutionConfig {
        population_size: 200,
        elite_count: 20,
        generation_limit: 50,
        mutation_rate: 0.2,
        crossover_rate: 0.8,
        seed: Some(42),
        tournament_size: Some(5),
        ..Default::default()
    };

    let mut rng = StdRng::seed_from_u64(42);
    // Construct the engine with all required components
    let mut engine = EvolutionEngine::new(evaluator, mutator, crossover, factory);
    c.bench_function("cvrp_moga", |b| {
        b.iter(|| {
            let _ = engine.run_ga_evolution(evo_config.clone());
        })
    });
    let ga_res = engine
        .run_ga_evolution(evo_config.clone())
        .expect("GA run failed");
    if let Some(total) = ga_res.global_best.result.metrics.get("total_distance") {
        println!("GA best distance: {:.4}", total);
    }
}

criterion_group!(benches, cvrp_benchmark);
criterion_main!(benches);
