use ultracrew::helpers::{generate_scenario, run_optimization};
use coralys_moga::config::EvolutionConfig;

#[test]
fn test_level1_determinism() {
    let num_runs = 10;
    
    let config = EvolutionConfig {
        population_size: 50,
        generation_limit: 50,
        mutation_rate: 0.1,
        crossover_rate: 0.7,
        elite_count: 5,
        seed: Some(42), // Fixed seed across all runs
    };

    let context = generate_scenario(20, 50, 6);

    let baseline_result = run_optimization(context.clone(), config.clone());

    for i in 1..num_runs {
        let result = run_optimization(context.clone(), config.clone());

        // 1. Final Best Fitness must be identical
        assert_eq!(
            baseline_result.global_best.fitness,
            result.global_best.fitness,
            "Run {} diverged in final fitness", i
        );

        // 2. Trajectory must be identical
        assert_eq!(
            baseline_result.generation_history.len(),
            result.generation_history.len(),
            "Run {} diverged in trajectory length", i
        );

        for gen in 0..baseline_result.generation_history.len() {
            let base_gen = &baseline_result.generation_history[gen];
            let new_gen = &result.generation_history[gen];
            assert_eq!(
                base_gen.fitness, new_gen.fitness,
                "Run {} diverged at generation {}", i, gen
            );
        }

        // 3. Genome assignments must be byte-for-byte identical
        for (shift_id, worker_id) in &baseline_result.global_best.schedule.assignments {
            let new_worker_id = result.global_best.schedule.assignments.get(shift_id).unwrap();
            assert_eq!(
                worker_id, new_worker_id,
                "Run {} diverged in schedule assignments", i
            );
        }
    }
}
