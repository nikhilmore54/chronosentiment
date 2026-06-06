//! # inrc_archive_forensics
//!
//! Sprint 3.6 — Archive Forensics Execution Binary
//!
//! Diagnoses SD-003 (Champion Retention Error): why top external champions
//! are not present in the final Pareto archive.
//!
//! ## Canonical Observer
//! Uses `ultracrew_server::inrc_observer::score_inrc_official()` exclusively.
//! Formula: hc_coverage + hc_skills + hc_one_shift_per_day
//!          + hc_forbidden_successions + soft_total
//! (HC components already penalty-weighted ×1000 by the evaluator.)
//! This matches `InrcExternalScorer::score_with_components()` line 70 in
//! `validation_pass.rs` — the source of the validated SD-003 observation.
//!
//! ## Evidence Artifacts Produced
//! - `champion_lifecycle.jsonl`   — one JSON record per champion lifecycle event
//! - `feasibility_snapshot.jsonl` — one JSON record per archive member at Gen N
//!
//! ## Run Modes
//! ```
//! # Smoke run (10 generations, fast verification)
//! cargo run --bin inrc_archive_forensics -- --gens 10
//!
//! # Full forensics run (5000 generations, single seed)
//! cargo run --bin inrc_archive_forensics
//! ```
//!
//! ## Classification Table (immutable — v1)
//! | Observation                              | Classification                    |
//! |------------------------------------------|-----------------------------------|
//! | admitted_at == null                      | Admission Geometry Failure        |
//! | exit_reason = Dominated                  | Proxy/External Misalignment       |
//! | exit_reason = Crowding                   | Memory Retention Failure          |
//! | exit_reason = ArchiveLimit               | Capacity Failure                  |
//! | feasible_count / archive_size < 5%       | Feasibility Representation Failure|

use ultracrew::inrc::parser::{parse_scenario, parse_week_data, parse_history};
use ultracrew::inrc::optimization::{InrcContext, InrcOptimizer};
use ultracrew::inrc::audit::{FeasibilitySnapshot, MemberSnapshot};
use ultracrew_server::simulation::generate_baseline_schedule;
use ultracrew_server::optimizer::{ScheduleGenome, UltraCrewEvaluator, UltraCrewMutator};
use ultracrew_server::inrc_observer::{score_inrc_official, OBSERVER_ID};
use coralys_moga::engine_proof::{EvolutionEngine, Evaluator, ParetoSolution};
use coralys_moga::ecology::champion::{ChampionTracker, ChampionStatus, ExitReason};
use std::sync::Arc;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::distributions::{WeightedIndex, Distribution};

// ── Frozen constants ───────────────────────────────────────────────────────────
const INSTANCE: &str = "n050w4";

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

// ── Sprint 3.7: Domination event reconstruction (local, fires once) ───────────
// Targeted snapshot for SD-006 Proxy Geometry Attribution.
// Fires exactly once: generation == 175 AND victim tracker_uid == 4.
// Not a general framework — discarded after gen175_domination_report.md is produced.

struct ExtMetrics {
    hc_coverage: usize,
    hc_skills: usize,
    hc_one_shift_per_day: usize,
    hc_forbidden_successions: usize,
    soft_total: i32,
    official_total: f64,
}

struct DominationEvent {
    eviction_gen: u64,
    victim_genome_hash: u64,
    victim_tracker_uid: u64,
    dominator_genome_hash: u64,
    victim_proxy: Vec<f64>,       // [O1=hc_coverage, O2=hc_skills, O3=hc_successions, O4=soft_total, O5=hc_violations]
    dominator_proxy: Vec<f64>,
    victim_external: ExtMetrics,
    dominator_external: ExtMetrics,
    victim_front_rank_prev_gen: usize,  // always 0 in a Pareto archive (all members non-dominated)
    victim_crowding_prev_gen: f64,      // crowding distance at end of (eviction_gen - 1)
}

// ── Sprint 3.8: Feasible genome chain-of-custody ──────────────────────────────
// Mirrors ChampionTracker discipline: one record per feasible genome discovered,
// with full lifecycle from discovery → admission → eviction.
struct FeasibleLifecycle {
    genome_hash: u64,
    discovered_at: u64,
    admitted_at: Option<u64>,
    evicted_at: Option<u64>,
    exit_reason: Option<ExitReason>,
    dominator_hash: Option<u64>,  // genome_hash of the dominator that evicted this genome (if Dominated)
    hc_total: usize,              // sum of all HC violation counts at discovery
    official_total: f64,          // official_total at discovery
    proxy: Vec<f64>,              // proxy objective vector at discovery
}

// Census sample: (generation, feasible_count, near_feasible_5, near_feasible_10, infeasible_count)
type CensusSample = (u64, usize, usize, usize, usize);

// HC distribution sample: (generation, min_hc, max_hc, sum_hc, count, hc0, hc_le5, hc_le10, hc_le20, hc_le50, hc_gt50)
// sum_hc/count = mean_hc; median requires sorting so we store raw sum for mean only.
type HcDistSample = (u64, usize, usize, u64, usize, usize, usize, usize, usize, usize, usize);

// Sprint 3.10: ΔHC offspring probe accumulators (accumulated across all generations)
// Measures P(child_hc < parent_hc) before selection — isolates RC-1 from RC-2.
struct DeltaHcProbe {
    total_offspring: u64,
    hc_improving: u64,          // child_hc < parent_hc
    hc_neutral: u64,            // child_hc == parent_hc
    hc_worsening: u64,          // child_hc > parent_hc
    hc_improving_inserted: u64, // child_hc < parent_hc AND was_inserted
    hc_worsening_inserted: u64, // child_hc > parent_hc AND was_inserted
    delta_sum: i64,             // sum of (child_hc - parent_hc) for mean delta
}

impl DeltaHcProbe {
    fn new() -> Self {
        Self {
            total_offspring: 0,
            hc_improving: 0,
            hc_neutral: 0,
            hc_worsening: 0,
            hc_improving_inserted: 0,
            hc_worsening_inserted: 0,
            delta_sum: 0,
        }
    }

    fn record(&mut self, parent_hc: usize, child_hc: usize, was_inserted: bool) {
        self.total_offspring += 1;
        let delta = child_hc as i64 - parent_hc as i64;
        self.delta_sum += delta;
        if delta < 0 {
            self.hc_improving += 1;
            if was_inserted { self.hc_improving_inserted += 1; }
        } else if delta == 0 {
            self.hc_neutral += 1;
        } else {
            self.hc_worsening += 1;
            if was_inserted { self.hc_worsening_inserted += 1; }
        }
    }

    fn p_improving(&self) -> f64 {
        if self.total_offspring == 0 { return 0.0; }
        self.hc_improving as f64 / self.total_offspring as f64
    }

    fn p_improving_inserted(&self) -> f64 {
        if self.hc_improving == 0 { return 0.0; }
        self.hc_improving_inserted as f64 / self.hc_improving as f64
    }

    fn mean_delta(&self) -> f64 {
        if self.total_offspring == 0 { return 0.0; }
        self.delta_sum as f64 / self.total_offspring as f64
    }
}

