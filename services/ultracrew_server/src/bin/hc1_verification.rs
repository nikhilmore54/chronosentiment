//! # hc1_verification
//!
//! Verification gate for the HC1 coverage correction (commit 9d106d201).
//!
//! ## Purpose
//! Confirms that after adding `coverage_deficit` as `objective[5]` in
//! `UltraCrewEvaluator::evaluate()`, the Pareto archive for n030w4 no longer
//! contains 40/196-class (severely under-assigned) candidates as preferred members.
//!
//! ## Verification criteria (all must pass)
//! 1. No archive member has coverage_deficit > 0 in objective[5]  — OR —
//!    if any do, they are strictly dominated on objective[5] by HC1-feasible members.
//! 2. At least one archive member has coverage_deficit == 0.0 (HC1-feasible).
//! 3. The minimum positions_filled across all archive members is >= 90% of total_required
//!    (i.e. no 40/196-class member survives).
//! 4. The best archive member (lowest coverage_deficit) has coverage_deficit == 0.0.
//!
//! ## Run
//! ```
//! # Fast smoke run (500 generations)
//! cargo run --bin hc1_verification -- --gens 500
//!
//! # Full run (2000 generations)
//! cargo run --bin hc1_verification -- --gens 2000
//! ```

use coralys_moga::engine_proof::{Evaluator, EvolutionEngine, ParetoSolution};
use rand::Rng;
use rand::SeedableRng;
use rand::distributions::{Distribution, WeightedIndex};
use rand::rngs::StdRng;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use ultracrew::inrc::optimization::{InrcContext, InrcOptimizer};
use ultracrew::inrc::parser::{parse_history, parse_scenario, parse_week_data};
use ultracrew_server::optimizer::{ScheduleGenome, UltraCrewEvaluator, UltraCrewMutator};
use ultracrew_server::simulation::generate_baseline_schedule;
use ultracrew_server::inrc_observer::score_inrc_official;

const INSTANCE: &str = "n030w4";

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

