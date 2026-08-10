use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;
use std::cmp::Ordering;
use std::collections::HashMap;

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

fn tournament_selection<'a>(evals: &'a [(usize, InrcEvaluation)], k: usize, rng: &mut StdRng) -> &'a (usize, InrcEvaluation) {
    let mut best: Option<&'a (usize, InrcEvaluation)> = None;
    for _ in 0..k {
        let idx = rng.gen_range(0..evals.len());
        let eval = &evals[idx];
        if best.is_none() || eval.1.fitness() > best.unwrap().1.fitness() {
            best = Some(eval);
        }
    }
    best.unwrap()
}

#[derive(Clone, Debug)]
struct GenomeMetadata {
    id: usize,
    birth_gen: usize,
    parent1: Option<usize>,
    parent2: Option<usize>,
    method: String,
    score: f64,
    rank_at_birth: usize,
}

fn get_score(eval: &InrcEvaluation) -> f64 {
    -eval.fitness()
}

fn run_pilot(seed: u64, out_csv: &mut File) {
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
    
    let mut global_gen = 0;
    let mut next_id = 1;
    
    let mut lineage_tree: HashMap<usize, GenomeMetadata> = HashMap::new();
    let mut best_overall_id: Option<usize> = None;
    let mut best_overall_eval: Option<InrcEvaluation> = None;

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

        let factory = EcologyGenomeFactory { num_nurses, num_days: 7, num_shifts };
        let mutator = EcologyMutator { num_nurses, num_days: 7, num_shifts };
        let crossover = InrcOptimizer { context: context.clone() };

        let mut rng = StdRng::seed_from_u64(seed + w as u64);
        
        let mut population = Vec::new();
        for _ in 0..100 {
            let cand = factory.create(&mut rng);
            let id = next_id;
            next_id += 1;
            population.push((id, cand));
        }
        
        for gen in 0..100 {
            global_gen += 1;
            
            let mut evals: Vec<(usize, InrcEvaluation)> = population.into_iter()
                .map(|(id, g)| (id, evaluator.evaluate(&g, &coralys_moga::runtime::optimization::metric::MetricReport::default())))
                .filter(|(_, e)| e.is_valid())
                .collect();
            
            if evals.is_empty() {
                population = Vec::new();
                for _ in 0..100 {
                    let cand = factory.create(&mut rng);
                    let id = next_id;
                    next_id += 1;
                    population.push((id, cand));
                }
                continue;
            }
            
            evals.sort_by(|a, b| b.1.fitness().partial_cmp(&a.1.fitness()).unwrap_or(Ordering::Equal));
            
            for (rank, (id, eval)) in evals.iter().enumerate() {
                if !lineage_tree.contains_key(id) {
                    lineage_tree.insert(*id, GenomeMetadata {
                        id: *id,
                        birth_gen: global_gen,
                        parent1: None,
                        parent2: None,
                        method: "random".to_string(),
                        score: get_score(eval),
                        rank_at_birth: rank + 1,
                    });
                } else {
                    if let Some(meta) = lineage_tree.get_mut(id) {
                        meta.rank_at_birth = rank + 1; // update rank if surviving
                    }
                }
            }
            
            let (gen_best_id, gen_best) = &evals[0];
            
            if best_overall_eval.is_none() || gen_best.fitness() > best_overall_eval.as_ref().unwrap().fitness() {
                best_overall_eval = Some(gen_best.clone());
                best_overall_id = Some(*gen_best_id);
            }
            
            let mut next_gen = Vec::with_capacity(100);
            next_gen.extend(evals.iter().take(5).map(|(id, e)| (*id, e.genome().clone())));
            
            while next_gen.len() < 100 {
                let p1 = tournament_selection(&evals, 3, &mut rng);
                let p2 = tournament_selection(&evals, 3, &mut rng);
                
                let mut c1 = p1.1.genome().clone();
                let mut c2 = p2.1.genome().clone();
                let mut c1_method = "mutation".to_string();
                let mut c2_method = "mutation".to_string();
                
                let mut p2_id = None;
                if rng.gen_bool(0.8) {
                    crossover.crossover(&mut c1, &mut c2, &mut rng);
                    c1_method = "crossover".to_string();
                    c2_method = "crossover".to_string();
                    p2_id = Some(p2.0);
                }
                
                mutator.mutate(&mut c1, &mut rng);
                mutator.mutate(&mut c2, &mut rng);
                
                let c1_id = next_id; next_id += 1;
                let c2_id = next_id; next_id += 1;
                
                lineage_tree.insert(c1_id, GenomeMetadata {
                    id: c1_id, birth_gen: global_gen, parent1: Some(p1.0), parent2: p2_id, method: c1_method, score: 9999999.0, rank_at_birth: 100
                });
                next_gen.push((c1_id, c1));
                
                if next_gen.len() < 100 {
                    lineage_tree.insert(c2_id, GenomeMetadata {
                        id: c2_id, birth_gen: global_gen, parent1: Some(p2.0), parent2: Some(p1.0), method: c2_method, score: 9999999.0, rank_at_birth: 100
                    });
                    next_gen.push((c2_id, c2));
                }
            }
            
            if global_gen % 10 == 0 {
                use rand::seq::SliceRandom;
                let num_samples = 3.min(evals.len());
                let samples: Vec<_> = evals.choose_multiple(&mut rng, num_samples).cloned().collect();
                
                for (sample_id, sample) in samples {
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
                    
                    if best_fit > sample.fitness() {
                        let new_id = next_id; next_id += 1;
                        let new_eval = evaluator.evaluate(&best_cand, &coralys_moga::runtime::optimization::metric::MetricReport::default());
                        let score = get_score(&new_eval);
                        
                        lineage_tree.insert(new_id, GenomeMetadata {
                            id: new_id, birth_gen: global_gen, parent1: Some(sample_id), parent2: None, method: "sa".to_string(), score, rank_at_birth: 1
                        });
                        
                        if best_fit > best_overall_eval.as_ref().unwrap().fitness() {
                            best_overall_eval = Some(new_eval.clone());
                            best_overall_id = Some(new_id);
                        }
                        
                        next_gen.pop();
                        next_gen.push((new_id, best_cand));
                    } else {
                        next_gen.pop();
                        next_gen.push((sample_id, sample.genome().clone()));
                    }
                }
            }
            population = next_gen;
        }
        
        let best = best_overall_eval.as_ref().unwrap();
        let next_hist = extract_next_history(&context, best.genome());
        
        total_score += best.soft_report.total_penalty;
        total_hard += best.hc_coverage + best.hc_skills + best.hc_one_shift_per_day + best.hc_forbidden_successions;
        current_history = next_hist;
    }
    
    // RECONSTRUCT ANCESTRY
    let mut lineage = Vec::new();
    let mut curr_id = best_overall_id.unwrap();
    
    while let Some(meta) = lineage_tree.get(&curr_id) {
        lineage.push(meta.clone());
        if let Some(p1) = meta.parent1 {
            curr_id = p1;
        } else {
            break;
        }
    }
    
    lineage.reverse(); // Now ordered from Root to Champion
    
    let root = &lineage[0];
    let champion = &lineage.last().unwrap();
    
    let time_to_peak = champion.birth_gen - root.birth_gen;
    let lineage_depth = lineage.len();
    
    let mut sa_interventions = 0;
    let mut ga_interventions = 0;
    let mut near_extinction_events = 0;
    
    let mut dormancy_period = time_to_peak;
    let initial_score = root.score;
    let mut meaningful_improvement_found = false;
    
    for (i, node) in lineage.iter().enumerate() {
        if node.method == "sa" { sa_interventions += 1; }
        if node.method == "mutation" || node.method == "crossover" { ga_interventions += 1; }
        
        // Elite threshold is top 20 (since population is 100)
        if node.rank_at_birth > 20 {
            near_extinction_events += 1;
        }
        
        if !meaningful_improvement_found && i > 0 {
            // Meaningful improvement = 10% better than root
            if node.score < initial_score * 0.90 {
                dormancy_period = node.birth_gen - root.birth_gen;
                meaningful_improvement_found = true;
            }
        }
    }
    
    let counterfactual_removal = if root.rank_at_birth > 20 { 1 } else { 0 };

    writeln!(out_csv, "{},{},{},{},{},{},{},{},{},{},{:.1},{:.1}", 
        seed, total_score, total_hard, root.birth_gen, time_to_peak, dormancy_period, 
        near_extinction_events, counterfactual_removal, lineage_depth, sa_interventions, 
        initial_score, champion.score).unwrap();
}

fn main() {
    let seeds = 2000..2010; // 10 seeds
    let output_file = "inrc_m22f1_2_ancestry.csv";
    
    let mut file = File::create(output_file).unwrap();
    writeln!(file, "seed,score,hard_penalty,discovery_gen,time_to_peak,dormancy,near_extinction,counterfactual_removal,lineage_depth,sa_interventions,initial_score,final_score").unwrap();

    println!("M22F-1.2 Champion Ancestry Reconstruction (10 seeds, MOGA+SA)");
    println!("  Output: {}", output_file);
    println!();

    for seed in seeds {
        let start = Instant::now();
        run_pilot(seed, &mut file);
        let elapsed = start.elapsed();
        println!("  Seed {} completed in {:.1}s", seed, elapsed.as_secs_f64());
    }
    
    println!("\nM22F-1.2 Ancestry Reconstruction completed successfully.");
}
