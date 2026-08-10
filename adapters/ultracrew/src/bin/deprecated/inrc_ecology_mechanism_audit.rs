use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::cell::Cell;
use std::rc::Rc;
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

// ── Telemetry Functions ───────────────────────────────────────────────────

fn mean_pairwise_hamming(population: &[InrcGenome]) -> f64 {
    if population.len() < 2 { return 0.0; }
    let mut sum = 0;
    let mut count = 0;
    for i in 0..population.len() {
        for j in (i+1)..population.len() {
            let b1 = &population[i].bits;
            let b2 = &population[j].bits;
            let dist = b1.iter().zip(b2.iter()).filter(|(x, y)| x != y).count();
            sum += dist;
            count += 1;
        }
    }
    sum as f64 / count as f64
}

fn genome_total_assignments(
    genome: &InrcGenome, 
    ecology_state: &EcologyState, 
    num_nurses: usize, 
    num_days: usize, 
    num_shifts: usize
) -> Vec<usize> {
    let mut counts = ecology_state.cumulative_assignments.clone();
    for n in 0..num_nurses {
        for d in 0..num_days {
            for s in 0..num_shifts {
                let idx = n * (num_days * num_shifts) + d * num_shifts + s;
                if genome.bits[idx] {
                    counts[n] += 1;
                }
            }
        }
    }
    counts
}

fn pop_mean_assignment_variance(population: &[InrcGenome], ecology_state: &EcologyState, num_nurses: usize, num_days: usize, num_shifts: usize) -> f64 {
    let mut sum_var = 0.0;
    for g in population {
        let counts = genome_total_assignments(g, ecology_state, num_nurses, num_days, num_shifts);
        let mean = counts.iter().sum::<usize>() as f64 / num_nurses as f64;
        let var = counts.iter().map(|&x| (x as f64 - mean).powi(2)).sum::<f64>() / num_nurses as f64;
        sum_var += var;
    }
    sum_var / population.len() as f64
}

fn pop_mean_gini(population: &[InrcGenome], ecology_state: &EcologyState, num_nurses: usize, num_days: usize, num_shifts: usize) -> f64 {
    let mut sum_gini = 0.0;
    for g in population {
        let counts = genome_total_assignments(g, ecology_state, num_nurses, num_days, num_shifts);
        sum_gini += calculate_gini(&counts);
    }
    sum_gini / population.len() as f64
}

fn pop_mean_coverage_ratio(population: &[InrcGenome], context: &Arc<InrcContext>) -> f64 {
    let mut sum = 0.0;
    for g in population {
        sum += compute_coverage_ratio(context, g);
    }
    sum / population.len() as f64
}

// ── GA Components ─────────────────────────────────────────────────────────

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
            let base_prob: f64 = 0.22;
            
            let aggressive_prob = if avg_assignments > 0.0 {
                let load = self.ecology.cumulative_assignments[n] as f64;
                let load_ratio = load / avg_assignments;
                let bias = (2.0 - load_ratio).max(0.7).min(1.3);
                (base_prob * bias).min(1.0)
            } else {
                base_prob
            };
            
            let prob = self.policy.interpolate(base_prob, aggressive_prob);
            
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
    mutation_attempts: Rc<Cell<usize>>,
    ecology_branch_entries: Rc<Cell<usize>>,
    ecology_branch_changed_bit: Rc<Cell<usize>>,
}

