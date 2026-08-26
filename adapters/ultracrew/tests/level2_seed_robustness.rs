use coralys_moga::config::EvolutionConfig;
use ultracrew::helpers::{generate_scenario, run_optimization};

#[test]
fn test_level2_seed_robustness() {
    let num_seeds = 50;

    let context = generate_scenario(20, 50, 6);

    let mut fitness_scores = Vec::new();
    let mut constraint_satisfaction_count = 0;

    for seed in 1..=num_seeds {
        let config = EvolutionConfig {
            population_size: 50,
            generation_limit: 50,
            mutation_rate: 0.1,
            crossover_rate: 0.7,
            elite_count: 5,
            seed: Some(seed),
            ..Default::default()
        };

        let result = run_optimization(context.clone(), config);
        let best = result.global_best;

        fitness_scores.push(best.fitness);

        if best.hc1_violations == 0 && best.hc2_violations == 0 && best.hc3_violations == 0 {
            constraint_satisfaction_count += 1;
        }
    }

    fitness_scores.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let min = fitness_scores[0];
    let max = fitness_scores[num_seeds as usize - 1];
    let median = fitness_scores[num_seeds as usize / 2];

    let mean: f64 = fitness_scores.iter().sum::<f64>() / num_seeds as f64;
    let variance = fitness_scores
        .iter()
        .map(|f| (f - mean).powi(2))
        .sum::<f64>()
        / num_seeds as f64;
    let std_dev = variance.sqrt();

    let satisfaction_rate = (constraint_satisfaction_count as f64 / num_seeds as f64) * 100.0;

    println!("--- LEVEL 2: SEED ROBUSTNESS (50 SEEDS) ---");
    println!("Mean Fitness:   {:.2}", mean);
    println!("Std Dev:        {:.2}", std_dev);
    println!("Median Fitness: {:.2}", median);
    println!("Worst Fitness:  {:.2}", min);
    println!("Best Fitness:   {:.2}", max);
    println!("Constraint Satisfaction Rate: {:.2}%", satisfaction_rate);

    // Baseline random fitness is usually ~ -25000. So every run should be > 0.
    assert!(min > 0.0, "Worst run failed to beat baseline (fitness < 0)");

    // Satisfaction rate should be very high (e.g. >90%)
    assert!(
        satisfaction_rate > 90.0,
        "Constraint Satisfaction Rate too low: {}%",
        satisfaction_rate
    );
}
