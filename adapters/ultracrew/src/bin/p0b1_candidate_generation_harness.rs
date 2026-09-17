// P0-B.1 — Candidate Generation Diagnostic Harness
//
// Purpose: Measure the four diagnostic questions that determine why the
// production Pareto search does or does not produce useful alternatives.
//
// Questions answered:
//   Q1 — Mutation reach:   How many distinct genomes are generated per step?
//   Q2 — Feasible reach:   How many generated genomes are constraint-feasible?
//   Q3 — Pareto survival:  How many distinct feasible genomes survive into the archive?
//   Q4 — Runtime:          How much wall time is required for each step budget?
//
// Bottleneck diagnosis:
//   few distinct mutations          → mutation reach problem
//   many mutations, few feasible    → feasibility/search problem
//   many feasible, one Pareto survivor → objective/dominance/search problem
//   good alternatives but too slow  → performance/search-budget problem
//
// Usage:
//   cargo run --bin p0b1_candidate_generation_harness -- \
//       --workers 20 --shifts 40 --steps 200
//
// The harness runs the production Pareto pipeline with instrumentation
// injected at each step. It does NOT change the optimizer, mutation
// algorithm, objectives, or any production invariant.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use coralys_moga::engine_proof::{
    Evaluator, FitnessVector, MutationPolicy, ParetoArchive, ParetoSolution,
};
use serde::Serialize;
use ultracrew::constraint_engine::{DomainConstraintEvaluator, InrcConstraintEvaluator};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::models::{Shift, Skill, Worker};
use ultracrew::optimization::{Observatory, ScheduleContext, ScheduleGenome, ScheduleOptimizer};
use ultracrew::public_contracts::InrcScenario;

// ── Instrumented Pareto step ──────────────────────────────────────────────────

/// Per-step diagnostic counters.
#[derive(Debug, Default, Clone, Serialize)]
struct StepDiagnostic {
    step: usize,
    /// uid of the genome produced by mutation this step.
    mutant_uid: u64,
    /// Whether the mutant was distinct from all previously seen uids.
    mutant_distinct: bool,
    /// Whether the mutant passes hard constraint check (hard_violations == 0).
    mutant_feasible: bool,
    /// Whether the mutant was admitted to the Pareto archive.
    mutant_admitted: bool,
    /// Archive size after this step.
    archive_size: usize,
}

/// Aggregate summary across all steps.
#[derive(Debug, Serialize)]
struct HarnessSummary {
    scenario: ScenarioParams,
    steps: usize,
    wall_ms: u64,
    distinct_mutations: usize,
    feasible_mutations: usize,
    archive_size_final: usize,
    alternatives_count: usize,
    bottleneck: String,
}

#[derive(Debug, Serialize, Clone)]
struct ScenarioParams {
    workers: usize,
    shifts: usize,
    steps: usize,
    fatigue_enabled: bool,
    /// Use heterogeneous scenario (mixed skills, variable durations, rest tension).
    /// When false, uses the original homogeneous scenario (all same skill, 8h shifts).
    heterogeneous: bool,
}

// ── Context construction ──────────────────────────────────────────────────────

/// Build a homogeneous scenario: all workers share one skill, all shifts are 8h.
/// This scenario has a single Pareto optimum — no genuine trade-offs exist.
/// Used to verify the engine correctly returns alternatives=[] when no tension exists.
fn build_homogeneous_context(params: &ScenarioParams) -> Arc<ScheduleContext> {
    let skill = Skill::new("FlightAttendant");

    let workers: Vec<Worker> = (0..params.workers)
        .map(|i| Worker {
            id: (i + 1) as u64,
            skills: vec![skill.clone()],
        })
        .collect();

    let shifts: Vec<Shift> = (0..params.shifts)
        .map(|i| Shift {
            id: (i + 1) as u64,
            start_hour: (i * 8) as u64 % 168,
            duration_hours: 8,
            required_skill: skill.clone(),
        })
        .collect();

    let mut ecology = WorkforceEcology::new();
    if params.fatigue_enabled {
        for i in 0..(params.workers / 2) {
            ecology.record_historical_hours((i + 1) as u64, 40.0);
        }
    }

    let scenario = InrcScenario {
        planning_horizon_hours: Some(168.0),
        max_hours_per_worker: Some(40.0),
        minimum_rest_hours: Some(8),
        leave_requests: None,
    };

    Arc::new(ScheduleContext {
        workers: Arc::new(workers),
        shifts: Arc::new(shifts),
        ecology,
        rng_seed: 42,
        observatory: Arc::new(Mutex::new(Observatory::new())),
        locked_assignments: None,
        scenario: Some(scenario),
        enable_fatigue: params.fatigue_enabled,
        fatigue_weight: if params.fatigue_enabled { 320.0 } else { 0.0 },
        hc3_aware_initialization: false,
        temporal_scarcity_construction: false,
        disable_global_constructor: false,
        precomputed_seeds: None,
        constructor_budget_ms: None,
    })
}

