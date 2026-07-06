/// M31.2A: Operator Engagement Audit
///
/// Measures exactly how much of M27's search is ecology-guided vs baseline,
/// per week. Tracks pressure distribution, imbalance evolution, and guidance
/// rate across the multi-week lifecycle.
///
/// Usage:
///   cargo run --release --bin m31_2a_engagement_audit -- --instance-prefix n050w4 --seed 42 --weeks 4

use clap::Parser;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use coralys_moga::traits::*;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use ultracrew::inrc::optimization::{InrcContext, InrcEvaluation, InrcGenome, InrcOptimizer};
use ultracrew::inrc::parser::{parse_history, parse_scenario, parse_week_data};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::workforce::WorkforceEcologyAdapter;
use ultracrew::inrc::history::extract_next_history;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    instance_prefix: String,
    #[arg(long, default_value_t = 42)]
    seed: u64,
    #[arg(long, default_value_t = 4)]
    weeks: usize,
    #[arg(long, default_value_t = 125)]
    generations: usize,
}

/// Telemetry returned from each mutation call
#[derive(Default)]
struct MutationTelemetry {
    attempted: u64,
    ecology_guided: u64,  // went through the ecology branch (avg_assignments > 0)
    baseline: u64,        // fell through to pure coin-flip
    directed: u64,        // ecology actually changed direction (not just coin-flip)
}

/// Instrument the mutator to produce telemetry, instead of a fire-and-forget MutationOperator impl.
fn mutate_with_telemetry(
    genome: &mut InrcGenome,
    adapter: &WorkforceEcologyAdapter,
    num_nurses: usize,
    num_days: usize,
    num_shifts: usize,
    rng: &mut StdRng,
) -> MutationTelemetry {
    let mut tel = MutationTelemetry::default();
    let rate = 1.0 / (genome.bits.len() as f64).max(1.0);

    // M27 branch: use accumulated ecology history for per-nurse loads
    let mut loads = vec![0.0f64; num_nurses];
    for n in 0..num_nurses {
        loads[n] = adapter.get_assignments(n);
    }
    let sum: f64 = loads.iter().sum();
    let avg_assignments = if num_nurses > 0 { sum / num_nurses as f64 } else { 0.0 };

    for i in 0..genome.bits.len() {
        if rng.gen_bool(rate) {
            tel.attempted += 1;
            let n = i / (num_days * num_shifts);

            if avg_assignments > 0.0 && rng.gen_bool(adapter.policy.alpha.min(1.0)) {
                // Ecology-guided branch
                tel.ecology_guided += 1;
                let load = loads[n];
                let is_working = genome.bits[i];
                let mut new_bit = is_working;

                if load > avg_assignments + 2.0 {
                    if is_working && rng.gen_bool(0.9) {
                        new_bit = false;
                    } else if !is_working && rng.gen_bool(0.1) {
                        new_bit = true;
                    }
                } else if load < avg_assignments - 2.0 {
                    if !is_working && rng.gen_bool(0.9) {
                        new_bit = true;
                    } else if is_working && rng.gen_bool(0.1) {
                        new_bit = false;
                    }
                } else {
                    new_bit = !is_working;
                }

                if new_bit != is_working {
                    genome.bits[i] = new_bit;
                    tel.directed += 1;
                }
            } else {
                // Baseline branch: pure bit flip
                tel.baseline += 1;
                genome.bits[i] = !genome.bits[i];
            }
        }
    }

    tel
}

fn tournament_select<'a>(evals: &'a [InrcEvaluation], k: usize, rng: &mut StdRng) -> &'a InrcEvaluation {
    let mut best_idx = rng.gen_range(0..evals.len());
    for _ in 1..k {
        let idx = rng.gen_range(0..evals.len());
        if evals[idx].fitness() > evals[best_idx].fitness() {
            best_idx = idx;
        }
    }
    &evals[best_idx]
}