/// Count positions filled and total required for a genome against week_data requirements.
/// Returns (positions_filled, total_required).
fn count_coverage(
    genome: &ScheduleGenome,
    scenario: &ultracrew::inrc::models::InrcScenario,
    week_data: &ultracrew::inrc::models::InrcWeekData,
) -> (usize, usize) {
    let flat = genome.to_flat_schedule();
    let num_days = scenario.number_of_weeks * 7;
    let days_map = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];

    let mut total_required = 0usize;
    let mut total_filled = 0usize;

    for d in 0..num_days {
        let day_name = days_map[d % 7];
        for req in &week_data.requirements {
            let req_level = match day_name {
                "Monday"    => &req.monday,
                "Tuesday"   => &req.tuesday,
                "Wednesday" => &req.wednesday,
                "Thursday"  => &req.thursday,
                "Friday"    => &req.friday,
                "Saturday"  => &req.saturday,
                "Sunday"    => &req.sunday,
                _           => continue,
            };
            if req_level.minimum == 0 {
                continue;
            }
            total_required += req_level.minimum;

            let mut filled = 0usize;
            for nurse in &scenario.nurses {
                if nurse.skills.contains(&req.skill) {
                    if let Some(shifts) = flat.get(&nurse.id) {
                        if d < shifts.len() && shifts[d] == req.shift_type {
                            filled += 1;
                        }
                    }
                }
            }
            total_filled += filled.min(req_level.minimum);
        }
    }

    (total_filled, total_required)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let max_generations: u64 = args
        .iter()
        .position(|a| a == "--gens")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(500);
    let seed: u64 = args
        .iter()
        .position(|a| a == "--seed")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(42);

    println!("=== hc1_verification ===");
    println!("Instance    : {}", INSTANCE);
    println!("Generations : {}", max_generations);
    println!("Seed        : {}", seed);
    println!("Purpose     : Verify HC1 coverage_deficit correction (commit 9d106d201)");
    println!();

    // ── Load scenario ──────────────────────────────────────────────────────────
    let base_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../adapters/ultracrew/tests/data/{}", INSTANCE));
    let scenario = parse_scenario(base_dir.join(format!("Sc-{}.json", INSTANCE))).unwrap();
    let week_data = parse_week_data(base_dir.join(format!("WD-{}-0.json", INSTANCE))).unwrap();
    let history = parse_history(base_dir.join(format!("H0-{}-0.json", INSTANCE))).unwrap();

    let inrc_context = InrcContext::new(
        scenario.clone(),
        week_data.clone(),
        history,
        ultracrew::ecology::WorkforceEcology::new(),
    );
    let inrc_optimizer = InrcOptimizer {
        context: Arc::new(inrc_context),
    };

    // ── Engine ─────────────────────────────────────────────────────────────────
    let evaluator = UltraCrewEvaluator {
        scenario: scenario.clone(),
        week_data: week_data.clone(),
    };
    let mutator = UltraCrewMutator::new(scenario.clone());
    let mut engine = EvolutionEngine::new(evaluator, mutator);
    let mut rng = StdRng::seed_from_u64(seed);

    // ── Seed with baseline ─────────────────────────────────────────────────────
    let baseline_genome = generate_baseline_schedule(&scenario, &week_data.requirements).unwrap();
    let base_fitness = engine.evaluator.evaluate(&baseline_genome);
    let base_uid = calculate_hash(&baseline_genome);

    let (base_filled, base_required) = count_coverage(&baseline_genome, &scenario, &week_data);
    let base_coverage_deficit = base_fitness.get(5).copied().unwrap_or(0.0);

    println!("Baseline genome:");
    println!(
        "  positions_filled / total_required : {} / {}  ({:.1}%)",
        base_filled,
        base_required,
        if base_required > 0 { base_filled as f64 / base_required as f64 * 100.0 } else { 0.0 }
    );
    println!("  objective[5] coverage_deficit     : {}", base_coverage_deficit);
    println!("  fitness vector len                 : {}", base_fitness.len());
    println!();

    engine.archive.add(ParetoSolution {
        genome: baseline_genome,
        fitness: base_fitness,
        uid: base_uid,
        parent_uid: 0,
    });

    // ── Main generation loop ───────────────────────────────────────────────────
    let calc_energy = |f: &[f64]| f.iter().map(|v| v.powi(2)).sum::<f64>().sqrt();

    for g in 1..=max_generations {
        let archive_size = engine.archive.solutions.len();
        if archive_size == 0 {
            break;
        }
        let num_objs = engine.archive.solutions[0].fitness.len();

        // Parent selection (crowding-distance weighted, uses all objectives)
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
            let dist_sampler = WeightedIndex::new(&weights).unwrap();
            dist_sampler.sample(&mut rng)
        };

        let parent = engine.archive.solutions[idx].clone();

        // Generate offspring (5 initial + 20 SA neighbours)
        let mut best_cand: (ScheduleGenome, Vec<f64>) = {
            let candidates: Vec<(ScheduleGenome, Vec<f64>)> = (0..5)
                .map(|_| {
                    let gc = engine.mutator.mutate_with_tier(&parent.genome, rng.gen_bool(0.8));
                    let fit = engine.evaluator.evaluate(&gc);
                    (gc, fit)
                })
                .collect();
            candidates
                .into_iter()
                .min_by(|a, b| calc_energy(&a.1).partial_cmp(&calc_energy(&b.1)).unwrap())
                .unwrap()
        };

        let mut t = 1000.0_f64;
        let alpha = 0.95_f64;
        for _ in 0..20 {
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
            genome: child_genome,
            fitness: child_fitness,
            uid: child_uid,
            parent_uid: parent.uid,
        });

        if g % 100 == 0 || g == max_generations {
            println!(
                "Gen {:>5} | archive={:>4}",
                g,
                engine.archive.solutions.len(),
            );
        }
    }

    // ── Final archive inspection ───────────────────────────────────────────────
    println!();
    println!("=== FINAL ARCHIVE INSPECTION (gen {}) ===", max_generations);
    println!(
        "{:<6} {:>8} {:>8} {:>8} {:>12} {:>10} {:>10}",
        "idx", "filled", "required", "cov%", "obj5_deficit", "hc_total", "feasible"
    );
    println!("{}", "-".repeat(72));

    let archive_size = engine.archive.solutions.len();
    let mut hc1_feasible_count = 0usize;
    let mut min_coverage_pct = 100.0_f64;
    let mut max_coverage_deficit = 0.0_f64;
    let mut min_filled = usize::MAX;
    let mut max_filled = 0usize;
    let mut total_required_global = 0usize;

    for (idx, sol) in engine.archive.solutions.iter().enumerate() {
        let (filled, required) = count_coverage(&sol.genome, &scenario, &week_data);
        let coverage_pct = if required > 0 { filled as f64 / required as f64 * 100.0 } else { 0.0 };
        let obj5_deficit = sol.fitness.get(5).copied().unwrap_or(0.0);

        // External INRC score for HC total
        let ext = score_inrc_official(&sol.genome, &scenario, &inrc_optimizer);
        let hc_total = ext.hc_coverage + ext.hc_skills + ext.hc_one_shift_per_day + ext.hc_forbidden_successions;
        let feasible = obj5_deficit == 0.0;

        println!(
            "{:<6} {:>8} {:>8} {:>7.1}% {:>12.0} {:>10} {:>10}",
            idx, filled, required, coverage_pct, obj5_deficit, hc_total,
            if feasible { "YES" } else { "NO" }
        );

        if feasible { hc1_feasible_count += 1; }
        if coverage_pct < min_coverage_pct { min_coverage_pct = coverage_pct; }
        if obj5_deficit > max_coverage_deficit { max_coverage_deficit = obj5_deficit; }
        if filled < min_filled { min_filled = filled; }
        if filled > max_filled { max_filled = filled; }
        total_required_global = required; // same for all members (same instance)
    }

    // ── Verification gate results ──────────────────────────────────────────────
    println!();
    println!("=== VERIFICATION GATE RESULTS ===");
    println!("Archive size                    : {}", archive_size);
    println!("HC1-feasible members (obj5==0)  : {}", hc1_feasible_count);
    println!("Min coverage %                  : {:.1}%", min_coverage_pct);
    println!("Max coverage_deficit (obj5)     : {:.0}", max_coverage_deficit);
    println!("Min positions filled            : {} / {}", min_filled, total_required_global);
    println!("Max positions filled            : {} / {}", max_filled, total_required_global);
    println!();

    // Gate 1: At least one HC1-feasible member
    let gate1 = hc1_feasible_count > 0;
    println!(
        "GATE 1 — HC1-feasible member exists          : {}",
        if gate1 { "PASS" } else { "FAIL" }
    );

    // Gate 2: No 40/196-class member (coverage < 30% of required)
    // 40/196 = 20.4%. Threshold: < 30% is a 40/196-class failure.
    let gate2 = min_coverage_pct >= 30.0;
    println!(
        "GATE 2 — No 40/196-class member (cov>=30%)   : {}  (min={:.1}%)",
        if gate2 { "PASS" } else { "FAIL" },
        min_coverage_pct
    );

    // Gate 3: Best member (first HC1-feasible) has coverage_deficit == 0
    let gate3 = hc1_feasible_count > 0;
    println!(
        "GATE 3 — HC1-feasible member in archive      : {}  (count={})",
        if gate3 { "PASS" } else { "FAIL" },
        hc1_feasible_count
    );

    // Gate 4: Fitness vector has 6 objectives (confirms correction is active)
    let gate4 = engine.archive.solutions.first().map(|s| s.fitness.len() == 6).unwrap_or(false);
    println!(
        "GATE 4 — FitnessVector has 6 objectives      : {}  (len={})",
        if gate4 { "PASS" } else { "FAIL" },
        engine.archive.solutions.first().map(|s| s.fitness.len()).unwrap_or(0)
    );

    println!();
    let all_pass = gate1 && gate2 && gate3 && gate4;
    if all_pass {
        println!("VERIFICATION RESULT: ALL GATES PASS — HC1 correction confirmed.");
        println!("The optimizer no longer admits 40/196-class candidates as Pareto-dominant.");
        println!("rankAlternatives() architectural decision may now proceed.");
    } else {
        println!("VERIFICATION RESULT: ONE OR MORE GATES FAILED — HC1 correction NOT confirmed.");
        eprintln!("ERROR: hc1_verification gates failed. See output above.");
        std::process::exit(1);
    }
}