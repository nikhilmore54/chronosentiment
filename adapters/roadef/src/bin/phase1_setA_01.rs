use std::fs;
use std::time::Instant;
use roadef::evaluator::RoadefEvaluator;
use roadef::moga_impl::{
    RoadefGenomeFactory, RoadefFitnessEvaluator, RoadefMutator, RoadefCrossover,
    EvolutionRunConfig, generate_gen0_population, ConstructionMode
};
use roadef::pipeline_impl::run_pipeline_evolution_v2;
use roadef::telemetry::{NullTelemetrySink, ComparatorMode};

fn load_network(path: &str) -> Result<roadef::models::Network, Box<dyn std::error::Error>> {
    let s = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&s)?)
}

fn load_traffic_matrix(path: &str) -> Result<roadef::models::TrafficMatrix, Box<dyn std::error::Error>> {
    let s = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&s)?)
}

fn load_scenario(path: &str) -> Result<roadef::models::Scenario, Box<dyn std::error::Error>> {
    let s = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&s)?)
}

fn main() {
    println!("Loading setA-01...");
    let net = load_network("adapters/roadef/repo/challenge-roadef-2026-main/setA/setA-01-net.json").unwrap();
    let tm = load_traffic_matrix("adapters/roadef/repo/challenge-roadef-2026-main/setA/setA-01-tm.json").unwrap();
    let scenario = load_scenario("adapters/roadef/repo/challenge-roadef-2026-main/setA/setA-01-scenario.json").unwrap();
    let evaluator = RoadefEvaluator::new(net, tm, scenario).unwrap();

    let config = EvolutionRunConfig {
        population_size: 50,
        generation_limit: 500,
        crossover_rate: 0.8,
        mutation_rate: 0.1,
        elite_count: 5,
        time_budget_secs: Some(3600), 
        max_evaluations: None,
        stagnation_limit: 500,
        comparator_mode: ComparatorMode::FeasibilityDriven,
    };

    println!("Generating gen0...");
    let factory = RoadefGenomeFactory::new(evaluator.clone(), 42);
    let fitness_eval = RoadefFitnessEvaluator::new(evaluator.clone());
    let mutator = RoadefMutator::new(evaluator.clone());
    let crossover = RoadefCrossover::new(evaluator.clone());

    let (gen0, _) = generate_gen0_population(&factory, &fitness_eval, 50, ConstructionMode::Legacy, 42);

    println!("Running pipeline...");
    let mut null_sink = NullTelemetrySink {};
    let t0 = Instant::now();
    let result = run_pipeline_evolution_v2(
        gen0,
        config,
        &fitness_eval,
        &mutator,
        &crossover,
        42,
        &mut null_sink,
        "setA-01",
    );
    let dt = t0.elapsed();

    println!("Result: {:?}", result);
    println!("Time: {:?}", dt);

    if !result.trajectory.is_empty() {
        let first_gen = &result.trajectory[0];
        let last_gen = &result.trajectory.last().unwrap();
        println!("Gen 0: {:?}", first_gen);
        println!("Last Gen: {:?}", last_gen);
        
        let total_gen_ms: u128 = result.trajectory.iter().map(|g| g.generation_runtime_ms).sum();
        let total_eval_ms: u128 = result.trajectory.iter().map(|g| g.evaluation_runtime_ms).sum();
        println!("Total Gen ms: {}", total_gen_ms);
        println!("Total Eval ms: {}", total_eval_ms);
        println!("Eval share: {:.2}%", (total_eval_ms as f64 / total_gen_ms as f64) * 100.0);
    }
}
