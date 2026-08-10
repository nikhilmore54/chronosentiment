use clap::Parser;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use coralys_moga::traits::*;
use coralys_v2::{ContextKey, OpportunityMemory, AdvisoryCandidate, AdvisoryRanker};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use ultracrew::inrc::optimization::{
    InrcContext, InrcEvaluation, InrcGenome, InrcOptimizer,
};
use ultracrew::inrc::parser::{parse_history, parse_scenario, parse_week_data};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::workforce::WorkforceEcologyAdapter;
use ultracrew::inrc::history::extract_next_history;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    m26: bool,
    #[arg(long)]
    m27: bool,
    #[arg(long)]
    m30: bool,
    #[arg(long, default_value_t = 0)]
    time_limit_secs: u64,
    #[arg(long)]
    instance_prefix: String,
    #[arg(long, default_value_t = 0)]
    week: usize, // Keeping this for CLI compat but ignoring, will run `weeks`
    #[arg(long, default_value_t = 42)]
    seed: u64,
    #[arg(long, default_value_t = 4)]
    weeks: usize,
    #[arg(long, default_value_t = 125)]
    generations: usize,
    #[arg(long)]
    historical_m30: bool,
    #[arg(long)]
    oracle_mode: bool,
    #[arg(long)]
    freeze_memory_after_week: Option<usize>,
}

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

#[derive(Clone)]
struct BenchmarkGenomeFactory {
    num_nurses: usize,
    num_days: usize,
    num_shifts: usize,
    adapter: WorkforceEcologyAdapter,
    m26_enabled: bool,
    m27_enabled: bool,
    historical_m30: bool,
}

impl GenomeFactory<InrcGenome> for BenchmarkGenomeFactory {
    fn create(&self, rng: &mut StdRng) -> InrcGenome {
        let size = self.num_nurses * self.num_days * self.num_shifts;
        let mut bits = vec![false; size];
        
        if self.historical_m30 {
            for i in 0..size {
                if rng.gen_bool(0.22) {
                    bits[i] = true;
                }
            }
            return InrcGenome { bits };
        }
        
        let avg_assignments: f64 = (0..self.num_nurses).map(|n| self.adapter.get_assignments(n)).sum::<f64>() / self.num_nurses as f64;
        
        for n in 0..self.num_nurses {
            let base_prob: f64 = 0.22;
            
            let prob = if self.m27_enabled {
                let aggressive_prob = if avg_assignments > 0.0 {
                    let signal = self.adapter.compute_signal(n, self.num_nurses);
                    let bias = (1.0 + signal.pressure as f64).max(0.7_f64).min(1.3_f64);
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
                    let idx = n * (self.num_days * self.num_shifts) + d * self.num_shifts + shift_idx;
                    bits[idx] = true;
                }
            }
        }
        InrcGenome { bits }
    }
}

#[derive(Clone)]
struct BenchmarkMutator {
    adapter: WorkforceEcologyAdapter,
    num_nurses: usize,
    num_days: usize,
    num_shifts: usize,
    m26_enabled: bool,
    m27_enabled: bool,
    historical_m30: bool,
}

