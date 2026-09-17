// Phase 3 — Production Pareto Pipeline
//
// Provides genuine multi-objective Pareto candidate generation for the production
// UltraCrew scheduling path, using the existing generic `coralys_moga::engine_proof`
// machinery.
//
// Architecture boundary (enforced):
//   - All UltraCrew domain logic stays in this adapter module.
//   - `coralys_moga::engine_proof` is used as a generic Pareto engine only.
//   - No domain logic enters `coralys-moga`.
//   - The existing scalar production path (`run_optimization`) is unchanged.
//
// Invariants:
//   I1 — Every `ProductionParetoSolution` comes from `ParetoArchive`.
//   I2 — `FitnessVector` is derived from constraint engine output directly,
//         never by decomposing the scalar weighted fitness.
//   I3 — Existing objectives only. No new objectives, no changed weights.
//   I4 — `coralys-moga` source is not modified.
//   I5 — Existing scalar path (`run_optimization`) is preserved and unchanged.

use crate::constraint_engine::{DomainConstraintEvaluator, InrcConstraintEvaluator};
use crate::decision_intelligence::analyze_solution;
use crate::optimization::{ScheduleContext, ScheduleGenome};
use crate::schedule_solution::ScheduleSolution;
use coralys_moga::engine_proof::{
    Evaluator, FitnessVector, MutationPolicy, ParetoArchive, ParetoSolution,
};
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

// ── Step 1: Hash for production ScheduleGenome ───────────────────────────────
//
// `HashMap<u64, u64>` is not `Hash` in Rust. We implement `Hash` for
// `ScheduleGenome` using a deterministic canonical representation: sort
// (shift_id, worker_id) pairs by shift_id, then hash each pair in order.
// This is stable across runs for the same assignment map.
//
// The meaning of assignments is unchanged — this is purely a hashing adapter.

impl Hash for ScheduleGenome {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut pairs: Vec<(u64, u64)> = self.assignments.iter().map(|(&k, &v)| (k, v)).collect();
        pairs.sort_unstable_by_key(|&(k, _)| k);
        for (shift_id, worker_id) in pairs {
            shift_id.hash(state);
            worker_id.hash(state);
        }
    }
}

// `engine_proof::Genome` requires: Clone + Send + Sync + Hash
// `ScheduleGenome` already derives Clone and is Send + Sync (no non-Send fields).
// Hash is now implemented above.
impl coralys_moga::engine_proof::Genome for ScheduleGenome {}

// ── Step 2: Production Pareto evaluator ──────────────────────────────────────
//
// Implements `engine_proof::Evaluator<ScheduleGenome>` returning a `FitnessVector`
// derived directly from the constraint engine output — not from the scalar fitness.
//
// Objective vector (6 components, matching existing production semantics):
//   [0] hard_violations   — total hard constraint violations (HC1+HC2+HC3+HC4)
//   [1] rest_violations   — rest period violations
//   [2] fairness_penalty  — workload fairness variance
//   [3] fatigue_penalty   — fatigue accumulation penalty
//   [4] hc1_violations    — minimum coverage violations (HC1)
//   [5] hc3_violations    — consecutive shift violations (HC3)
//
// These are the same components already tracked in `ScheduleEvaluation` and
// `ConstraintReport`. No new objectives are introduced.

pub struct ProductionParetoEvaluator {
    pub context: Arc<ScheduleContext>,
}

impl Evaluator<ScheduleGenome> for ProductionParetoEvaluator {
    fn evaluate(&self, genome: &ScheduleGenome) -> FitnessVector {
        let evaluator: Box<dyn DomainConstraintEvaluator> =
            Box::new(InrcConstraintEvaluator::new(self.context.clone()));
        let report = evaluator.evaluate(genome);

        // Derive FitnessVector from constraint engine output directly.
        // Lower is better for all objectives (Pareto minimization).
        // The ParetoArchive::add() dominance check uses: sol.fitness[d] < other.fitness[d]
        // means `sol` is better on dimension d — consistent with minimization.
        vec![
            report.hard_violations as f64,
            report.rest_violations as f64,
            report.fairness_penalty,
            report.fatigue_penalty,
            report.hc1_violations as f64,
            report.hc3_violations as f64,
        ]
    }
}

