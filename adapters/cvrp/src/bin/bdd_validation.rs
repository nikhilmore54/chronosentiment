use coralys_moga::traits::FitnessEvaluator;
use coralys_moga::{EvolutionConfig, EvolutionEngineBuilder};
use cvrp::moga_impl::{CvrpCrossover, CvrpEvaluator, CvrpLocalSearch, CvrpMutator};
use cvrp::{CvrpCandidate, CvrpGenomeFactory, CvrpInstance, DistanceMetric};

fn main() {
    println!("Starting Dual Distance Metric Validation on A-n32-k5...");

    // 1. Define Official BKS routes
    let bks_routes = vec![
        vec![22, 32, 20, 18, 14, 8, 27],
        vec![13, 2, 17, 31],
        vec![28, 25],
        vec![30, 19, 9, 10, 23, 16, 11, 26, 6, 21],
        vec![15, 29, 12, 5, 24, 4, 3, 7],
    ];

    // Compute official BKS values
    let instance_for_bks = CvrpInstance::a_n32_k5();
    let bks_float =
        instance_for_bks.evaluate_routes_distance(&bks_routes, DistanceMetric::EuclideanFloat);
    let bks_integer =
        instance_for_bks.evaluate_routes_distance(&bks_routes, DistanceMetric::TspLibEuc2D);

    // 2. Setup and run GA in Floating-Point Mode
    let mut instance_float = CvrpInstance::a_n32_k5();
    instance_float.distance_metric = DistanceMetric::EuclideanFloat;
    let eval_float = CvrpEvaluator {
        instance: instance_float.clone(),
    };
    let mutator_float = CvrpMutator::new(instance_float.clone(), cvrp::RadiusPolicy::Control);
    let crossover_float = CvrpCrossover;
    let factory_float = CvrpGenomeFactory {
        num_customers: instance_float.customers.len(),
    };
    let ls_float = CvrpLocalSearch {
        instance: instance_float.clone(),
    };

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

    let engine_float = EvolutionEngineBuilder::new()
        .with_evaluator(eval_float)
        .with_mutator(mutator_float)
        .with_crossover(crossover_float)
        .with_factory(factory_float)
        .build()
        .expect("Failed to build float engine");

    let res_float = engine_float.run_ga_evolution(evo_config.clone()).unwrap();
    // Evaluate the best float solution under integer metric
    let best_cand_float = res_float.global_best.eval.candidate.clone();
    let evaluator_for_best = CvrpEvaluator {
        instance: instance_float.clone(),
    };
    let best_eval_float = evaluator_for_best
        .evaluate(
            &best_cand_float,
            &coralys_moga::runtime::optimization::metric::MetricReport::default(),
        )
        .eval;
    let float_best_as_float = best_eval_float.total_distance_float;
    let float_best_as_integer = best_eval_float.total_distance_integer;

    // 3. Setup and run GA in Integer Mode
    let mut instance_int = CvrpInstance::a_n32_k5();
    instance_int.distance_metric = DistanceMetric::TspLibEuc2D;
    let eval_int = CvrpEvaluator {
        instance: instance_int.clone(),
    };
    let mutator_int = CvrpMutator::new(instance_int.clone(), cvrp::RadiusPolicy::Control);
    let crossover_int = CvrpCrossover;
    let factory_int = CvrpGenomeFactory {
        num_customers: instance_int.customers.len(),
    };
    let ls_int = CvrpLocalSearch {
        instance: instance_int.clone(),
    };

    let engine_int = EvolutionEngineBuilder::new()
        .with_evaluator(eval_int)
        .with_mutator(mutator_int)
        .with_crossover(crossover_int)
        .with_factory(factory_int)
        .build()
        .expect("Failed to build int engine");

    let res_int = engine_int.run_ga_evolution(evo_config).unwrap();
    let best_cand_int = res_int.global_best.eval.candidate.clone();
    let evaluator_for_best_int = CvrpEvaluator {
        instance: instance_int.clone(),
    };
    let best_eval_int = evaluator_for_best_int
        .evaluate(
            &best_cand_int,
            &coralys_moga::runtime::optimization::metric::MetricReport::default(),
        )
        .eval;
    let int_best_as_float = best_eval_int.total_distance_float;
    let int_best_as_integer = best_eval_int.total_distance_integer;

    println!("\n=========================================================");
    println!("CVRP DUAL METRIC VALIDATION REPORT");
    println!("=========================================================");
    println!("Distance edge calculations:");
    // Print a few sample edge calculations to satisfy Task 1 & 4
    let n1 = &instance_for_bks.depot;
    let n2 = &instance_for_bks.customers[0]; // Customer ID 2
    let f_dist = ((n1.x - n2.x).powi(2) + (n1.y - n2.y).powi(2)).sqrt();
    let i_dist = f_dist.round();
    println!(
        "  Depot (82, 76) -> Cust 2 (96, 44): Float={:.4}, Int={:.4}",
        f_dist, i_dist
    );

    let n3 = &instance_for_bks.customers[11]; // Cust ID 13 (98, 52)
    let f_dist2 = ((n2.x - n3.x).powi(2) + (n2.y - n3.y).powi(2)).sqrt();
    let i_dist2 = f_dist2.round();
    println!(
        "  Cust 2 (96, 44) -> Cust 13 (98, 52): Float={:.4}, Int={:.4}",
        f_dist2, i_dist2
    );

    println!("\nRoute totals for Official BKS:");
    for (i, route) in bks_routes.iter().enumerate() {
        let mut sub_routes = vec![route.clone()];
        let r_float =
            instance_for_bks.evaluate_routes_distance(&sub_routes, DistanceMetric::EuclideanFloat);
        let r_int =
            instance_for_bks.evaluate_routes_distance(&sub_routes, DistanceMetric::TspLibEuc2D);
        println!(
            "  Route #{}: Nodes={:?}, Float={:.4}, Int={:.4}",
            i + 1,
            route,
            r_float,
            r_int
        );
    }

    println!("\nValidation Table:");
    println!(
        "| Solution | Official Integer (Benchmark) | Floating Euclidean (Research) | Difference | % Diff |"
    );
    println!("| :--- | :---: | :---: | :---: | :---: |");

    let diff_bks = bks_float - bks_integer;
    let pct_bks = (diff_bks / bks_integer) * 100.0;
    println!(
        "| Official BKS | {:.2} | {:.4} | {:.4} | {:.2}% |",
        bks_integer, bks_float, diff_bks, pct_bks
    );

    let diff_coralys_f = float_best_as_float - float_best_as_integer;
    let pct_coralys_f = (diff_coralys_f / float_best_as_integer) * 100.0;
    println!(
        "| Coralys Best (Float mode) | {:.2} | {:.4} | {:.4} | {:.2}% |",
        float_best_as_integer, float_best_as_float, diff_coralys_f, pct_coralys_f
    );

    let diff_coralys_i = int_best_as_float - int_best_as_integer;
    let pct_coralys_i = (diff_coralys_i / int_best_as_integer) * 100.0;
    println!(
        "| Coralys Best (Integer mode) | {:.2} | {:.4} | {:.4} | {:.2}% |",
        int_best_as_integer, int_best_as_float, diff_coralys_i, pct_coralys_i
    );

    println!("=========================================================");

    let matches_bks = int_best_as_integer <= 784.0;
    println!("Final Conclusion:");
    if matches_bks {
        println!(
            "MATCHES OFFICIAL BKS\nCoralys reaches the official benchmark optimum under the official TSPLIB metric."
        );
    } else {
        println!("OPTIMIZATION STILL REQUIRED\nCoralys has not yet reached the official optimum.");
    }
}
