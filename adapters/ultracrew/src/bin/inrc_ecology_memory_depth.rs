use std::cmp::Ordering;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;

use coralys_moga::config::EvolutionConfig;
use coralys_moga::traits::{
    CrossoverOperator, Evaluated, FitnessEvaluator, Genome, GenomeFactory, MutationOperator,
};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use coralys_moga::ecology::distribution_gini;
use ultracrew::ecology::WorkforceEcology;
use ultracrew::inrc::history::extract_next_history;
use ultracrew::inrc::optimization::{InrcContext, InrcEvaluation, InrcGenome, InrcOptimizer};
use ultracrew::inrc::parser::{parse_history, parse_scenario, parse_week_data};
use ultracrew::workforce::{NurseId, WorkforceEcologyAdapter};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum DepthArm {
    Off,
    Depth1w,
    Depth2w,
    Depth4w,
    DepthFull,
}

// ── Metrics ────────────────────────────────────────────────────────────────

// calculate_gini was here but we will use distribution_gini

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

            let mut available_nurses: Vec<usize> = Vec::new();
            for n in 0..num_nurses {
                let idx = n * (num_days * num_shifts) + d * num_shifts + s;
                if genome.bits[idx] {
                    available_nurses.push(n);
                }
            }

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
        1.0
    } else {
        total_fulfilled as f64 / total_required as f64
    }
}

// ── GA Components ─────────────────────────────────────────────────────────

#[derive(Clone)]
struct EcologyGenomeFactory {
    num_nurses: usize,
    num_days: usize,
    num_shifts: usize,
    adapter: WorkforceEcologyAdapter,
    arm: DepthArm,
}

impl GenomeFactory<InrcGenome> for EcologyGenomeFactory {
    fn create(&self, rng: &mut StdRng) -> InrcGenome {
        let size = self.num_nurses * self.num_days * self.num_shifts;
        let mut bits = vec![false; size];
        let avg_assignments: f64 = (0..self.num_nurses)
            .map(|n| self.adapter.get_assignments(n))
            .sum::<f64>()
            / self.num_nurses as f64;

        for n in 0..self.num_nurses {
            let base_prob: f64 = 0.22;

            let prob = if self.arm != DepthArm::Off {
                let aggressive_prob = if avg_assignments > 0.0 {
                    let signal = self.adapter.compute_signal(n, self.num_nurses);
                    let bias = (1.0 + signal.pressure).max(0.7).min(1.3);
                    (base_prob * bias).min(1.0)
                } else {
                    base_prob
                };

                let alpha = self.adapter.policy.alpha.max(0.0).min(1.0);
                alpha * aggressive_prob + (1.0 - alpha) * base_prob
            } else {
                base_prob
            };

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
    adapter: WorkforceEcologyAdapter,
    num_nurses: usize,
    num_days: usize,
    num_shifts: usize,
    arm: DepthArm,
}

impl MutationOperator<InrcGenome> for EcologyMutator {
    fn mutate(&self, genome: &mut InrcGenome, rng: &mut StdRng) {
        let rate = 1.0 / (genome.bits.len() as f64).max(1.0);

        if self.arm == DepthArm::Off {
            for i in 0..genome.bits.len() {
                if rng.gen_bool(rate) {
                    genome.bits[i] = !genome.bits[i];
                }
            }
            return;
        }

        let (mut nurse_loads, mut avg_assignments) = {
            let mut loads = vec![0.0; self.num_nurses];
            for n in 0..self.num_nurses {
                loads[n] = self.adapter.get_assignments(n);
            }
            let sum: f64 = loads.iter().sum();
            (loads, sum / self.num_nurses as f64)
        };

        for i in 0..genome.bits.len() {
            if rng.gen_bool(rate) {
                let n = i / (self.num_days * self.num_shifts);

                if avg_assignments > 0.0 && rng.gen_bool(self.adapter.policy.alpha.min(1.0)) {
                    let load = nurse_loads[n];
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
                    }
                } else {
                    genome.bits[i] = !genome.bits[i];
                }
            }
        }
    }
}

// ── Custom GA Loop ────────────────────────────────────────────────────────

fn tournament_selection<'a>(
    evals: &'a [InrcEvaluation],
    k: usize,
    rng: &mut StdRng,
) -> &'a InrcEvaluation {
    let mut best: Option<&'a InrcEvaluation> = None;
    for _ in 0..k {
        let idx = rng.gen_range(0..evals.len());
        let eval = &evals[idx];
        if best.is_none() || eval.fitness() > best.unwrap().fitness() {
            best = Some(eval);
        }
    }
    best.unwrap()
}