// ── Step 3: MutationPolicy adapter ───────────────────────────────────────────
//
// `engine_proof::MutationPolicy` requires: `&G → G` (immutable, returns new genome).
// `MutationOperator` requires: `&mut G` (in-place mutation).
//
// This adapter clones the genome, applies the existing in-place mutation via
// `ScheduleOptimizer::mutate()`, and returns the mutated clone.
// Mutation semantics are unchanged.
//
// P0-B.3: Apply k perturbations per Pareto mutation step.
//
// A single-step perturbation from the scalar GA optimum almost never escapes
// Pareto dominance: the seed dominates every single-assignment mutant on all
// six objectives simultaneously. Applying k=3 perturbations per step creates
// genomes that trade off differently across objectives, giving the Pareto
// archive meaningful mutation reach.
//
// This is an adapter-level execution parameter only. The mutation algorithm
// (operators, probabilities, RNG) is unchanged. The scalar GA is unchanged.
// The Pareto archive and dominance semantics are unchanged.
const PARETO_MUTATION_PERTURBATIONS: usize = 3;

pub struct ProductionMutationAdapter {
    pub context: Arc<ScheduleContext>,
}

impl MutationPolicy<ScheduleGenome> for ProductionMutationAdapter {
    fn mutate(&self, genome: &ScheduleGenome) -> ScheduleGenome {
        use coralys_moga::traits::MutationOperator;
        use crate::optimization::ScheduleOptimizer;

        let mut child = genome.clone();
        let optimizer = ScheduleOptimizer::new(self.context.clone());
        let mut rng = StdRng::from_entropy();
        for _ in 0..PARETO_MUTATION_PERTURBATIONS {
            optimizer.mutate(&mut child, &mut rng);
        }
        child
    }
}

// ── Step 4: run_pareto_pipeline() ────────────────────────────────────────────
//
// Production equivalent of `run_inrc_startup_pipeline()`.
// Uses the existing production genome and constraint engine.
// The existing scalar path (`run_optimization`) is not touched.

/// Factual constraint profile for a Pareto candidate.
///
/// P0-C.2 (revised): The adapter does not decide whether a candidate is
/// acceptable. It exposes the factual constraint profile so the scheduler
/// can make that operational decision with full information.
///
/// `is_feasible` is `true` when `hard_violations == 0`, matching the
/// existing `validate_schedule()` semantics. It is provided as a convenience
/// flag — the scheduler may override this judgment based on operational context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateFeasibility {
    /// True when hard_violations == 0 (same as validate_schedule()).
    /// Provided as a convenience flag; the scheduler makes the final decision.
    pub is_feasible: bool,
    /// Total hard constraint violations (HC1+HC2+HC3+HC4+rest).
    pub hard_violations: usize,
    /// Rest period violations specifically.
    pub rest_violations: usize,
    /// HC1 (minimum coverage) violations.
    pub hc1_violations: usize,
    /// HC3 (forbidden shift succession) violations.
    pub hc3_violations: usize,
}

/// A single genuine Pareto candidate from the production optimizer.
/// Every field is derived from the optimizer — nothing is fabricated.
///
/// The adapter does not rank, score, or filter candidates beyond excluding
/// the primary schedule. The scheduler receives the complete factual picture
/// and makes the operational decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionParetoSolution {
    /// Complete shift→worker assignment map (same representation as `ScheduleResponse.schedule`).
    pub schedule: HashMap<u64, u64>,
    /// Raw multi-objective fitness vector (6 components; lower is better for all).
    /// [hard_violations, rest_violations, fairness_penalty, fatigue_penalty,
    ///  hc1_violations, hc3_violations]
    pub objectives: Vec<f64>,
    /// Analyzed metrics (same keys as `ScheduleResponse.metrics`).
    pub metrics: HashMap<String, f64>,
    /// Factual constraint profile. The scheduler uses this to decide whether
    /// the candidate is operationally acceptable in context.
    pub feasibility: CandidateFeasibility,
}

/// Number of diverse genomes used to initialize the Pareto archive.
///
/// P0-B.4: The scalar GA produces a perfect-score genome (all objectives = 0).
/// Seeding the Pareto engine with only that genome means every perturbation is
/// dominated — the archive never grows. Seeding with N diverse feasible genomes
/// instead gives the Pareto search a non-dominated initial front with genuine
/// trade-offs across the six objectives.
///
/// Each genome is generated by `ScheduleOptimizer::create()` — the same
/// constraint-aware factory used by the scalar GA — so all seeds are feasible
/// under the existing UltraCrew constraints. The scalar optimum (primary) is
/// still excluded from alternatives via the existing `seed_uid` filter.
const PARETO_DIVERSE_SEED_COUNT: usize = 20;

