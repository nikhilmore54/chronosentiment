/// Multi-seed historical runner for Coralys policy validation.
///
/// For each seed, this binary:
///   1. Loads the canonical INRC-II instance (n050w4).
///   2. Runs the seeded MOGA evolution loop.
///   3. Captures SearchObservation telemetry and builds EcologyState.
///   4. Calls EcologyRecommender to produce a RecommendationReport.
///   5. Calls RecommendationMirrorPolicy to produce a PolicyDecision.
///   6. Writes a JSON file `seed_{N}_report.json` for each seed.
///   7. Prints a concise stdout summary and aggregated statistics.
///
/// The output corpus is the first governance dataset for coralys-policy.
/// Its primary purpose is to answer: do similar pathologies produce consistent decisions?
use ultracrew::inrc::parser::{parse_scenario, parse_week_data, parse_history};
use ultracrew::inrc::optimization::{InrcContext, InrcOptimizer};
use ultracrew_server::simulation::generate_baseline_schedule;
use ultracrew_server::optimizer::{ScheduleGenome, UltraCrewEvaluator, UltraCrewMutator};
use ultracrew_server::inrc_observer::score_inrc_official;
use coralys_moga::engine_proof::{EvolutionEngine, ParetoSolution, Evaluator};
use coralys_ecology::diagnostics::{
    SearchObservation, CandidateObservation, ObjectiveVector, DiagnosticResult, DiagnosticDetector,
    AttractorDetector, TradeoffBasinDetector, EcologyLockInDetector,
    AccumulationFailureDetector, ProxySuppressionDetector, EcologyState,
};
use coralys_recommendation::EcologyRecommender;
use coralys_policy::{RecommendationMirrorPolicy, PolicyEngine, PolicyDecision};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use coralys_core::memory::InnovationTracker;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::distributions::{WeightedIndex, Distribution};
use rand::Rng;

const INSTANCE: &str = "n050w4";
const MAX_GENERATIONS: usize = 1000;
const EVAL_WINDOW: usize = 100;
const OBS_WINDOW: usize = 200;