impl MutationOperator<InrcGenome> for BenchmarkMutator {
    fn mutate(&self, genome: &mut InrcGenome, rng: &mut StdRng) {
        if self.historical_m30 {
            let rate = 1.0 / (genome.bits.len() as f64).max(1.0);
            for i in 0..genome.bits.len() {
                if rng.gen_bool(rate) {
                    genome.bits[i] = !genome.bits[i];
                }
            }
            return;
        }
        if !self.m26_enabled && !self.m27_enabled {
            let rate = 1.0 / (genome.bits.len() as f64).max(1.0);
            for i in 0..genome.bits.len() {
                if rng.gen_bool(rate) {
                    genome.bits[i] = !genome.bits[i];
                }
            }
            return;
        }

        let rate = 1.0 / (genome.bits.len() as f64).max(1.0);
        
        let (mut loads, mut avg_assignments) = if self.m26_enabled && !self.m27_enabled {
            let mut l = vec![0.0; self.num_nurses];
            let mut total = 0.0;
            for n in 0..self.num_nurses {
                let mut count = 0;
                for d in 0..self.num_days {
                    for s in 0..self.num_shifts {
                        let idx = n * (self.num_days * self.num_shifts) + d * self.num_shifts + s;
                        if genome.bits[idx] { count += 1; }
                    }
                }
                l[n] = count as f64;
                total += count as f64;
            }
            (l, total / self.num_nurses as f64)
        } else {
            let mut l = vec![0.0; self.num_nurses];
            for n in 0..self.num_nurses {
                l[n] = self.adapter.get_assignments(n);
            }
            let sum: f64 = l.iter().sum();
            (l, sum / self.num_nurses as f64)
        };
        
        for i in 0..genome.bits.len() {
            if rng.gen_bool(rate) {
                let n = i / (self.num_days * self.num_shifts);
                
                if avg_assignments > 0.0 && rng.gen_bool(self.adapter.policy.alpha.min(1.0)) {
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
                        if self.m26_enabled && !self.m27_enabled {
                            if new_bit {
                                loads[n] += 1.0;
                                avg_assignments += 1.0 / self.num_nurses as f64;
                            } else {
                                loads[n] -= 1.0;
                                avg_assignments -= 1.0 / self.num_nurses as f64;
                            }
                        }
                    }
                } else {
                    let new_bit = !genome.bits[i];
                    genome.bits[i] = new_bit;
                    if self.m26_enabled && !self.m27_enabled {
                        if new_bit {
                            loads[n] += 1.0;
                            avg_assignments += 1.0 / self.num_nurses as f64;
                        } else {
                            loads[n] -= 1.0;
                            avg_assignments -= 1.0 / self.num_nurses as f64;
                        }
                    }
                }
            }
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
        false
    }
    
    fn fallback_cmp(&self, other: &Self) -> Ordering {
        self.eval.fitness().partial_cmp(&other.eval.fitness()).unwrap_or(Ordering::Equal)
    }
}

fn tournament_selection<'a>(evals: &'a [EvaluatedOffspring], k: usize, rng: &mut StdRng) -> &'a EvaluatedOffspring {
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

