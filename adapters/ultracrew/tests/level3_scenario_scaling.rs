use ultracrew::helpers::{generate_scenario, run_optimization};
use coralys_moga::config::EvolutionConfig;

#[test]
fn test_level3_scenario_scaling() {
    let scenarios = vec![
        ("Small", 20, 50, 50, 50),
        ("Medium", 50, 100, 100, 100),
        ("Large", 100, 250, 200, 200),
    ];

    println!("--- LEVEL 3: SCENARIO ROBUSTNESS ---");

    for (name, workers, shifts, pop, gen) in scenarios {
        let context = generate_scenario(workers, shifts, workers / 3);
        
        let config = EvolutionConfig {
            population_size: pop,
            generation_limit: gen,
            mutation_rate: 0.1,
            crossover_rate: 0.7,
            elite_count: 5,
            seed: Some(42),
        };

        let result = run_optimization(context, config);
        let best = result.global_best;
        let gen0_best = result.generation_history.first().unwrap();

        let total_hc_start = gen0_best.hc1_violations + gen0_best.hc2_violations + gen0_best.hc3_violations;
        let total_hc_end = best.hc1_violations + best.hc2_violations + best.hc3_violations;

        let reduction_pct = if total_hc_start == 0 {
            100.0
        } else {
            ((total_hc_start - total_hc_end) as f64 / total_hc_start as f64) * 100.0
        };

        println!("[{}] Workers: {}, Shifts: {} | HC Reductions: {} -> {} ({:.2}%)", 
            name, workers, shifts, total_hc_start, total_hc_end, reduction_pct);

        assert!(
            total_hc_end == 0 || reduction_pct > 95.0,
            "{} scenario failed to satisfy scaling target. Final HC: {}, Reduction: {:.2}%", 
            name, total_hc_end, reduction_pct
        );
    }
}
