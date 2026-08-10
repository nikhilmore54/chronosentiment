use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;
use std::cmp::Ordering;

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use ultracrew::inrc::optimization::{
    InrcContext, InrcOptimizer, InrcGenome, InrcEvaluation
};
use ultracrew::inrc::parser::{parse_scenario, parse_history, parse_week_data};
use ultracrew::ecology::{WorkforceEcology};
use coralys_moga::ecology::distribution_gini;
use ultracrew::inrc::history::extract_next_history;
use coralys_moga::traits::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Baseline,
    Archive,
    SA,
    ArchiveSA,
    PureSA,
}

#[derive(Clone)]
struct EcologyGenomeFactory {
    num_nurses: usize,
    num_days: usize,
    num_shifts: usize,
}

impl EcologyGenomeFactory {
    fn create(&self, rng: &mut StdRng) -> InrcGenome {
        let size = self.num_nurses * self.num_days * self.num_shifts;
        let mut bits = vec![false; size];
        for i in 0..size {
            if rng.gen_bool(0.22) {
                bits[i] = true;
            }
        }
        InrcGenome { bits }
    }
}

#[derive(Clone)]
struct EcologyMutator {
    num_nurses: usize,
    num_days: usize,
    num_shifts: usize,
}

impl EcologyMutator {
    fn mutate(&self, genome: &mut InrcGenome, rng: &mut StdRng) {
        let rate = 1.0 / (genome.bits.len() as f64).max(1.0);
        for i in 0..genome.bits.len() {
            if rng.gen_bool(rate) {
                genome.bits[i] = !genome.bits[i];
            }
        }
    }
}

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

struct Family {
    id: usize,
    canonical_genome: InrcGenome,
    birth_gen: usize,
    last_seen_gen: usize,
    initial_score: f64,
    peak_score: f64,
    peak_gen: usize,
    was_refined_by_sa: bool,
    source: String,
}

struct AttributionStats {
    archive_pulls: usize,
    archive_pulls_to_new_best: usize,
    population_pulls: usize,
    population_pulls_to_new_best: usize,
    sa_counterfactual_gains: Vec<f64>,
    champion_lineage_depth: usize,
    improvements_by_offspring: usize,
    improvements_by_mutation: usize,
    improvements_by_sa: usize,
}

