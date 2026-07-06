use ultracrew::inrc::models::InrcScenario;
use ultracrew::inrc::parser::{parse_scenario, parse_week_data};
use ultracrew_server::simulation::generate_baseline_schedule;
use ultracrew_server::optimizer::{UltraCrewEvaluator, UltraCrewMutator};
use coralys_moga::engine_proof::{EvolutionEngine, Evaluator};
use std::fs::File;
use std::io::Write;
use rand::Rng;
use rand::distributions::{WeightedIndex, Distribution};
use std::collections::HashMap;

#[derive(Clone, Default)]
struct EpochStats {
    selections: u64,
    total_yield: f64,
    admissions: u64,
}

#[derive(Clone)]
struct ParentStats {
    insertion_epoch: usize,
    epochs: HashMap<usize, EpochStats>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let instance = if args.len() > 1 { &args[1] } else { "n050w4" };
    let sa_mode = "memetic"; // Enforce memetic
    
    let base_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../adapters/ultracrew/tests/data/{}", instance));
    let scenario = parse_scenario(base_dir.join(format!("Sc-{}.json", instance))).unwrap();
    let week_data = parse_week_data(base_dir.join(format!("WD-{}-0.json", instance))).unwrap();
    
    println!("Starting Sprint 3.5C Persistence 25k Sweep for Scenario {} with Mode: {}", instance, sa_mode);
    
    let baseline_genome = generate_baseline_schedule(&scenario, &week_data.requirements).unwrap();
    let evaluator = UltraCrewEvaluator { scenario: scenario.clone() };
    
    let mutator = UltraCrewMutator::new(scenario.clone());
    let mut engine = EvolutionEngine::new(evaluator, mutator);
    let fitness = engine.evaluator.evaluate(&baseline_genome);
    let mut next_uid = 2;
    engine.archive.add(coralys_moga::engine_proof::ParetoSolution {
        genome: baseline_genome,
        fitness,
        uid: 1,
        parent_uid: 0,
    });
    
    let generations = 25000;
    
    let mut parent_tracker: HashMap<u64, ParentStats> = HashMap::new();
    parent_tracker.insert(1, ParentStats {
        insertion_epoch: 1,
        epochs: HashMap::new(),
    });
    
