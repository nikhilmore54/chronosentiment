use cvrp::{CvrpDecisionPlugin, CvrpInstance, CvrpGenomeFactory};
use cvrp::moga_impl::{CvrpMutator, CvrpCrossover, CvrpLocalSearch};
use coralys_moga::{EvolutionConfig, EvolutionEngineBuilder};
use coralys_moga::engine::PluginFitnessEvaluator;
use coralys_core::DecisionPlugin;

fn main() {
    let instance = CvrpInstance::a_n32_k5();
    let plugin = CvrpDecisionPlugin::new(instance.clone());

    let state = plugin.current_state();
    let evaluator = PluginFitnessEvaluator {
        plugin: &plugin,
        state: &state,
        _marker: std::marker::PhantomData,
    };

    let mutator = CvrpMutator::new(instance.clone(), cvrp::RadiusPolicy::Control);
    let crossover = CvrpCrossover;
    let factory = CvrpGenomeFactory { num_customers: instance.customers.len() };
    let local_search = CvrpLocalSearch { instance: instance.clone() };

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

    let engine = EvolutionEngineBuilder::new()
        .with_evaluator(evaluator)
        .with_mutator(mutator)
        .with_crossover(crossover)
        .with_factory(factory)
        .with_improvement(local_search)
        .build()
        .expect("Builder build failed");

    let ga_res = engine.run_ga_evolution(evo_config).expect("GA run failed");
    
    let mut ga_distance = 0.0;
    if let Some(&total) = ga_res.global_best.result.metrics.get("total_distance") {
        ga_distance = total;
    }

    // BKS for A-n32-k5 is 784.0 (often represented as 784 in benchmarks)
    let bks_distance = 784.0;
    let gap = ((ga_distance - bks_distance) / bks_distance) * 100.0;

    println!("========================================");
    println!("CVRP GA vs BKS Comparison");
    println!("========================================");
    println!("BKS Distance: {:.2}", bks_distance);
    println!("GA Distance:  {:.2}", ga_distance);
    println!("Percentage Gap: {:.2}%", gap);
    println!("========================================");
}