fn main() {
    let args = Args::parse();

    let base_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data").join(&args.instance_prefix);

    let sc_path = base_dir.join(format!("Sc-{}.json", args.instance_prefix));
    let scenario = parse_scenario(&sc_path).unwrap();
    let num_nurses = scenario.nurses.len();
    let num_shifts = scenario.shift_types.len();
    let num_days = 7;

    let mut adapter = WorkforceEcologyAdapter::new(num_nurses, 1.0);

    let h0_path = base_dir.join(format!("H0-{}-0.json", args.instance_prefix));
    let h0 = parse_history(&h0_path).unwrap();
    let mut current_history = h0.clone();

    eprintln!("M31.2A Operator Engagement Audit");
    eprintln!("Instance: {}, Seed: {}, Weeks: {}, Gens/week: {}",
        args.instance_prefix, args.seed, args.weeks, args.generations);
    eprintln!("{:-<80}", "");

    let mut all_weeks = vec![];

    for w in 0..args.weeks {
        let wd_path = base_dir.join(format!("WD-{}-{}.json", args.instance_prefix, w));
        let week_data = parse_week_data(&wd_path).unwrap();

        let context = Arc::new(InrcContext::new(
            scenario.clone(), week_data, current_history.clone(), WorkforceEcology::new()
        ));
        let evaluator = InrcOptimizer { context: context.clone() };
        let mut rng = StdRng::seed_from_u64(args.seed + w as u64);

        // Snapshot ecology state at start of week
        let week_start_assignments: Vec<f64> = (0..num_nurses).map(|n| adapter.get_assignments(n)).collect();
        let total_assignments: f64 = week_start_assignments.iter().sum();
        let mean_assignments = if num_nurses > 0 { total_assignments / num_nurses as f64 } else { 0.0 };

        // Compute imbalance: std dev of loads
        let variance: f64 = week_start_assignments.iter()
            .map(|&x| (x - mean_assignments).powi(2))
            .sum::<f64>() / num_nurses as f64;
        let std_dev = variance.sqrt();

        // Count nurses with non-zero signal
        let nurses_with_signal = week_start_assignments.iter()
            .filter(|&&x| (x - mean_assignments).abs() > 2.0)
            .count();

        // Run evolution with telemetry
        let population_size = 100;
        let mut population: Vec<InrcGenome> = (0..population_size)
            .map(|_| {
                let size = num_nurses * num_days * num_shifts;
                let bits = (0..size).map(|_| rng.gen_bool(0.22)).collect();
                InrcGenome { bits }
            })
            .collect();

        let mut week_tel = MutationTelemetry::default();
        let mut global_best: Option<InrcEvaluation> = None;

        for _gen in 0..args.generations {
            let evals: Vec<InrcEvaluation> = population.iter()
                .map(|g| evaluator.evaluate(g))
                .filter(|e| e.is_valid())
                .collect();

            if evals.is_empty() {
                population = (0..population_size)
                    .map(|_| {
                        let size = num_nurses * num_days * num_shifts;
                        let bits = (0..size).map(|_| rng.gen_bool(0.22)).collect();
                        InrcGenome { bits }
                    })
                    .collect();
                continue;
            }

            let gen_best = evals.iter().max_by(|a, b|
                a.fitness().partial_cmp(&b.fitness()).unwrap()
            ).unwrap();

            if global_best.is_none() || gen_best.fitness() > global_best.as_ref().unwrap().fitness() {
                global_best = Some(gen_best.clone());
            }

            // Build next gen with telemetry
            let mut next_gen = vec![];

            // Elites
            let mut sorted = evals.clone();
            sorted.sort_by(|a, b| b.fitness().partial_cmp(&a.fitness()).unwrap());
            for i in 0..5.min(sorted.len()) {
                next_gen.push(sorted[i].genome().clone());
            }

            while next_gen.len() < population_size {
                let p1 = tournament_select(&evals, 3, &mut rng);
                let p2 = tournament_select(&evals, 3, &mut rng);

                let mut c1 = p1.genome().clone();
                let mut c2 = p2.genome().clone();

                // Simple crossover
                if rng.gen_bool(0.8) {
                    let point = rng.gen_range(0..c1.bits.len());
                    for i in point..c1.bits.len() {
                        let tmp = c1.bits[i];
                        c1.bits[i] = c2.bits[i];
                        c2.bits[i] = tmp;
                    }
                }

                let t1 = mutate_with_telemetry(&mut c1, &adapter, num_nurses, num_days, num_shifts, &mut rng);
                let t2 = mutate_with_telemetry(&mut c2, &adapter, num_nurses, num_days, num_shifts, &mut rng);

                week_tel.attempted += t1.attempted + t2.attempted;
                week_tel.ecology_guided += t1.ecology_guided + t2.ecology_guided;
                week_tel.baseline += t1.baseline + t2.baseline;
                week_tel.directed += t1.directed + t2.directed;

                next_gen.push(c1);
                if next_gen.len() < population_size {
                    next_gen.push(c2);
                }
            }

            population = next_gen;
        }

        let best_eval = global_best.unwrap();

        // Compute ecology guidance rate
        let guidance_rate = if week_tel.attempted > 0 {
            week_tel.ecology_guided as f64 / week_tel.attempted as f64 * 100.0
        } else { 0.0 };
        let direction_rate = if week_tel.ecology_guided > 0 {
            week_tel.directed as f64 / week_tel.ecology_guided as f64 * 100.0
        } else { 0.0 };

        println!("Week {w}:");
        println!("  Ecology State:  mean_assignments={mean_assignments:.1}  std_dev={std_dev:.2}  nurses_with_signal={nurses_with_signal}/{num_nurses}");
        println!("  Mutations:      attempted={attempted}  ecology_guided={guided}  baseline={baseline}",
            attempted = week_tel.attempted,
            guided = week_tel.ecology_guided,
            baseline = week_tel.baseline);
        println!("  Guidance Rate:  {guidance_rate:.1}%  (of those, direction_changed={:.1}%)", direction_rate);
        println!("  Week Objective: {}", best_eval.fitness() as i64);
        println!();

        all_weeks.push((
            w, mean_assignments, std_dev, nurses_with_signal,
            guidance_rate, direction_rate, best_eval.fitness() as i64
        ));

        // Advance ecology
        let next_hist = extract_next_history(&context, best_eval.genome());
        for n in 0..num_nurses {
            adapter.accumulate_assignments(n,
                next_hist.nurse_history[n].number_of_assignments
                    - current_history.nurse_history[n].number_of_assignments);
            adapter.accumulate_weekends(n,
                next_hist.nurse_history[n].number_of_working_weekends
                    - current_history.nurse_history[n].number_of_working_weekends);
        }
        current_history = next_hist;
    }

    // Summary table as JSON for easy parsing
    let rows: Vec<serde_json::Value> = all_weeks.iter().map(|&(w, mean, std, signal_nurses, guidance, direction, obj)| {
        serde_json::json!({
            "week": w,
            "mean_assignments": mean,
            "assignment_std_dev": std,
            "nurses_with_signal": signal_nurses,
            "guidance_rate_pct": guidance,
            "direction_rate_pct": direction,
            "objective": obj
        })
    }).collect();

    println!("JSON_SUMMARY:");
    println!("{}", serde_json::to_string_pretty(&rows).unwrap());
}