impl MutationOperator<InrcGenome> for EcologyMutator {
    fn mutate(&self, genome: &mut InrcGenome, rng: &mut StdRng) {
        let rate = 1.0 / (genome.bits.len() as f64).max(1.0);
        let avg_assignments = self.ecology.mean_assignments();
        
        for i in 0..genome.bits.len() {
            if rng.gen_bool(rate) {
                self.mutation_attempts.set(self.mutation_attempts.get() + 1);
                
                let n = i / (self.num_days * self.num_shifts);
                
                if self.policy.alpha > 0.0
                    && avg_assignments > 0.0
                    && rng.gen_bool(self.policy.alpha.min(1.0))
                {
                    self.ecology_branch_entries.set(self.ecology_branch_entries.get() + 1);
                    
                    let load = self.ecology.cumulative_assignments[n] as f64;
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
                    
                    // Effective ecology rate: ecology branch altered search trajectory
                    // compared to the neutral strategy (which ALWAYS flips the bit).
                    if new_bit != !is_working {
                        self.ecology_branch_changed_bit.set(self.ecology_branch_changed_bit.get() + 1);
                    }
                    genome.bits[i] = new_bit;
                } else {
                    genome.bits[i] = !genome.bits[i];
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

fn run_audit(seed: u64, alpha: f64, out_csv: &mut File) {
    let base_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/n030w4");
    let scenario = parse_scenario(base_dir.join("Sc-n030w4.json")).unwrap();
    let num_nurses = scenario.nurses.len();
    let num_shifts = scenario.shift_types.len();
    
    let mut ecology_state = EcologyState::new(num_nurses);
    let policy = EcologyPolicy::new(alpha);
    
    let h0 = parse_history(base_dir.join("H0-n030w4-0.json")).unwrap();
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
        };

        let mutation_attempts = Rc::new(Cell::new(0usize));
        let ecology_branch_entries = Rc::new(Cell::new(0usize));
        let ecology_branch_changed_bit = Rc::new(Cell::new(0usize));

        let mutator = EcologyMutator {
            ecology: ecology_state.clone(),
            policy: policy.clone(),
            num_nurses,
            num_days: 7,
            num_shifts,
            mutation_attempts: mutation_attempts.clone(),
            ecology_branch_entries: ecology_branch_entries.clone(),
            ecology_branch_changed_bit: ecology_branch_changed_bit.clone(),
        };
        
        let crossover = InrcOptimizer { context: context.clone() };

        let mut rng = StdRng::seed_from_u64(seed + w as u64);
        let mut population = (0..100).map(|_| factory.create(&mut rng)).collect::<Vec<_>>();
        let mut best_overall: Option<InrcEvaluation> = None;
        
        for gen in 0..100 {
            let mut evals: Vec<InrcEvaluation> = population.iter()
                .map(|g| evaluator.evaluate(g, &coralys_moga::runtime::optimization::metric::MetricReport::default()))
                .filter(|e| e.is_valid())
                .collect();
            
            if evals.is_empty() {
                population = (0..100).map(|_| factory.create(&mut rng)).collect();
                continue;
            }
            
            evals.sort_by(|a, b| b.fitness().partial_cmp(&a.fitness()).unwrap_or(Ordering::Equal));
            let gen_best = evals[0].clone();
            
            // Telemetry calculations
            let best_score = gen_best.soft_report.total_penalty;
            let mean_score = evals.iter().map(|e| e.soft_report.total_penalty as f64).sum::<f64>() / evals.len() as f64;
            let score_variance = evals.iter().map(|e| (e.soft_report.total_penalty as f64 - mean_score).powi(2)).sum::<f64>() / evals.len() as f64;
            
            let current_pop: Vec<InrcGenome> = evals.iter().map(|e| e.genome().clone()).collect();
            
            let mean_assignment_variance = pop_mean_assignment_variance(&current_pop, &ecology_state, num_nurses, 7, num_shifts);
            let mean_gini = pop_mean_gini(&current_pop, &ecology_state, num_nurses, 7, num_shifts);
            let hamming = mean_pairwise_hamming(&current_pop);
            
            let mean_cov = pop_mean_coverage_ratio(&current_pop, &context);
            let best_cov = compute_coverage_ratio(&context, gen_best.genome());
            
            let attempts = mutator.mutation_attempts.get();
            let entries = mutator.ecology_branch_entries.get();
            let changed = mutator.ecology_branch_changed_bit.get();
            
            let branch_rate = if attempts > 0 { entries as f64 / attempts as f64 } else { 0.0 };
            let effective_rate = if attempts > 0 { changed as f64 / attempts as f64 } else { 0.0 };
            
            // Write to CSV
            writeln!(out_csv, "{},{:.2},{},{},{},{:.2},{:.2},{:.4},{:.4},{:.4},{:.4},{:.4},{},{},{},{:.4},{:.4}",
                seed, alpha, w, gen,
                best_score, mean_score, score_variance,
                mean_assignment_variance, mean_gini,
                hamming, mean_cov, best_cov,
                attempts, entries, changed, branch_rate, effective_rate
            ).unwrap();
            
            // Reset counters for next gen
            mutator.mutation_attempts.set(0);
            mutator.ecology_branch_entries.set(0);
            mutator.ecology_branch_changed_bit.set(0);
            
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
        for n in 0..num_nurses {
            ecology_state.cumulative_assignments[n] += next_hist.nurse_history[n].number_of_assignments - current_history.nurse_history[n].number_of_assignments;
            ecology_state.cumulative_weekends[n] += next_hist.nurse_history[n].number_of_working_weekends - current_history.nurse_history[n].number_of_working_weekends;
        }
        current_history = next_hist;
    }
}

fn main() {
    let seed = 12346;
    let alphas = vec![0.0, 0.4, 1.0];
    let output_file = "mechanism_audit_seed12346.csv";
    
    let mut file = File::create(output_file).unwrap();
    writeln!(file, "seed,alpha,week,generation,best_score,mean_score,score_variance,mean_assignment_variance,mean_gini,mean_pairwise_hamming_distance,mean_coverage_ratio,best_coverage_ratio,mutation_attempts,ecology_branch_entries,ecology_branch_changed_bit,branch_rate,effective_ecology_rate").unwrap();

    println!("F.2D.2 Mechanism Audit");
    println!("  Seed: {}", seed);
    println!("  Alphas: {:?}", alphas);
    println!("  Output: {}", output_file);
    println!();

    for &alpha in &alphas {
        let start = Instant::now();
        run_audit(seed, alpha, &mut file);
        let elapsed = start.elapsed();
        println!("  Completed alpha={:.2} in {:.1}s", alpha, elapsed.as_secs_f64());
    }
    
    println!("\nMechanism Audit completed successfully.");
}