fn run_pass(args: &Args, initial_memory: Option<OpportunityMemory<InrcContextKey>>, record_oracle: bool) -> (String, HashSet<InrcContextKey>) {
    let mut oracle_set = HashSet::new();

        
    let base_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data").join(&args.instance_prefix);
        
    let sc_path = base_dir.join(format!("Sc-{}.json", args.instance_prefix));
    let scenario = parse_scenario(&sc_path).unwrap();
    let num_nurses = scenario.nurses.len();
    let num_shifts = scenario.shift_types.len();
    
    let mut adapter = WorkforceEcologyAdapter::new(num_nurses, 1.0);
    
    let h0_path = base_dir.join(format!("H0-{}-0.json", args.instance_prefix));
    let h0 = parse_history(&h0_path).unwrap();
    let mut current_history = h0.clone();
    
    let mut total_objective = 0.0;
    let mut overall_time_to_best = 0;
    
    let start_time = Instant::now();
    let max_duration = std::time::Duration::from_secs(if args.time_limit_secs > 0 { args.time_limit_secs } else { 86400 });
    let max_gen_per_week = args.generations;
    
    // We will share OpportunityMemory across weeks
    let mut memory = initial_memory.unwrap_or_else(|| OpportunityMemory::new(500.0, 0.0048));
    
    // Telemetry for Diversity Audit
    let mut unique_contexts = HashSet::new();
    let mut context_visits = HashMap::new();
    let mut parent_diversity = HashSet::new();
    
    let mut w3_soft = 0;
    let mut w3_hard = 0;
    let mut champion_count = 0;
    let mut total_evals = 0;
    let mut known_evals = 0;
    let mut tie_frequency = 0;
    let mut opportunity_utilization = 0;
    
    for w in 0..args.weeks {
        let wd_path = base_dir.join(format!("WD-{}-{}.json", args.instance_prefix, w));
        let week_data = parse_week_data(&wd_path).unwrap();
        
        let context = Arc::new(InrcContext::new(scenario.clone(), week_data, current_history.clone(), WorkforceEcology::new()));
        let evaluator = InrcOptimizer { context: context.clone() };
        
        let mut rng = StdRng::seed_from_u64(args.seed + w as u64);
        
        let factory = BenchmarkGenomeFactory {
            num_nurses, num_days: 7, num_shifts,
            adapter: adapter.clone(), m26_enabled: args.m26,
            m27_enabled: args.m27, historical_m30: args.historical_m30,
        };
        let mut mutator = BenchmarkMutator {
            num_nurses, num_days: 7, num_shifts,
            adapter: adapter.clone(), m26_enabled: args.m26, m27_enabled: args.m27, historical_m30: args.historical_m30,
        };
        
        let population_size = 100;
        
        let mut population: Vec<Offspring> = (0..population_size)
            .map(|_| Offspring {
                genome: factory.create(&mut rng),
                parent_ctx: None,
            })
            .collect();
            
        let mut global_best: Option<InrcEvaluation> = None;
        let mut week_best_objective = -f64::MAX;
        
        for gen in 0..max_gen_per_week {
            if start_time.elapsed() >= max_duration {
                break; // Hard budget stop
            }
            
            let mut evals: Vec<EvaluatedOffspring> = population.into_iter()
                .map(|off| EvaluatedOffspring {
                    eval: evaluator.evaluate(&off.genome, &coralys_moga::runtime::optimization::metric::MetricReport::default()),
                    parent_ctx: off.parent_ctx,
                })
                .filter(|e| e.eval.is_valid())
                .collect();
                
            if evals.is_empty() {
                population = (0..population_size).map(|_| Offspring {
                    genome: factory.create(&mut rng),
                    parent_ctx: None,
                }).collect();
                continue;
            }
            
            let memory_engage_gen = if args.historical_m30 { 100 } else { 20 };
            
            if args.m30 && gen >= memory_engage_gen { // Engage memory
                let default_ctx = InrcContextKey { hc_cov:0, hc_skills:0, hc_1shift:0, hc_forb:0, soft_pen_bucket:0};
                AdvisoryRanker::sort(&mut evals, &memory, &default_ctx);
                for e in evals.iter() {
                    if let Some(ctx) = &e.parent_ctx {
                        if memory.is_known(ctx) && memory.score(ctx) > memory.score(&default_ctx) {
                            opportunity_utilization += 1;
                        }
                    }
                }
            } else {
                evals.sort_by(|a, b| b.eval.fitness().partial_cmp(&a.eval.fitness()).unwrap_or(Ordering::Equal));
            }
            
            let gen_best = evals[0].eval.clone();
            let mut new_global_best = false;
            if global_best.is_none() || gen_best.fitness() > global_best.as_ref().unwrap().fitness() {
                if let Some(gb) = &global_best {
                    if (gen_best.fitness() - gb.fitness()).abs() < 1e-6 {
                        tie_frequency += 1;
                    }
                }
                global_best = Some(gen_best.clone());
                week_best_objective = gen_best.fitness();
                overall_time_to_best = w * max_gen_per_week + gen;
                new_global_best = true;
                champion_count += 1;
            }
            
            for (rank, e) in evals.iter().enumerate() {
                if let Some(ctx) = &e.parent_ctx {
                    if gen >= memory_engage_gen {
                        total_evals += 1;
                        unique_contexts.insert(ctx.clone());
                        *context_visits.entry(ctx.clone()).or_insert(0) += 1;
                        if memory.is_known(ctx) {
                            known_evals += 1;
                        }
                    }
                    if record_oracle && rank == 0 && new_global_best {
                        oracle_set.insert(ctx.clone());
                    }
                    
                    let mut should_record = true;
                    if let Some(freeze_week) = args.freeze_memory_after_week {
                        if w > freeze_week {
                            should_record = false;
                        }
                    }
                    if args.oracle_mode && !record_oracle {
                        should_record = false; // Freeze memory during oracle evaluation
                    }
                    
                    if should_record {
                        memory.record(ctx.clone(), rank == 0 && new_global_best);
                    }
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
                mutator.mutate(&mut c1, &mut rng);
                mutator.mutate(&mut c2, &mut rng);
                
                let ctx1 = InrcContextKey::from_eval(&p1.eval);
                parent_diversity.insert(ctx1.clone());
                next_gen.push(Offspring {
                    genome: c1,
                    parent_ctx: Some(ctx1),
                });
                if next_gen.len() < population_size {
                    let ctx2 = InrcContextKey::from_eval(&p2.eval);
                    parent_diversity.insert(ctx2.clone());
                    next_gen.push(Offspring {
                        genome: c2,
                        parent_ctx: Some(ctx2),
                    });
                }
            }
            population = next_gen;
        }
        
        let best_eval = global_best.unwrap();
        total_objective += best_eval.fitness();
        
        if w == 3 {
            w3_soft = best_eval.soft_report.total_penalty;
            w3_hard = best_eval.hc_coverage + best_eval.hc_skills + best_eval.hc_one_shift_per_day + best_eval.hc_forbidden_successions;
        }
        
        let next_hist = extract_next_history(&context, best_eval.genome());
        for n in 0..num_nurses {
            adapter.accumulate_assignments(n, next_hist.nurse_history[n].number_of_assignments - current_history.nurse_history[n].number_of_assignments);
            adapter.accumulate_weekends(n, next_hist.nurse_history[n].number_of_working_weekends - current_history.nurse_history[n].number_of_working_weekends);
        }
        current_history = next_hist;
    }
    
    let elapsed = start_time.elapsed().as_secs_f64();
    let known_ratio = if total_evals > 0 { known_evals as f64 / total_evals as f64 } else { 0.0 };
    
    // Calculate context entropy
    let mut context_entropy = 0.0;
    let total_visits: usize = context_visits.values().sum();
    if total_visits > 0 {
        for &visits in context_visits.values() {
            let p = visits as f64 / total_visits as f64;
            if p > 0.0 {
                context_entropy -= p * p.log2();
            }
        }
    }
    
    let result_json = format!("{{\"objective\": {}, \"time_to_best\": {}, \"elapsed\": {:.2}, \"w3_soft\": {}, \"w3_hard\": {}, \"champion_count\": {}, \"known_ratio\": {:.4}, \"tie_frequency\": {}, \"opportunity_utilization\": {}, \"unique_contexts\": {}, \"context_entropy\": {:.4}, \"parent_diversity\": {}}}", 
        total_objective, overall_time_to_best, elapsed, w3_soft, w3_hard, champion_count, known_ratio, tie_frequency, opportunity_utilization, unique_contexts.len(), context_entropy, parent_diversity.len());
        
    (result_json, oracle_set)
}

fn main() {
    let mut args = Args::parse();
    if args.oracle_mode {
        let orig_m30 = args.m30;
        args.m30 = false; // Run M27 without memory to collect oracle contexts
        let (_, oracle_contexts) = run_pass(&args, None, true);
        args.m30 = orig_m30;
        
        let mut oracle_memory = OpportunityMemory::new(500.0, 0.0048);
        for ctx in oracle_contexts {
            for _ in 0..100 {
                oracle_memory.record(ctx.clone(), true);
            }
        }
        let (res, _) = run_pass(&args, Some(oracle_memory), false);
        println!("{}", res);
    } else {
        let (res, _) = run_pass(&args, None, false);
        println!("{}", res);
    }
}

