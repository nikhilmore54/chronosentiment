use std::cmp::Ordering;
use std::collections::HashMap;
use std::time::Instant;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use coralys_moga::traits::*;
use coralys_v2::{AdvisoryCandidate, AdvisoryRanker, ContextKey, OpportunityMemory};
use cvrp::moga_impl::{CvrpCrossoverRoutePreserving, CvrpEvaluator, CvrpRouteAwareMutator};
use cvrp::{CvrpCandidate, CvrpEvaluation, CvrpGenomeFactory, CvrpInstance};

#[derive(Clone, Hash, Eq, PartialEq)]
struct CvrpContextKey {
    route_count: usize,
    longest_route: usize,
    route_balance: i64,
}

impl ContextKey for CvrpContextKey {}

impl CvrpContextKey {
    fn from_eval(eval: &CvrpEvaluation) -> Self {
        Self {
            route_count: eval.num_vehicles,
            longest_route: eval.routes.iter().map(|r| r.len()).max().unwrap_or(0),
            route_balance: compute_route_balance(&eval.routes),
        }
    }
}

fn compute_route_balance(routes: &[Vec<usize>]) -> i64 {
    if routes.is_empty() {
        return 0;
    }
    let mean = routes.iter().map(|r| r.len() as f64).sum::<f64>() / routes.len() as f64;
    let var = routes
        .iter()
        .map(|r| (r.len() as f64 - mean).powi(2))
        .sum::<f64>()
        / routes.len() as f64;
    var.sqrt().round() as i64
}

struct MemoryStats {
    obs: usize,
    champs: usize,
}

struct Offspring {
    genome: CvrpCandidate,
    parent_ctx: Option<CvrpContextKey>,
}

struct EvaluatedOffspring {
    eval: CvrpEvaluation,
    parent_ctx: Option<CvrpContextKey>,
}

impl AdvisoryCandidate for EvaluatedOffspring {
    type Context = CvrpContextKey;

    fn fitness_bucket(&self) -> i64 {
        (self.eval.total_distance / 1.0).floor() as i64
    }

    fn parent_context(&self) -> Option<&Self::Context> {
        self.parent_ctx.as_ref()
    }

    fn lower_is_better() -> bool {
        true
    }

    fn fallback_cmp(&self, other: &Self) -> Ordering {
        self.eval
            .total_distance
            .partial_cmp(&other.eval.total_distance)
            .unwrap_or(Ordering::Equal)
    }
}

fn tournament_selection<'a>(
    evals: &'a [EvaluatedOffspring],
    k: usize,
    rng: &mut StdRng,
) -> &'a EvaluatedOffspring {
    let mut best: Option<&'a EvaluatedOffspring> = None;
    for _ in 0..k {
        let idx = rng.gen_range(0..evals.len());
        let e = &evals[idx];
        // Fitness in CVRP is typically inverse distance
        if best.is_none() || e.eval.total_distance < best.unwrap().eval.total_distance {
            best = Some(e);
        }
    }
    best.unwrap()
}

struct RunMetrics {
    best_objective: f64,
    time_to_best: usize,
    champion_count: usize,
    known_evals: usize,
    total_evals: usize,
    total_generated_score: f64,
    total_selected_score: f64,
}

