use std::fs;
use std::time::Instant;
use std::sync::Arc;
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
    
    let num_demands = tm.demands.len();
    let num_time_slots = tm.num_time_slots;
    let node_ids: Vec<u64> = net.nodes.iter().map(|n| n.id).collect();

    let evaluator = Arc::new(RoadefEvaluator::new(&net, tm, scenario));

    let config = EvolutionRunConfig {
        population_size: 50,
        generation_limit: 500,
        crossover_rate: 0.8,
        mutation_rate: 0.1,
        elite_count: 5,
        no_improvement_limit: 20,
        seed: Some(42),
        log_interval: 100,
        health_interval: 200,
        max_runtime: None,
        comparator_mode: ComparatorMode::Scalar,
        peak_demand_set: None,
    };

    println!("Generating gen0...");
    let factory = RoadefGenomeFactory {
        num_demands,
        num_time_slots,
        node_ids: node_ids.clone(),
        mode: ConstructionMode::Random,
        greedy_data: None,
    };
    
    let fitness_eval = RoadefFitnessEvaluator { evaluator: Arc::clone(&evaluator) };
    let mutator = RoadefMutator { node_ids: node_ids.clone() };
    let crossover = RoadefCrossover;

    let init_pop = generate_gen0_population(&factory, &fitness_eval, Some(42), 50);

    println!("Running pipeline...");
    let mut null_sink = NullTelemetrySink {};
    let mut log_buf = Vec::new();
    
    let pipeline = coralys_core::pipeline::EvolutionaryPipeline {
        constraint_model: roadef::constraints::RoadefConstraintModel { evaluator: evaluator.clone() },
        repair_operators: vec![Box::new(roadef::operators::RoadefRepair)],
        improvement_operators: vec![Box::new(roadef::operators::RoadefImprovement)],
        repair_budget: coralys_core::operators::OperatorBudget { max_iterations: 10, max_time_ms: 100 },
        improve_budget: coralys_core::operators::OperatorBudget { max_iterations: 10, max_time_ms: 100 },
    };
    
    let t0 = Instant::now();
    let result = run_pipeline_evolution_v2(
        &factory,
        &fitness_eval,
        &mutator,
        &crossover,
        &pipeline,
        &config,
        init_pop,
        "setA-01",
        &mut log_buf,
        &mut null_sink,
    );
    let dt = t0.elapsed();

    println!("Best Obj: {}", result.best_obj);
    println!("Time: {:?}", dt);

    if !result.trajectory.is_empty() {
        let first_gen = &result.trajectory[0];
        let last_gen = &result.trajectory.last().unwrap();
        println!("Gen 0: {:?}", first_gen);
        println!("Last Gen: {:?}", last_gen);
        
        let total_gen_ms: f64 = result.trajectory.iter().map(|g| g.generation_runtime_ms).sum();
        let total_eval_ms: f64 = result.trajectory.iter().map(|g| g.evaluation_runtime_ms).sum();
        let total_evals: usize = result.trajectory.iter().map(|g| g.n_eval).sum();
        let total_cache_hits: usize = result.trajectory.iter().map(|g| g.cache_hits).sum();
        
        let total_cache_lookup_ms: f64 = result.trajectory.iter().map(|g| g.cache_lookup_ms).sum();
        let total_cache_hit_materialize_ms: f64 = result.trajectory.iter().map(|g| g.cache_hit_materialize_ms).sum();
        let total_cache_insert_ms: f64 = result.trajectory.iter().map(|g| g.cache_insert_ms).sum();
        
        println!("Total Gen ms: {}", total_gen_ms);
        println!("Total Eval ms: {}", total_eval_ms);
        println!("Total Actual Evals: {}", total_evals);
        println!("Total Cache Hits: {}", total_cache_hits);
        println!("Cache Lookup ms: {}", total_cache_lookup_ms);
        println!("Cache Materialize ms: {}", total_cache_hit_materialize_ms);
        println!("Cache Insert ms: {}", total_cache_insert_ms);
        println!("Eval share: {:.2}%", (total_eval_ms / total_gen_ms) * 100.0);
    }
}
