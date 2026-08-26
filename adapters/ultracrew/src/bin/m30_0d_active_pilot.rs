use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use coralys_moga::traits::*;
use coralys_v2::{AdvisoryCandidate, AdvisoryRanker, ContextKey, OpportunityMemory};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::inrc::optimization::{InrcContext, InrcEvaluation, InrcGenome, InrcOptimizer};
use ultracrew::inrc::parser::{parse_history, parse_scenario, parse_week_data};

#[derive(Clone, Hash, Eq, PartialEq)]
struct InrcContextKey {
    hc_cov: usize,
    hc_skills: usize,
    hc_1shift: usize,
    hc_forb: usize,
    soft_pen_bucket: i32,
}

impl ContextKey for InrcContextKey {}

impl InrcContextKey {
    fn from_eval(eval: &InrcEvaluation) -> Self {
        Self {
            hc_cov: eval.hc_coverage,
            hc_skills: eval.hc_skills,
            hc_1shift: eval.hc_one_shift_per_day,
            hc_forb: eval.hc_forbidden_successions,
            soft_pen_bucket: eval.soft_report.total_penalty / 250 * 250,
        }
    }
}

struct Offspring {
    genome: InrcGenome,
    parent_ctx: Option<InrcContextKey>,
}

struct EvaluatedOffspring {
    eval: InrcEvaluation,
    parent_ctx: Option<InrcContextKey>,
}

impl AdvisoryCandidate for EvaluatedOffspring {
    type Context = InrcContextKey;

    fn fitness_bucket(&self) -> i64 {
        (self.eval.fitness() / 100.0).floor() as i64
    }

    fn parent_context(&self) -> Option<&Self::Context> {
        self.parent_ctx.as_ref()
    }

    fn lower_is_better() -> bool {
        false // UltraCrew maximizes fitness
    }

    fn fallback_cmp(&self, other: &Self) -> Ordering {
        self.eval
            .fitness()
            .partial_cmp(&other.eval.fitness())
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
        if best.is_none() || e.eval.fitness() > best.unwrap().eval.fitness() {
            best = Some(e);
        }
    }
    best.unwrap()
}

fn run_pilot(use_advisory: bool, seed: u64) -> RunMetrics {
    let base_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/n030w4");

    let scenario = parse_scenario(base_dir.join("Sc-n030w4.json")).unwrap();
    let current_history = parse_history(base_dir.join("H0-n030w4-0.json")).unwrap();
    let week_data = parse_week_data(base_dir.join("WD-n030w4-0.json")).unwrap();

    let context = Arc::new(InrcContext::new(
        scenario,
        week_data,
        current_history,
        WorkforceEcology::new(),
    ));
    let evaluator = InrcOptimizer {
        context: context.clone(),
    };

    let mut rng = StdRng::seed_from_u64(seed);

    let population_size = 100;
    let generations = 500;

    let mut memory = OpportunityMemory::new(500.0, 0.0048);

    let mut population: Vec<Offspring> = (0..population_size)
        .map(|_| Offspring {
            genome: evaluator.create(&mut rng),
            parent_ctx: None,
        })
        .collect();

    let mut global_best: Option<InrcEvaluation> = None;
    let mut metrics = RunMetrics {
        best_objective: -f64::MAX,
        time_to_best: 0,
        champion_count: 0,
        known_evals: 0,
        total_evals: 0,
        total_generated_score: 0.0,
        total_selected_score: 0.0,
    };

    for gen in 0..generations {
        let mut evals: Vec<EvaluatedOffspring> = population
            .drain(..)
            .map(|off| EvaluatedOffspring {
                eval: evaluator.evaluate(
                    &off.genome,
                    &coralys_moga::runtime::optimization::metric::MetricReport::default(),
                ),
                parent_ctx: off.parent_ctx,
            })
            .filter(|e| e.eval.is_valid())
            .collect();

        if evals.is_empty() {
            population = (0..population_size)
                .map(|_| Offspring {
                    genome: evaluator.create(&mut rng),
                    parent_ctx: None,
                })
                .collect();
            continue;
        }

        if gen >= 100 {
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

        if use_advisory && gen >= 100 {
            let default_ctx = InrcContextKey {
                hc_cov: 0,
                hc_skills: 0,
                hc_1shift: 0,
                hc_forb: 0,
                soft_pen_bucket: 0,
            };
            AdvisoryRanker::sort(&mut evals, &memory, &default_ctx);
        } else {
            evals.sort_by(|a, b| {
                b.eval
                    .fitness()
                    .partial_cmp(&a.eval.fitness())
                    .unwrap_or(Ordering::Equal)
            });
        }

        let gen_best = evals[0].eval.clone();

        let mut new_global_best = false;
        if global_best.is_none() || gen_best.fitness() > global_best.as_ref().unwrap().fitness() {
            global_best = Some(gen_best.clone());
            new_global_best = true;
            metrics.best_objective = gen_best.fitness();
            metrics.time_to_best = gen;
            metrics.champion_count += 1;
        }

        for (rank, e) in evals.iter().enumerate() {
            if gen >= 100 && rank < population_size / 2 {
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
                genome: evals[i].eval.genome().clone(),
                parent_ctx: evals[i].parent_ctx.clone(),
            });
        }

        while next_gen.len() < population_size {
            let p1 = tournament_selection(&evals, 3, &mut rng);
            let p2 = tournament_selection(&evals, 3, &mut rng);

            let mut c1 = p1.eval.genome().clone();
            let mut c2 = p2.eval.genome().clone();

            if rng.gen_bool(0.8) {
                evaluator.crossover(&mut c1, &mut c2, &mut rng);
            }
            evaluator.mutate(&mut c1, &mut rng);
            evaluator.mutate(&mut c2, &mut rng);

            next_gen.push(Offspring {
                genome: c1,
                parent_ctx: Some(InrcContextKey::from_eval(&p1.eval)),
            });
            if next_gen.len() < population_size {
                next_gen.push(Offspring {
                    genome: c2,
                    parent_ctx: Some(InrcContextKey::from_eval(&p2.eval)),
                });
            }
        }
        population = next_gen;
    }
    metrics
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

fn main() {
    println!("=== M30.0D.4 Robustness Audit ===");

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

        let delta = treatment.best_objective - control.best_objective;
        if delta > 0.0 {
            wins += 1;
        } else if delta < worst_regression {
            worst_regression = delta;
        }

        println!(
            "  Control: {}, Treatment: {} (Delta: {})",
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
    println!("Mean Delta:     {:.2}", t_mean - c_mean);
    println!(
        "Win Rate:       {}/{} ({:.1}%)",
        wins,
        seeds.len(),
        (wins as f64 / seeds.len() as f64) * 100.0
    );
    println!("Worst Regress:  {:.2}", worst_regression);
    println!("\nCompleted in {:.2?}", el);
}