/// Full per-seed report written to JSON.
/// Preserves the complete diagnostic → recommendation → decision chain.
#[derive(Serialize, Deserialize, Debug)]
struct SeedRunReport {
    seed: u64,
    total_generations: usize,
    /// Raw detector results from the final evaluation window.
    diagnostics: HashMap<String, DiagnosticResult>,
    memory_ecology: coralys_core::memory::InnovationTelemetry,
    recommendation: coralys_recommendation::RecommendationReport,
    decision: PolicyDecision,
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn main() {
    let seeds: Vec<u64> = vec![61, 42, 100, 999, 12_345];

    println!("=== Coralys Policy Validation – Multi-Seed Historical Runner ===");
    println!("Instance    : {}", INSTANCE);
    println!("Generations : {}", MAX_GENERATIONS);
    println!("Seeds       : {:?}", seeds);
    println!();

    // Load scenario data once — types are inferred, not named, so they remain private.
    let base_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../adapters/ultracrew/tests/data/{}", INSTANCE));
    let scenario  = parse_scenario(base_dir.join(format!("Sc-{}.json",  INSTANCE))).unwrap();
    let week_data = parse_week_data(base_dir.join(format!("WD-{}-0.json", INSTANCE))).unwrap();
    let history   = parse_history(base_dir.join(format!("H0-{}-0.json",  INSTANCE))).unwrap();

    let mut all_reports: Vec<SeedRunReport> = Vec::new();

    for seed in &seeds {
        println!("--- Running seed {} ---", seed);

        // ── Engine setup ────────────────────────────────────────────────────────
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

        // Seed archive with baseline schedule
        let baseline   = generate_baseline_schedule(&scenario, &week_data.requirements).unwrap();
        let base_fitness = engine.evaluator.evaluate(&baseline);
        let base_uid   = calculate_hash(&baseline);
        engine.archive.add(ParetoSolution {
            genome: baseline, fitness: base_fitness, uid: base_uid, parent_uid: 0,
        });

        let mut telemetry_stream: Vec<SearchObservation> = Vec::new();
        let mut ecology_state = EcologyState::new(MAX_GENERATIONS);

        let attractor_detector   = AttractorDetector::default();
        let tradeoff_detector    = TradeoffBasinDetector::new(1, 3, -0.5);
        let lockin_detector      = EcologyLockInDetector::new(0, 0.15, 100);
        let accum_detector       = AccumulationFailureDetector::new(0, 0.75);
        let suppression_detector = ProxySuppressionDetector::new(0, vec![3], 0.80);

        let mut innovation_tracker = InnovationTracker::new();
        let mut final_innovation_telemetry = coralys_core::memory::InnovationTelemetry::default();
        
        let mut progress_tracker = coralys_ecology::progress::ProgressTracker::new();

        let get_obj_vec = |fit: &[f64], official_hc: f64| -> ObjectiveVector {
            let mut values = vec![official_hc];
            values.extend_from_slice(fit);
            ObjectiveVector::new(values)
        };

        // ── Evolution loop ──────────────────────────────────────────────────────
        for g in 1..=MAX_GENERATIONS {
            let archive_size = engine.archive.solutions.len();
            if archive_size == 0 { break; }
            let num_objs = engine.archive.solutions[0].fitness.len();

            // Crowding-distance parent selection
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

            // Generate offspring with simulated annealing refinement
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

            let child_score  = score_inrc_official(&child_genome, &scenario, &inrc_optimizer);
            let parent_score = score_inrc_official(&parent.genome, &scenario, &inrc_optimizer);

            let parent_hc = (parent_score.hc_coverage + parent_score.hc_skills
                + parent_score.hc_one_shift_per_day + parent_score.hc_forbidden_successions) as f64;
            let child_hc = (child_score.hc_coverage + child_score.hc_skills
                + child_score.hc_one_shift_per_day + child_score.hc_forbidden_successions) as f64;

            let was_inserted = engine.archive.add(ParetoSolution {
                genome: child_genome.clone(), fitness: child_fitness.clone(),
                uid: child_uid, parent_uid: parent.uid,
            });

            let candidate = CandidateObservation {
                objectives: get_obj_vec(&child_fitness, child_hc),
                admitted:   was_inserted,
                feasible:   child_score.feasible,
                parent_objectives: Some(get_obj_vec(&parent.fitness, parent_hc)),
            };
            let archive_objectives: Vec<ObjectiveVector> = engine.archive.solutions.iter()
                .map(|sol| {
                    let sc = score_inrc_official(&sol.genome, &scenario, &inrc_optimizer);
                    let hc = (sc.hc_coverage + sc.hc_skills
                        + sc.hc_one_shift_per_day + sc.hc_forbidden_successions) as f64;
                    get_obj_vec(&sol.fitness, hc)
                })
                .collect();
                
            let gen_signatures: Vec<u64> = engine.archive.solutions.iter()
                .flat_map(|sol| sol.genome.signatures())
                .collect();
                
            let innovation_telemetry = innovation_tracker.observe(&gen_signatures);
            final_innovation_telemetry = innovation_telemetry.clone();
            
            let global_best_fitness = engine.archive.solutions[0].fitness[0]; // Wait, is fitness[0] the main objective (cost)? Yes, for UltraCrew it's cost.
            let progress_telemetry = progress_tracker.observe_minimization(g, global_best_fitness);

            let attachments = vec![
                coralys_core::telemetry::TelemetryAttachment {
                    namespace: "memory.ecology.v1".to_string(),
                    version: 1,
                    payload: serde_json::to_value(&innovation_telemetry).unwrap(),
                },
                coralys_core::telemetry::TelemetryAttachment {
                    namespace: "progress.ecology.v1".to_string(),
                    version: 1,
                    payload: serde_json::to_value(&progress_telemetry).unwrap(),
                }
            ];

            telemetry_stream.push(SearchObservation {
                generation: g, archive_size, diversity_score: 0.0,
                candidates: vec![candidate], archive_objectives,
                telemetry: Some(coralys_core::telemetry::SearchTelemetry {
                    generation: g,
                    population: coralys_core::telemetry::PopulationTelemetry {
                        best_fitness: engine.archive.solutions[0].fitness[0],
                        median_fitness: 0.0,
                        worst_fitness: 0.0,
                        diversity_score: 0.0,
                        elite_diversity_score: 0.0,
                        feasible_ratio: 1.0,
                    },
                    attachments,
                }),
            });

            // Record per-generation diagnostics into EcologyState
            let start_idx = telemetry_stream.len().saturating_sub(OBS_WINDOW);
            let obs_slice = &telemetry_stream[start_idx..];
            let mut results = HashMap::new();
            results.insert("Attractor".to_string(),           attractor_detector.evaluate(obs_slice));
            results.insert("TradeoffBasin".to_string(),       tradeoff_detector.evaluate(obs_slice));
            results.insert("EcologyLockIn".to_string(),       lockin_detector.evaluate(obs_slice));
            results.insert("AccumulationFailure".to_string(), accum_detector.evaluate(obs_slice));
            results.insert("ProxySuppression".to_string(),    suppression_detector.evaluate(obs_slice));
            ecology_state.record(g, results);

            if g % 200 == 0 {
                println!("  Generation {:>4} / {}...", g, MAX_GENERATIONS);
            }
        }

        // ── Final diagnostics snapshot ──────────────────────────────────────────
        let final_start = telemetry_stream.len().saturating_sub(OBS_WINDOW);
        let final_obs   = &telemetry_stream[final_start..];
        let mut final_diagnostics = HashMap::new();
        final_diagnostics.insert("Attractor".to_string(),           attractor_detector.evaluate(final_obs));
        final_diagnostics.insert("TradeoffBasin".to_string(),       tradeoff_detector.evaluate(final_obs));
        final_diagnostics.insert("EcologyLockIn".to_string(),       lockin_detector.evaluate(final_obs));
        final_diagnostics.insert("AccumulationFailure".to_string(), accum_detector.evaluate(final_obs));
        final_diagnostics.insert("ProxySuppression".to_string(),    suppression_detector.evaluate(final_obs));

        // ── Recommendation ──────────────────────────────────────────────────────
        let recommender = EcologyRecommender::new(0.5, EVAL_WINDOW);
        let rec_report  = recommender.recommend(&ecology_state);

        // ── Policy decision (read-only mirror) ──────────────────────────────────
        let policy   = RecommendationMirrorPolicy;
        let decision = policy.evaluate(&ecology_state, &rec_report);

        // ── Stdout summary ──────────────────────────────────────────────────────
        println!("  Generations completed : {}", telemetry_stream.len());
        println!("  Policy action         : {}", decision.action);
        println!("  Policy confidence     : {:.4}", decision.confidence);
        for (i, r) in decision.rationales.iter().enumerate() {
            println!("  Rationale [{}]         : {}", i, r);
        }
        if !rec_report.recommendations.is_empty() {
            println!("  Recommendations       : {}", rec_report.recommendations.len());
        } else {
            println!("  Recommendations       : none (all detectors below threshold)");
        }

        // ── Write per-seed JSON ─────────────────────────────────────────────────
        let report = SeedRunReport {
            seed: *seed,
            total_generations: telemetry_stream.len(),
            diagnostics: final_diagnostics,
            memory_ecology: final_innovation_telemetry,
            recommendation: rec_report,
            decision,
        };
        let filename = format!("seed_{}_report.json", seed);
        let json = serde_json::to_string_pretty(&report).expect("JSON serialization failed");
        std::fs::write(&filename, &json).expect("Failed to write seed JSON");
        println!("  Written: {}\n", filename);

        all_reports.push(report);
    }

    // ── Aggregated statistics ────────────────────────────────────────────────────
    let total_runs = all_reports.len();
    let mut action_counts:     HashMap<String, usize> = HashMap::new();
    let mut action_confidence: HashMap<String, f64>   = HashMap::new();
    let mut confidence_sum = 0.0_f64;

    for r in &all_reports {
        *action_counts.entry(r.decision.action.clone()).or_insert(0) += 1;
        *action_confidence.entry(r.decision.action.clone()).or_insert(0.0) += r.decision.confidence;
        confidence_sum += r.decision.confidence;
    }

    let avg_conf = if total_runs > 0 { confidence_sum / total_runs as f64 } else { 0.0 };

    let confidence_by_action: HashMap<String, f64> = action_confidence
        .iter()
        .map(|(k, v)| (k.clone(), v / *action_counts.get(k).unwrap_or(&1) as f64))
        .collect();

    let highest_conf_action = confidence_by_action
        .iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(k, _)| k.clone());

