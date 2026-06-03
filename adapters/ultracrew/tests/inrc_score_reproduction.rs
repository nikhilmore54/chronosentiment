use ultracrew::inrc::parser::{parse_scenario, parse_history, parse_week_data};
use ultracrew::inrc::optimization::{InrcContext, InrcOptimizer};
use ultracrew::ecology::WorkforceEcology;
use coralys_moga::engine::EvolutionEngine;
use coralys_moga::config::EvolutionConfig;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn test_f2c_bronze_feasibility() {
    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("data").join("n030w4");
    
    let scenario = parse_scenario(base_dir.join("Sc-n030w4.json")).unwrap();
    let week_data = parse_week_data(base_dir.join("WD-n030w4-0.json")).unwrap();
    let history = parse_history(base_dir.join("H0-n030w4-0.json")).unwrap();
    
    // Ecology
    let mut ecology = WorkforceEcology::new();
    
    let context = InrcContext::new(scenario, week_data, history, ecology);
    
    let optimizer = InrcOptimizer {
        context: Arc::new(context),
    };
    
    let engine = EvolutionEngine::new(
        optimizer.clone(),
        optimizer.clone(),
        optimizer.clone(),
        optimizer.clone(),
    );
    
    let config = EvolutionConfig {
        population_size: 500,
        generation_limit: 300,
        seed: Some(42),
        ..Default::default()
    };
    
    println!("Starting F.2C Feasibility Test on n030w4");
    let result = engine.run_ga_evolution(config);
    
    // Print the Feasibility Dashboard for each generation
    for (gen, eval) in result.generation_history.iter().enumerate() {
        println!("Gen {}: Fit={} | Cov={} Skills={} OneShift={} Succ={}", 
            gen, 
            eval.fitness,
            eval.hc_coverage,
            eval.hc_skills,
            eval.hc_one_shift_per_day,
            eval.hc_forbidden_successions
        );
    }
    
    let best = &result.global_best;
    println!("Best Feasibility: Fit={} | Cov={} Skills={} OneShift={} Succ={}", 
        best.fitness,
        best.hc_coverage,
        best.hc_skills,
        best.hc_one_shift_per_day,
        best.hc_forbidden_successions
    );

    assert!(best.is_feasible(), "Failed to reach 0 hard constraints. Best: {:?}", best);
}