fn run_ablation(seed: u64, arm: DepthArm, out_csv: &mut File) {
    let base_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/n030w4");
    let scenario = parse_scenario(base_dir.join("Sc-n030w4.json")).unwrap();
    let num_nurses = scenario.nurses.len();
    let num_shifts = scenario.shift_types.len();

    let h0 = parse_history(base_dir.join("H0-n030w4-0.json")).unwrap();
    let mut current_history = h0.clone();

    let mut week_history: Vec<Vec<f64>> = Vec::new();
    // Synthesize weeks -4, -3, -2, -1
    for _ in 0..4 {
        let mut h = vec![0.0; num_nurses];
        for n in 0..num_nurses {
            h[n] = current_history.nurse_history[n].number_of_assignments as f64 / 4.0;
        }
        week_history.push(h);
    }

    for w in 0..4 {
        let wd_path = base_dir.join(format!("WD-n030w4-{}.json", w));
        let week_data = parse_week_data(wd_path).unwrap();
        let context = Arc::new(InrcContext::new(
            scenario.clone(),
            week_data,
            current_history.clone(),
            WorkforceEcology::new(),
        ));
        let evaluator = InrcOptimizer {
            context: context.clone(),
        };

        let mut adapter = WorkforceEcologyAdapter::new(num_nurses, 1.0);
        let depth = match arm {
            DepthArm::Off => 0,
            DepthArm::Depth1w => 1,
            DepthArm::Depth2w => 2,
            DepthArm::Depth4w => 4,
            DepthArm::DepthFull => week_history.len(),
        };

        if depth > 0 {
            let start_idx = if week_history.len() > depth {
                week_history.len() - depth
            } else {
                0
            };
            for hist_w in start_idx..week_history.len() {
                for n in 0..num_nurses {
                    adapter
                        .memory
                        .accumulate(n, "assignments", week_history[hist_w][n]);
                }
            }
        }

        let factory = EcologyGenomeFactory {
            num_nurses,
            num_days: 7,
            num_shifts,
            adapter: adapter.clone(),
            arm,
        };

        let mutator = EcologyMutator {
            adapter: adapter.clone(),
            num_nurses,
            num_days: 7,
            num_shifts,
            arm,
        };

        let crossover = InrcOptimizer {
            context: context.clone(),
        };

        let mut rng = StdRng::seed_from_u64(seed + w as u64);
        let mut population = (0..100)
            .map(|_| factory.create(&mut rng))
            .collect::<Vec<_>>();
        let mut best_overall: Option<InrcEvaluation> = None;

        for gen in 0..100 {
            let mut evals: Vec<InrcEvaluation> = population
                .iter()
                .map(|g| {
                    evaluator.evaluate(
                        g,
                        &coralys_moga::runtime::optimization::metric::MetricReport::default(),
                    )
                })
                .filter(|e| e.is_valid())
                .collect();

            if evals.is_empty() {
                population = (0..100).map(|_| factory.create(&mut rng)).collect();
                continue;
            }

            evals.sort_by(|a, b| {
                b.fitness()
                    .partial_cmp(&a.fitness())
                    .unwrap_or(Ordering::Equal)
            });
            let gen_best = evals[0].clone();

            if best_overall.is_none()
                || gen_best.fitness() > best_overall.as_ref().unwrap().fitness()
            {
                best_overall = Some(gen_best.clone());
            }

            // Selection & Next Gen
            let mut next_gen = Vec::with_capacity(100);
            next_gen.extend(evals.iter().take(5).map(|e| e.genome().clone()));

            while next_gen.len() < 100 {
                let p1 = tournament_selection(&evals, 3, &mut rng);
                let p2 = tournament_selection(&evals, 3, &mut rng);
                let mut c1 = p1.genome().clone();
                let mut c2 = p2.genome().clone();
                if rng.gen_bool(0.8) {
                    crossover.crossover(&mut c1, &mut c2, &mut rng);
                }
                mutator.mutate(&mut c1, &mut rng);
                mutator.mutate(&mut c2, &mut rng);
                next_gen.push(c1);
                if next_gen.len() < 100 {
                    next_gen.push(c2);
                }
            }
            population = next_gen;
        }

        let best = best_overall.unwrap();
        let next_hist = extract_next_history(&context, best.genome());

        if w == 3 {
            // Final metrics
            let score = best.soft_report.total_penalty;
            let hard = best.hc_coverage
                + best.hc_skills
                + best.hc_one_shift_per_day
                + best.hc_forbidden_successions;

            // We use next_hist to get total assignments across all 4 weeks + H0.
            let mut counts = vec![0; num_nurses];
            for n in 0..num_nurses {
                counts[n] = next_hist.nurse_history[n].number_of_assignments;
            }
            let mean = counts.iter().sum::<usize>() as f64 / num_nurses as f64;
            let cv = (counts
                .iter()
                .map(|&x| (x as f64 - mean).powi(2))
                .sum::<f64>()
                / num_nurses as f64)
                .sqrt()
                / mean;
            let gini = distribution_gini(&counts);

            let cov = compute_coverage_ratio(&context, best.genome());

            writeln!(
                out_csv,
                "{},{:?},{},{},{},{},{},{},{:.4},{:.4},{:.4}",
                seed,
                arm,
                score,
                hard,
                best.hc_coverage,
                best.hc_skills,
                best.hc_one_shift_per_day,
                best.hc_forbidden_successions,
                gini,
                cv,
                cov
            )
            .unwrap();
        }

        // Accumulate state for this week to use in future weeks
        let mut this_week = vec![0.0; num_nurses];
        for n in 0..num_nurses {
            this_week[n] = (next_hist.nurse_history[n].number_of_assignments
                - current_history.nurse_history[n].number_of_assignments)
                as f64;
        }
        week_history.push(this_week);
        current_history = next_hist;
    }
}

fn main() {
    let seeds = 1000..1030; // 30 seeds
    let arms = vec![
        DepthArm::Off,
        DepthArm::Depth1w,
        DepthArm::Depth2w,
        DepthArm::Depth4w,
        DepthArm::DepthFull,
    ];
    let output_file = "memory_depth_ablation_30seed.csv";

    let mut file = File::create(output_file).unwrap();
    writeln!(file, "seed,arm,score,hard,hc_coverage,hc_skills,hc_one_shift_per_day,hc_forbidden_successions,gini,cv,coverage").unwrap();

    println!("F.2D.8D Memory Depth Ablation");
    println!("  Output: {}", output_file);
    println!();

    for seed in seeds {
        for &arm in &arms {
            let start = Instant::now();
            run_ablation(seed, arm, &mut file);
            let elapsed = start.elapsed();
            println!(
                "  Seed {} Arm {:?} completed in {:.1}s",
                seed,
                arm,
                elapsed.as_secs_f64()
            );
        }
    }

    println!("\nAblation Matrix completed successfully.");
}
