use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;
use std::cmp::Ordering;

use coralys_moga::config::EvolutionConfig;
use coralys_moga::traits::{GenomeFactory, MutationOperator, CrossoverOperator, FitnessEvaluator, Evaluated, Genome};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use ultracrew::inrc::optimization::{
    InrcContext, InrcOptimizer, InrcGenome, InrcEvaluation
};
use ultracrew::inrc::parser::{parse_scenario, parse_history, parse_week_data};
use ultracrew::ecology::{WorkforceEcology, EcologyState, EcologyPolicy};
use ultracrew::inrc::history::extract_next_history;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    Off,
    WeekLocal,
    FullEcology,
}

// ── Metrics ────────────────────────────────────────────────────────────────

fn calculate_gini(values: &[usize]) -> f64 {
    if values.is_empty() { return 0.0; }
    let mut sorted = values.to_vec();
    sorted.sort();
    let n = sorted.len();
    let sum: usize = sorted.iter().sum();
    if sum == 0 { return 0.0; }
    
    let mut index_sum = 0.0;
    for (i, val) in sorted.iter().enumerate() {
        index_sum += (i as f64 + 1.0) * (*val as f64);
    }
    
    let n_f64 = n as f64;
    let sum_f64 = sum as f64;
    
    (2.0 * index_sum) / (n_f64 * sum_f64) - (n_f64 + 1.0) / n_f64
}