    let summary = serde_json::json!({
        "total_runs": total_runs,
        "action_frequency": action_counts,
        "average_confidence": avg_conf,
        "highest_confidence_action": highest_conf_action,
        "confidence_by_action": confidence_by_action,
    });
    let summary_json = serde_json::to_string_pretty(&summary).expect("Summary JSON failed");
    std::fs::write("policy_validation_summary.json", &summary_json)
        .expect("Failed to write summary JSON");

    println!("=== Policy Validation Summary ===");
    println!("Total runs           : {}", total_runs);
    println!("Average confidence   : {:.4}", avg_conf);
    if let Some(ref top) = highest_conf_action {
        println!("Highest conf action  : {}", top);
    }
    println!("Action frequencies:");
    for (action, count) in &action_counts {
        let pct   = *count as f64 / total_runs as f64 * 100.0;
        let avg_c = confidence_by_action.get(action).copied().unwrap_or(0.0);
        println!("  [{:>3.0}%  conf={:.3}]  {}", pct, avg_c, action);
    }
    println!();
    println!("Written: policy_validation_summary.json");
    println!();
    println!("Policy stability check:");
    println!("  Do similar pathologies produce consistent decisions?");
    let all_same = all_reports.windows(2).all(|w| w[0].decision.action == w[1].decision.action);
    if all_same && total_runs > 0 {
        println!("  ✅ YES – all {} seeds produced the same policy action.", total_runs);
    } else {
        println!("  ⚠️  Actions diverged across seeds. Review per-seed JSONs.");
        for r in &all_reports {
            println!("    seed {:>6} → {}", r.seed, r.decision.action);
        }
    }
}
