use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;

use coralys_moga::config::EvolutionConfig;
use coralys_moga::engine::EvolutionEngine;
use coralys_moga::traits::{CrossoverOperator, GenomeFactory, MutationOperator};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use ultracrew::ecology::{EcologyPolicy, EcologyState, WorkforceEcology};
use ultracrew::inrc::history::extract_next_history;
use ultracrew::inrc::optimization::{InrcContext, InrcGenome, InrcOptimizer};
use ultracrew::inrc::parser::{parse_history, parse_scenario, parse_week_data};

// ── GA Components with Ecology Interpolation ───────────────────────────────
//
// The factory and mutator each define "neutral" and "aggressive" behavior,
// then use EcologyPolicy::interpolate to blend between them.
//
// Invariants:
//   alpha = 0.0 → neutral only → identical to STATE_ONLY / no-ecology search
//   alpha = 1.0 → aggressive only → identical to original FULL_ECOLOGY

#[derive(Clone)]
struct EcologyGenomeFactory {
    num_nurses: usize,
    num_days: usize,
    num_shifts: usize,
    ecology: EcologyState,
    policy: EcologyPolicy,
}

impl GenomeFactory<InrcGenome> for EcologyGenomeFactory {
    fn create(&self, rng: &mut StdRng) -> InrcGenome {
        let size = self.num_nurses * self.num_days * self.num_shifts;
        let mut bits = vec![false; size];
        let avg_assignments = self.ecology.mean_assignments();

        for n in 0..self.num_nurses {
            let base_prob: f64 = 0.22; // neutral probability

            // Compute the aggressive (full ecology) probability for this nurse
            let aggressive_prob = if avg_assignments > 0.0 {
                let load = self.ecology.cumulative_assignments[n] as f64;
                let load_ratio = load / avg_assignments;
                // Original FULL_ECOLOGY formula (at old alpha=1.0):
                // bias = clamp(2.0 - load_ratio, 0.7, 1.3)
                let bias = (2.0 - load_ratio).max(0.7).min(1.3);
                (base_prob * bias).min(1.0)
            } else {
                base_prob
            };

            // Interpolate: alpha=0 → base_prob, alpha=1 → aggressive_prob
            let prob = self.policy.interpolate(base_prob, aggressive_prob);

            for d in 0..self.num_days {
                if rng.gen_bool(prob.max(0.0).min(1.0)) {
                    let shift_idx = rng.gen_range(0..self.num_shifts);
                    let idx =
                        n * (self.num_days * self.num_shifts) + d * self.num_shifts + shift_idx;
                    bits[idx] = true;
                }
            }
        }
        InrcGenome { bits }
    }
}

#[derive(Clone)]
struct EcologyMutator {
    ecology: EcologyState,
    policy: EcologyPolicy,
    num_nurses: usize,
    num_days: usize,
    num_shifts: usize,
}

impl MutationOperator<InrcGenome> for EcologyMutator {
    fn mutate(&self, genome: &mut InrcGenome, rng: &mut StdRng) {
        let rate = 1.0 / (genome.bits.len() as f64).max(1.0);
        let avg_assignments = self.ecology.mean_assignments();

        for i in 0..genome.bits.len() {
            if rng.gen_bool(rate) {
                let n = i / (self.num_days * self.num_shifts);

                // Neutral behavior: simple bit flip (STATE_ONLY equivalent)
                // Aggressive behavior: load-steered flip (FULL_ECOLOGY equivalent)
                //
                // With probability alpha, use aggressive steering.
                // With probability (1-alpha), use neutral flip.
                // This ensures alpha=0 gives pure neutral and alpha=1 gives pure aggressive.

                if self.policy.alpha > 0.0
                    && avg_assignments > 0.0
                    && rng.gen_bool(self.policy.alpha.min(1.0))
                {
                    let load = self.ecology.cumulative_assignments[n] as f64;
                    let is_working = genome.bits[i];

                    if load > avg_assignments + 2.0 {
                        // High load: strongly prefer removing assignments
                        if is_working && rng.gen_bool(0.9) {
                            genome.bits[i] = false;
                        } else if !is_working && rng.gen_bool(0.1) {
                            genome.bits[i] = true;
                        }
                    } else if load < avg_assignments - 2.0 {
                        // Low load: strongly prefer adding assignments
                        if !is_working && rng.gen_bool(0.9) {
                            genome.bits[i] = true;
                        } else if is_working && rng.gen_bool(0.1) {
                            genome.bits[i] = false;
                        }
                    } else {
                        // Balanced load: normal flip
                        genome.bits[i] = !genome.bits[i];
                    }
                } else {
                    // Neutral: simple bit flip
                    genome.bits[i] = !genome.bits[i];
                }
            }
        }
    }
}