/// Runs the production Pareto pipeline.
///
/// Seeds the Pareto engine with N diverse feasible genomes (not just the scalar
/// optimum), runs `steps` evolution steps, and returns all non-dominated
/// solutions from the archive, excluding the primary schedule.
///
/// The existing scalar production path is not affected.
pub fn run_pareto_pipeline(
    context: Arc<ScheduleContext>,
    seed_genome: ScheduleGenome,
    additional_seeds: Vec<ScheduleGenome>,
    steps: usize,
) -> Vec<ProductionParetoSolution> {
    use crate::optimization::ScheduleOptimizer;
    use coralys_moga::engine_proof::EvolutionEngine;
    use coralys_moga::traits::GenomeFactory;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use std::collections::hash_map::DefaultHasher;

    // Compute the seed uid before moving seed_genome into the engine.
    // Used to exclude the primary/seed genome from the alternatives list:
    // the primary is Pareto-optimal but is not "an alternative to itself".
    // Filtering happens after the archive is produced — Pareto dominance
    // semantics are not changed.
    let seed_uid = {
        let mut h = DefaultHasher::new();
        seed_genome.hash(&mut h);
        h.finish()
    };

    let evaluator = ProductionParetoEvaluator {
        context: context.clone(),
    };
    let mutator = ProductionMutationAdapter {
        context: context.clone(),
    };

    let mut engine = EvolutionEngine::new(evaluator, mutator);

    // P0-B.4: Seed the Pareto archive with N diverse feasible genomes.
    //
    // The scalar GA optimum has objectives [0,0,0,0,0,0] — a perfect score.
    // Seeding only that genome means every perturbation is dominated and the
    // archive never grows. Instead, we seed with PARETO_DIVERSE_SEED_COUNT
    // randomly-generated feasible genomes using the existing constraint-aware
    // factory. The non-dominated front across these diverse genomes will contain
    // genuine trade-offs, giving the Pareto search a meaningful starting point.
    //
    // The scalar optimum (primary) is seeded first so its uid is recorded, then
    // diverse genomes are added. The primary is excluded from alternatives by the
    // existing seed_uid filter in archive_to_solutions().
    engine.seed(seed_genome);

    let factory = ScheduleOptimizer::new(context.clone());
    // Use a fixed seed for reproducibility of the diverse population.
    let mut rng = StdRng::seed_from_u64(context.rng_seed.wrapping_add(1));
    for _ in 0..PARETO_DIVERSE_SEED_COUNT {
        let diverse_genome = factory.create(&mut rng);
        engine.seed(diverse_genome);
    }

    // P0-F: Seed the Pareto archive with the scalar GA's top_10 candidates.
    // These are genuine optimizer-generated schedules from the scalar GA's
    // final population. Seeding them gives the Pareto search a richer initial
    // front, increasing the probability of finding non-dominated alternatives
    // even for homogeneous datasets.
    //
    // Provenance: these genomes come directly from GaResult.top_10 — they are
    // not mutated or synthesized during this seeding step.
    for genome in additional_seeds {
        engine.seed(genome);
    }

    for _ in 0..steps {
        engine.step();
    }

    archive_to_solutions(&engine.archive, seed_uid, &context)
}

