use coralys_ecology::diagnostics::{
    AccumulationFailureDetector, AttractorDetector, CandidateObservation, DiagnosticDetector,
    EcologyLockInDetector, EcologyState, ObjectiveVector, ProxySuppressionDetector,
    SearchObservation, TradeoffBasinDetector,
};
use coralys_moga::engine_proof::{Evaluator, EvolutionEngine, ParetoSolution};
use coralys_recommendation::EcologyRecommender;
use rand::Rng;
use rand::SeedableRng;
use rand::distributions::{Distribution, WeightedIndex};
use rand::rngs::StdRng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use ultracrew::inrc::optimization::{InrcContext, InrcOptimizer};
use ultracrew::inrc::parser::{parse_history, parse_scenario, parse_week_data};
use ultracrew_server::inrc_observer::score_inrc_official;
use ultracrew_server::optimizer::{ScheduleGenome, UltraCrewEvaluator, UltraCrewMutator};
use ultracrew_server::simulation::generate_baseline_schedule;

const INSTANCE: &str = "n050w4";

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let max_generations: usize = args
        .iter()
        .position(|a| a == "--gens")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(1000);
    let seed: u64 = args
        .iter()
        .position(|a| a == "--seed")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(61); // Canonical SD-007 seed

    let mut rng = StdRng::seed_from_u64(seed);

    println!("=== Ecology Diagnostics Validation Run ===");
    println!("Instance    : {}", INSTANCE);
    println!("Generations : {}", max_generations);
    println!("Seed        : {}", seed);
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
    };
    let mutator = UltraCrewMutator::new(scenario.clone());
    let mut engine = EvolutionEngine::new(evaluator, mutator);

    // Seed with baseline
    let baseline_genome = generate_baseline_schedule(&scenario, &week_data.requirements).unwrap();
    let base_fitness = engine.evaluator.evaluate(&baseline_genome);
    let base_uid = calculate_hash(&baseline_genome);

    engine.archive.add(ParetoSolution {
        genome: baseline_genome,
        fitness: base_fitness,
        uid: base_uid,
        parent_uid: 0,
    });

    let mut telemetry_stream = Vec::new();
    let mut ecology_state = EcologyState::new(max_generations);

    let attractor_detector = AttractorDetector::default();
    let tradeoff_detector = TradeoffBasinDetector::new(1, 3, -0.5);
    let lockin_detector = EcologyLockInDetector::new(0, 0.15, 100);
    let accum_detector = AccumulationFailureDetector::new(0, 0.75);
    let suppression_detector = ProxySuppressionDetector::new(0, vec![3], 0.80);

    // ── Evolution Loop ─────────────────────────────────────────────────────────
    for g in 1..=max_generations {
        let archive_size = engine.archive.solutions.len();
        if archive_size == 0 {
            break;
        }
        let num_objs = engine.archive.solutions[0].fitness.len();

        // Parent selection
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
                    if i == j {
                        continue;
                    }
                    let dist = (0..num_objs)
                        .map(|d| {
                            let ni =
                                (engine.archive.solutions[i].fitness[d] - min_vals[d]) / ranges[d];
                            let nj =
                                (engine.archive.solutions[j].fitness[d] - min_vals[d]) / ranges[d];
                            (ni - nj).powi(2)
                        })
                        .sum::<f64>()
                        .sqrt();
                    if dist < min_dist {
                        min_dist = dist;
                    }
                }
                weights.push((min_dist + 1e-9).powf(0.5));
            }
            let total_w: f64 = weights.iter().sum();
            for w in weights.iter_mut() {
                *w /= total_w;
            }
            let dist_sampler = WeightedIndex::new(&weights).unwrap();
            dist_sampler.sample(&mut rng)
        };

        let parent = engine.archive.solutions[idx].clone();

        // Generate offspring
        let calc_energy = |f: &[f64]| f.iter().map(|v| v.powi(2)).sum::<f64>().sqrt();

        let mut best_cand: (ScheduleGenome, Vec<f64>) = {
            let candidates: Vec<(ScheduleGenome, Vec<f64>)> = (0..5)
                .map(|_| {
                    let gc = engine
                        .mutator
                        .mutate_with_tier(&parent.genome, rng.gen_bool(0.8));
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
            let neighbour = engine
                .mutator
                .mutate_with_tier(&best_cand.0, rng.gen_bool(0.8));
            let n_fit = engine.evaluator.evaluate(&neighbour);
            let delta = calc_energy(&n_fit) - calc_energy(&best_cand.1);
            if delta < 0.0 || rng.gen_range(0.0..1.0) < (-delta / t).exp() {
                best_cand = (neighbour, n_fit);
            }
            t *= alpha;
        }

        let (child_genome, child_fitness) = best_cand;
        let child_uid = calculate_hash(&child_genome);

        let child_score = score_inrc_official(&child_genome, &scenario, &inrc_optimizer);
        let parent_score = score_inrc_official(&parent.genome, &scenario, &inrc_optimizer);

        // Admission
        let was_inserted = engine.archive.add(ParetoSolution {
            genome: child_genome.clone(),
            fitness: child_fitness.clone(),
            uid: child_uid,
            parent_uid: parent.uid,
        });

        // ── Map INRC-II objectives to generic ObjectiveVector ────────────────
        // Index 0: target objective (HC_Total)
        // Index 1-5: proxy objectives (O1 to O5)
        let get_objective_vector = |fit: &[f64], official_hc: f64| -> ObjectiveVector {
            let mut values = vec![official_hc];
            values.extend_from_slice(fit);
            ObjectiveVector::new(values)
        };

        // Compute total HC for parent & child
        let parent_hc = (parent_score.hc_coverage
            + parent_score.hc_skills
            + parent_score.hc_one_shift_per_day
            + parent_score.hc_forbidden_successions) as f64;
        let child_hc = (child_score.hc_coverage
            + child_score.hc_skills
            + child_score.hc_one_shift_per_day
            + child_score.hc_forbidden_successions) as f64;

        let candidate = CandidateObservation {
            objectives: get_objective_vector(&child_fitness, child_hc),
            admitted: was_inserted,
            feasible: child_score.feasible,
            parent_objectives: Some(get_objective_vector(&parent.fitness, parent_hc)),
        };

        let archive_objectives: Vec<ObjectiveVector> = engine
            .archive
            .solutions
            .iter()
            .map(|sol| {
                let sc = score_inrc_official(&sol.genome, &scenario, &inrc_optimizer);
                let hc = (sc.hc_coverage
                    + sc.hc_skills
                    + sc.hc_one_shift_per_day
                    + sc.hc_forbidden_successions) as f64;
                get_objective_vector(&sol.fitness, hc)
            })
            .collect();

        telemetry_stream.push(SearchObservation {
            generation: g,
            archive_size,
            diversity_score: 0.0,
            candidates: vec![candidate],
            archive_objectives,
            telemetry: None,
        });

        // Record diagnostic results in EcologyState chronologically
        let obs_window = 200;
        let start_idx = telemetry_stream.len().saturating_sub(obs_window);
        let obs_slice = &telemetry_stream[start_idx..];

        let mut results = std::collections::HashMap::new();
        results.insert(
            "Attractor".to_string(),
            attractor_detector.evaluate(obs_slice),
        );
        results.insert(
            "TradeoffBasin".to_string(),
            tradeoff_detector.evaluate(obs_slice),
        );
        results.insert(
            "EcologyLockIn".to_string(),
            lockin_detector.evaluate(obs_slice),
        );
        results.insert(
            "AccumulationFailure".to_string(),
            accum_detector.evaluate(obs_slice),
        );
        results.insert(
            "ProxySuppression".to_string(),
            suppression_detector.evaluate(obs_slice),
        );

        ecology_state.record(g, results);

        if g % 200 == 0 {
            println!("Generation {:>4} / {}...", g, max_generations);
        }
    }

    println!(
        "\nTelemetry Stream complete ({} observations captured).",
        telemetry_stream.len()
    );
    println!("Running diagnostic detectors...");

    // ── Evaluate Detectors ─────────────────────────────────────────────────────

    // 1. AttractorDetector
    let attractor_detector = AttractorDetector::default();
    let res_attractor = attractor_detector.evaluate(&telemetry_stream);
    println!("\n--- Attractor Detector ---");
    println!("Confidence: {:.4}", res_attractor.confidence);
    println!("Severity  : {:.4}", res_attractor.severity);
    if let Some(metric) = res_attractor
        .supporting_metrics
        .iter()
        .find(|m| m.name == "attractor_index")
    {
        let idx = metric.value as usize;
        let labels = [
            "Target (HC_Total)",
            "O1 (Assignments)",
            "O2 (Weekends)",
            "O3 (Successions)",
            "O4 (Workload)",
            "O5 (Temporal)",
        ];
        println!("Attractor Index: {} ({})", idx, labels[idx]);
    }
    for m in &res_attractor.supporting_metrics {
        if m.name.starts_with("correlation_obj_") {
            println!("  {}: {:.4}", m.name, m.value);
        }
    }

    // 2. TradeoffBasinDetector
    // X is O1 (Assignments, index 1), Y is O3 (Successions, index 3)
    let tradeoff_detector = TradeoffBasinDetector::new(1, 3, -0.5);
    let res_tradeoff = tradeoff_detector.evaluate(&telemetry_stream);
    println!("\n--- Tradeoff Basin Detector (O1 vs O3) ---");
    println!("Confidence: {:.4}", res_tradeoff.confidence);
    println!("Severity  : {:.4}", res_tradeoff.severity);
    for m in &res_tradeoff.supporting_metrics {
        println!("  {}: {:.4}", m.name, m.value);
    }

    // 3. EcologyLockInDetector
    let lockin_detector = EcologyLockInDetector::new(0, 0.15, 100);
    let res_lockin = lockin_detector.evaluate(&telemetry_stream);
    println!("\n--- Ecology Lock-In Detector ---");
    println!("Confidence: {:.4}", res_lockin.confidence);
    println!("Severity  : {:.4}", res_lockin.severity);
    for m in &res_lockin.supporting_metrics {
        println!("  {}: {:.4}", m.name, m.value);
    }

    // 4. AccumulationFailureDetector
    let accum_detector = AccumulationFailureDetector::new(0, 0.75);
    let res_accum = accum_detector.evaluate(&telemetry_stream);
    println!("\n--- Accumulation Failure Detector ---");
    println!("Confidence: {:.4}", res_accum.confidence);
    println!("Severity  : {:.4}", res_accum.severity);
    for m in &res_accum.supporting_metrics {
        println!("  {}: {:.4}", m.name, m.value);
    }

    // 5. ProxySuppressionDetector
    // Target index is 0, Proxy is O3 (Successions, index 3)
    let suppression_detector = ProxySuppressionDetector::new(0, vec![3], 0.80);
    let res_suppression = suppression_detector.evaluate(&telemetry_stream);
    println!("\n--- Proxy Suppression Detector (Target vs O3) ---");
    println!("Confidence: {:.4}", res_suppression.confidence);
    println!("Severity  : {:.4}", res_suppression.severity);
    for m in &res_suppression.supporting_metrics {
        println!("  {}: {:.4}", m.name, m.value);
    }

    // ── Generate & Print Recommendations ──────────────────────────────────────────
    println!("\n=== Search Governance Recommendations ===");
    let recommender = EcologyRecommender::new(0.5, 100);
    let report = recommender.recommend(&ecology_state);

    if report.recommendations.is_empty() {
        println!("No recommendations triggered (all confidence levels below threshold).");
    } else {
        println!(
            "Generated at generation {} (using last {} generations history):",
            report.generated_at_generation, recommender.evaluation_window
        );
        for (i, rec) in report.recommendations.iter().enumerate() {
            println!("\n[Recommendation {}]", i + 1);
            println!("  Action:     {}", rec.action);
            println!("  Rationale:  {}", rec.rationale);
            println!("  Confidence: {:.4}", rec.confidence);
            println!("  Evidence:");
            for ev in &rec.evidence {
                println!("    - {}", ev);
            }
        }
    }
}