fn run_pilot(seed: u64, mode: Mode, out_csv: &mut File) {
    let base_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/n030w4");
    let mut scenario = parse_scenario(base_dir.join("Sc-n030w4.json")).unwrap();
    let num_nurses = scenario.nurses.len();
    let num_shifts = scenario.shift_types.len();
    
    let num_weeks = 52;
    let scaling = num_weeks / 4;
    scenario.number_of_weeks = num_weeks;
    for contract in &mut scenario.contracts {
        contract.min_assignments *= scaling;
        contract.max_assignments *= scaling;
        contract.max_working_weekends *= scaling;
    }
    
    let h0 = parse_history(base_dir.join("H0-n030w4-0.json")).unwrap();
    let mut current_history = h0.clone();
    for n in 0..num_nurses {
        current_history.nurse_history[n].number_of_assignments = 0;
        current_history.nurse_history[n].number_of_working_weekends = 0;
    }
    
    let base_week_data = parse_week_data(base_dir.join("WD-n030w4-0.json")).unwrap();
    let mut rng_env = StdRng::seed_from_u64(seed);
    
    let mut total_score = 0;
    let mut total_hard = 0;
    
    let mut families: Vec<Family> = Vec::new();
    let mut global_gen = 0;
    let mut stats = AttributionStats {
        archive_pulls: 0,
        archive_pulls_to_new_best: 0,
        population_pulls: 0,
        population_pulls_to_new_best: 0,
        sa_counterfactual_gains: Vec::new(),
        champion_lineage_depth: 0,
        improvements_by_offspring: 0,
        improvements_by_mutation: 0,
        improvements_by_sa: 0,
    };

    let get_score = |eval: &InrcEvaluation| -> f64 {
        // Higher is worse for score, but fitness is negative. We use positive score here.
        -eval.fitness()
    };

    let is_similar = |a: &InrcGenome, b: &InrcGenome| -> bool {
        a.bits.iter().zip(b.bits.iter()).filter(|(x, y)| x != y).count() < 10
    };

    for w in 0..num_weeks {
        let mut week_data = base_week_data.clone();
        
        if rng_env.gen_bool(0.2) {
            let sick_nurse_idx = rng_env.gen_range(0..scenario.nurses.len());
            let sick_nurse = &scenario.nurses[sick_nurse_idx];
            let days = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
            for day in days {
                for shift_type in &scenario.shift_types {
                    week_data.shift_off_requests.push(ultracrew::inrc::models::InrcShiftOffRequest {
                        nurse: sick_nurse.id.clone(),
                        shift_type: shift_type.id.clone(),
                        day: day.to_string(),
                    });
                }
            }
        }
        
        if rng_env.gen_bool(0.3) {
            let req_idx = rng_env.gen_range(0..week_data.requirements.len());
            week_data.requirements[req_idx].monday.optimal += 1;
            week_data.requirements[req_idx].monday.minimum += 1;
        }
        
        let context = Arc::new(InrcContext::new(scenario.clone(), week_data, current_history.clone(), WorkforceEcology::new()));
        let evaluator = InrcOptimizer { context: context.clone() };

        let factory = EcologyGenomeFactory {
            num_nurses,
            num_days: 7,
            num_shifts,
        };

        let mutator = EcologyMutator {
            num_nurses,
            num_days: 7,
            num_shifts,
        };
        
        let crossover = InrcOptimizer { context: context.clone() };

        let mut rng = StdRng::seed_from_u64(seed + w as u64);
        let mut best_overall: Option<InrcEvaluation> = None;
        
        if mode == Mode::PureSA {
            let mut current_cand = factory.create(&mut rng);
            use coralys_moga::traits::FitnessEvaluator;
            let mut current_eval = evaluator.evaluate(&current_cand, &coralys_moga::runtime::optimization::metric::MetricReport::default());
            let mut best_cand = current_cand.clone();
            let mut best_eval = current_eval.clone();
            
            let t_start = 1000.0;
            let sa_steps = 10000;
            let alpha = (0.01_f64 / t_start).powf(1.0 / sa_steps as f64);
            let mut t = t_start;
            
            for gen in 0..sa_steps {
                global_gen += 1;
                let mut neighbor = current_cand.clone();
                mutator.mutate(&mut neighbor, &mut rng);
                let neighbor_eval = evaluator.evaluate(&neighbor, &coralys_moga::runtime::optimization::metric::MetricReport::default());
                let delta = neighbor_eval.fitness() - current_eval.fitness();
                
                if delta > 0.0 || rng.gen_range(0.0..1.0) < (delta / t).exp() {
                    current_cand = neighbor;
                    current_eval = neighbor_eval;
                    if current_eval.fitness() > best_eval.fitness() {
                        let score_before = get_score(&best_eval);
                        best_eval = current_eval.clone();
                        best_cand = current_cand.clone();
                        let score_after = get_score(&best_eval);
                        stats.sa_counterfactual_gains.push(score_before - score_after);
                        stats.improvements_by_sa += 1;
                    }
                }
                t *= alpha;
                
                if gen % 100 == 0 {
                    let mut found = false;
                    for f in &mut families {
                        if is_similar(&best_cand, &f.canonical_genome) {
                            f.last_seen_gen = global_gen;
                            let score = get_score(&best_eval);
                            if score < f.peak_score { 
                                f.peak_score = score;
                                f.peak_gen = global_gen;
                            }
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        families.push(Family {
                            id: families.len(),
                            canonical_genome: best_cand.clone(),
                            birth_gen: global_gen,
                            last_seen_gen: global_gen,
                            initial_score: get_score(&best_eval),
                            peak_score: get_score(&best_eval),
                            peak_gen: global_gen,
                            was_refined_by_sa: true,
                            source: "sa".to_string(),
                        });
                    }
                }
            }
            best_overall = Some(best_eval);
        } else {
            let mut population = (0..100).map(|_| factory.create(&mut rng)).collect::<Vec<_>>();
            let mut passive_archive: Vec<InrcEvaluation> = Vec::new();
            
            use coralys_moga::traits::FitnessEvaluator;
            for gen in 0..100 {
                global_gen += 1;
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
                
                if best_overall.is_none() || gen_best.fitness() > best_overall.as_ref().unwrap().fitness() {
                    best_overall = Some(gen_best.clone());
                    stats.improvements_by_offspring += 1; // Simplification, hard to track exactly if mutation vs crossover won.
                    stats.champion_lineage_depth += 1;
                }
                
                let num_elites = (evals.len() / 5).max(1);
                for i in 0..num_elites {
                    let mut found = false;
                    for f in &mut families {
                        if is_similar(evals[i].genome(), &f.canonical_genome) {
                            f.last_seen_gen = global_gen;
                            let score = get_score(&evals[i]);
                            if score < f.peak_score { 
                                f.peak_score = score;
                                f.peak_gen = global_gen;
                            }
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        families.push(Family {
                            id: families.len(),
                            canonical_genome: evals[i].genome().clone(),
                            birth_gen: global_gen,
                            last_seen_gen: global_gen,
                            initial_score: get_score(&evals[i]),
                            peak_score: get_score(&evals[i]),
                            peak_gen: global_gen,
                            was_refined_by_sa: false,
                            source: "ga".to_string(),
                        });
                    }
                }
                
                let mut next_gen = Vec::with_capacity(100);
                next_gen.extend(evals.iter().take(5).map(|e| e.genome().clone()));
                
                use coralys_moga::traits::CrossoverOperator;
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
                
                if mode == Mode::Archive || mode == Mode::ArchiveSA {
                    for eval in &evals {
                        let mut novel = true;
                        for cand in &passive_archive {
                            let diff = eval.genome().bits.iter().zip(cand.genome().bits.iter()).filter(|(a, b)| a != b).count();
                            if diff < 5 {
                                novel = false;
                                break;
                            }
                        }
                        if novel {
                            if passive_archive.len() < 50 {
                                passive_archive.push(eval.clone());
                            } else {
                                let mut worst_idx = 0;
                                let mut worst_fit = passive_archive[0].fitness();
                                for i in 1..passive_archive.len() {
                                    if passive_archive[i].fitness() < worst_fit {
                                        worst_fit = passive_archive[i].fitness();
                                        worst_idx = i;
                                    }
                                }
                                if eval.fitness() > worst_fit {
                                    passive_archive[worst_idx] = eval.clone();
                                }
                            }
                        }
                    }
                }
                
                if gen % 10 == 0 {
                    use rand::seq::SliceRandom;
                    let mut samples = Vec::new();
                    let mut from_archive = false;
                    
                    if (mode == Mode::Archive || mode == Mode::ArchiveSA) && !passive_archive.is_empty() {
                        let num_samples = 3.min(passive_archive.len());
                        samples = passive_archive.choose_multiple(&mut rng, num_samples).cloned().collect();
                        from_archive = true;
                        stats.archive_pulls += num_samples;
                    } else if mode == Mode::SA {
                        let num_samples = 3.min(evals.len());
                        samples = evals.choose_multiple(&mut rng, num_samples).cloned().collect();
                        stats.population_pulls += num_samples;
                    }
                    
                    if mode == Mode::SA || mode == Mode::ArchiveSA {
                        for sample in &mut samples {
                            let score_before_sa = get_score(sample);
                            let mut current_cand = sample.genome().clone();
                            let mut current_fit = sample.fitness();
                            let mut best_cand = current_cand.clone();
                            let mut best_fit = current_fit;
                            
                            let t_start = 500.0;
                            let sa_steps = 250;
                            let alpha = (0.01_f64 / t_start).powf(1.0 / sa_steps as f64);
                            let mut t = t_start;
                            
                            for _ in 0..sa_steps {
                                let mut neighbor = current_cand.clone();
                                mutator.mutate(&mut neighbor, &mut rng);
                                let neighbor_eval = evaluator.evaluate(&neighbor, &coralys_moga::runtime::optimization::metric::MetricReport::default());
                                let new_fit = neighbor_eval.fitness();
                                
                                let delta = new_fit - current_fit;
                                if delta > 0.0 || rng.gen_range(0.0..1.0) < (delta / t).exp() {
                                    current_cand = neighbor;
                                    current_fit = new_fit;
                                    if current_fit > best_fit {
                                        best_fit = current_fit;
                                        best_cand = current_cand.clone();
                                    }
                                }
                                t *= alpha;
                            }
                            
                            let score_after_sa = -best_fit;
                            if best_fit > sample.fitness() {
                                *sample = evaluator.evaluate(&best_cand, &coralys_moga::runtime::optimization::metric::MetricReport::default());
                                
                                if best_fit > best_overall.as_ref().unwrap().fitness() {
                                    best_overall = Some(sample.clone());
                                    stats.improvements_by_sa += 1;
                                    stats.champion_lineage_depth += 1;
                                    stats.sa_counterfactual_gains.push(score_before_sa - score_after_sa);
                                    
                                    if from_archive {
                                        stats.archive_pulls_to_new_best += 1;
                                    } else {
                                        stats.population_pulls_to_new_best += 1;
                                    }
                                }
                                
                                for f in &mut families {
                                    if is_similar(&best_cand, &f.canonical_genome) {
                                        f.was_refined_by_sa = true;
                                        if score_after_sa < f.peak_score {
                                            f.peak_score = score_after_sa;
                                            f.peak_gen = global_gen;
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    
                    for sample in samples {
                        next_gen.pop();
                        next_gen.push(sample.genome().clone());
                    }
                }
                population = next_gen;
            }
        }
        
        let best = best_overall.unwrap();
        let next_hist = extract_next_history(&context, best.genome());
        
        total_score += best.soft_report.total_penalty;
        total_hard += best.hc_coverage + best.hc_skills + best.hc_one_shift_per_day + best.hc_forbidden_successions;
        
        if w == num_weeks - 1 {
            let mut counts = vec![0; num_nurses];
            for n in 0..num_nurses {
                counts[n] = next_hist.nurse_history[n].number_of_assignments;
            }
            let gini = distribution_gini(&counts);
            
            let mut lifetimes: Vec<_> = families.iter().map(|f| f.last_seen_gen - f.birth_gen).collect();
            lifetimes.sort_unstable();
            let median_lt = if lifetimes.is_empty() { 0 } else { lifetimes[lifetimes.len() / 2] };
            let p90_lt = if lifetimes.is_empty() { 0 } else { lifetimes[(lifetimes.len() as f64 * 0.9) as usize] };
            let max_lt = if lifetimes.is_empty() { 0 } else { *lifetimes.last().unwrap() };
            let discovery_rate = families.len();
            
            let mut delayed_payoff_count = 0;
            let mut total_time_to_peak = 0;
            let mut sa_conversions = 0;
            
            for f in &families {
                if f.peak_gen > f.birth_gen + 50 {
                    delayed_payoff_count += 1;
                }
                total_time_to_peak += f.peak_gen - f.birth_gen;
                if f.was_refined_by_sa {
                    sa_conversions += 1;
                }
            }
            
            let delayed_payoff_ratio = if discovery_rate > 0 { delayed_payoff_count as f64 / discovery_rate as f64 } else { 0.0 };
            let avg_time_to_peak = if discovery_rate > 0 { total_time_to_peak as f64 / discovery_rate as f64 } else { 0.0 };
            let sa_conversion_rate = if discovery_rate > 0 { sa_conversions as f64 / discovery_rate as f64 } else { 0.0 };
            
            let mut sa_gains = stats.sa_counterfactual_gains.clone();
            sa_gains.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
            let median_sa_gain = if sa_gains.is_empty() { 0.0 } else { sa_gains[sa_gains.len() / 2] };
            let p95_sa_gain = if sa_gains.is_empty() { 0.0 } else { sa_gains[(sa_gains.len() as f64 * 0.95) as usize] };

            writeln!(out_csv, "{},{:?},{},{},{:.4},{},{},{},{},{:.4},{:.1},{:.4},{:.1},{:.1},{},{},{},{},{},{}", 
                seed, mode, total_score, total_hard, gini, discovery_rate, median_lt, p90_lt, max_lt,
                delayed_payoff_ratio, avg_time_to_peak, sa_conversion_rate, median_sa_gain, p95_sa_gain,
                stats.improvements_by_offspring, stats.improvements_by_sa, stats.champion_lineage_depth,
                stats.archive_pulls, stats.archive_pulls_to_new_best, stats.population_pulls_to_new_best).unwrap();
        }
        
        current_history = next_hist;
    }
}

fn main() {
    let seeds = 2000..2010; // 10 seeds
    let modes = vec![Mode::Baseline, Mode::Archive, Mode::SA, Mode::ArchiveSA, Mode::PureSA];
    let output_file = "inrc_m22f1_deep_attribution.csv";
    
    let mut file = File::create(output_file).unwrap();
    writeln!(file, "seed,mode,soft_penalty,hard_penalty,gini,discovery,median_lifetime,p90_lifetime,max_lifetime,delayed_payoff_ratio,avg_time_to_peak,sa_conversion_rate,median_sa_gain,p95_sa_gain,imp_offspring,imp_sa,lineage_depth,archive_pulls,archive_success,pop_success").unwrap();

    println!("M22F-1.1 INRC-II Contribution Attribution (10 seeds, 52 weeks)");
    println!("  Output: {}", output_file);
    println!();

    for seed in seeds {
        for &mode in &modes {
            let start = Instant::now();
            run_pilot(seed, mode, &mut file);
            let elapsed = start.elapsed();
            println!("  Seed {} Mode {:?} completed in {:.1}s", seed, mode, elapsed.as_secs_f64());
        }
    }
    
    println!("\nM22F-1.1 Benchmark completed successfully.");
}
