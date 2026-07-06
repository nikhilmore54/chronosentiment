use ultracrew::inrc::parser::{parse_scenario, parse_week_data, parse_history};
use ultracrew::inrc::optimization::{InrcContext, InrcOptimizer};
use ultracrew_server::simulation::generate_baseline_schedule;
use ultracrew_server::optimizer::{ScheduleGenome, UltraCrewEvaluator, UltraCrewMutator};
use coralys_moga::engine_proof::{EvolutionEngine, ParetoSolution, Evaluator};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::distributions::{WeightedIndex, Distribution};
use rand::Rng;

const INSTANCE: &str = "n050w4";
const MAX_GENERATIONS: usize = 1000;

#[derive(Serialize)]
struct SeedProgressReport {
    seed: u64,
    final_score: f64,
    progress_at_100: coralys_ecology::progress::ProgressObservation,
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn main() {
    let seeds: Vec<u64> = (1..=100).collect();

    println!("=== M8G: UltraCrew Progress Validation ===");
    println!("Instance    : {}", INSTANCE);
    println!("Generations : {}", MAX_GENERATIONS);
    println!("Seeds       : {} total", seeds.len());
    println!();

    let base_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../adapters/ultracrew/tests/data/{}", INSTANCE));
    let scenario  = parse_scenario(base_dir.join(format!("Sc-{}.json",  INSTANCE))).unwrap();
    let week_data = parse_week_data(base_dir.join(format!("WD-{}-0.json", INSTANCE))).unwrap();
    let history   = parse_history(base_dir.join(format!("H0-{}-0.json",  INSTANCE))).unwrap();

    let mut all_reports: Vec<SeedProgressReport> = Vec::new();

    for seed in &seeds {
        println!("--- Running seed {} ---", seed);

        let mut rng = StdRng::seed_from_u64(*seed);

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
            genome: baseline, fitness: base_fitness, uid: base_uid, parent_uid: 0,
        });

        let mut progress_tracker = coralys_ecology::progress::ProgressTracker::new();
        let mut progress_at_100 = None;
        let mut final_score = 0.0;

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

            let mut best_cand: (ScheduleGenome, Vec<f64>) = {
                let candidates: Vec<(ScheduleGenome, Vec<f64>)> = (0..5)
                    .map(|_| {
                        let gc  = engine.mutator.mutate_with_tier(&parent.genome, rng.gen_bool(0.8));
                        let fit = engine.evaluator.evaluate(&gc);
                        (gc, fit)
                    })
                    .collect();
                candidates.into_iter()
                    .min_by(|a, b| calc_energy(&a.1).partial_cmp(&calc_energy(&b.1)).unwrap())
                    .unwrap()
            };
            let mut t = 1000.0_f64;
            let alpha = 0.95_f64;
            for _ in 0..2 {
                let neighbour = engine.mutator.mutate_with_tier(&best_cand.0, rng.gen_bool(0.8));
                let n_fit = engine.evaluator.evaluate(&neighbour);
                let delta = calc_energy(&n_fit) - calc_energy(&best_cand.1);
                if delta < 0.0 || rng.gen_range(0.0..1.0) < (-delta / t).exp() {
                    best_cand = (neighbour, n_fit);
                }
                t *= alpha;
            }

            let (child_genome, child_fitness) = best_cand;
            let child_uid = calculate_hash(&child_genome);

            engine.archive.add(ParetoSolution {
                genome: child_genome.clone(), fitness: child_fitness.clone(),
                uid: child_uid, parent_uid: parent.uid,
            });

            let best_sol = &engine.archive.solutions[0];
            let sc = ultracrew_server::inrc_observer::score_inrc_official(&best_sol.genome, &scenario, &inrc_optimizer);
            let global_best_fitness = sc.official_total;
            
            let progress_telemetry = progress_tracker.observe_minimization(g, global_best_fitness);
            
            if g == 100 {
                progress_at_100 = Some(progress_telemetry);
            }
            if g == MAX_GENERATIONS {
                final_score = global_best_fitness;
            }

            if g % 100 == 0 {
                println!("  Generation {:>4} / {}... best cost: {}", g, MAX_GENERATIONS, global_best_fitness);
            }
        }

        all_reports.push(SeedProgressReport {
            seed: *seed,
            final_score,
            progress_at_100: progress_at_100.unwrap(),
        });
    }

    let json = serde_json::to_string_pretty(&all_reports).unwrap();
    let filename = "m8g_ultracrew_report.json";
    std::fs::write(&filename, &json).expect("Failed to write seed JSON");
    println!("Wrote report to {}", filename);
}
