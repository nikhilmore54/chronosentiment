use cvrp::{
    CvrpCandidate, CvrpGenomeFactory, CvrpInstance,
    moga_impl::{CvrpCrossover, CvrpCrossoverVariant, CvrpEvaluator, CvrpMutator},
};

use coralys_moga::traits::FitnessEvaluator;
use rand::SeedableRng;
use std::time::Instant;

fn apply_true_local_search(
    mut candidate: CvrpCandidate,
    instance: &CvrpInstance,
) -> (CvrpCandidate, f64) {
    let eval = CvrpEvaluator {
        instance: instance.clone(),
    };
    let mut current_best = eval
        .evaluate(
            &candidate,
            &coralys_moga::runtime::optimization::metric::MetricReport::default(),
        )
        .eval
        .total_distance;

    let mut improving = true;
    while improving {
        improving = false;
        let n = candidate.permutation.len();

        // Exhaustive 2-opt approx (inversion)
        for i in 0..n {
            for j in (i + 1)..n {
                let mut test_cand = candidate.clone();
                test_cand.permutation[i..=j].reverse();
                let dist = eval
                    .evaluate(
                        &test_cand,
                        &coralys_moga::runtime::optimization::metric::MetricReport::default(),
                    )
                    .eval
                    .total_distance;
                if dist < current_best {
                    current_best = dist;
                    candidate = test_cand;
                    improving = true;
                }
            }
        }

        // Exhaustive Swap
        for i in 0..n {
            for j in (i + 1)..n {
                let mut test_cand = candidate.clone();
                test_cand.permutation.swap(i, j);
                let dist = eval
                    .evaluate(
                        &test_cand,
                        &coralys_moga::runtime::optimization::metric::MetricReport::default(),
                    )
                    .eval
                    .total_distance;
                if dist < current_best {
                    current_best = dist;
                    candidate = test_cand;
                    improving = true;
                }
            }
        }

        // Exhaustive Relocate
        for i in 0..n {
            for j in 0..n {
                if i == j {
                    continue;
                }
                let mut test_cand = candidate.clone();
                let val = test_cand.permutation.remove(i);
                let insert_pos = if j > i { j - 1 } else { j };
                test_cand.permutation.insert(insert_pos, val);
                let dist = eval
                    .evaluate(
                        &test_cand,
                        &coralys_moga::runtime::optimization::metric::MetricReport::default(),
                    )
                    .eval
                    .total_distance;
                if dist < current_best {
                    current_best = dist;
                    candidate = test_cand;
                    improving = true;
                }
            }
        }
    }

    (candidate, current_best)
}

fn main() {
    let instance = CvrpInstance::a_n32_k5();
    let evaluator = CvrpEvaluator {
        instance: instance.clone(),
    };
    let mutator = CvrpMutator::new(instance.clone(), cvrp::RadiusPolicy::Control);
    let factory = CvrpGenomeFactory {
        num_customers: instance.customers.len(),
    };
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let mut best_cand = coralys_moga::traits::GenomeFactory::create(&factory, &mut rng);
    let mut best_dist = evaluator
        .evaluate(
            &best_cand,
            &coralys_moga::runtime::optimization::metric::MetricReport::default(),
        )
        .eval
        .total_distance;

    // Quick random search for a decent starting point (e.g. 5000 randoms)
    for _ in 0..5000 {
        let cand = coralys_moga::traits::GenomeFactory::create(&factory, &mut rng);
        let dist = evaluator
            .evaluate(
                &cand,
                &coralys_moga::runtime::optimization::metric::MetricReport::default(),
            )
            .eval
            .total_distance;
        if dist < best_dist {
            best_dist = dist;
            best_cand = cand;
        }
    }

    println!("Best random distance after 5000 gens: {:.2}", best_dist);

    println!(
        "Running exhaustive deterministic Local Search (2-opt, Swap, Relocate) on incumbent..."
    );
    let start = Instant::now();
    let (_, after_dist) = apply_true_local_search(best_cand.clone(), &instance);

    println!("Before : {:.2}", best_dist);
    println!("After  : {:.2}", after_dist);
    println!(
        "Improvement: {:.2} (Took {:?})",
        best_dist - after_dist,
        start.elapsed()
    );
}