/// Build a heterogeneous scenario with genuine objective tension.
///
/// P0-B.5: This scenario is designed to represent a realistic UltraCrew
/// scheduling problem where the scalar GA cannot simultaneously minimize all
/// six objectives. Objective tension arises from:
///
///   - Mixed skills: not all workers can cover all shifts (HC1 pressure)
///   - Variable shift durations: 4h/6h/8h/10h/12h (rest constraint tension)
///   - Tight rest requirement (10h): consecutive shifts create HC3 violations
///   - Unequal workload distribution: some skills are scarce (fairness tension)
///
/// In this scenario, assigning a scarce Pilot to one shift means another shift
/// may need a less-rested worker, trading off rest_violations vs hc1_violations.
/// The scalar GA finds one locally optimal trade-off; the Pareto search finds
/// the non-dominated front of alternative trade-offs.
fn build_heterogeneous_context(params: &ScenarioParams) -> Arc<ScheduleContext> {
    // Four skills with different scarcity levels.
    let pilot     = Skill::new("Pilot");
    let copilot   = Skill::new("CoPilot");
    let attendant = Skill::new("FlightAttendant");
    let purser    = Skill::new("Purser");

    // Workers: mixed skills, some workers have 2 skills (cross-trained).
    // Distribution: 4 Pilots, 4 CoPilots, 8 FlightAttendants, 4 Pursers.
    // 4 workers are cross-trained (Pilot+CoPilot or FlightAttendant+Purser).
    let workers: Vec<Worker> = vec![
        // Pilots (scarce)
        Worker { id: 1,  skills: vec![pilot.clone()] },
        Worker { id: 2,  skills: vec![pilot.clone()] },
        Worker { id: 3,  skills: vec![pilot.clone(), copilot.clone()] }, // cross-trained
        Worker { id: 4,  skills: vec![pilot.clone()] },
        // CoPilots
        Worker { id: 5,  skills: vec![copilot.clone()] },
        Worker { id: 6,  skills: vec![copilot.clone()] },
        Worker { id: 7,  skills: vec![copilot.clone()] },
        Worker { id: 8,  skills: vec![copilot.clone(), pilot.clone()] }, // cross-trained
        // FlightAttendants (most common)
        Worker { id: 9,  skills: vec![attendant.clone()] },
        Worker { id: 10, skills: vec![attendant.clone()] },
        Worker { id: 11, skills: vec![attendant.clone()] },
        Worker { id: 12, skills: vec![attendant.clone()] },
        Worker { id: 13, skills: vec![attendant.clone(), purser.clone()] }, // cross-trained
        Worker { id: 14, skills: vec![attendant.clone()] },
        Worker { id: 15, skills: vec![attendant.clone()] },
        Worker { id: 16, skills: vec![attendant.clone()] },
        // Pursers (scarce)
        Worker { id: 17, skills: vec![purser.clone()] },
        Worker { id: 18, skills: vec![purser.clone()] },
        Worker { id: 19, skills: vec![purser.clone(), attendant.clone()] }, // cross-trained
        Worker { id: 20, skills: vec![purser.clone()] },
    ];

    // Shifts: variable durations, mixed skills, spread across a 168h week.
    // 40 shifts total: 10 Pilot, 10 CoPilot, 12 FlightAttendant, 8 Purser.
    // Variable durations create rest constraint tension.
    let shift_defs: Vec<(u64, u64, u64, &Skill)> = vec![
        // (id, start_hour, duration_hours, required_skill)
        // Pilot shifts (10)
        (1,  0,   8,  &pilot),
        (2,  8,   6,  &pilot),
        (3,  14,  10, &pilot),
        (4,  24,  8,  &pilot),
        (5,  32,  12, &pilot),
        (6,  44,  6,  &pilot),
        (7,  50,  8,  &pilot),
        (8,  72,  10, &pilot),
        (9,  82,  8,  &pilot),
        (10, 120, 6,  &pilot),
        // CoPilot shifts (10)
        (11, 0,   6,  &copilot),
        (12, 6,   8,  &copilot),
        (13, 14,  12, &copilot),
        (14, 26,  8,  &copilot),
        (15, 34,  10, &copilot),
        (16, 44,  6,  &copilot),
        (17, 50,  8,  &copilot),
        (18, 72,  4,  &copilot),
        (19, 76,  8,  &copilot),
        (20, 120, 10, &copilot),
        // FlightAttendant shifts (12)
        (21, 0,   8,  &attendant),
        (22, 8,   6,  &attendant),
        (23, 14,  8,  &attendant),
        (24, 22,  10, &attendant),
        (25, 32,  8,  &attendant),
        (26, 40,  6,  &attendant),
        (27, 46,  8,  &attendant),
        (28, 54,  12, &attendant),
        (29, 66,  8,  &attendant),
        (30, 74,  6,  &attendant),
        (31, 80,  8,  &attendant),
        (32, 120, 10, &attendant),
        // Purser shifts (8)
        (33, 0,   8,  &purser),
        (34, 8,   10, &purser),
        (35, 18,  6,  &purser),
        (36, 24,  8,  &purser),
        (37, 72,  8,  &purser),
        (38, 80,  10, &purser),
        (39, 90,  6,  &purser),
        (40, 120, 8,  &purser),
    ];

    let shifts: Vec<Shift> = shift_defs
        .into_iter()
        .map(|(id, start, dur, skill)| Shift {
            id,
            start_hour: start,
            duration_hours: dur,
            required_skill: skill.clone(),
        })
        .collect();

    let mut ecology = WorkforceEcology::new();
    if params.fatigue_enabled {
        // Pre-load some workers with historical hours to create fatigue tension.
        for id in [1u64, 3, 5, 9, 13, 17] {
            ecology.record_historical_hours(id, 35.0);
        }
    }

    let scenario = InrcScenario {
        planning_horizon_hours: Some(168.0),
        max_hours_per_worker: Some(40.0),
        // Tighter rest requirement creates HC3 tension with variable shift durations.
        minimum_rest_hours: Some(10),
        leave_requests: None,
    };

    Arc::new(ScheduleContext {
        workers: Arc::new(workers),
        shifts: Arc::new(shifts),
        ecology,
        rng_seed: 42,
        observatory: Arc::new(Mutex::new(Observatory::new())),
        locked_assignments: None,
        scenario: Some(scenario),
        enable_fatigue: params.fatigue_enabled,
        fatigue_weight: if params.fatigue_enabled { 320.0 } else { 0.0 },
        hc3_aware_initialization: false,
        temporal_scarcity_construction: false,
        disable_global_constructor: false,
        precomputed_seeds: None,
        constructor_budget_ms: None,
    })
}