/// Compute crowding distance for a specific solution (by uid) within the archive.
/// Uses the same normalised Euclidean neighbour distance as the parent selector.
/// Returns f64::INFINITY for boundary solutions; 0.0 if uid not found or archive < 2.
fn crowding_distance_for(solutions: &[ParetoSolution<ScheduleGenome>], target_uid: u64) -> f64 {
    let n = solutions.len();
    if n < 2 { return 0.0; }
    let num_objs = solutions[0].fitness.len();

    let target_idx = match solutions.iter().position(|s| s.uid == target_uid) {
        Some(i) => i,
        None => return 0.0,
    };

    let mut distance = 0.0_f64;

    for obj in 0..num_objs {
        let mut sorted: Vec<(usize, f64)> = solutions.iter()
            .enumerate()
            .map(|(i, s)| (i, s.fitness[obj]))
            .collect();
        sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let min_val = sorted[0].1;
        let max_val = sorted[n - 1].1;
        let range = max_val - min_val + 1e-9;

        let pos = sorted.iter().position(|(i, _)| *i == target_idx).unwrap();

        if pos == 0 || pos == n - 1 {
            return f64::INFINITY;
        }

        let prev_val = sorted[pos - 1].1;
        let next_val = sorted[pos + 1].1;
        distance += (next_val - prev_val) / range;
    }

    distance
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let max_generations: u64 = args.iter()
        .position(|a| a == "--gens")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(5000);
    let seed: u64 = args.iter()
        .position(|a| a == "--seed")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(42);

    // Single seeded RNG — makes every run fully reproducible given the same seed.
    // Use --seed <N> to reproduce a specific run. Default seed=42.
    let mut rng = StdRng::seed_from_u64(seed);

    println!("=== inrc_archive_forensics ===");
    println!("Instance    : {}", INSTANCE);
    println!("Observer    : {}", OBSERVER_ID);
    println!("Formula     : hc_coverage + hc_skills + hc_one_shift_per_day + hc_forbidden_successions + soft_total");
    println!("Generations : {}", max_generations);
    println!("Seed        : {}", seed);
    println!();

    // ── Load scenario ──────────────────────────────────────────────────────────
    let base_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../adapters/ultracrew/tests/data/{}", INSTANCE));
    let scenario = parse_scenario(
        base_dir.join(format!("Sc-{}.json", INSTANCE))
    ).unwrap();
    let week_data = parse_week_data(
        base_dir.join(format!("WD-{}-0.json", INSTANCE))
    ).unwrap();
    let history = parse_history(
        base_dir.join(format!("H0-{}-0.json", INSTANCE))
    ).unwrap();

    let inrc_context = InrcContext::new(
        scenario.clone(),
        week_data.clone(),
        history,
        ultracrew::ecology::WorkforceEcology::new(),
    );
    let inrc_optimizer = InrcOptimizer { context: Arc::new(inrc_context) };

    // ── Engine ─────────────────────────────────────────────────────────────────
    let evaluator = UltraCrewEvaluator { scenario: scenario.clone() };
    let mutator = UltraCrewMutator::new(scenario.clone());
    let mut engine = EvolutionEngine::new(evaluator, mutator);

    // ── Audit instruments ──────────────────────────────────────────────────────
    let mut champion_tracker = ChampionTracker::new();

    // Full-coverage archive membership map: genome_hash → (tracker_uid, ext_score).
    // Populated for EVERY archive insertion (not just best-ever champions).
    // This is the correct eviction detection surface — the archive contains O(100s)
    // of members, but ChampionTracker only records O(10s) of best-ever champions.
    // We need to know when ANY archive member is evicted so we can cross-reference
    // against the best-ever champion records.
    //
    // tracker_uid is Some(uid) only for genomes that were best-ever at admission time.
    // For all other archive members, tracker_uid is None.
    let mut archive_members: HashMap<u64, (Option<u64>, f64)> = HashMap::new();

    // Coverage counters for the coverage audit printed at end.
    let mut total_insertions: u64 = 0;
    let mut total_evictions: u64 = 0;
    let mut tracked_evictions: u64 = 0;

    // ── Sprint 3.7: Best-ever champion domination event reconstruction ─────────
    // Seed-agnostic: fires when the best-ever champion (whatever tracker UID it has)
    // is evicted from the archive. Does not fire more than once.
    let mut best_ever_genome_hash: Option<u64> = None;
    let mut best_ever_genome_cache: Option<ScheduleGenome> = None;
    let mut best_ever_crowding_prev_gen: f64 = 0.0;
    let mut best_ever_tracker_uid: Option<u64> = None;
    let mut best_ever_eviction_gen: u64 = 0;
    let mut domination_event: Option<DominationEvent> = None;

    // ── Sprint 3.8: Feasible lifecycle tracking ────────────────────────────────
    // HashMap<genome_hash, FeasibleLifecycle> — keyed by genome hash for deduplication.
    // A feasible genome may be discovered, evicted, and rediscovered across 5000 gens;
    // the map ensures only one lifecycle record per unique genome hash.
    // If a genome is rediscovered after eviction, the existing record is updated
    // (admitted_at reset, evicted_at/exit_reason cleared) to reflect the latest admission.
    //
    // feasible_archive_members: genome_hash → genome_hash (identity map) for genomes
    // currently in the archive, used to detect evictions.
    //
    // Frozen definitions (from sd005_sprint38_charter.md):
    //   first feasible = earliest discovered_at
    //   best feasible  = lowest official_total
    let mut feasible_lifecycles: HashMap<u64, FeasibleLifecycle> = HashMap::new();
    let mut feasible_archive_members: std::collections::HashSet<u64> = std::collections::HashSet::new();
    let mut census_timeline: Vec<CensusSample> = Vec::new();
    // Sprint 3.9: HC_Total distribution timeline — sampled every 100 gens (same cadence as census)
    let mut hc_dist_timeline: Vec<HcDistSample> = Vec::new();
    // Sprint 3.9: gen=0 initialization snapshot — HC_Total of all initial archive members
    let mut init_hc_values: Vec<usize> = Vec::new();
    // Sprint 3.10: ΔHC offspring probe — accumulated across all generations
    let mut delta_hc_probe = DeltaHcProbe::new();

    // ── Seed with baseline ─────────────────────────────────────────────────────
    let baseline_genome =
        generate_baseline_schedule(&scenario, &week_data.requirements).unwrap();
    let base_fitness = engine.evaluator.evaluate(&baseline_genome);
    let base_uid = calculate_hash(&baseline_genome);

    // Canonical scoring via shared module
    let base_score = score_inrc_official(&baseline_genome, &scenario, &inrc_optimizer);

    println!("Baseline official_total : {}", base_score.official_total);
    println!("Baseline feasible       : {}", base_score.feasible);
    println!("  HC_Coverage           : {}", base_score.hc_coverage);
    println!("  HC_Skills             : {}", base_score.hc_skills);
    println!("  HC_OneShiftPerDay     : {}", base_score.hc_one_shift_per_day);
    println!("  HC_ForbiddenSucc      : {}", base_score.hc_forbidden_successions);
    println!("  SoftTotal             : {}", base_score.soft_total);
    println!();

    let base_status = ChampionStatus {
        uid: base_uid,
        feasible: base_score.feasible,
        viability_score: 0.0,
        archive_member: true,
        pareto_rank: 0,
        crowding_distance: 0.0,
    };
    let (tracker_uid, _is_new) = champion_tracker.observe(
        0,
        OBSERVER_ID,
        base_score.official_total,
        base_fitness.clone(),
        true,
        Some(base_status),
    );
    // Register baseline in full-coverage map with its tracker_uid (it IS the first best-ever)
    archive_members.insert(base_uid, (Some(tracker_uid), base_score.official_total));
    total_insertions += 1;

    engine.archive.add(ParetoSolution {
        genome: baseline_genome,
        fitness: base_fitness,
        uid: base_uid,
        parent_uid: 0,
    });

    // ── Main generation loop ───────────────────────────────────────────────────
    let mut best_external_ever: f64 = base_score.official_total;

    for g in 1..=max_generations {
        let archive_size = engine.archive.solutions.len();
        if archive_size == 0 { break; }
        let num_objs = engine.archive.solutions[0].fitness.len();

        // ── Parent selection (crowding-distance weighted) ──────────────────────
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
                            let ni = (engine.archive.solutions[i].fitness[d]
                                - min_vals[d]) / ranges[d];
                            let nj = (engine.archive.solutions[j].fitness[d]
                                - min_vals[d]) / ranges[d];
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

        // ── Generate offspring (5 initial + 20 SA neighbours) ─────────────────
        let calc_energy = |f: &[f64]| f.iter().map(|v| v.powi(2)).sum::<f64>().sqrt();

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

        // ── Canonical external score via shared module ─────────────────────────
        let child_score = score_inrc_official(&child_genome, &scenario, &inrc_optimizer);

        // ── Sprint 3.7: Capture best-ever proxy before potential eviction ───────
        // Must happen before engine.archive.add() so the victim is still present.
        // Runs every generation (cheap: just a find + clone if present).
        let best_ever_proxy_before: Option<Vec<f64>> = best_ever_genome_hash.and_then(|h| {
            engine.archive.solutions.iter()
                .find(|s| s.uid == h)
                .map(|s| s.fitness.clone())
        });

        // ── Sprint 3.10: ΔHC offspring probe — score parent BEFORE archive.add() ──
        // parent_hc is computed here (before admission) so it reflects the pre-selection state.
        // child_hc comes from child_score (already computed above).
        let parent_score = score_inrc_official(&parent.genome, &scenario, &inrc_optimizer);
        let parent_hc = parent_score.hc_coverage + parent_score.hc_skills
            + parent_score.hc_one_shift_per_day + parent_score.hc_forbidden_successions;
        let child_hc_for_probe = child_score.hc_coverage + child_score.hc_skills
            + child_score.hc_one_shift_per_day + child_score.hc_forbidden_successions;

        // ── Archive admission ──────────────────────────────────────────────────
        let old_uids: Vec<u64> = engine.archive.solutions.iter().map(|s| s.uid).collect();
        let was_inserted = engine.archive.add(ParetoSolution {
            genome: child_genome.clone(),
            fitness: child_fitness.clone(),
            uid: child_uid,
            parent_uid: parent.uid,
        });
        let new_uids: Vec<u64> = engine.archive.solutions.iter().map(|s| s.uid).collect();

        // ── ChampionTracker: observe only best-ever candidates ─────────────────
        // ChampionTracker is designed to track best-ever external score champions.
        // We call it for every candidate so it can detect new bests, but only
        // best-ever candidates create new tracker records.
        let child_status = ChampionStatus {
            uid: child_uid,
            feasible: child_score.feasible,
            viability_score: 0.0,
            archive_member: was_inserted,
            pareto_rank: 0,
            crowding_distance: 0.0,
        };
        let (tracker_uid, is_new_record) = champion_tracker.observe(
            g,
            OBSERVER_ID,
            child_score.official_total,
            child_fitness.clone(),
            was_inserted,
            Some(child_status),
        );

        if is_new_record {
            best_external_ever = child_score.official_total;
        }

        // ── Sprint 3.7: Track best-ever champion genome hash ──────────────────
        // Updated every time a new best-ever is found (is_new_record=true).
        if is_new_record {
            best_ever_genome_hash = Some(child_uid);
            best_ever_tracker_uid = Some(tracker_uid);
        }

        // ── Sprint 3.10: Record ΔHC probe entry (fires after was_inserted is known) ──
        delta_hc_probe.record(parent_hc, child_hc_for_probe, was_inserted);

        // ── Full-coverage archive membership tracking ──────────────────────────
        // Register EVERY archive insertion in archive_members map.
        // For best-ever champions (is_new_record=true), store their tracker_uid.
        // For all other insertions, store None as tracker_uid.
        if was_inserted {
            let t_uid = if is_new_record { Some(tracker_uid) } else { None };
            archive_members.insert(child_uid, (t_uid, child_score.official_total));
            total_insertions += 1;
        }

        // ── Sprint 3.8: Record feasible genome discovery ───────────────────────
        // Fires for every feasible child, whether or not it was admitted.
        // HashMap deduplication: if genome_hash already exists (rediscovery after eviction),
        // update admitted_at and clear eviction fields rather than creating a duplicate.
        if child_score.feasible {
            let hc = child_score.hc_coverage
                + child_score.hc_skills
                + child_score.hc_one_shift_per_day
                + child_score.hc_forbidden_successions;
            if let Some(existing) = feasible_lifecycles.get_mut(&child_uid) {
                // Rediscovery: update admission fields if newly admitted
                if was_inserted {
                    existing.admitted_at = Some(g);
                    existing.evicted_at = None;
                    existing.exit_reason = None;
                    existing.dominator_hash = None;
                    feasible_archive_members.insert(child_uid);
                }
            } else {
                // First discovery
                let lc = FeasibleLifecycle {
                    genome_hash: child_uid,
                    discovered_at: g,
                    admitted_at: if was_inserted { Some(g) } else { None },
                    evicted_at: None,
                    exit_reason: None,
                    dominator_hash: None,
                    hc_total: hc,
                    official_total: child_score.official_total,
                    proxy: child_fitness.clone(),
                };
                feasible_lifecycles.insert(child_uid, lc);
                if was_inserted {
                    feasible_archive_members.insert(child_uid);
                }
            }
        }

        // ── Notify evictions using full-coverage map ───────────────────────────
        // Detect any archive member evicted this generation.
        // If the evicted member was a best-ever champion (has Some(tracker_uid)),
        // notify the ChampionTracker so it records the eviction reason.
        for old_uid in &old_uids {
            if !new_uids.contains(old_uid) {
                total_evictions += 1;
                let reason = if was_inserted {
                    ExitReason::Dominated
                } else {
                    ExitReason::Unknown
                };
                if let Some(&(Some(t_uid), _ext)) = archive_members.get(old_uid) {
                    champion_tracker.notify_eviction(t_uid, g, reason.clone());
                    tracked_evictions += 1;

                    // ── Sprint 3.7: Fire DominationEvent for best-ever champion ─
                    // Fires once when the current best-ever champion is evicted.
                    // Seed-agnostic: uses best_ever_tracker_uid, not hardcoded UID 4.
                    if Some(t_uid) == best_ever_tracker_uid && domination_event.is_none() {
                        best_ever_eviction_gen = g;
                        if let Some(ref v_proxy) = best_ever_proxy_before {
                            if let Some(ref v_genome) = best_ever_genome_cache {
                                let v_ext = score_inrc_official(v_genome, &scenario, &inrc_optimizer);
                                domination_event = Some(DominationEvent {
                                    eviction_gen: g,
                                    victim_genome_hash: *old_uid,
                                    victim_tracker_uid: t_uid,
                                    dominator_genome_hash: child_uid,
                                    victim_proxy: v_proxy.clone(),
                                    dominator_proxy: child_fitness.clone(),
                                    victim_external: ExtMetrics {
                                        hc_coverage: v_ext.hc_coverage,
                                        hc_skills: v_ext.hc_skills,
                                        hc_one_shift_per_day: v_ext.hc_one_shift_per_day,
                                        hc_forbidden_successions: v_ext.hc_forbidden_successions,
                                        soft_total: v_ext.soft_total,
                                        official_total: v_ext.official_total,
                                    },
                                    dominator_external: ExtMetrics {
                                        hc_coverage: child_score.hc_coverage,
                                        hc_skills: child_score.hc_skills,
                                        hc_one_shift_per_day: child_score.hc_one_shift_per_day,
                                        hc_forbidden_successions: child_score.hc_forbidden_successions,
                                        soft_total: child_score.soft_total,
                                        official_total: child_score.official_total,
                                    },
                                    victim_front_rank_prev_gen: 0,
                                    victim_crowding_prev_gen: best_ever_crowding_prev_gen,
                                });
                            }
                        }
                    }
                }
                // ── Sprint 3.8: Record feasible genome eviction ───────────────
                if feasible_archive_members.contains(old_uid) {
                    if let Some(lc) = feasible_lifecycles.get_mut(old_uid) {
                        lc.evicted_at = Some(g);
                        lc.exit_reason = Some(reason);
                        // Record dominator identity for Retention Failure causal chain
                        if was_inserted {
                            lc.dominator_hash = Some(child_uid);
                        }
                    }
                    feasible_archive_members.remove(old_uid);
                }

                // Remove from membership map — this genome is no longer in the archive
                archive_members.remove(old_uid);
            }
        }

        // ── Sprint 3.8/3.9: Census + HC distribution sampling every 100 generations ──
        // Full-archive feasibility scoring is O(archive_size) per gen — sample
        // every 100 gens to keep the 5000-gen run tractable.
        if g % 100 == 0 || g == max_generations {
            let mut feasible_n = 0usize;
            let mut near5 = 0usize;
            let mut near10 = 0usize;
            let mut infeasible_n = 0usize;
            // Sprint 3.9: HC distribution accumulators
            let mut hc_min = usize::MAX;
            let mut hc_max = 0usize;
            let mut hc_sum = 0u64;
            let mut hc_count = 0usize;
            let mut hc0 = 0usize;
            let mut hc_le5 = 0usize;
            let mut hc_le10 = 0usize;
            let mut hc_le20 = 0usize;
            let mut hc_le50 = 0usize;
            let mut hc_gt50 = 0usize;
            for sol in &engine.archive.solutions {
                let sc = score_inrc_official(&sol.genome, &scenario, &inrc_optimizer);
                let hc = sc.hc_coverage + sc.hc_skills
                    + sc.hc_one_shift_per_day + sc.hc_forbidden_successions;
                if sc.feasible        { feasible_n  += 1; }
                else if hc <= 5      { near5       += 1; }
                else if hc <= 10     { near10      += 1; }
                else                 { infeasible_n += 1; }
                // HC distribution
                if hc < hc_min { hc_min = hc; }
                if hc > hc_max { hc_max = hc; }
                hc_sum += hc as u64;
                hc_count += 1;
                if hc == 0       { hc0     += 1; }
                else if hc <= 5  { hc_le5  += 1; }
                else if hc <= 10 { hc_le10 += 1; }
                else if hc <= 20 { hc_le20 += 1; }
                else if hc <= 50 { hc_le50 += 1; }
                else             { hc_gt50 += 1; }
            }
            census_timeline.push((g, feasible_n, near5, near10, infeasible_n));
            let safe_min = if hc_count == 0 { 0 } else { hc_min };
            hc_dist_timeline.push((g, safe_min, hc_max, hc_sum, hc_count,
                                   hc0, hc_le5, hc_le10, hc_le20, hc_le50, hc_gt50));
            // Sprint 3.9: Capture gen=0 initialization snapshot (first census point)
            if g == 0 {
                for sol in &engine.archive.solutions {
                    let sc = score_inrc_official(&sol.genome, &scenario, &inrc_optimizer);
                    let hc = sc.hc_coverage + sc.hc_skills
                        + sc.hc_one_shift_per_day + sc.hc_forbidden_successions;
                    init_hc_values.push(hc);
                }
            }
        }

        // ── Sprint 3.7: Update best-ever crowding + genome cache each generation ─
        // Computed at END of each generation (post-insertion archive state).
        // Stored as "prev gen" value — used if the champion is evicted next generation.
        // Front rank is always 0 in a Pareto archive (all members non-dominated).
        if let Some(h) = best_ever_genome_hash {
            best_ever_crowding_prev_gen = crowding_distance_for(&engine.archive.solutions, h);
            if let Some(sol) = engine.archive.solutions.iter().find(|s| s.uid == h) {
                best_ever_genome_cache = Some(sol.genome.clone());
            }
        }

        // ── Progress heartbeat ─────────────────────────────────────────────────
        if g % 500 == 0 || g == max_generations {
            println!(
                "Gen {:>5} | archive={:>4} | best_ext_ever={:.0}",
                g,
                engine.archive.solutions.len(),
                best_external_ever,
            );
        }
    }

    // ── Finalize champion tracker ──────────────────────────────────────────────
    let best_in_final = engine.archive.solutions.iter()
        .map(|sol| score_inrc_official(&sol.genome, &scenario, &inrc_optimizer).official_total)
        .fold(f64::MAX, f64::min);

    champion_tracker.finalize(best_in_final);

    // ── FeasibilitySnapshot at final generation ────────────────────────────────
    let mut feasibility_snap = FeasibilitySnapshot::new(max_generations);

    for (idx, sol) in engine.archive.solutions.iter().enumerate() {
        let sc = score_inrc_official(&sol.genome, &scenario, &inrc_optimizer);
        let member_snap = MemberSnapshot {
            archive_index: idx,
            hc_coverage: sc.hc_coverage,
            hc_skills: sc.hc_skills,
            hc_one_shift_per_day: sc.hc_one_shift_per_day,
            hc_forbidden_successions: sc.hc_forbidden_successions,
            total_hc_violations: sc.total_hc_penalty,
            soft_total: sc.soft_total,
            official_total: sc.official_total as i64,
            feasible: sc.feasible,
            objective_vector: sol.fitness.clone(),
        };
        feasibility_snap.add_member(member_snap);
    }

    // ── Write JSONL artifacts ──────────────────────────────────────────────────
    let champion_jsonl = champion_tracker.to_json_lines();
    let mut champ_file = File::create("champion_lifecycle.jsonl").unwrap();
    champ_file.write_all(champion_jsonl.as_bytes()).unwrap();
    println!("\nWrote champion_lifecycle.jsonl ({} bytes)", champion_jsonl.len());

    let feasibility_jsonl = feasibility_snap.to_json_lines();
    let mut feas_file = File::create("feasibility_snapshot.jsonl").unwrap();
    feas_file.write_all(feasibility_jsonl.as_bytes()).unwrap();
    println!("Wrote feasibility_snapshot.jsonl ({} bytes)", feasibility_jsonl.len());

    // ── Sprint 3.7: Write gen{N}_domination_report.md ─────────────────────────
    if let Some(ref ev) = domination_event {
        let obj_names = ["O1 (HC_Coverage)", "O2 (HC_Skills)", "O3 (HC_Successions)", "O4 (SoftTotal)", "O5 (HC_Violations)"];

        // Dominance proof: dominator[i] <= victim[i] for all i, strict < for at least one
        let all_leq = ev.dominator_proxy.iter().zip(ev.victim_proxy.iter())
            .all(|(d, v)| d <= v);
        let any_lt = ev.dominator_proxy.iter().zip(ev.victim_proxy.iter())
            .any(|(d, v)| d < v);
        let dominance_holds = all_leq && any_lt;

        let mut report = String::new();
        report.push_str(&format!("# Gen-{} Domination Report — SD-006 Proxy Geometry Attribution\n\n", ev.eviction_gen));
        report.push_str("**Sprint:** 3.7  \n");
        report.push_str(&format!("**Seed:** {}  \n", seed));
        report.push_str("**Instance:** n050w4  \n");
        report.push_str("**Observer:** `inrc_official_total`  \n");
        report.push_str(&format!("**Event:** Archive eviction of best-ever champion at generation {}  \n\n", ev.eviction_gen));
        report.push_str("---\n\n");

        // Section 1: Victim
        report.push_str(&format!("## 1. Victim — Tracker UID {}\n\n", ev.victim_tracker_uid));
        report.push_str("| Field | Value |\n|---|---|\n");
        report.push_str(&format!("| Genome hash | {} |\n", ev.victim_genome_hash));
        report.push_str(&format!("| Tracker UID | {} |\n", ev.victim_tracker_uid));
        report.push_str(&format!("| OfficialTotal | {:.0} |\n", ev.victim_external.official_total));
        report.push_str(&format!("| HC_Coverage | {} |\n", ev.victim_external.hc_coverage));
        report.push_str(&format!("| HC_Skills | {} |\n", ev.victim_external.hc_skills));
        report.push_str(&format!("| HC_OneShiftPerDay | {} |\n", ev.victim_external.hc_one_shift_per_day));
        report.push_str(&format!("| HC_ForbiddenSuccessions | {} |\n", ev.victim_external.hc_forbidden_successions));
        report.push_str(&format!("| SoftTotal | {} |\n", ev.victim_external.soft_total));
        report.push_str("\n");

        // Section 2: Dominator
        report.push_str("## 2. Dominating Genome\n\n");
        report.push_str("| Field | Value |\n|---|---|\n");
        report.push_str(&format!("| Genome hash | {} |\n", ev.dominator_genome_hash));
        report.push_str(&format!("| OfficialTotal | {:.0} |\n", ev.dominator_external.official_total));
        report.push_str(&format!("| HC_Coverage | {} |\n", ev.dominator_external.hc_coverage));
        report.push_str(&format!("| HC_Skills | {} |\n", ev.dominator_external.hc_skills));
        report.push_str(&format!("| HC_OneShiftPerDay | {} |\n", ev.dominator_external.hc_one_shift_per_day));
        report.push_str(&format!("| HC_ForbiddenSuccessions | {} |\n", ev.dominator_external.hc_forbidden_successions));
        report.push_str(&format!("| SoftTotal | {} |\n", ev.dominator_external.soft_total));
        report.push_str("\n");

        // Section 3: Proxy delta table
        report.push_str("## 3. Proxy Delta Table (ΔO1–ΔO5)\n\n");
        report.push_str("Δ = Dominator − Victim. Negative = dominator improved on this objective.\n\n");
        report.push_str("| Objective | Victim | Dominator | Δ | Direction |\n|---|---|---|---|---|\n");
        for (i, name) in obj_names.iter().enumerate() {
            let v = ev.victim_proxy[i];
            let d = ev.dominator_proxy[i];
            let delta = d - v;
            let dir = if delta < -0.5 { "↓ improved" } else if delta > 0.5 { "↑ worsened" } else { "= equal" };
            report.push_str(&format!("| {} | {:.0} | {:.0} | {:+.0} | {} |\n", name, v, d, delta, dir));
        }
        report.push_str("\n");

        // Section 4: External delta table
        report.push_str("## 4. External Delta Table\n\n");
        report.push_str("Δ = Dominator − Victim. Positive = dominator is worse externally.\n\n");
        report.push_str("| Metric | Victim | Dominator | Δ |\n|---|---|---|---|\n");
        // usize fields
        let vc = ev.victim_external.hc_coverage as i64;
        let dc = ev.dominator_external.hc_coverage as i64;
        report.push_str(&format!("| HC_Coverage | {} | {} | {:+} |\n", vc, dc, dc - vc));
        let vs = ev.victim_external.hc_skills as i64;
        let ds = ev.dominator_external.hc_skills as i64;
        report.push_str(&format!("| HC_Skills | {} | {} | {:+} |\n", vs, ds, ds - vs));
        let vo = ev.victim_external.hc_one_shift_per_day as i64;
        let do_ = ev.dominator_external.hc_one_shift_per_day as i64;
        report.push_str(&format!("| HC_OneShiftPerDay | {} | {} | {:+} |\n", vo, do_, do_ - vo));
        let vf = ev.victim_external.hc_forbidden_successions as i64;
        let df = ev.dominator_external.hc_forbidden_successions as i64;
        report.push_str(&format!("| HC_ForbiddenSucc | {} | {} | {:+} |\n", vf, df, df - vf));
        // i32 field
        let vst = ev.victim_external.soft_total as i64;
        let dst = ev.dominator_external.soft_total as i64;
        report.push_str(&format!("| SoftTotal | {} | {} | {:+} |\n", vst, dst, dst - vst));
        // f64 field
        let vot = ev.victim_external.official_total;
        let dot = ev.dominator_external.official_total;
        report.push_str(&format!("| OfficialTotal | {:.0} | {:.0} | {:+.0} |\n", vot, dot, dot - vot));
        report.push_str("\n");

        // Section 5: Dominance proof
        report.push_str("## 5. Dominance Proof\n\n");
        report.push_str("For Pareto domination: dominator ≤ victim on ALL objectives, strict < on at least one.\n\n");
        report.push_str("| Objective | Victim | Dominator | Relation | Holds? |\n|---|---|---|---|---|\n");
        for (i, name) in obj_names.iter().enumerate() {
            let v = ev.victim_proxy[i];
            let d = ev.dominator_proxy[i];
            let rel = if (d - v).abs() < 0.5 { "=" } else if d < v { "<" } else { ">" };
            let holds = d <= v;
            report.push_str(&format!("| {} | {:.0} | {:.0} | {} | {} |\n",
                name, v, d, rel, if holds { "✓" } else { "✗" }));
        }
        report.push_str(&format!("\n**Domination holds: {}**\n\n", if dominance_holds { "YES" } else { "NO — INSTRUMENTATION ERROR" }));

        // Section 6: Archive rank before eviction
        let prev_gen = ev.eviction_gen.saturating_sub(1);
        report.push_str(&format!("## 6. Archive Rank Before Eviction (Gen {})\n\n", prev_gen));
        report.push_str("All members of a Pareto archive are non-dominated by definition (Front 0).\n");
        report.push_str("Crowding distance measures isolation within the front.\n\n");
        report.push_str("| Field | Value |\n|---|---|\n");
        report.push_str(&format!("| Pareto Front Rank | {} |\n", ev.victim_front_rank_prev_gen));
        let crowding_str = if ev.victim_crowding_prev_gen.is_infinite() {
            "∞ (boundary solution)".to_string()
        } else {
            format!("{:.4}", ev.victim_crowding_prev_gen)
        };
        report.push_str(&format!("| Crowding Distance | {} |\n", crowding_str));
        report.push_str("\n");
        let uid_label = format!("UID {}", ev.victim_tracker_uid);
        if ev.victim_crowding_prev_gen.is_infinite() {
            report.push_str(&format!("{} was a **boundary solution** at gen {} — it occupied an extreme position in at least one proxy objective dimension. This indicates it was NOT marginal before eviction; it was a structurally important archive member.\n\n", uid_label, prev_gen));
        } else if ev.victim_crowding_prev_gen > 1.0 {
            report.push_str(&format!("{} had **high crowding distance** at gen {} — it was well-isolated in proxy space, indicating it was NOT marginal before eviction.\n\n", uid_label, prev_gen));
        } else {
            report.push_str(&format!("{} had **low crowding distance** at gen {} — it was in a dense region of proxy space, suggesting it may have been marginal before eviction.\n\n", uid_label, prev_gen));
        }

        // Attribution summary
        report.push_str("---\n\n");
        report.push_str("## Attribution Summary\n\n");
        // Find which objectives drove the domination
        let driving_objs: Vec<&str> = obj_names.iter().enumerate()
            .filter(|(i, _)| ev.dominator_proxy[*i] < ev.victim_proxy[*i] - 0.5)
            .map(|(_, name)| *name)
            .collect();
        let delta_official = ev.dominator_external.official_total - ev.victim_external.official_total;
        if driving_objs.is_empty() {
            report.push_str("**WARNING**: No single objective shows a clear improvement. Domination may be marginal (Case D).\n\n");
        } else {
            report.push_str(&format!(
                "UID {} was evicted because improving **{}** was considered worth sacrificing **{:+.0} points** of official quality.\n\n",
                ev.victim_tracker_uid,
                driving_objs.join(", "),
                delta_official
            ));
        }
        report.push_str("```\n");
        for (i, name) in obj_names.iter().enumerate() {
            let delta = ev.dominator_proxy[i] - ev.victim_proxy[i];
            report.push_str(&format!("Δ{} = {:+.0}\n", name, delta));
        }
        report.push_str(&format!("\nwhich produced Pareto domination,\n\nwhile causing\n\nΔOfficialTotal = {:+.0}\n", delta_official));
        report.push_str("```\n");

        let report_path = format!("gen{}_domination_report.md", ev.eviction_gen);
        let mut report_file = File::create(&report_path).unwrap();
        report_file.write_all(report.as_bytes()).unwrap();
        println!("\nWrote {} ({} bytes)", &report_path, report.len());

        // Console echo of key numbers
        println!("\n=== Sprint 3.7: Gen-{} Domination Event ===", ev.eviction_gen);
        println!("Victim  UID-4 official_total : {:.0}", ev.victim_external.official_total);
        println!("Dominator official_total     : {:.0}", ev.dominator_external.official_total);
        println!("ΔOfficialTotal               : {:+.0}", ev.dominator_external.official_total - ev.victim_external.official_total);
        println!("Victim crowding (gen 174)     : {}", crowding_str);
        println!("Domination holds             : {}", dominance_holds);
        for (i, name) in obj_names.iter().enumerate() {
            println!("  Δ{} = {:+.0}", name, ev.dominator_proxy[i] - ev.victim_proxy[i]);
        }
    } else {
        println!("\n[Sprint 3.7] WARNING: DominationEvent was NOT fired.");
        println!("  best_ever_genome_hash set : {}", best_ever_genome_hash.is_some());
        println!("  best_ever_genome_cache set: {}", best_ever_genome_cache.is_some());
        println!("  best_ever_tracker_uid     : {:?}", best_ever_tracker_uid);
        println!("  best_ever_eviction_gen    : {}", best_ever_eviction_gen);
        println!("  Possible causes:");
        println!("    - No champion retention error in this run (best_ever == best_final)");
        println!("    - Best-ever champion was never evicted (still in archive at end)");
        println!("    - best_ever_proxy_before was None at eviction time (champion not in archive before add())");
        println!("    - best_ever_genome_cache was None (champion not seen in archive at prev gen end)");
    }

    // ── Sprint 3.8: Write feasible_lineage_report.md ──────────────────────────
    {
        let obj_names = ["O1 (HC_Coverage)", "O2 (HC_Skills)", "O3 (HC_Successions)", "O4 (SoftTotal)", "O5 (HC_Violations)"];

        // Frozen definitions (sd005_sprint38_charter.md):
        //   first feasible = earliest discovered_at
        //   best feasible  = lowest official_total
        let first_feasible: Option<&FeasibleLifecycle> = feasible_lifecycles.values()
            .min_by_key(|lc| lc.discovered_at);
        let best_feasible: Option<&FeasibleLifecycle> = feasible_lifecycles.values()
            .min_by(|a, b| a.official_total.partial_cmp(&b.official_total).unwrap());

        let fmt_exit = |lc: &FeasibleLifecycle| -> &'static str {
            match lc.exit_reason {
                Some(ExitReason::Dominated)      => "Dominated",
                Some(ExitReason::Crowding)       => "Crowding",
                Some(ExitReason::ArchiveLimit)   => "ArchiveLimit",
                Some(ExitReason::Unknown)        => "Unknown",
                Some(ExitReason::NeverAdmitted)  => "NeverAdmitted",
                None => if lc.evicted_at.is_none() && lc.admitted_at.is_some() {
                    "Still in archive"
                } else if lc.admitted_at.is_none() {
                    "Never admitted"
                } else {
                    "None"
                },
            }
        };

        let fmt_lifetime = |lc: &FeasibleLifecycle| -> String {
            match (lc.admitted_at, lc.evicted_at) {
                (Some(a), Some(e)) => format!("{}", e.saturating_sub(a)),
                (Some(a), None)    => format!("{} (still in archive)", max_generations.saturating_sub(a)),
                (None, _)          => "N/A (never admitted)".to_string(),
            }
        };

        let mut report = String::new();
        report.push_str("# Feasible Lineage Report — SD-005 Causal Dependency Investigation\n\n");
        report.push_str("**Sprint:** 3.8  \n");
        report.push_str(&format!("**Seed:** {}  \n", seed));
        report.push_str("**Instance:** n050w4  \n");
        report.push_str("**Observer:** `inrc_official_total`  \n");
        report.push_str(&format!("**Generations:** {}  \n", max_generations));
        report.push_str(&format!("**Total feasible genomes discovered:** {}  \n\n", feasible_lifecycles.len()));
        report.push_str("---\n\n");

        // ── Section 1: First Feasible Genome ──────────────────────────────────
        report.push_str("## 1. First Feasible Genome\n\n");
        if let Some(lc) = first_feasible {
            report.push_str("| Field | Value |\n|---|---|\n");
            report.push_str(&format!("| Genome hash | {} |\n", lc.genome_hash));
            report.push_str(&format!("| Discovered at generation | {} |\n", lc.discovered_at));
            report.push_str(&format!("| HC_Total at discovery | {} |\n", lc.hc_total));
            report.push_str(&format!("| OfficialTotal at discovery | {:.0} |\n", lc.official_total));
            report.push_str(&format!("| Archive admitted? | {} |\n", lc.admitted_at.is_some()));
            match lc.admitted_at {
                Some(a) => report.push_str(&format!("| Admitted at generation | {} |\n", a)),
                None    => report.push_str("| Admitted at generation | N/A |\n"),
            }
            report.push_str(&format!("| Archive lifetime (gens) | {} |\n", fmt_lifetime(lc)));
            match lc.evicted_at {
                Some(e) => report.push_str(&format!("| Evicted at generation | {} |\n", e)),
                None    => report.push_str("| Evicted at generation | N/A |\n"),
            }
            report.push_str(&format!("| Exit reason | {} |\n", fmt_exit(lc)));
            match lc.dominator_hash {
                Some(dh) => report.push_str(&format!("| Dominator genome hash | {} |\n", dh)),
                None     => report.push_str("| Dominator genome hash | N/A |\n"),
            }
            report.push_str("\n**Proxy objective vector at discovery:**\n\n");
            report.push_str("| Objective | Value |\n|---|---|\n");
            for (i, name) in obj_names.iter().enumerate() {
                if i < lc.proxy.len() {
                    report.push_str(&format!("| {} | {:.0} |\n", name, lc.proxy[i]));
                }
            }
            report.push_str("\n");
        } else {
            report.push_str(&format!(
                "**No feasible genome was ever discovered.**\n\n\
                 Classification: **Discovery Failure** — the evaluator never returned `feasible=true` \
                 in {} generations.\n\n",
                max_generations
            ));
        }

        // ── Section 2: Best Feasible Genome ───────────────────────────────────
        report.push_str("## 2. Best Feasible Genome\n\n");
        if let (Some(first), Some(best)) = (first_feasible, best_feasible) {
            if best.genome_hash == first.genome_hash {
                report.push_str("Same as Section 1 (only one feasible genome discovered, or first == best by official_total).\n\n");
            } else {
                report.push_str("| Field | Value |\n|---|---|\n");
                report.push_str(&format!("| Genome hash | {} |\n", best.genome_hash));
                report.push_str(&format!("| Discovered at generation | {} |\n", best.discovered_at));
                report.push_str(&format!("| OfficialTotal at discovery | {:.0} |\n", best.official_total));
                report.push_str(&format!("| Archive lifetime (gens) | {} |\n", fmt_lifetime(best)));
                report.push_str(&format!("| Exit reason | {} |\n", fmt_exit(best)));
                match best.dominator_hash {
                    Some(dh) => report.push_str(&format!("| Dominator genome hash | {} |\n", dh)),
                    None     => report.push_str("| Dominator genome hash | N/A |\n"),
                }
                report.push_str("\n");
            }
        } else if first_feasible.is_none() {
            report.push_str("No feasible genome discovered — see Section 1.\n\n");
        }

        // ── Section 3: Feasibility Census Timeline ─────────────────────────────
        report.push_str("## 3. Feasibility Census Timeline\n\n");
        report.push_str("Sampled every 100 generations. `near_feasible_5` = HC_Total ≤ 5; `near_feasible_10` = HC_Total ≤ 10.\n\n");
        report.push_str("| Generation | feasible_count | near_feasible_5 | near_feasible_10 | infeasible_count |\n");
        report.push_str("|---|---|---|---|---|\n");
        for &(cgen, f, n5, n10, inf) in &census_timeline {
            report.push_str(&format!("| {} | {} | {} | {} | {} |\n", cgen, f, n5, n10, inf));
        }
        report.push_str("\n");

        // ── Section 4: SD-005 Classification ──────────────────────────────────
        report.push_str("## 4. SD-005 Classification\n\n");
        report.push_str("Applying frozen classification table from `sd005_sprint38_charter.md`.\n\n");

        let total_feasible      = feasible_lifecycles.len();
        let total_admitted      = feasible_lifecycles.values().filter(|lc| lc.admitted_at.is_some()).count();
        let total_evicted       = feasible_lifecycles.values().filter(|lc| lc.evicted_at.is_some()).count();
        let still_in_archive    = feasible_archive_members.len();
        let dominated_evictions = feasible_lifecycles.values()
            .filter(|lc| matches!(lc.exit_reason, Some(ExitReason::Dominated)))
            .count();

        report.push_str("| Metric | Count |\n|---|---|\n");
        report.push_str(&format!("| Total feasible genomes discovered | {} |\n", total_feasible));
        report.push_str(&format!("| Admitted to archive | {} |\n", total_admitted));
        report.push_str(&format!("| Evicted from archive | {} |\n", total_evicted));
        report.push_str(&format!("| Evicted by Dominated | {} |\n", dominated_evictions));
        report.push_str(&format!("| Still in archive at gen {} | {} |\n\n", max_generations, still_in_archive));

        let classification = if total_feasible == 0 {
            "**Discovery Failure** — No feasible genome was ever produced by the evaluator. \
             The O3 mechanism is not implicated; the root cause is upstream of the archive \
             (evaluator landscape, mutation operators, or constraint structure).".to_string()
        } else if total_admitted == 0 {
            "**Admission Failure** — Feasible genomes were produced but never admitted to the archive. \
             The Pareto archive rejected all feasible candidates at the admission boundary.".to_string()
        } else if total_evicted > 0 && still_in_archive == 0 {
            "**Retention Failure** — Feasible genomes were admitted then evicted. \
             The archive admitted feasibility but could not retain it under selection pressure.".to_string()
        } else if still_in_archive > 0 {
            "**SD-005 Falsified** — Feasible solutions persist in the archive at the final generation.".to_string()
        } else {
            "**Representation Failure** — Feasible solutions survive but the archive remains ~0% feasible.".to_string()
        };

        report.push_str(&format!("### Classification\n\n{}\n\n", classification));

        if total_evicted > 0 && dominated_evictions > 0 {
            report.push_str("### Causal Chain Evidence\n\n");
            report.push_str("At least one feasible genome was evicted by Pareto domination. \
                             Combined with SD-006 (O3 pressure causes champion eviction), \
                             this is consistent with the unified causal chain:\n\n");
            report.push_str("```\n");
            report.push_str("O3 pressure\n    ↓\nProxy domination\n    ↓\n");
            report.push_str("Champion eviction (SD-006, CLOSED)\n    ↓\n");
            report.push_str("Feasible genome eviction (SD-005)\n    ↓\n0% feasible archive\n");
            report.push_str("```\n\n");
            report.push_str("See `gen283_domination_report.md` for the canonical SD-006 domination event.\n\n");
        }

        let report_path = "feasible_lineage_report.md";
        let mut report_file = File::create(report_path).unwrap();
        report_file.write_all(report.as_bytes()).unwrap();
        println!("\nWrote {} ({} bytes)", report_path, report.len());
        println!("Feasible genomes discovered : {}", total_feasible);
        println!("Feasible genomes admitted   : {}", total_admitted);
        println!("Feasible genomes evicted    : {}", total_evicted);
        println!("Still in archive            : {}", still_in_archive);
    }

    // ── Sprint 3.9: Write hc_distribution_report.md ───────────────────────────
    {
        let mut report = String::new();
        report.push_str("# HC_Total Distribution Report — SD-007 Constraint Landscape Probe\n\n");
        report.push_str("**Sprint:** 3.9  \n");
        report.push_str(&format!("**Seed:** {}  \n", seed));
        report.push_str("**Instance:** n050w4  \n");
        report.push_str(&format!("**Generations:** {}  \n", max_generations));
        report.push_str("**Probe:** HC_Total = hc_coverage + hc_skills + hc_one_shift_per_day + hc_forbidden_successions  \n\n");
        report.push_str("---\n\n");

        // Section 1: Initialization snapshot (gen=0)
        report.push_str("## 1. Initialization Snapshot (gen=0)\n\n");
        if init_hc_values.is_empty() {
            report.push_str("*No gen=0 archive members recorded (archive empty at gen=0 census).*\n\n");
        } else {
            let n = init_hc_values.len();
            let sum: usize = init_hc_values.iter().sum();
            let mean = sum as f64 / n as f64;
            let mut sorted = init_hc_values.clone();
            sorted.sort_unstable();
            let median = if n % 2 == 0 {
                (sorted[n/2 - 1] + sorted[n/2]) as f64 / 2.0
            } else {
                sorted[n/2] as f64
            };
            let min_hc = sorted[0];
            let max_hc = sorted[n - 1];
            report.push_str("| Metric | Value |\n|---|---|\n");
            report.push_str(&format!("| Archive members at gen=0 | {} |\n", n));
            report.push_str(&format!("| Min HC_Total | {} |\n", min_hc));
            report.push_str(&format!("| Max HC_Total | {} |\n", max_hc));
            report.push_str(&format!("| Mean HC_Total | {:.2} |\n", mean));
            report.push_str(&format!("| Median HC_Total | {:.1} |\n", median));
            report.push_str("\n**RC-3 Assessment (Initialization Depth):**  \n");
            if median > 50.0 {
                report.push_str(&format!("Median HC_Total = {:.1} > 50 → RC-3 candidate confirmed (deep initialization).\n\n", median));
            } else if median > 20.0 {
                report.push_str(&format!("Median HC_Total = {:.1} (moderate initialization depth; RC-3 possible).\n\n", median));
            } else {
                report.push_str(&format!("Median HC_Total = {:.1} ≤ 20 → RC-3 not primary cause (initialization is shallow).\n\n", median));
            }
        }

        // Section 2: HC_Total trajectory across 5000 generations
        report.push_str("## 2. HC_Total Trajectory (every 100 generations)\n\n");
        report.push_str("| Gen | Min | Max | Mean | HC=0 | HC≤5 | HC≤10 | HC≤20 | HC≤50 | HC>50 |\n");
        report.push_str("|---|---|---|---|---|---|---|---|---|---|\n");
        for &(cgen, min_hc, max_hc, hc_sum, hc_count, hc0, hc_le5, hc_le10, hc_le20, hc_le50, hc_gt50) in &hc_dist_timeline {
            let mean_hc = if hc_count > 0 { hc_sum as f64 / hc_count as f64 } else { 0.0 };
            report.push_str(&format!(
                "| {} | {} | {} | {:.1} | {} | {} | {} | {} | {} | {} |\n",
                cgen, min_hc, max_hc, mean_hc, hc0, hc_le5, hc_le10, hc_le20, hc_le50, hc_gt50
            ));
        }
        report.push_str("\n");

        // Section 3: RC-1 assessment (operator incapacity — is HC_Total decreasing?)
        report.push_str("## 3. RC-1 Assessment — Operator Incapacity\n\n");
        if hc_dist_timeline.len() >= 2 {
            let first = &hc_dist_timeline[0];
            let last = &hc_dist_timeline[hc_dist_timeline.len() - 1];
            let first_mean = if first.4 > 0 { first.3 as f64 / first.4 as f64 } else { 0.0 };
            let last_mean = if last.4 > 0 { last.3 as f64 / last.4 as f64 } else { 0.0 };
            let delta = last_mean - first_mean;
            report.push_str(&format!("Mean HC_Total at gen={}: {:.2}  \n", first.0, first_mean));
            report.push_str(&format!("Mean HC_Total at gen={}: {:.2}  \n", last.0, last_mean));
            report.push_str(&format!("Δ mean HC_Total (last − first): {:.2}  \n\n", delta));
            if delta >= -1.0 {
                report.push_str("**RC-1 CONFIRMED:** Mean HC_Total shows no downward trend (Δ ≈ 0 or positive). \
                                  Mutation operators are not reducing HC violations over 5000 generations. \
                                  Operator incapacity is the primary root cause.\n\n");
            } else if delta < -10.0 {
                report.push_str("**RC-1 FALSIFIED:** Mean HC_Total shows clear downward trend. \
                                  Operators are making progress; stochastic factors or proxy misalignment \
                                  may be preventing feasibility crossing.\n\n");
            } else {
                report.push_str(&format!("**RC-1 PARTIAL:** Mean HC_Total decreased by {:.2} over {} generations. \
                                  Progress is slow — operators may be weakly capable but insufficient \
                                  for the n050w4 constraint landscape.\n\n", delta.abs(), max_generations));
            }
        } else {
            report.push_str("*Insufficient data points for trajectory analysis.*\n\n");
        }

        // Section 4: Summary and classification
        report.push_str("## 4. Summary\n\n");
        let first_mean = hc_dist_timeline.first()
            .map(|s| if s.4 > 0 { s.3 as f64 / s.4 as f64 } else { 0.0 })
            .unwrap_or(0.0);
        let last_min = hc_dist_timeline.last().map(|s| s.1).unwrap_or(0);
        let last_max = hc_dist_timeline.last().map(|s| s.2).unwrap_or(0);
        let last_mean = hc_dist_timeline.last()
            .map(|s| if s.4 > 0 { s.3 as f64 / s.4 as f64 } else { 0.0 })
            .unwrap_or(0.0);
        report.push_str("| Metric | Value |\n|---|---|\n");
        report.push_str(&format!("| Mean HC_Total at gen=0 | {:.2} |\n", first_mean));
        report.push_str(&format!("| Mean HC_Total at gen={} | {:.2} |\n", max_generations, last_mean));
        report.push_str(&format!("| Min HC_Total at gen={} | {} |\n", max_generations, last_min));
        report.push_str(&format!("| Max HC_Total at gen={} | {} |\n", max_generations, last_max));
        report.push_str(&format!("| HC=0 (feasible) at gen={} | {} |\n",
            max_generations,
            hc_dist_timeline.last().map(|s| s.5).unwrap_or(0)));
        report.push_str("\n");

        let report_path = "hc_distribution_report.md";
        let mut report_file = File::create(report_path).unwrap();
        report_file.write_all(report.as_bytes()).unwrap();
        println!("\nWrote {} ({} bytes)", report_path, report.len());
    }

    // ── Sprint 3.10: Write delta_hc_report.md ─────────────────────────────────
    // Classification table (frozen — sd007_resolution.md):
    //   P(delta_hc < 0) ≈ 0                                    → RC-1 CONFIRMED
    //   P(delta_hc < 0) > 0.1 AND P(improving AND inserted) ≈ 0 → RC-2 CONFIRMED
    //   Both probabilities > 0                                   → RC-1 + RC-2 interaction
    {
        let p_imp = delta_hc_probe.p_improving();
        let p_imp_ins = delta_hc_probe.p_improving_inserted();
        let mean_d = delta_hc_probe.mean_delta();

        let classification = if p_imp < 0.01 {
            "**RC-1 CONFIRMED** — Mutation operator is incapable of reducing HC_Total. \
             P(child_hc < parent_hc) ≈ 0 across all evaluated offspring. \
             The operator cannot navigate toward feasibility regardless of selection pressure."
        } else if p_imp >= 0.10 && p_imp_ins < 0.01 {
            "**RC-2 CONFIRMED** — Mutation operator CAN reduce HC_Total (P > 0.10), \
             but HC-improving offspring are almost never admitted to the archive. \
             Selection/proxy pressure is suppressing feasibility-directed progress."
        } else if p_imp >= 0.01 && p_imp < 0.10 {
            "**RC-1 PARTIAL + RC-2 PLAUSIBLE** — Mutation operator rarely reduces HC_Total \
             (P < 0.10), and the few improving offspring that exist may also face selection \
             pressure. Both RC-1 and RC-2 contribute."
        } else {
            "**RC-1 + RC-2 INTERACTION** — Both P(improving) and P(improving AND inserted) \
             are non-trivial. Operator has some capacity but selection further suppresses it."
        };

        let mut report = String::new();
        report.push_str("# ΔHC Offspring Probe Report — SD-007 Root Cause Isolation\n\n");
        report.push_str("**Sprint:** 3.10  \n");
        report.push_str(&format!("**Seed:** {}  \n", seed));
        report.push_str("**Instance:** n050w4  \n");
        report.push_str(&format!("**Generations:** {}  \n", max_generations));
        report.push_str("**Probe:** ΔHC = child_hc − parent_hc, measured BEFORE archive.add()  \n");
        report.push_str("**Scope:** All evaluated offspring (not just archive-admitted ones)  \n\n");
        report.push_str("---\n\n");

        // Section 1: Raw counts
        report.push_str("## 1. Raw Offspring Counts\n\n");
        report.push_str("| Category | Count | % of Total |\n|---|---|---|\n");
        let total = delta_hc_probe.total_offspring.max(1);
        report.push_str(&format!("| Total offspring evaluated | {} | 100.0% |\n", delta_hc_probe.total_offspring));
        report.push_str(&format!("| HC-improving (child_hc < parent_hc) | {} | {:.2}% |\n",
            delta_hc_probe.hc_improving,
            delta_hc_probe.hc_improving as f64 / total as f64 * 100.0));
        report.push_str(&format!("| HC-neutral (child_hc == parent_hc) | {} | {:.2}% |\n",
            delta_hc_probe.hc_neutral,
            delta_hc_probe.hc_neutral as f64 / total as f64 * 100.0));
        report.push_str(&format!("| HC-worsening (child_hc > parent_hc) | {} | {:.2}% |\n",
            delta_hc_probe.hc_worsening,
            delta_hc_probe.hc_worsening as f64 / total as f64 * 100.0));
        report.push_str("\n**Archive admission breakdown:**\n\n");
        report.push_str("| Category | Count |\n|---|---|\n");
        report.push_str(&format!("| HC-improving AND admitted | {} |\n", delta_hc_probe.hc_improving_inserted));
        report.push_str(&format!("| HC-worsening AND admitted | {} |\n", delta_hc_probe.hc_worsening_inserted));
        report.push_str("\n");

        // Section 2: Key probabilities
        report.push_str("## 2. Key Probabilities\n\n");
        report.push_str("| Probability | Value | Interpretation |\n|---|---|---|\n");
        report.push_str(&format!(
            "| P(child_hc < parent_hc) | {:.6} ({:.4}%) | Probability mutation reduces HC_Total |\n",
            p_imp, p_imp * 100.0));
        report.push_str(&format!(
            "| P(admitted \\| improving) | {:.6} ({:.4}%) | Of HC-improving offspring, fraction admitted |\n",
            p_imp_ins, p_imp_ins * 100.0));
        report.push_str(&format!(
            "| Mean ΔHC per offspring | {:.4} | Positive = operator drifts away from feasibility |\n",
            mean_d));
        report.push_str("\n");

        // Section 3: RC-1 vs RC-2 classification
        report.push_str("## 3. Root Cause Classification\n\n");
        report.push_str("**Frozen classification table (sd007_resolution.md):**\n\n");
        report.push_str("| Condition | Classification |\n|---|---|\n");
        report.push_str("| P(delta_hc < 0) ≈ 0 | RC-1 CONFIRMED (operator incapacity) |\n");
        report.push_str("| P(delta_hc < 0) > 0.1 AND P(improving AND inserted) ≈ 0 | RC-2 CONFIRMED (selection suppression) |\n");
        report.push_str("| Both probabilities > 0 | RC-1 + RC-2 interaction |\n\n");
        report.push_str(&format!("**Observed:** P(improving) = {:.6}, P(admitted | improving) = {:.6}  \n\n", p_imp, p_imp_ins));
        report.push_str(&format!("{}\n\n", classification));

        // Section 4: Mean ΔHC interpretation
        report.push_str("## 4. Mean ΔHC Interpretation\n\n");
        report.push_str(&format!("Mean ΔHC = {:.4} (penalty-weighted units; divide by 1000 for actual violation count delta)  \n\n", mean_d));
        if mean_d > 0.0 {
            report.push_str(&format!(
                "Mean ΔHC is **positive** ({:.4}): on average, each mutation step moves the offspring \
                 **away** from feasibility. The search is diverging from the feasibility boundary. \
                 This is consistent with RC-1 (operator incapacity) and/or RC-2 (selection pressure \
                 favouring HC-worsening offspring that improve other proxy objectives).\n\n",
                mean_d));
        } else if mean_d < -1.0 {
            report.push_str(&format!(
                "Mean ΔHC is **negative** ({:.4}): on average, each mutation step moves the offspring \
                 **toward** feasibility. Operator has directional capacity. If feasibility is still \
                 not reached, RC-2 (selection suppression) is the primary cause.\n\n",
                mean_d));
        } else {
            report.push_str(&format!(
                "Mean ΔHC ≈ 0 ({:.4}): mutations are HC-neutral on average. \
                 The operator neither improves nor worsens HC systematically.\n\n",
                mean_d));
        }

        let report_path = "delta_hc_report.md";
        let mut report_file = File::create(report_path).unwrap();
        report_file.write_all(report.as_bytes()).unwrap();
        println!("Wrote {} ({} bytes)", report_path, report.len());

        // Console echo of key numbers
        println!("\n=== Sprint 3.10: ΔHC Offspring Probe ===");
        println!("Total offspring evaluated    : {}", delta_hc_probe.total_offspring);
        println!("HC-improving                 : {} ({:.4}%)", delta_hc_probe.hc_improving, p_imp * 100.0);
        println!("HC-neutral                   : {}", delta_hc_probe.hc_neutral);
        println!("HC-worsening                 : {}", delta_hc_probe.hc_worsening);
        println!("HC-improving AND admitted    : {}", delta_hc_probe.hc_improving_inserted);
        println!("P(child_hc < parent_hc)      : {:.6}", p_imp);
        println!("P(admitted | improving)      : {:.6}", p_imp_ins);
        println!("Mean ΔHC                     : {:.4}", mean_d);
        println!("Classification               : {}", if p_imp < 0.01 { "RC-1 CONFIRMED" } else if p_imp >= 0.10 && p_imp_ins < 0.01 { "RC-2 CONFIRMED" } else { "RC-1+RC-2 INTERACTION" });
    }

    // ── Console summary ────────────────────────────────────────────────────────
    println!("\n=== Forensics Summary ===");
    println!("Best external ever seen : {:.0}", best_external_ever);
    println!("Best external in archive: {:.0}", best_in_final);
    let retention_error = if (best_in_final - best_external_ever).abs() < 1.0 { 0 } else { 1 };
    println!("Champion Retention Error: {}", retention_error);
    println!();

    // ── Coverage audit ────────────────────────────────────────────────────────
    println!("=== Coverage Audit ===");
    println!("Total archive insertions : {}", total_insertions);
    println!("Total archive evictions  : {}", total_evictions);
    println!("Tracked evictions        : {}", tracked_evictions);
    println!("Untracked evictions      : {}", total_evictions.saturating_sub(tracked_evictions));
    println!("Champions tracked        : {}", champion_tracker.records.len());
    println!("Archive size (final)     : {}", engine.archive.solutions.len());
    println!("Archive members map size : {}", archive_members.len());
    println!();

    feasibility_snap.print_report();

    // ── SD-003 pre-classification hint ────────────────────────────────────────
    println!("\n=== SD-003 Pre-Classification ===");
    let archive_size = engine.archive.solutions.len();
    let feasible_pct = if archive_size > 0 {
        feasibility_snap.feasible_count as f64 / archive_size as f64
    } else {
        0.0
    };

    if feasible_pct < 0.05 {
        println!("HINT: Feasibility Representation Failure (feasible={:.1}% < 5%)", feasible_pct * 100.0);
    }
    println!("See champion_lifecycle.jsonl for per-champion exit classification.");
    println!("Apply classification table v1 from forensics_manifest.txt.");
}