    for g in 1..=generations {
        let current_epoch = (g - 1) / 1000 + 1;
        let archive_size = engine.archive.solutions.len();
        if archive_size == 0 { break; }
        
        let mut idx = 0;
        let mut min_vals = vec![f64::INFINITY; 5];
        let mut max_vals = vec![0.0_f64; 5];
        let mut ranges = vec![1e-9; 5];
        
        for d in 0..5 {
            let vals: Vec<f64> = engine.archive.solutions.iter().map(|s| s.fitness[d]).collect();
            min_vals[d] = vals.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            max_vals[d] = vals.iter().fold(0.0_f64, |a, &b| a.max(b));
            ranges[d] = max_vals[d] - min_vals[d] + 1e-9;
        }
        
        let mut weights = Vec::with_capacity(archive_size);
        if archive_size == 1 {
            idx = 0;
            weights.push(1.0);
        } else {
            let mut normalized_coords = vec![vec![0.0; 5]; archive_size];
            for d in 0..5 {
                for i in 0..archive_size {
                    normalized_coords[i][d] = (engine.archive.solutions[i].fitness[d] - min_vals[d]) / ranges[d];
                }
            }
            
            for i in 0..archive_size {
                let mut min_dist = f64::INFINITY;
                for j in 0..archive_size {
                    if i == j { continue; }
                    let dist = (0..5).map(|d| (normalized_coords[i][d] - normalized_coords[j][d]).powi(2)).sum::<f64>().sqrt();
                    if dist < min_dist { min_dist = dist; }
                }
                let novelty = (min_dist + 1e-9).powf(0.5);
                weights.push(novelty);
            }
            
            let total_weight: f64 = weights.iter().sum();
            for w in weights.iter_mut() {
                *w /= total_weight;
            }
            
            let dist = WeightedIndex::new(&weights).unwrap();
            let mut rng = rand::thread_rng();
            idx = dist.sample(&mut rng);
        }
        
        let parent = engine.archive.solutions[idx].clone();
        if let Some(stats) = parent_tracker.get_mut(&parent.uid) {
            stats.epochs.entry(current_epoch).or_insert(EpochStats::default()).selections += 1;
        }
        
        let mut rng = rand::thread_rng();
        let num_offspring = 5;
        let mut best_candidate_genome = parent.genome.clone();
        let mut best_candidate_fitness = vec![f64::INFINITY; 5];
        let mut best_candidate_energy = f64::INFINITY;
        
        let calc_raw_energy = |fitness: &[f64]| -> f64 {
            fitness.iter().map(|v| v.powi(2)).sum::<f64>().sqrt()
        };
        
        for _ in 0..num_offspring {
            let use_tier1 = rng.gen_bool(0.8);
            let candidate_genome = engine.mutator.mutate_with_tier(&parent.genome, use_tier1);
            let candidate_fitness = engine.evaluator.evaluate(&candidate_genome);
            let energy = calc_raw_energy(&candidate_fitness);
            
            if energy < best_candidate_energy {
                best_candidate_energy = energy;
                best_candidate_fitness = candidate_fitness;
                best_candidate_genome = candidate_genome;
            }
        }
        
        let mut candidate_genome = best_candidate_genome;
        let mut candidate_fitness = best_candidate_fitness;
        
        let mut t = 1000.0;
        let alpha = 0.95;
        let sa_steps = 20;
        let mut current_energy = calc_raw_energy(&candidate_fitness);
        
        for _ in 0..sa_steps {
            let use_t1 = rng.gen_bool(0.8);
            let neighbor_genome = engine.mutator.mutate_with_tier(&candidate_genome, use_t1);
            let neighbor_fitness = engine.evaluator.evaluate(&neighbor_genome);
            let neighbor_energy = calc_raw_energy(&neighbor_fitness);
            
            let delta = neighbor_energy - current_energy;
            if delta < 0.0 || rng.gen_range(0.0..1.0) < (-delta / t).exp() {
                candidate_genome = neighbor_genome;
                candidate_fitness = neighbor_fitness;
                current_energy = neighbor_energy;
            }
            t *= alpha;
        }
        
        let child_inrc = candidate_fitness[0] + candidate_fitness[1] + candidate_fitness[2];
        let parent_inrc = parent.fitness[0] + parent.fitness[1] + parent.fitness[2];
        let inrc_improvement = if parent_inrc > child_inrc { parent_inrc - child_inrc } else { 0.0 };
        
        let was_inserted = engine.archive.add(coralys_moga::engine_proof::ParetoSolution {
            genome: candidate_genome,
            fitness: candidate_fitness,
            uid: next_uid,
            parent_uid: parent.uid,
        });
        
        if was_inserted {
            if let Some(stats) = parent_tracker.get_mut(&parent.uid) {
                let e_stats = stats.epochs.entry(current_epoch).or_insert(EpochStats::default());
                e_stats.total_yield += inrc_improvement;
                e_stats.admissions += 1;
            }
            
            parent_tracker.insert(next_uid, ParentStats {
                insertion_epoch: current_epoch,
                epochs: HashMap::new(),
            });
            next_uid += 1;
        }
        
        if g % 1000 == 0 {
            let best_inrc = engine.archive.solutions.iter().map(|s| s.fitness[0] + s.fitness[1] + s.fitness[2]).fold(f64::INFINITY, |a,b| a.min(b));
            println!("Epoch {} complete. Archive: {}, Best INRC: {:.1}", current_epoch, archive_size, best_inrc);
        }
    }
    
    // Computing correlations
    let mut file = File::create(format!("persistence_25k_{}.txt", instance)).unwrap();
    writeln!(file, "Yield Autocorrelation (Persistence)").unwrap();
    
    let pearson = |x: &Vec<f64>, y: &Vec<f64>| -> f64 {
        if x.len() < 2 { return 0.0; }
        let mx = x.iter().sum::<f64>() / x.len() as f64;
        let my = y.iter().sum::<f64>() / y.len() as f64;
        let mut cov = 0.0;
        let mut var_x = 0.0;
        let mut var_y = 0.0;
        for (vx, vy) in x.iter().zip(y.iter()) {
            let dx = vx - mx;
            let dy = vy - my;
            cov += dx * dy;
            var_x += dx * dx;
            var_y += dy * dy;
        }
        if var_x == 0.0 || var_y == 0.0 { return 0.0; }
        cov / (var_x * var_y).sqrt()
    };
    