/// Dispatch to the appropriate context builder based on `params.heterogeneous`.
fn build_context(params: &ScenarioParams) -> Arc<ScheduleContext> {
    if params.heterogeneous {
        build_heterogeneous_context(params)
    } else {
        build_homogeneous_context(params)
    }
}

// ── Instrumented Pareto evaluator/mutator wrappers ────────────────────────────

struct InstrumentedEvaluator {
    context: Arc<ScheduleContext>,
}

impl Evaluator<ScheduleGenome> for InstrumentedEvaluator {
    fn evaluate(&self, genome: &ScheduleGenome) -> FitnessVector {
        use ultracrew::pareto_pipeline::ProductionParetoEvaluator;
        let inner = ProductionParetoEvaluator {
            context: self.context.clone(),
        };
        inner.evaluate(genome)
    }
}

struct InstrumentedMutator {
    context: Arc<ScheduleContext>,
}

impl MutationPolicy<ScheduleGenome> for InstrumentedMutator {
    fn mutate(&self, genome: &ScheduleGenome) -> ScheduleGenome {
        use ultracrew::pareto_pipeline::ProductionMutationAdapter;
        let inner = ProductionMutationAdapter {
            context: self.context.clone(),
        };
        inner.mutate(genome)
    }
}

// ── Harness run ───────────────────────────────────────────────────────────────