fn compute_coverage_ratio(
    context: &Arc<InrcContext>,
    genome: &InrcGenome,
) -> f64 {
    let num_nurses = context.num_nurses;
    let num_days = context.num_days;
    let num_shifts = context.shift_types.len();
    let days_map = vec!["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];

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
    ecology: EcologyState,
    policy: EcologyPolicy,
    arm: Arm,
}

impl GenomeFactory<InrcGenome> for EcologyGenomeFactory {
    fn create(&self, rng: &mut StdRng) -> InrcGenome {
        let size = self.num_nurses * self.num_days * self.num_shifts;
        let mut bits = vec![false; size];
        let avg_assignments = self.ecology.mean_assignments();
        
        for n in 0..self.num_nurses {
            let base_prob: f64 = 0.22;
            
            let prob = if self.arm == Arm::FullEcology {
                let aggressive_prob = if avg_assignments > 0.0 {
                    let load = self.ecology.cumulative_assignments[n] as f64;
                    let load_ratio = load / avg_assignments;
                    let bias = (2.0 - load_ratio).max(0.7).min(1.3);
                    (base_prob * bias).min(1.0)
                } else {
                    base_prob
                };
                self.policy.interpolate(base_prob, aggressive_prob)
            } else {
                base_prob
            };
            
            for d in 0..self.num_days {
                if rng.gen_bool(prob.max(0.0).min(1.0)) {
                    let shift_idx = rng.gen_range(0..self.num_shifts);
                    let idx = n * (self.num_days * self.num_shifts) + d * self.num_shifts + shift_idx;
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
    arm: Arm,
}

impl MutationOperator<InrcGenome> for EcologyMutator {
    fn mutate(&self, genome: &mut InrcGenome, rng: &mut StdRng) {
        if self.arm == Arm::Off {
            let rate = 1.0 / (genome.bits.len() as f64).max(1.0);
            for i in 0..genome.bits.len() {
                if rng.gen_bool(rate) {
                    genome.bits[i] = !genome.bits[i];
                }
            }
            return;
        }

        let rate = 1.0 / (genome.bits.len() as f64).max(1.0);
        
        let (mut nurse_loads, mut avg_assignments) = if self.arm == Arm::WeekLocal {
            let mut loads = vec![0.0; self.num_nurses];
            let mut total = 0.0;
            for n in 0..self.num_nurses {
                let mut count = 0;
                for d in 0..self.num_days {
                    for s in 0..self.num_shifts {
                        let idx = n * (self.num_days * self.num_shifts) + d * self.num_shifts + s;
                        if genome.bits[idx] {
                            count += 1;
                        }
                    }
                }
                loads[n] = count as f64;
                total += count as f64;
            }
            (loads, total / self.num_nurses as f64)
        } else {
            let mut loads = vec![0.0; self.num_nurses];
            for n in 0..self.num_nurses {
                loads[n] = self.ecology.cumulative_assignments[n] as f64;
            }
            (loads, self.ecology.mean_assignments())
        };
        
        for i in 0..genome.bits.len() {
            if rng.gen_bool(rate) {
                let n = i / (self.num_days * self.num_shifts);
                
                if avg_assignments > 0.0 && rng.gen_bool(self.policy.alpha.min(1.0)) {
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
                        if self.arm == Arm::WeekLocal {
                            if new_bit {
                                nurse_loads[n] += 1.0;
                                avg_assignments += 1.0 / self.num_nurses as f64;
                            } else {
                                nurse_loads[n] -= 1.0;
                                avg_assignments -= 1.0 / self.num_nurses as f64;
                            }
                        }
                    }
                } else {
                    let new_bit = !genome.bits[i];
                    genome.bits[i] = new_bit;
                    if self.arm == Arm::WeekLocal {
                        if new_bit {
                            nurse_loads[n] += 1.0;
                            avg_assignments += 1.0 / self.num_nurses as f64;
                        } else {
                            nurse_loads[n] -= 1.0;
                            avg_assignments -= 1.0 / self.num_nurses as f64;
                        }
                    }
                }
            }
        }
    }
}

// ── Custom GA Loop ────────────────────────────────────────────────────────

fn tournament_selection<'a>(evals: &'a [InrcEvaluation], k: usize, rng: &mut StdRng) -> &'a InrcEvaluation {
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

fn run_ablation(seed: u64, arm: Arm, regime: &str, out_csv: &mut File) {
    let base_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/n030w4");
    let scenario = parse_scenario(base_dir.join("Sc-n030w4.json")).unwrap();
    let num_nurses = scenario.nurses.len();
    let num_shifts = scenario.shift_types.len();
    
    let mut ecology_state = EcologyState::new(num_nurses);
    let policy = EcologyPolicy::new(1.0); // alpha is always 1.0 for ON arms
    
    let h0_filename = if regime == "skewed" {
        "H0-n030w4-0.json".to_string()
    } else {
        format!("H0-n030w4-0_{}.json", regime)
    };
    
    let h0 = parse_history(base_dir.join(&h0_filename)).unwrap();
    let mut current_history = h0.clone();
    
    for w in 0..4 {
        let wd_path = base_dir.join(format!("WD-n030w4-{}.json", w));
        let week_data = parse_week_data(wd_path).unwrap();
        
        let ecology = WorkforceEcology::new();
        let context = Arc::new(InrcContext::new(scenario.clone(), week_data, current_history.clone(), ecology.clone()));
        let evaluator = InrcOptimizer { context: context.clone() };

        let factory = EcologyGenomeFactory {
            num_nurses,
            num_days: 7,
            num_shifts,
            ecology: ecology_state.clone(),
            policy: policy.clone(),
            arm,
        };

        let mutator = EcologyMutator {
            ecology: ecology_state.clone(),
            policy: policy.clone(),
            num_nurses,
            num_days: 7,
            num_shifts,
            arm,
        };
        
        let crossover = InrcOptimizer { context: context.clone() };

        let mut rng = StdRng::seed_from_u64(seed + w as u64);
        let mut population = (0..100).map(|_| factory.create(&mut rng)).collect::<Vec<_>>();
        let mut best_overall: Option<InrcEvaluation> = None;
        
        for gen in 0..100 {
            let mut evals: Vec<InrcEvaluation> = population.iter()
                .map(|g| evaluator.evaluate(g))
                .filter(|e| e.is_valid())
                .collect();
            
            if evals.is_empty() {
                population = (0..100).map(|_| factory.create(&mut rng)).collect();
                continue;
            }
            
            evals.sort_by(|a, b| b.fitness().partial_cmp(&a.fitness()).unwrap_or(Ordering::Equal));
            let gen_best = evals[0].clone();
            
            if best_overall.is_none() || gen_best.fitness() > best_overall.as_ref().unwrap().fitness() {
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
            let hard = best.hc_coverage + best.hc_skills + best.hc_one_shift_per_day + best.hc_forbidden_successions;
            
            // We use next_hist to get total assignments across all 4 weeks + H0.
            let mut counts = vec![0; num_nurses];
            for n in 0..num_nurses {
                counts[n] = next_hist.nurse_history[n].number_of_assignments;
            }
            let mean = counts.iter().sum::<usize>() as f64 / num_nurses as f64;
            let cv = (counts.iter().map(|&x| (x as f64 - mean).powi(2)).sum::<f64>() / num_nurses as f64).sqrt() / mean;
            let gini = calculate_gini(&counts);
            
            let cov = compute_coverage_ratio(&context, best.genome());
            
            writeln!(out_csv, "{},{},{:?},{},{},{},{},{},{},{:.4},{:.4},{:.4}", 
                regime, seed, arm, score, hard, 
                best.hc_coverage, best.hc_skills, best.hc_one_shift_per_day, best.hc_forbidden_successions,
                gini, cv, cov).unwrap();
        }
        
        // Accumulate state
        for n in 0..num_nurses {
            ecology_state.cumulative_assignments[n] += next_hist.nurse_history[n].number_of_assignments - current_history.nurse_history[n].number_of_assignments;
            ecology_state.cumulative_weekends[n] += next_hist.nurse_history[n].number_of_working_weekends - current_history.nurse_history[n].number_of_working_weekends;
        }
        current_history = next_hist;
    }
}

fn main() {
    let seeds = 1000..1030; // 30 seeds
    let arms = vec![Arm::Off, Arm::FullEcology];
    let regimes = vec!["balanced", "skewed", "extreme"];
    let output_file = "history_test_n030w4.csv";
    
    let mut file = File::create(output_file).unwrap();
    writeln!(file, "regime,seed,arm,score,hard,hc_coverage,hc_skills,hc_one_shift_per_day,hc_forbidden_successions,gini,cv,coverage").unwrap();

    println!("F.2D.8B History Regimes Test");
    println!("  Output: {}", output_file);
    println!();

    for regime in &regimes {
        for seed in seeds.clone() {
            for &arm in &arms {
                let start = Instant::now();
                run_ablation(seed, arm, regime, &mut file);
                let elapsed = start.elapsed();
                println!("  Regime {} Seed {} Arm {:?} completed in {:.1}s", regime, seed, arm, elapsed.as_secs_f64());
            }
        }
    }
    
    println!("\nHistory Test completed successfully.");
}
