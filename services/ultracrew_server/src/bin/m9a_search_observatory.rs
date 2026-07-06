use ultracrew::inrc::parser::{parse_scenario, parse_week_data, parse_history};
use ultracrew::inrc::optimization::{InrcContext, InrcOptimizer};
use ultracrew_server::simulation::generate_baseline_schedule;
use ultracrew_server::optimizer::{ScheduleGenome, UltraCrewEvaluator, UltraCrewMutator};
use coralys_moga::engine_proof::{EvolutionEngine, ParetoSolution, Evaluator};
use coralys_ecology::state::{SearchStateObservatory, SearchSnapshot};
use coralys_moga::ecology::{EcologyMemory, distribution_gini};
use serde::Serialize;
use std::sync::Arc;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, VecDeque};
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::distributions::{WeightedIndex, Distribution};
use rand::Rng;

const MAX_GENERATIONS: usize = 10000;

#[derive(Serialize, Clone, Copy, PartialEq)]
enum CoolingMode {
    Local,
    Global,
    Hybrid,
}

#[derive(Serialize)]
struct SeedStateReport {
    seed: u64,
    instance: String,
    mode: String,
    history: Vec<SearchSnapshot>,
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn hamming_distance(g1: &ScheduleGenome, g2: &ScheduleGenome) -> f64 {
    let mut diffs = 0;
    for (s1, s2) in g1.slots.iter().zip(g2.slots.iter()) {
        if s1.assigned_nurse != s2.assigned_nurse {
            diffs += 1;
        }
    }
    diffs as f64 / g1.slots.len().max(1) as f64
}

fn run_instance(instance: &str, seeds: &[u64], mode: CoolingMode) -> Vec<SeedStateReport> {
    let mut all_reports = Vec::new();
    
    let base_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../adapters/ultracrew/tests/data/{}", instance));
    
    let scenario_path = base_dir.join(format!("Sc-{}.json", instance));
    if !scenario_path.exists() {
        println!("Warning: Instance {} not found at {:?}. Skipping.", instance, scenario_path);
        return all_reports;
    }
    
    let scenario  = parse_scenario(scenario_path).unwrap();
    let week_data = parse_week_data(base_dir.join(format!("WD-{}-0.json", instance))).unwrap();
    let history   = parse_history(base_dir.join(format!("H0-{}-0.json", instance))).unwrap();

    for &seed in seeds {
        let mode_name = match mode {
            CoolingMode::Local => "Local",
            CoolingMode::Global => "Global",
            CoolingMode::Hybrid => "Hybrid",
        };
        println!("--- Running {} (seed={}, mode={}) ---", instance, seed, mode_name);

        let mut rng = StdRng::seed_from_u64(seed);

        let inrc_context = InrcContext::new(
            scenario.clone(),
            week_data.clone(),
            history.clone(),
            ultracrew::ecology::WorkforceEcology::new(),
        );
        let inrc_optimizer = InrcOptimizer { context: Arc::new(inrc_context) };

        let evaluator = UltraCrewEvaluator { scenario: scenario.clone() };
        let mutator   = UltraCrewMutator::new(scenario.clone());
        let mut engine = EvolutionEngine::new(evaluator, mutator);

        let baseline   = generate_baseline_schedule(&scenario, &week_data.requirements).unwrap();
        let base_fitness = engine.evaluator.evaluate(&baseline);
        let base_uid   = calculate_hash(&baseline);
        engine.archive.add(ParetoSolution {
            genome: baseline.clone(), fitness: base_fitness, uid: base_uid, parent_uid: 0,
        });

        let mut observatory = SearchStateObservatory::new();
        let mut memory = EcologyMemory::<usize>::new();
        
        let mut nurse_to_idx = HashMap::new();
        for (i, n) in scenario.nurses.iter().enumerate() {
            nurse_to_idx.insert(n.id.clone(), i);
        }
        
        let mut history_buffer: VecDeque<ScheduleGenome> = VecDeque::new();
        let mut recent_revisits: VecDeque<bool> = VecDeque::new();
        let history_k = 10;
        
        let mut global_t = 1000.0_f64;

        for g in 1..=MAX_GENERATIONS {
            let archive_size = engine.archive.solutions.len();
            if archive_size == 0 { break; }
            let num_objs = engine.archive.solutions[0].fitness.len();

            let idx = if archive_size == 1 {
                0
            } else {
                let mut min_vals = vec![f64::INFINITY; num_objs];
                let mut max_vals = vec![0.0_f64; num_objs];
                for d in 0..num_objs {
                    for sol in &engine.archive.solutions {
                        min_vals[d] = min_vals[d].min(sol.fitness[d]);
                        max_vals[d] = max_vals[d].max(sol.fitness[d]);
                    }
                }
                let ranges: Vec<f64> = (0..num_objs)
                    .map(|d| max_vals[d] - min_vals[d] + 1e-9)
                    .collect();
                let mut weights = Vec::with_capacity(archive_size);
                for i in 0..archive_size {
                    let mut min_dist = f64::INFINITY;
                    for j in 0..archive_size {
                        if i == j { continue; }
                        let dist = (0..num_objs)
                            .map(|d| {
                                let ni = (engine.archive.solutions[i].fitness[d] - min_vals[d]) / ranges[d];
                                let nj = (engine.archive.solutions[j].fitness[d] - min_vals[d]) / ranges[d];
                                (ni - nj).powi(2)
                            })
                            .sum::<f64>()
                            .sqrt();
                        if dist < min_dist { min_dist = dist; }
                    }
                    weights.push((min_dist + 1e-9).powf(0.5));
                }
                let total_w: f64 = weights.iter().sum();
                for w in weights.iter_mut() { *w /= total_w; }
                WeightedIndex::new(&weights).unwrap().sample(&mut rng)
            };

            let parent = engine.archive.solutions[idx].clone();
            let calc_energy = |f: &[f64]| f.iter().map(|v| v.powi(2)).sum::<f64>().sqrt();

            let mut tier1_attempts = 0;
            let mut tier1_acceptances = 0;
            let mut tier1_improvements = 0;
            let mut tier2_attempts = 0;
            let mut tier2_acceptances = 0;
            let mut tier2_improvements = 0;

            let mut best_cand: (ScheduleGenome, Vec<f64>, String) = {
                let candidates: Vec<(ScheduleGenome, Vec<f64>, String)> = (0..5)
                    .map(|_| {
                        let is_tier1 = rng.gen_bool(0.5);
                        let (gc, op) = engine.mutator.mutate_with_tier_logged(&parent.genome, is_tier1);
                        if op.contains("Tier1") { tier1_attempts += 1; } else { tier2_attempts += 1; }
                        let fit = engine.evaluator.evaluate(&gc);
                        (gc, fit, op)
                    })
                    .collect();
                candidates.into_iter()
                    .min_by(|a, b| calc_energy(&a.1).partial_cmp(&calc_energy(&b.1)).unwrap())
                    .unwrap()
            };
            
            let mut t = match mode {
                CoolingMode::Local => 1000.0_f64,
                _ => global_t,
            };
            let alpha = 0.95_f64;
            let mut total_moves = 0;
            let mut accepted_worse_moves = 0;
            let mut accepted_better_moves = 0;
            let mut operator_counts = HashMap::new();
            
            for _ in 0..2 {
                let is_tier1 = rng.gen_bool(0.5);
                let (neighbour, op) = engine.mutator.mutate_with_tier_logged(&best_cand.0, is_tier1);
                if op.contains("Tier1") { tier1_attempts += 1; } else { tier2_attempts += 1; }
                
                let n_fit = engine.evaluator.evaluate(&neighbour);
                let delta = calc_energy(&n_fit) - calc_energy(&best_cand.1);
                total_moves += 1;
                
                let mut accepted = false;
                let mut is_improvement = false;
                if delta < 0.0 {
                    accepted = true;
                    is_improvement = true;
                    accepted_better_moves += 1;
                } else if rng.gen_range(0.0..1.0) < (-delta / t).exp() {
                    accepted = true;
                    accepted_worse_moves += 1;
                }
                
                if accepted {
                    best_cand = (neighbour, n_fit, op.clone());
                    *operator_counts.entry(op.clone()).or_insert(0) += 1;
                    if op.contains("Tier1") {
                        tier1_acceptances += 1;
                        if is_improvement { tier1_improvements += 1; }
                    } else {
                        tier2_acceptances += 1;
                        if is_improvement { tier2_improvements += 1; }
                    }
                }
                
                if mode == CoolingMode::Local {
                    t *= alpha;
                }
            }
            
            match mode {
                CoolingMode::Global => {
                    global_t *= 0.99;
                    t = global_t;
                },
                CoolingMode::Hybrid => {
                    global_t *= 0.9995;
                    t = global_t;
                },
                CoolingMode::Local => {}
            }
            
            if total_moves == 0 || (accepted_better_moves == 0 && accepted_worse_moves == 0) {
               *operator_counts.entry(best_cand.2.clone()).or_insert(0) += 1;
               if best_cand.2.contains("Tier1") { tier1_acceptances += 1; } else { tier2_acceptances += 1; }
            }

            let (child_genome, child_fitness, _) = best_cand;
            let child_uid = calculate_hash(&child_genome);
            
            for slot in &child_genome.slots {
                if let Some(&idx) = nurse_to_idx.get(&slot.assigned_nurse) {
                    memory.accumulate(idx, "assignments", 1.0);
                }
            }
            let mut loads = Vec::new();
            for i in 0..scenario.nurses.len() {
                loads.push(memory.get_measure(i, "assignments") as usize);
            }
            let memory_novelty_proxy = 1.0 - distribution_gini(&loads);
            
            let mut min_dist_to_history = 1.0;
            let mut sum_dist = 0.0;
            let mut is_revisit = false;
            
            if history_buffer.is_empty() {
                min_dist_to_history = hamming_distance(&child_genome, &baseline);
                sum_dist = min_dist_to_history;
            } else {
                for hist_genome in &history_buffer {
                    let d = hamming_distance(&child_genome, hist_genome);
                    if d == 0.0 { is_revisit = true; }
                    if d < min_dist_to_history { min_dist_to_history = d; }
                    sum_dist += d;
                }
            }
            let history_novelty = if history_buffer.is_empty() {
                sum_dist
            } else {
                sum_dist / history_buffer.len() as f64
            };
            
            recent_revisits.push_back(is_revisit);
            if recent_revisits.len() > history_k { recent_revisits.pop_front(); }
            let revisit_rate = recent_revisits.iter().filter(|&&r| r).count() as f64 / recent_revisits.len() as f64;
            
            history_buffer.push_back(child_genome.clone());
            if history_buffer.len() > history_k { history_buffer.pop_front(); }

            let acceptance_rate = (accepted_worse_moves + accepted_better_moves) as f64 / total_moves.max(1) as f64;
            let worse_acceptance_rate = accepted_worse_moves as f64 / total_moves.max(1) as f64;
            let better_acceptance_rate = accepted_better_moves as f64 / total_moves.max(1) as f64;

            engine.archive.add(ParetoSolution {
                genome: child_genome.clone(), fitness: child_fitness.clone(),
                uid: child_uid, parent_uid: parent.uid,
            });

            let best_sol = &engine.archive.solutions[0];
            let sc = ultracrew_server::inrc_observer::score_inrc_official(&best_sol.genome, &scenario, &inrc_optimizer);
            let global_best_fitness = sc.official_total;

            let mut centroid = vec![0.0_f64; num_objs];
            for sol in &engine.archive.solutions {
                for d in 0..num_objs {
                    centroid[d] += sol.fitness[d];
                }
            }
            for d in 0..num_objs {
                centroid[d] /= engine.archive.solutions.len() as f64;
            }
            let diversity: f64 = engine.archive.solutions.iter()
                .map(|sol| {
                    (0..num_objs).map(|d| (sol.fitness[d] - centroid[d]).powi(2)).sum::<f64>().sqrt()
                })
                .sum::<f64>() / (engine.archive.solutions.len() as f64 + 1e-9);

            let child_sc = ultracrew_server::inrc_observer::score_inrc_official(&child_genome, &scenario, &inrc_optimizer);
            let distance_to_incumbent_best = child_sc.official_total - global_best_fitness;

            observatory.observe_minimization(
                g,
                global_best_fitness,
                diversity,
                memory_novelty_proxy,
                history_novelty,
                revisit_rate,
                distance_to_incumbent_best,
                1.0,
                &operator_counts,
                t,
                accepted_worse_moves,
                accepted_better_moves,
                acceptance_rate,
                worse_acceptance_rate,
                better_acceptance_rate,
                tier1_attempts,
                tier1_acceptances,
                tier1_improvements,
                tier2_attempts,
                tier2_acceptances,
                tier2_improvements,
            );
        }

        all_reports.push(SeedStateReport {
            seed,
            instance: instance.to_string(),
            mode: mode_name.to_string(),
            history: observatory.get_history().to_vec(),
        });
    }
    
    all_reports
}

fn main() {
    println!("=== M9A.1: Large Scale SearchState Observatory ===");
    
    let instances = vec!["n030w4", "n050w4", "n080w8"];
    let seeds: Vec<u64> = (1..=30).collect();
    let mut all_reports = Vec::new();
    
    for instance in instances {
        let mut rep_global = run_instance(instance, &seeds, CoolingMode::Global);
        all_reports.append(&mut rep_global);
    }
    
    let json = serde_json::to_string_pretty(&all_reports).unwrap();
    let filename = "m9a_ultracrew_state_large.json";
    std::fs::write(&filename, &json).expect("Failed to write state JSON");
    println!("Wrote report to {}", filename);
}