    let compute_decay = |x: &Vec<f64>, y: &Vec<f64>| -> (f64, f64, f64, f64) {
        let mut decays = Vec::new();
        for (vx, vy) in x.iter().zip(y.iter()) {
            if *vx > 0.0 {
                decays.push(*vy / *vx);
            }
        }
        if decays.is_empty() { return (0.0, 0.0, 0.0, 0.0); }
        decays.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mean = decays.iter().sum::<f64>() / decays.len() as f64;
        let p25 = decays[(decays.len() as f64 * 0.25) as usize];
        let med = decays[decays.len() / 2];
        let p75 = decays[(decays.len() as f64 * 0.75) as usize];
        (mean, med, p25, p75)
    };
    
    for lag in 1..=24 {
        let mut lag_x = Vec::new();
        let mut lag_y = Vec::new();
        for stats in parent_tracker.values() {
            for e in 1..=(25 - lag) {
                if let Some(s1) = stats.epochs.get(&e) {
                    if s1.selections > 0 {
                        let y1 = s1.total_yield / s1.selections as f64;
                        if let Some(s2) = stats.epochs.get(&(e+lag)) {
                            if s2.selections > 0 {
                                let y2 = s2.total_yield / s2.selections as f64;
                                lag_x.push(y1); lag_y.push(y2);
                            }
                        }
                    }
                }
            }
        }
        
        let corr = pearson(&lag_x, &lag_y);
        let (mean, med, p25, p75) = compute_decay(&lag_x, &lag_y);
        writeln!(file, "Lag {:02}: Corr={:.4}, N={}", lag, corr, lag_x.len()).unwrap();
        writeln!(file, "Lag {:02} Decay: Mean={:.4}, Median={:.4}, P25={:.4}, P75={:.4}", lag, mean, med, p25, p75).unwrap();
    }
    

    
    // Compute Survival Probability (Top/Bottom 10%)
    let mut early_parents = Vec::new();
    for (uid, stats) in &parent_tracker {
        if stats.insertion_epoch <= 5 {
            // Compute yield in insertion epoch
            if let Some(es) = stats.epochs.get(&stats.insertion_epoch) {
                if es.selections > 0 {
                    let yield_0 = es.total_yield / es.selections as f64;
                    early_parents.push((*uid, yield_0, stats.insertion_epoch));
                }
            }
        }
    }
    
    early_parents.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    
    let top_10_idx = early_parents.len() - (early_parents.len() / 10);
    let bottom_10_idx = early_parents.len() / 10;
    
    let mut top_survivors = 0;
    let mut bottom_survivors = 0;
    
    for i in top_10_idx..early_parents.len() {
        let (uid, _, birth) = early_parents[i];
        let stats = parent_tracker.get(&uid).unwrap();
        let target_epoch = birth + 5;
        if let Some(es) = stats.epochs.get(&target_epoch) {
            if es.selections > 0 {
                top_survivors += 1;
            }
        }
    }
    
    for i in 0..bottom_10_idx {
        let (uid, _, birth) = early_parents[i];
        let stats = parent_tracker.get(&uid).unwrap();
        let target_epoch = birth + 5;
        if let Some(es) = stats.epochs.get(&target_epoch) {
            if es.selections > 0 {
                bottom_survivors += 1;
            }
        }
    }
    
    let top_surv_rate = top_survivors as f64 / (early_parents.len() - top_10_idx) as f64;
    let bot_surv_rate = bottom_survivors as f64 / bottom_10_idx as f64;
    
    writeln!(file, "\nSurvival Probability (Productive 5000 gens later)").unwrap();
    writeln!(file, "Top 10% Initial Yield: {:.2}%", top_surv_rate * 100.0).unwrap();
    writeln!(file, "Bottom 10% Initial Yield: {:.2}%", bot_surv_rate * 100.0).unwrap();
    
    println!("Persistence evaluation complete.");
}