/// Converts a `ParetoArchive<ScheduleGenome>` to `Vec<ProductionParetoSolution>`,
/// excluding only the seed/primary genome (identified by `seed_uid`).
///
/// **P0-C.2 (revised) — Adapter describes; scheduler decides:**
/// All non-dominated candidates (except the primary) are returned, each with a
/// `CandidateFeasibility` struct populated from the authoritative
/// `InrcConstraintEvaluator`. The adapter does not filter by feasibility —
/// that decision belongs to the scheduler, who has the operational context.
///
/// `CandidateFeasibility.is_feasible` mirrors `validate_schedule()` semantics
/// (`hard_violations == 0`) as a convenience flag. The scheduler may override
/// this judgment based on situational context.
///
/// This preserves the complete decision provenance: the scheduler can
/// demonstrate that multiple materially different solutions were generated and
/// considered before a final roster was selected.
fn archive_to_solutions(
    archive: &ParetoArchive<ScheduleGenome>,
    seed_uid: u64,
    context: &Arc<ScheduleContext>,
) -> Vec<ProductionParetoSolution> {
    let constraint_eval = InrcConstraintEvaluator::new(context.clone());
    archive
        .solutions
        .iter()
        .filter(|sol| sol.uid != seed_uid)
        .map(|sol: &ParetoSolution<ScheduleGenome>| {
            // Evaluate constraint profile for this candidate using the
            // authoritative InrcConstraintEvaluator. This is the same evaluator
            // used by validate_schedule() and decision_support.
            let report = constraint_eval.evaluate(&sol.genome);
            let feasibility = CandidateFeasibility {
                is_feasible: report.hard_violations == 0,
                hard_violations: report.hard_violations,
                rest_violations: report.rest_violations,
                hc1_violations: report.hc1_violations,
                hc3_violations: report.hc3_violations,
            };

            // Build a ScheduleSolution directly from the Pareto solution's data
            // to reuse analyze_solution() for metrics without constructing a
            // ScheduleEvaluation (which has different field semantics).
            let schedule_solution = ScheduleSolution {
                assignments: sol.genome.assignments.clone(),
                fitness: 0.0, // not used for metrics
                hard_violations: report.hard_violations,
                fairness_penalty: sol.fitness.get(2).copied().unwrap_or(0.0),
                fatigue_penalty: sol.fitness.get(3).copied().unwrap_or(0.0),
                rest_violations: report.rest_violations,
                recommendations: None,
                telemetry: None,
                top_candidates: Vec::new(),
            };
            let metrics = analyze_solution(&schedule_solution);

            ProductionParetoSolution {
                schedule: sol.genome.assignments.clone(),
                objectives: sol.fitness.clone(),
                metrics,
                feasibility,
            }
        })
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Shift, Skill, Worker};
    use crate::optimization::{Observatory, ScheduleContext};
    use std::sync::{Arc, Mutex};

    fn make_context(workers: Vec<Worker>, shifts: Vec<Shift>) -> Arc<ScheduleContext> {
        Arc::new(ScheduleContext {
            workers: Arc::new(workers),
            shifts: Arc::new(shifts),
            ecology: crate::ecology::WorkforceEcology::new(),
            rng_seed: 42,
            observatory: Arc::new(Mutex::new(Observatory::new())),
            locked_assignments: None,
            scenario: None,
            enable_fatigue: false,
            fatigue_weight: 0.0,
            hc3_aware_initialization: false,
            temporal_scarcity_construction: false,
            disable_global_constructor: false,
            constructor_budget_ms: None,
            precomputed_seeds: None,
        })
    }

    fn make_worker(id: u64, skill: &str) -> Worker {
        Worker {
            id,
            skills: vec![Skill::new(skill)],
        }
    }

    fn make_shift(id: u64, skill: &str, start: u64, duration: u64) -> Shift {
        Shift {
            id,
            required_skill: Skill::new(skill),
            start_hour: start,
            duration_hours: duration,
        }
    }

    /// P3-G1: ScheduleGenome Hash is deterministic — same assignments produce same hash.
    #[test]
    fn p3_g1_schedule_genome_hash_is_deterministic() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut assignments = HashMap::new();
        assignments.insert(1u64, 10u64);
        assignments.insert(2u64, 20u64);
        let genome = ScheduleGenome { assignments };

        let mut h1 = DefaultHasher::new();
        genome.hash(&mut h1);
        let hash1 = h1.finish();

        let mut h2 = DefaultHasher::new();
        genome.hash(&mut h2);
        let hash2 = h2.finish();

        assert_eq!(hash1, hash2, "Hash must be deterministic for same genome");
    }

    /// P3-G2: Different assignments produce different hashes.
    #[test]
    fn p3_g2_different_assignments_produce_different_hashes() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut a1 = HashMap::new();
        a1.insert(1u64, 10u64);
        let g1 = ScheduleGenome { assignments: a1 };

        let mut a2 = HashMap::new();
        a2.insert(1u64, 20u64); // different worker
        let g2 = ScheduleGenome { assignments: a2 };

        let mut h1 = DefaultHasher::new();
        g1.hash(&mut h1);

        let mut h2 = DefaultHasher::new();
        g2.hash(&mut h2);

        assert_ne!(
            h1.finish(),
            h2.finish(),
            "Different assignments must produce different hashes"
        );
    }

    /// P3-G3: run_pareto_pipeline completes without panic and returns valid solutions.
    ///
    /// A 2-worker / 2-shift scenario with 10 steps may produce zero alternatives —
    /// the seed (scalar GA optimum) dominates all single-perturbation mutants in a
    /// minimal scenario. The correct product behaviour is `alternatives=[]` (UI hard
    /// stop). This test verifies the pipeline runs without panic and that any returned
    /// solutions have the correct structure; it does NOT assert non-empty output.
    ///
    /// Non-empty output is verified by the P0-B.1 harness against a realistic
    /// 20-worker / 40-shift scenario.
    #[test]
    fn p3_g3_pareto_pipeline_returns_solutions() {
        let workers = vec![
            make_worker(1, "Pilot"),
            make_worker(2, "Pilot"),
        ];
        let shifts = vec![
            make_shift(1, "Pilot", 0, 8),
            make_shift(2, "Pilot", 8, 8),
        ];
        let context = make_context(workers, shifts);

        let mut assignments = HashMap::new();
        assignments.insert(1u64, 1u64);
        assignments.insert(2u64, 2u64);
        let seed = ScheduleGenome { assignments };

        // Pipeline must complete without panic. Empty result is valid for a minimal scenario.
        let solutions = run_pareto_pipeline(context, seed, Vec::new(), 10);
        for sol in &solutions {
            assert_eq!(sol.schedule.len(), 2, "Each solution must cover all shifts");
            assert_eq!(sol.objectives.len(), 6, "Each solution must have 6 objectives");
        }
    }

    /// P3-G4: Each solution has a complete schedule (same shift count as input).
    #[test]
    fn p3_g4_solutions_have_complete_schedules() {
        let workers = vec![
            make_worker(1, "Pilot"),
            make_worker(2, "Pilot"),
        ];
        let shifts = vec![
            make_shift(1, "Pilot", 0, 8),
            make_shift(2, "Pilot", 8, 8),
        ];
        let context = make_context(workers, shifts);

        let mut assignments = HashMap::new();
        assignments.insert(1u64, 1u64);
        assignments.insert(2u64, 2u64);
        let seed = ScheduleGenome { assignments };

        let solutions = run_pareto_pipeline(context, seed, Vec::new(), 10);
        for sol in &solutions {
            assert_eq!(
                sol.schedule.len(),
                2,
                "Each solution must have an assignment for every shift"
            );
        }
    }

    /// P3-G5: Each solution's objectives vector has exactly 6 components.
    #[test]
    fn p3_g5_solutions_have_six_objectives() {
        let workers = vec![
            make_worker(1, "Pilot"),
            make_worker(2, "Pilot"),
        ];
        let shifts = vec![
            make_shift(1, "Pilot", 0, 8),
            make_shift(2, "Pilot", 8, 8),
        ];
        let context = make_context(workers, shifts);

        let mut assignments = HashMap::new();
        assignments.insert(1u64, 1u64);
        assignments.insert(2u64, 2u64);
        let seed = ScheduleGenome { assignments };

        let solutions = run_pareto_pipeline(context, seed, Vec::new(), 10);
        for sol in &solutions {
            assert_eq!(
                sol.objectives.len(),
                6,
                "Each solution must have exactly 6 objective components"
            );
        }
    }

    /// P3-G6: No solution in the archive is dominated by another on all objectives.
    #[test]
    fn p3_g6_archive_solutions_are_pareto_nondominated() {
        let workers = vec![
            make_worker(1, "Pilot"),
            make_worker(2, "Pilot"),
            make_worker(3, "Pilot"),
        ];
        let shifts = vec![
            make_shift(1, "Pilot", 0, 8),
            make_shift(2, "Pilot", 8, 8),
            make_shift(3, "Pilot", 16, 8),
        ];
        let context = make_context(workers, shifts);

        let mut assignments = HashMap::new();
        assignments.insert(1u64, 1u64);
        assignments.insert(2u64, 2u64);
        assignments.insert(3u64, 3u64);
        let seed = ScheduleGenome { assignments };

        let solutions = run_pareto_pipeline(context, seed, Vec::new(), 20);

        // Verify Pareto correctness: no solution dominates another on all objectives.
        for (i, a) in solutions.iter().enumerate() {
            for (j, b) in solutions.iter().enumerate() {
                if i == j {
                    continue;
                }
                // Check if a dominates b (a is better or equal on all, strictly better on at least one)
                let a_dominates_b = a.objectives.iter().zip(b.objectives.iter())
                    .all(|(ai, bi)| ai <= bi)
                    && a.objectives.iter().zip(b.objectives.iter())
                    .any(|(ai, bi)| ai < bi);
                assert!(
                    !a_dominates_b,
                    "Solution {} dominates solution {} — archive is not Pareto-correct",
                    i, j
                );
            }
        }
    }
}