// ── Metrics ────────────────────────────────────────────────────────────────

fn calculate_gini(values: &[usize]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort();
    let n = sorted.len();
    let sum: usize = sorted.iter().sum();
    if sum == 0 {
        return 0.0;
    }

    let mut index_sum = 0.0;
    for (i, val) in sorted.iter().enumerate() {
        index_sum += (i as f64 + 1.0) * (*val as f64);
    }

    let n_f64 = n as f64;
    let sum_f64 = sum as f64;

    (2.0 * index_sum) / (n_f64 * sum_f64) - (n_f64 + 1.0) / n_f64
}

/// Compute coverage_ratio = fulfilled_demand / required_demand
/// where fulfilled_demand = sum(min(assigned, required)) over all (day, shift, skill)
/// and required_demand = sum(required) over all (day, shift, skill).
/// Uses hard-constraint minimum only, NOT optimal targets.
fn compute_coverage_ratio(context: &Arc<InrcContext>, genome: &InrcGenome) -> f64 {
    let num_nurses = context.num_nurses;
    let num_days = context.num_days;
    let num_shifts = context.shift_types.len();
    let days_map = vec![
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ];

    let mut total_required: usize = 0;
    let mut total_fulfilled: usize = 0;

    for d in 0..num_days {
        let day_name = days_map[d];
        for s in 0..num_shifts {
            let shift_name = &context.shift_types[s];

            // Collect demands for this (day, shift)
            let mut demands = Vec::new();
            for req in &context.week_data.requirements {
                if req.shift_type == *shift_name {
                    let req_level = match day_name {
                        "Monday" => &req.monday,
                        "Tuesday" => &req.tuesday,
                        "Wednesday" => &req.wednesday,
                        "Thursday" => &req.thursday,
                        "Friday" => &req.friday,
                        "Saturday" => &req.saturday,
                        "Sunday" => &req.sunday,
                        _ => unreachable!(),
                    };
                    if req_level.minimum > 0 {
                        demands.push((&req.skill, req_level.minimum));
                    }
                }
            }

            // Collect available assigned nurses for this (day, shift)
            let mut available_nurses: Vec<usize> = Vec::new();
            for n in 0..num_nurses {
                let idx = n * (num_days * num_shifts) + d * num_shifts + s;
                if genome.bits[idx] {
                    available_nurses.push(n);
                }
            }

            // Greedy fulfillment (same algorithm as evaluator)
            for (skill, min_count) in demands {
                total_required += min_count;

                let mut fulfilled = 0;
                let mut to_remove = Vec::new();
                for (i, &n) in available_nurses.iter().enumerate() {
                    let nurse = &context.scenario.nurses[n];
                    if nurse.skills.contains(skill) {
                        fulfilled += 1;
                        to_remove.push(i);
                        if fulfilled == min_count {
                            break;
                        }
                    }
                }
                total_fulfilled += fulfilled;

                for &i in to_remove.iter().rev() {
                    available_nurses.remove(i);
                }
            }
        }
    }

    if total_required == 0 {
        1.0 // No demand → fully covered by definition
    } else {
        total_fulfilled as f64 / total_required as f64
    }
}

#[derive(Debug)]
struct RunMetrics {
    cum_score: i32,
    cum_hard: i32,
    assignment_variance: f64,
    weekend_concentration: f64,
    best_weekly: i32,
    mean_weekly: f64,
    gini: f64,
    persistence_score: f64,
    assignment_range: usize,
    normalized_assignment_range: f64,
    coefficient_of_variation: f64,
    coverage_ratio: f64,
}