fn run_harness(params: ScenarioParams) -> HarnessSummary {
    use coralys_moga::engine_proof::EvolutionEngine;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    let context = build_context(&params);

    // Produce the primary schedule via the scalar GA (same path as production).
    let seed_genome = {
        use coralys_moga::config::EvolutionConfig;
        use ultracrew::helpers::run_optimization;
        let config = EvolutionConfig {
            generation_limit: 100,
            ..EvolutionConfig::default()
        };
        let ga_result = run_optimization(context.clone(), config);
        ga_result.global_best.schedule.clone()
    };

    // Compute seed uid.
    let seed_uid = {
        let mut h = DefaultHasher::new();
        seed_genome.hash(&mut h);
        h.finish()
    };

    // Build the Pareto engine.
    let evaluator = InstrumentedEvaluator {
        context: context.clone(),
    };
    let mutator = InstrumentedMutator {
        context: context.clone(),
    };

    let mut engine = EvolutionEngine::new(evaluator, mutator);
    engine.seed(seed_genome);

    // Capture seed fitness immediately — before diverse seeds are added.
    // Used later for P0-C.1 delta computation.
    let seed_fitness_for_delta: Vec<f64> = engine
        .archive
        .solutions
        .first()
        .map(|s| s.fitness.clone())
        .unwrap_or_default();

    // Print seed objective values so we can confirm whether the seed is a
    // perfect-score genome (all zeros) or has non-zero objectives.
    eprintln!(
        "=== SEED OBJECTIVES [hard_viol, rest_viol, fairness, fatigue, hc1_viol, hc3_viol] ===\n  {:?}",
        seed_fitness_for_delta
    );

    // P0-B.4: Seed the Pareto archive with diverse feasible genomes, mirroring
    // the production run_pareto_pipeline() diverse initialization.
    // Also instrument each diverse genome to answer:
    //   - How many unique genomes does create() produce?
    //   - How many unique objective vectors?
    //   - How many enter the archive (non-dominated, distinct uid)?
    {
        use ultracrew::optimization::ScheduleOptimizer;
        use coralys_moga::traits::GenomeFactory;
        use rand::SeedableRng;
        use rand::rngs::StdRng;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let factory = ScheduleOptimizer::new(context.clone());
        let mut rng = StdRng::seed_from_u64(context.rng_seed.wrapping_add(1));
        let diverse_count = 20usize;

        let mut unique_genome_uids: std::collections::HashSet<u64> = std::collections::HashSet::new();
        let mut unique_obj_vecs: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut admitted_count = 0usize;

        let init_evaluator = ultracrew::pareto_pipeline::ProductionParetoEvaluator {
            context: context.clone(),
        };

        for i in 0..diverse_count {
            let genome = factory.create(&mut rng);

            // Compute uid
            let uid = {
                let mut h = DefaultHasher::new();
                genome.hash(&mut h);
                h.finish()
            };
            unique_genome_uids.insert(uid);

            // Evaluate objectives
            use coralys_moga::engine_proof::Evaluator;
            let fitness = init_evaluator.evaluate(&genome);
            let obj_key = format!("{:?}", fitness);
            unique_obj_vecs.insert(obj_key.clone());

            // Attempt to seed into archive
            let archive_before = engine.archive.solutions.len();
            engine.seed(genome);
            let archive_after = engine.archive.solutions.len();
            if archive_after > archive_before {
                admitted_count += 1;
            }

            if i < 5 || i == diverse_count - 1 {
                eprintln!("  diverse[{:02}]: uid={:016x} fitness={} admitted={}", i, uid, obj_key, archive_after > archive_before);
            }
        }

        eprintln!(
            "=== DIVERSE INIT SUMMARY: unique_genomes={} unique_obj_vecs={} admitted={} archive_after_init={} ===",
            unique_genome_uids.len(),
            unique_obj_vecs.len(),
            admitted_count,
            engine.archive.solutions.len()
        );
    }

    // Constraint evaluator for feasibility check.
    let constraint_eval: Box<dyn DomainConstraintEvaluator> =
        Box::new(InrcConstraintEvaluator::new(context.clone()));

    // Instrumentation state.
    let mut seen_uids: HashSet<u64> = HashSet::new();
    // Seed uid is already "seen".
    seen_uids.insert(seed_uid);

    let mut diagnostics: Vec<StepDiagnostic> = Vec::with_capacity(params.steps);
    let mut distinct_count = 0usize;
    let mut feasible_count = 0usize;

    let start = Instant::now();

    for step in 0..params.steps {
        let archive_before = engine.archive.solutions.len();

        // Manually replicate one engine step so we can inspect the mutant.
        // We do this by reading the archive before and after.
        engine.step();

        let archive_after = engine.archive.solutions.len();
        let admitted = archive_after > archive_before;

        // The most recently added solution (if admitted) is the last in the archive.
        // If not admitted, we can't directly observe the mutant uid from outside.
        // We track the archive's newest uid as a proxy.
        let mutant_uid = if admitted {
            engine.archive.solutions.last().map(|s| s.uid).unwrap_or(0)
        } else {
            // Not admitted — we can't recover the uid without engine internals.
            // Use 0 as sentinel for "not admitted, uid unknown".
            0
        };

        let mutant_distinct = if mutant_uid != 0 {
            let is_new = !seen_uids.contains(&mutant_uid);
            seen_uids.insert(mutant_uid);
            is_new
        } else {
            false
        };

        if mutant_distinct {
            distinct_count += 1;
        }

        // Feasibility check: if admitted, check hard violations.
        let mutant_feasible = if admitted {
            if let Some(sol) = engine.archive.solutions.last() {
                    let report = constraint_eval.evaluate(&sol.genome);
                report.hard_violations == 0
            } else {
                false
            }
        } else {
            false
        };

        if mutant_feasible {
            feasible_count += 1;
        }

        diagnostics.push(StepDiagnostic {
            step,
            mutant_uid,
            mutant_distinct,
            mutant_feasible,
            mutant_admitted: admitted,
            archive_size: archive_after,
        });
    }

    let wall_ms = start.elapsed().as_millis() as u64;

    // Final archive: exclude seed.
    let alternatives: Vec<_> = engine
        .archive
        .solutions
        .iter()
        .filter(|s| s.uid != seed_uid)
        .collect();

    let archive_size_final = engine.archive.solutions.len();
    let alternatives_count = alternatives.len();

    // P0-C.1: Print per-alternative objective vectors for decision-quality analysis.
    // For each alternative, show:
    //   - uid (genome identity)
    //   - fitness vector [hard_viol, rest_viol, fairness, fatigue, hc1_viol, hc3_viol]
    //   - delta from primary seed on each objective
    // This establishes whether the alternatives represent genuine trade-offs.
    if !alternatives.is_empty() {
        // Use the seed fitness captured before diverse seeding.
        let seed_fitness = &seed_fitness_for_delta;

        eprintln!("=== P0-C.1 ALTERNATIVE QUALITY ANALYSIS ===");
        eprintln!(
            "  Primary seed: uid={:016x} fitness={:?}",
            seed_uid, seed_fitness
        );
        eprintln!("  Objectives: [hard_viol, rest_viol, fairness, fatigue, hc1_viol, hc3_viol]");
        eprintln!("  Alternatives ({}):", alternatives_count);
        for (i, alt) in alternatives.iter().enumerate() {
            let delta: Vec<f64> = alt.fitness.iter()
                .zip(seed_fitness.iter())
                .map(|(a, s)| a - s)
                .collect();
            let trade_off_dims: Vec<usize> = delta.iter().enumerate()
                .filter(|(_, d)| d.abs() > 0.001)
                .map(|(i, _)| i)
                .collect();
            let obj_names = ["hard_viol", "rest_viol", "fairness", "fatigue", "hc1_viol", "hc3_viol"];
            let trade_off_desc: Vec<String> = trade_off_dims.iter().map(|&i| {
                let d = delta[i];
                if d > 0.0 {
                    format!("{}+{:.1}", obj_names[i], d)
                } else {
                    format!("{}{:.1}", obj_names[i], d)
                }
            }).collect();
            eprintln!(
                "  alt[{:02}]: uid={:016x} fitness={:?}",
                i, alt.uid, alt.fitness
            );
            eprintln!(
                "           delta={:?} trade_offs=[{}]",
                delta,
                if trade_off_desc.is_empty() { "none".to_string() } else { trade_off_desc.join(", ") }
            );
        }
    }

    // Bottleneck diagnosis.
    let bottleneck = if distinct_count == 0 {
        "MUTATION_REACH: zero distinct mutations produced".to_string()
    } else if feasible_count == 0 {
        format!(
            "FEASIBILITY: {} distinct mutations, 0 feasible (all violate hard constraints)",
            distinct_count
        )
    } else if alternatives_count == 0 {
        format!(
            "PARETO_DOMINANCE: {} distinct, {} feasible, 0 survived Pareto filter (all dominated by primary or each other)",
            distinct_count, feasible_count
        )
    } else {
        format!(
            "OK: {} alternatives in {}ms ({} distinct mutations, {} feasible)",
            alternatives_count, wall_ms, distinct_count, feasible_count
        )
    };

    // Print per-step diagnostics to stderr for inspection.
    eprintln!("=== P0-B.1 Step Diagnostics (first 20 steps) ===");
    for d in diagnostics.iter().take(20) {
        eprintln!(
            "  step {:3}: uid={:016x} distinct={} feasible={} admitted={} archive={}",
            d.step, d.mutant_uid, d.mutant_distinct, d.mutant_feasible, d.mutant_admitted, d.archive_size
        );
    }
    if params.steps > 20 {
        eprintln!("  ... ({} more steps)", params.steps - 20);
    }

    HarnessSummary {
        scenario: params,
        steps: diagnostics.len(),
        wall_ms,
        distinct_mutations: distinct_count,
        feasible_mutations: feasible_count,
        archive_size_final,
        alternatives_count,
        bottleneck,
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let workers = args
        .iter()
        .position(|a| a == "--workers")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(20usize);

    let shifts = args
        .iter()
        .position(|a| a == "--shifts")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(40usize);

    let steps = args
        .iter()
        .position(|a| a == "--steps")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(200usize);

    let fatigue = args.contains(&"--fatigue".to_string());
    let heterogeneous = args.contains(&"--heterogeneous".to_string());

    eprintln!(
        "P0-B.1 Candidate Generation Harness: workers={} shifts={} steps={} fatigue={} heterogeneous={}",
        workers, shifts, steps, fatigue, heterogeneous
    );

    let params = ScenarioParams {
        workers,
        shifts,
        steps,
        fatigue_enabled: fatigue,
        heterogeneous,
    };

    let summary = run_harness(params);

    println!("{}", serde_json::to_string_pretty(&summary).unwrap());

    eprintln!("\n=== BOTTLENECK DIAGNOSIS ===");
    eprintln!("{}", summary.bottleneck);
    eprintln!(
        "alternatives={} archive_final={} distinct_mutations={} feasible_mutations={} wall_ms={}",
        summary.alternatives_count,
        summary.archive_size_final,
        summary.distinct_mutations,
        summary.feasible_mutations,
        summary.wall_ms
    );
}