fn run_pilot(use_advisory: bool, seed: u64) -> RunMetrics {
    let instance = CvrpInstance::a_n32_k5();
    let evaluator = CvrpEvaluator {
        instance: instance.clone(),
    };
    let mutator = CvrpRouteAwareMutator {
        instance: instance.clone(),
    };
    let crossover = CvrpCrossoverRoutePreserving {
        instance: instance.clone(),
    };
    let factory = CvrpGenomeFactory {
        num_customers: instance.customers.len(),
    };

    let mut rng = StdRng::seed_from_u64(seed);

    let population_size = 100;
    let generations = 500;

    let mut memory = OpportunityMemory::new(500.0, 0.00088);

    let mut population: Vec<Offspring> = (0..population_size)
        .map(|_| Offspring {
            genome: factory.create(&mut rng),
            parent_ctx: None,
        })
        .collect();

    let mut global_best: Option<CvrpEvaluation> = None;
    let mut metrics = RunMetrics {
        best_objective: std::f64::MAX,
        time_to_best: 0,
        champion_count: 0,
        known_evals: 0,
        total_evals: 0,
        total_generated_score: 0.0,
        total_selected_score: 0.0,
    };

    for generation in 0..generations {
        let mut evals: Vec<EvaluatedOffspring> = population
            .drain(..)
            .map(|off| EvaluatedOffspring {
                eval: evaluator
                    .evaluate(
                        &off.genome,
                        &coralys_moga::runtime::optimization::metric::MetricReport::default(),
                    )
                    .eval,
                parent_ctx: off.parent_ctx,
            })
            .collect();

        if generation >= 100 {
            for e in &evals {
                if let Some(ctx) = &e.parent_ctx {
                    metrics.total_evals += 1;
                    if memory.is_known(ctx) {
                        metrics.known_evals += 1;
                    }
                    metrics.total_generated_score += memory.score(ctx);
                }
            }
        }

        if use_advisory && generation >= 100 {
            let default_ctx = CvrpContextKey {
                route_count: 0,
                longest_route: 0,
                route_balance: 0,
            };
            AdvisoryRanker::sort(&mut evals, &memory, &default_ctx);
        } else {
            evals.sort_by(|a, b| {
                a.eval
                    .total_distance
                    .partial_cmp(&b.eval.total_distance)
                    .unwrap_or(Ordering::Equal)
            });
        }

        let gen_best = evals[0].eval.clone();

        let mut new_global_best = false;
        if global_best.is_none()
            || gen_best.total_distance < global_best.as_ref().unwrap().total_distance
        {
            global_best = Some(gen_best.clone());
            new_global_best = true;
            metrics.best_objective = gen_best.total_distance;
            metrics.time_to_best = generation;
            metrics.champion_count += 1;
        }

        for (rank, e) in evals.iter().enumerate() {
            if generation >= 100 && rank < population_size / 2 {
                if let Some(ctx) = &e.parent_ctx {
                    metrics.total_selected_score += memory.score(ctx);
                }
            }
            if let Some(ctx) = &e.parent_ctx {
                memory.record(ctx.clone(), rank == 0 && new_global_best);
            }
        }

        let mut next_gen = Vec::with_capacity(population_size);
        for i in 0..std::cmp::min(5, evals.len()) {
            next_gen.push(Offspring {
                genome: evals[i].eval.candidate.clone(),
                parent_ctx: evals[i].parent_ctx.clone(),
            });
        }

        while next_gen.len() < population_size {
            let p1 = tournament_selection(&evals, 3, &mut rng);
            let p2 = tournament_selection(&evals, 3, &mut rng);

            let (mut c1, mut c2) = if rng.gen_bool(0.8) {
                crossover.crossover(&p1.eval.candidate, &p2.eval.candidate, &mut rng)
            } else {
                (p1.eval.candidate.clone(), p2.eval.candidate.clone())
            };

            mutator.mutate(&mut c1, &mut rng);
            mutator.mutate(&mut c2, &mut rng);

            next_gen.push(Offspring {
                genome: c1,
                parent_ctx: Some(CvrpContextKey::from_eval(&p1.eval)),
            });
            if next_gen.len() < population_size {
                next_gen.push(Offspring {
                    genome: c2,
                    parent_ctx: Some(CvrpContextKey::from_eval(&p2.eval)),
                });
            }
        }
        population = next_gen;
    }
    metrics
}

fn main() {
    println!("=== M30.2B CVRP Active Pilot (Robustness Audit) ===");
    let seeds = vec![42, 100, 256, 512, 1024, 2048, 4096, 8192, 12345, 99999];

    let mut control_objs = Vec::new();
    let mut treatment_objs = Vec::new();
    let mut wins = 0;
    let mut worst_regression = 0.0;

    let start = Instant::now();

    for (i, &seed) in seeds.iter().enumerate() {
        println!("Seed {}/10 (u64: {})...", i + 1, seed);
        let control = run_pilot(false, seed);
        let treatment = run_pilot(true, seed);

        control_objs.push(control.best_objective);
        treatment_objs.push(treatment.best_objective);

        // Lower is better for CVRP distance
        let delta = control.best_objective - treatment.best_objective;
        if delta > 0.0 {
            wins += 1;
        } else if delta < worst_regression {
            worst_regression = delta;
        }

        println!(
            "  Control: {:.2}, Treatment: {:.2} (Delta: {:.2})",
            control.best_objective, treatment.best_objective, delta
        );
    }

    let c_mean = control_objs.iter().sum::<f64>() / seeds.len() as f64;
    let t_mean = treatment_objs.iter().sum::<f64>() / seeds.len() as f64;

    let c_var = control_objs
        .iter()
        .map(|v| (v - c_mean).powi(2))
        .sum::<f64>()
        / seeds.len() as f64;
    let t_var = treatment_objs
        .iter()
        .map(|v| (v - t_mean).powi(2))
        .sum::<f64>()
        / seeds.len() as f64;

    let el = start.elapsed();

    println!("\n=== Robustness Audit Results ===");
    println!("Control Mean:   {:.2} (std: {:.2})", c_mean, c_var.sqrt());
    println!("Treatment Mean: {:.2} (std: {:.2})", t_mean, t_var.sqrt());
    println!(
        "Mean Delta:     {:.2} (Positive is Improvement)",
        c_mean - t_mean
    );
    println!(
        "Win Rate:       {}/{} ({:.1}%)",
        wins,
        seeds.len(),
        (wins as f64 / seeds.len() as f64) * 100.0
    );
    println!("Worst Regress:  {:.2}", worst_regression);
    println!("\nCompleted in {:.2?}", el);
}