/// Run a 4-week simulation with the given alpha.
/// alpha = 0.0 reproduces STATE_ONLY behavior exactly.
/// alpha = 1.0 reproduces FULL_ECOLOGY behavior exactly.
fn run_ablation(seed: u64, alpha: f64) -> RunMetrics {
    let base_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/n030w4");
    let scenario = parse_scenario(base_dir.join("Sc-n030w4.json")).unwrap();
    let num_nurses = scenario.nurses.len();

    let mut ecology_state = EcologyState::new(num_nurses);
    let policy = EcologyPolicy::new(alpha);

    // Initial H0 assignments (for persistence score)
    let h0 = parse_history(base_dir.join("H0-n030w4-0.json")).unwrap();
    let mut init_assignments = vec![0; num_nurses];
    for n in 0..num_nurses {
        init_assignments[n] = h0.nurse_history[n].number_of_assignments;
    }

    let mut current_history = h0.clone();

    let mut cum_score = 0;
    let mut cum_hard = 0;
    let mut best_weekly = i32::MAX;
    let mut cum_coverage_fulfilled: usize = 0;
    let mut cum_coverage_required: usize = 0;

    for w in 0..4 {
        let wd_path = base_dir.join(format!("WD-n030w4-{}.json", w));
        let week_data = parse_week_data(wd_path).unwrap();

        let ecology = WorkforceEcology::new();
        let context = Arc::new(InrcContext::new(
            scenario.clone(),
            week_data,
            current_history.clone(),
            ecology.clone(),
        ));
        let evaluator = InrcOptimizer {
            context: context.clone(),
        };

        let factory = EcologyGenomeFactory {
            num_nurses,
            num_days: 7,
            num_shifts: scenario.shift_types.len(),
            ecology: ecology_state.clone(),
            policy: policy.clone(),
        };

        let mutator = EcologyMutator {
            ecology: ecology_state.clone(),
            policy: policy.clone(),
            num_nurses,
            num_days: 7,
            num_shifts: scenario.shift_types.len(),
        };

        // Use InrcOptimizer for crossover (standard uniform)
        let crossover = InrcOptimizer {
            context: context.clone(),
        };

        let config = EvolutionConfig {
            population_size: 100,
            generation_limit: 100,
            elite_count: 5,
            seed: Some(seed + w as u64),
            ..Default::default()
        };

        let engine = EvolutionEngine::new(evaluator.clone(), mutator, crossover, factory);
        let result = engine.run_ga_evolution(config);

        let best = result.expect("Evolution engine failed").global_best;

        cum_score += best.soft_report.total_penalty;
        let hc = best.hc_coverage
            + best.hc_skills
            + best.hc_one_shift_per_day
            + best.hc_forbidden_successions;
        cum_hard += hc as i32;
        if best.soft_report.total_penalty < best_weekly {
            best_weekly = best.soft_report.total_penalty;
        }

        // Compute per-week coverage ratio and accumulate
        let week_coverage = compute_coverage_ratio(&context, &best.genome);
        // For cumulative coverage, re-derive the raw counts
        let num_days = 7;
        let num_shifts = context.shift_types.len();
        let days_map = vec![
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
            "Sunday",
        ];
        for d in 0..num_days {
            let day_name = days_map[d];
            for s in 0..num_shifts {
                let shift_name = &context.shift_types[s];
                for req in &context.week_data.requirements {
                    if req.shift_type == *shift_name {
                        let req_level = match day_name {
                            "Monday" => &req.monday,
                            "Tuesday" => &req.tuesday,
                            "Wednesday" => &req.wednesday,
                            "Thursday" => &req.thursday,
                            "Friday" => &req.friday,
                            "Saturday" => &req.saturday,
                            "Sunday" => &req.sunday,
                            _ => unreachable!(),
                        };
                        if req_level.minimum > 0 {
                            cum_coverage_required += req_level.minimum;
                            // Count fulfilled (greedy, mirroring evaluator logic)
                            let mut available: Vec<usize> = Vec::new();
                            for n in 0..num_nurses {
                                let idx = n * (num_days * num_shifts) + d * num_shifts + s;
                                if best.genome.bits[idx] {
                                    available.push(n);
                                }
                            }
                            let mut fulfilled = 0;
                            for &n in &available {
                                let nurse = &context.scenario.nurses[n];
                                if nurse.skills.contains(&req.skill) {
                                    fulfilled += 1;
                                    if fulfilled == req_level.minimum {
                                        break;
                                    }
                                }
                            }
                            cum_coverage_fulfilled += fulfilled;
                        }
                    }
                }
            }
        }

        let next_hist = extract_next_history(&context, &best.genome);

        // Update cumulative ecology state
        for n in 0..num_nurses {
            let week_assignments = next_hist.nurse_history[n].number_of_assignments
                - current_history.nurse_history[n].number_of_assignments;
            let week_weekends = next_hist.nurse_history[n].number_of_working_weekends
                - current_history.nurse_history[n].number_of_working_weekends;

            ecology_state.cumulative_assignments[n] += week_assignments;
            ecology_state.cumulative_weekends[n] += week_weekends;
        }

        current_history = next_hist;
    }

    let mean_weekly = cum_score as f64 / 4.0;

    // ── Fairness Metrics ───────────────────────────────────────────────────
    let mean_assignments = ecology_state.mean_assignments();
    let assignment_variance = ecology_state
        .cumulative_assignments
        .iter()
        .map(|&x| (x as f64 - mean_assignments).powi(2))
        .sum::<f64>()
        / num_nurses as f64;

    let mean_weekends =
        ecology_state.cumulative_weekends.iter().sum::<usize>() as f64 / num_nurses as f64;
    let weekend_concentration = ecology_state
        .cumulative_weekends
        .iter()
        .map(|&x| (x as f64 - mean_weekends).powi(2))
        .sum::<f64>()
        / num_nurses as f64;

    let gini = calculate_gini(&ecology_state.cumulative_assignments);

    // Assignment range
    let max_a = *ecology_state
        .cumulative_assignments
        .iter()
        .max()
        .unwrap_or(&0);
    let min_a = *ecology_state
        .cumulative_assignments
        .iter()
        .min()
        .unwrap_or(&0);
    let assignment_range = max_a - min_a;
    let normalized_assignment_range = if mean_assignments > 0.0 {
        assignment_range as f64 / mean_assignments
    } else {
        0.0
    };

    // Coefficient of variation
    let std_dev = assignment_variance.sqrt();
    let coefficient_of_variation = if mean_assignments > 0.0 {
        std_dev / mean_assignments
    } else {
        0.0
    };

    // Coverage ratio (cumulative)
    let coverage_ratio = if cum_coverage_required == 0 {
        1.0
    } else {
        cum_coverage_fulfilled as f64 / cum_coverage_required as f64
    };

    // Persistence score
    let mut init_ranked: Vec<usize> = (0..num_nurses).collect();
    init_ranked.sort_by_key(|&i| init_assignments[i]);
    let mut final_ranked: Vec<usize> = (0..num_nurses).collect();
    final_ranked.sort_by_key(|&i| ecology_state.cumulative_assignments[i]);

    let mut rank_change_sum = 0;
    for n in 0..num_nurses {
        let init_rank = init_ranked.iter().position(|&x| x == n).unwrap();
        let final_rank = final_ranked.iter().position(|&x| x == n).unwrap();
        rank_change_sum += (init_rank as i32 - final_rank as i32).abs();
    }
    let persistence_score = rank_change_sum as f64 / num_nurses as f64;

    RunMetrics {
        cum_score,
        cum_hard,
        assignment_variance,
        weekend_concentration,
        best_weekly,
        mean_weekly,
        gini,
        persistence_score,
        assignment_range,
        normalized_assignment_range,
        coefficient_of_variation,
        coverage_ratio,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Parse CLI flags
    let mut num_seeds: u64 = 10;
    let mut alphas: Vec<f64> = vec![0.0, 0.1, 0.2, 0.4, 0.6, 0.8, 1.0];
    let mut output_file = "ablation_multi_results.csv".to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--seeds" => {
                i += 1;
                num_seeds = args[i].parse().expect("--seeds requires a number");
            }
            "--alphas" => {
                i += 1;
                alphas = args[i]
                    .split(',')
                    .map(|s| {
                        s.trim()
                            .parse::<f64>()
                            .expect("--alphas requires comma-separated floats")
                    })
                    .collect();
            }
            "--output" => {
                i += 1;
                output_file = args[i].clone();
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    println!("F.2D.1 Multi-Week Ecology Alpha Sweep");
    println!("  Seeds: {}", num_seeds);
    println!("  Alphas: {:?}", alphas);
    println!("  Output: {}", output_file);
    println!();

    let mut file = File::create(&output_file).unwrap();
    writeln!(
        file,
        "seed,alpha,cum_score,cum_hard,assignment_variance,weekend_concentration,\
        best_weekly,mean_weekly,gini,persistence_score,assignment_range,\
        normalized_assignment_range,coefficient_of_variation,coverage_ratio"
    )
    .unwrap();

    for i in 0..num_seeds {
        let seed = 12345 + i;

        for &alpha in &alphas {
            let start = Instant::now();
            let metrics = run_ablation(seed, alpha);
            let elapsed = start.elapsed();

            println!("  Seed {} alpha={:.2} → score={} hard={} gini={:.4} cv={:.4} coverage={:.4} [{:.1}s]",
                seed, alpha, metrics.cum_score, metrics.cum_hard,
                metrics.gini, metrics.coefficient_of_variation, metrics.coverage_ratio,
                elapsed.as_secs_f64());

            writeln!(
                file,
                "{},{:.2},{},{},{:.4},{:.4},{},{:.4},{:.4},{:.4},{},{:.4},{:.4},{:.4}",
                seed,
                alpha,
                metrics.cum_score,
                metrics.cum_hard,
                metrics.assignment_variance,
                metrics.weekend_concentration,
                metrics.best_weekly,
                metrics.mean_weekly,
                metrics.gini,
                metrics.persistence_score,
                metrics.assignment_range,
                metrics.normalized_assignment_range,
                metrics.coefficient_of_variation,
                metrics.coverage_ratio
            )
            .unwrap();
        }
    }
    println!("\nAlpha sweep completed successfully.");
}
