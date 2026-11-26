/// RP-410 Search Dynamics Telemetry
///
/// Provides zero-overhead-when-disabled telemetry for the ROADEF evolution loop.
///
/// # Design
///
/// Two record types are emitted:
///
/// - `MoveRecord`       — emitted once per accepted improvement (global-best change).
/// - `GenerationRecord` — emitted once per generation for the population best.
///
/// Both are serialised as newline-delimited JSON (JSONL) by `JsonlTelemetrySink`.
/// When telemetry is disabled, `NullTelemetrySink` is used and the compiler
/// eliminates all telemetry branches (zero allocation, zero branching in hot path).
///
/// # Zone Definitions (from RP-406C §14)
///
/// The load vector is the sorted arc-saturation vector (descending).
/// Zones are defined by rank (1-indexed):
///
///   Rank 1          → Peak
///   Ranks 2–20      → Shoulder
///   Ranks 21–100    → Transition
///   Ranks 101+      → Tail
///
/// Delta values are (new − old): negative = improvement.
///
/// # Move Classification
///
/// Each accepted move is classified by which zone shows the largest absolute
/// improvement. Mixed improvements (multiple zones improve by comparable amounts)
/// are recorded as `mixed`. Neutral moves (no zone improves by more than ε) are
/// recorded as `neutral`.
use std::io::Write;

// ---------------------------------------------------------------------------
// RP-408A: ComparatorMode
// ---------------------------------------------------------------------------

/// Which objective comparator is active for this run.
///
/// This is the sole manipulated variable in RP-408. All other subsystems
/// (constructor, operators, repair, evaluator, promotion pipeline) are
/// identical between modes. The comparator is selected once per run and
/// recorded in every telemetry record so that Scalar and Lexicographic
/// runs can be distinguished in post-hoc analysis without relying on
/// filenames or directory structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparatorMode {
    /// Scalar comparator: compares by `fitness()` (= −obj) only.
    /// This is the RP-406C baseline behaviour.
    Scalar,
    /// Lexicographic comparator: compares by the full sorted load vector
    /// (descending), breaking ties zone by zone (Peak → Shoulder → Transition → Tail).
    /// Introduced in RP-408.
    Lexicographic,
}

impl std::fmt::Display for ComparatorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComparatorMode::Scalar => write!(f, "scalar"),
            ComparatorMode::Lexicographic => write!(f, "lexicographic"),
        }
    }
}

// ---------------------------------------------------------------------------
// Load vector helpers
// ---------------------------------------------------------------------------

/// Compute the sorted arc-saturation load vector (descending) from a flat
/// slice of arc saturations. Returns a `Vec<f64>` of length `arc_sats.len()`.
///
/// This is the same vector used in RP-406C benchmark comparisons.
pub fn sorted_load_vector(arc_sats: &[f64]) -> Vec<f64> {
    let mut v: Vec<f64> = arc_sats.to_vec();
    v.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    v
}

/// Compute the Shoulder Dominance Index (SDI) from a load vector.
///
/// SDI = Σ(i=2..20) wᵢ · Lᵢ   where wᵢ = 1/(i−1), Lᵢ = load_vector[i−1].
///
/// This is the absolute SDI for a single vector. The delta SDI between two
/// vectors is computed by the caller.
pub fn compute_sdi(load_vector: &[f64]) -> f64 {
    let mut sdi = 0.0f64;
    for i in 2..=20usize {
        if i - 1 < load_vector.len() {
            let w = 1.0 / (i - 1) as f64;
            sdi += w * load_vector[i - 1];
        }
    }
    sdi
}

/// Zone delta record: change in cumulative load per zone between two load vectors.
/// Negative values indicate improvement (load decreased).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ZoneDeltas {
    /// Δ load at Rank 1 (Peak zone). Negative = improvement.
    pub delta_rank1: f64,
    /// Δ cumulative load at Ranks 2–20 (Shoulder zone). Negative = improvement.
    pub delta_2_20: f64,
    /// Δ cumulative load at Ranks 21–100 (Transition zone). Negative = improvement.
    pub delta_21_100: f64,
    /// Δ cumulative load at Ranks 101+ (Tail zone). Negative = improvement.
    pub delta_tail: f64,
}

impl ZoneDeltas {
    /// Compute zone deltas between two load vectors (new − old).
    /// Both vectors must be sorted descending (as returned by `sorted_load_vector`).
    pub fn compute(old: &[f64], new: &[f64]) -> Self {
        let zone_sum = |v: &[f64], lo: usize, hi: usize| -> f64 {
            // lo and hi are 1-indexed rank bounds (inclusive).
            let start = lo.saturating_sub(1);
            let end = hi.min(v.len());
            if start >= end {
                0.0
            } else {
                v[start..end].iter().sum()
            }
        };

        let delta_rank1 = zone_sum(new, 1, 1) - zone_sum(old, 1, 1);
        let delta_2_20 = zone_sum(new, 2, 20) - zone_sum(old, 2, 20);
        let delta_21_100 = zone_sum(new, 21, 100) - zone_sum(old, 21, 100);
        let delta_tail = zone_sum(new, 101, usize::MAX) - zone_sum(old, 101, usize::MAX);

        Self {
            delta_rank1,
            delta_2_20,
            delta_21_100,
            delta_tail,
        }
    }

    /// Classify the move based on which zone shows the largest absolute improvement.
    ///
    /// Returns one of: "peak", "shoulder", "transition", "tail", "mixed", "neutral".
    pub fn classify(&self, epsilon: f64) -> &'static str {
        let improvements = [
            ("peak", -self.delta_rank1),
            ("shoulder", -self.delta_2_20),
            ("transition", -self.delta_21_100),
            ("tail", -self.delta_tail),
        ];

        // Count zones that improved by more than epsilon
        let improved: Vec<(&str, f64)> = improvements
            .iter()
            .filter(|(_, imp)| *imp > epsilon)
            .cloned()
            .collect();

        match improved.len() {
            0 => "neutral",
            1 => improved[0].0,
            _ => {
                // Multiple zones improved — find the dominant one
                let max_imp = improved
                    .iter()
                    .map(|(_, v)| *v)
                    .fold(f64::NEG_INFINITY, f64::max);
                let dominant: Vec<&str> = improved
                    .iter()
                    .filter(|(_, v)| *v >= max_imp * 0.5) // within 50% of max
                    .map(|(name, _)| *name)
                    .collect();
                if dominant.len() == 1 {
                    dominant[0]
                } else {
                    "mixed"
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Record types
// ---------------------------------------------------------------------------

/// Emitted once per accepted improvement (global-best change) in the evolution loop.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MoveRecord {
    /// Record type tag for JSONL filtering.
    pub record_type: &'static str,
    /// Unique identifier for this run (UUID v4). Groups all records from one
    /// execution together without relying on filenames or directory structure.
    /// Added in RP-408A.
    pub run_uuid: String,
    /// Which comparator was active for this run. Added in RP-408A.
    pub comparator_mode: ComparatorMode,
    /// Instance identifier (e.g. "setA-06").
    pub instance: String,
    /// Random seed for the run.
    pub seed: u64,
    /// Generation index at which the improvement was accepted.
    pub generation: u32,
    /// Operator type that produced the move (currently "evolution" — will be refined in RP-409).
    pub operator: &'static str,
    /// Zone deltas (new − old load vector).
    pub deltas: ZoneDeltas,
    /// Move classification.
    pub move_class: String,
    /// New objective value (scalar, lower = better).
    pub new_obj: f64,
    /// Previous objective value.
    pub prev_obj: f64,
    /// New MLU.
    pub new_mlu: f64,
    /// New SDI (Shoulder Dominance Index).
    pub new_sdi: f64,
}

/// Emitted once per generation for the population best.
#[derive(Debug, Clone, serde::Serialize)]
pub struct GenerationRecord {
    /// Record type tag for JSONL filtering.
    pub record_type: &'static str,
    /// Unique identifier for this run (UUID v4). Added in RP-408A.
    pub run_uuid: String,
    /// Which comparator was active for this run. Added in RP-408A.
    pub comparator_mode: ComparatorMode,
    /// Instance identifier.
    pub instance: String,
    /// Random seed.
    pub seed: u64,
    /// Generation index.
    pub generation: u32,
    /// Best objective in this generation (scalar).
    pub best_obj: f64,
    /// Best MLU in this generation.
    pub best_mlu: f64,
    /// SDI of the best individual.
    pub best_sdi: f64,
    /// Top-20 prefix of the best individual's load vector.
    pub top20_prefix: Vec<f64>,
    /// Number of valid individuals in the population.
    pub valid_count: usize,
    /// Population size.
    pub population_size: usize,
    /// Number of unique fitness values (diversity proxy).
    pub unique_fitness_count: usize,
    /// Stagnation counter at this generation.
    pub stagnation: usize,
    // --- RP-410 per-generation improvement histogram ---
    /// Accepted global-best improvements this generation classified as Peak.
    pub moves_peak: u32,
    /// Accepted global-best improvements this generation classified as Shoulder.
    pub moves_shoulder: u32,
    /// Accepted global-best improvements this generation classified as Transition.
    pub moves_transition: u32,
    /// Accepted global-best improvements this generation classified as Tail.
    pub moves_tail: u32,
    /// Accepted global-best improvements this generation classified as Mixed.
    pub moves_mixed: u32,
    /// Accepted global-best improvements this generation classified as Neutral.
    pub moves_neutral: u32,
    // --- RP-410 per-generation operator usage counts ---
    /// Number of crossover operations applied this generation.
    pub crossover_count: u32,
    /// Number of mutation-only operations applied this generation.
    pub mutation_count: u32,
    // --- RP-407 construction diagnostics ---
    /// Number of valid individuals in the population at generation 0, before any
    /// selection or variation. Only meaningful when `generation == 0`; set to 0
    /// for all subsequent generations. This is the correct metric for Initial
    /// Feasibility Rate (valid / population_size at gen 0), as distinct from
    /// `valid_count` which reflects the current generation's population state.
    pub generation0_valid_count: usize,
    // --- RP-411 Execution Throughput ---
    /// Wall-clock time spent in fitness evaluation this generation (milliseconds).
    pub eval_time_ms: f64,
    /// Wall-clock time spent in crossover this generation (milliseconds).
    pub crossover_time_ms: f64,
    /// Wall-clock time spent in mutation this generation (milliseconds).
    pub mutation_time_ms: f64,
    /// Wall-clock time spent in repair this generation (milliseconds).
    pub repair_time_ms: f64,
    /// Wall-clock time spent in selection/replacement this generation (milliseconds).
    pub selection_time_ms: f64,
    /// Wall-clock time spent writing telemetry this generation (milliseconds).
    pub telemetry_time_ms: f64,
    /// Unaccounted wall-clock time this generation (milliseconds).
    pub other_time_ms: f64,
    /// Total wall-clock time for this generation (milliseconds).
    pub total_gen_time_ms: f64,
    // --- P10-B repair decomposition counters ---
    /// Number of offspring that were infeasible before process_offspring() was called.
    /// These are the individuals that triggered the repair path.
    pub p10b_infeasible_entering_repair: u32,
    /// Number of offspring that were already feasible before process_offspring() was called.
    /// These went directly to the improvement path (no repair invoked).
    pub p10b_feasible_entering_repair: u32,
    /// Number of repair attempts (= infeasible_entering_repair, since repair is called once per infeasible individual).
    pub p10b_repair_attempts: u32,
    /// Number of repair attempts that succeeded (process_offspring returned Ok(true) for a previously-infeasible individual).
    pub p10b_repair_successes: u32,
    /// Number of repair attempts that failed (process_offspring returned Ok(false) for a previously-infeasible individual).
    pub p10b_repair_failures: u32,
    /// Total wall-clock time spent in process_offspring() for infeasible individuals only (milliseconds).
    /// This isolates repair cost from improvement cost.
    pub p10b_repair_ms: f64,
    /// Total wall-clock time spent in process_offspring() for feasible individuals only (milliseconds).
    /// This isolates improvement cost from repair cost.
    pub p10b_improve_ms: f64,
    /// Repair work per infeasible individual: p10b_repair_ms / p10b_infeasible_entering_repair.
    /// NaN when p10b_infeasible_entering_repair == 0.
    pub p10b_repair_ms_per_infeasible: f64,
    // --- P10-C0 repair-effectiveness counters ---
    // These are observational only — no repair behavior is changed.
    // Measured by calling evaluate_violations() before and after process_offspring()
    // on infeasible offspring, and comparing waypoint fingerprints.
    /// Number of failed repair attempts where the genome waypoints changed.
    /// A changed genome means repair made structural modifications (SegmentLimit/Connectivity path).
    pub p10c0_genome_changed_count: u32,
    /// Number of failed repair attempts where the genome waypoints were identical before and after.
    /// An unchanged genome means repair made no structural modifications (Capacity-only path).
    pub p10c0_genome_unchanged_count: u32,
    /// Number of failed repair attempts where violation count decreased after repair.
    pub p10c0_violation_count_improved: u32,
    /// Number of failed repair attempts where violation count was unchanged after repair.
    pub p10c0_violation_count_unchanged: u32,
    /// Number of failed repair attempts where violation count increased after repair.
    pub p10c0_violation_count_worsened: u32,
    /// Sum of max capacity saturation (flow/capacity) across all failed repairs, before repair.
    /// Divide by p10b_repair_failures to get mean. NaN when p10b_repair_failures == 0.
    pub p10c0_sum_max_sat_before: f64,
    /// Sum of max capacity saturation (flow/capacity) across all failed repairs, after repair.
    /// Divide by p10b_repair_failures to get mean. NaN when p10b_repair_failures == 0.
    pub p10c0_sum_max_sat_after: f64,
}

// ---------------------------------------------------------------------------
// RP-410B Candidate pipeline record
// ---------------------------------------------------------------------------

/// Emitted once per generated child, before selection.
///
/// Each record is a **DecisionEvent** — a single row in the causal graph of the
/// evolutionary run. Because every candidate carries its parent IDs and tournament
/// ID, the full genealogy of any accepted improvement can be reconstructed post-hoc.
///
/// # Schema (RP-410C)
///
/// Fields are grouped by pipeline stage:
///   Identity:   record_type, instance, seed, generation, candidate_id
///   Lineage:    parent1, parent2, operator, tournament_id
///   Objective:  deltas, move_class, obj
///   Feasibility: valid
///   Promotion:  won_tournament, population_slot, elite_slot, became_global_best
///   Diagnosis:  decision_stage, reason
///
/// # Volume warning
///
/// This record is emitted `population_size` times per generation. For a
/// population of 100 and 10,000 generations, that is 1,000,000 records.
/// Enable only when RP-410C analysis is required; disable for production runs.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CandidateRecord {
    /// Record type tag for JSONL filtering.
    pub record_type: &'static str,
    /// Unique identifier for this run (UUID v4). Added in RP-408A.
    pub run_uuid: String,
    /// Which comparator was active for this run. Added in RP-408A.
    pub comparator_mode: ComparatorMode,
    /// Instance identifier.
    pub instance: String,
    /// Random seed.
    pub seed: u64,
    /// Generation index at which this candidate was produced.
    pub generation: u32,
    /// Unique candidate identifier within this run (sequential counter).
    /// Enables downstream genealogy reconstruction.
    pub candidate_id: u64,
    /// `candidate_id` of the first parent (crossover) or self (mutation / elite).
    /// `0` when no parent is tracked (e.g. initial population).
    pub parent1: u64,
    /// `candidate_id` of the second parent (crossover), or `0` if not applicable.
    pub parent2: u64,
    /// Operator that produced this candidate.
    pub operator: &'static str,
    /// Tournament slot index this candidate participated in.
    /// `0` when tournament tracking is not yet wired (RP-410C Phase 1 stub).
    pub tournament_id: u32,
    /// Zone deltas relative to the current global best (new − best).
    /// Negative = improvement over global best.
    pub deltas: ZoneDeltas,
    /// Move classification relative to global best.
    pub move_class: String,
    /// Candidate's objective value (scalar, lower = better; inf if invalid).
    pub obj: f64,
    /// Whether the candidate is feasible (passes all hard constraints).
    pub valid: bool,
    /// Whether the candidate won its tournament comparison.
    /// `false` when tournament tracking is not yet wired (RP-410C Phase 1 stub).
    pub won_tournament: bool,
    /// Population slot entered by this candidate, or `None` if it did not enter
    /// the population. `None` when population-slot tracking is not yet wired.
    pub population_slot: Option<usize>,
    /// Elite slot displaced by this candidate, or `None` if no elite displacement.
    /// `None` when elite-slot tracking is not yet wired.
    pub elite_slot: Option<usize>,
    /// Whether this candidate became the new global-best solution.
    pub became_global_best: bool,
    /// Stage at which the candidate's fate was decided.
    ///
    /// Values: `"Evaluation"` | `"Tournament"` | `"Population"` | `"Elite"` |
    ///         `"GlobalBest"` | `"Accepted"`
    ///
    /// `"Accepted"` means the candidate survived all stages and became global best.
    /// All other values indicate the stage at which it was eliminated.
    /// `"GlobalBest"` is used when the candidate improved the global best.
    /// Until RP-410C wires full tournament/population tracking, this field is set
    /// to `"GlobalBest"` for accepted candidates and `"Tournament"` for all others
    /// (conservative stub — will be refined when per-stage tracking is complete).
    pub decision_stage: &'static str,
    /// Why the candidate was rejected at its `decision_stage`, or `null` if accepted.
    ///
    /// Example values: `"WorseScalarFitness"`, `"CapacityViolation"`, `"WorseThanElite"`.
    /// `null` when the candidate was accepted (became global best).
    pub reason: Option<&'static str>,
}

// ---------------------------------------------------------------------------
// RP-412 Construction diagnostics record
// ---------------------------------------------------------------------------

/// Emitted once per run, immediately after the initial population is evaluated.
///
/// Captures construction-phase diagnostics for RP-412. Fields that require
/// deeper evaluator instrumentation (per-constraint violation categories,
/// repair attempt counts) are reserved as `0` until the evaluator exposes them.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConstructionRecord {
    /// Record type tag for JSONL filtering.
    pub record_type: &'static str,
    /// Unique identifier for this run (UUID v4). Added in RP-408A.
    pub run_uuid: String,
    /// Which comparator was active for this run. Added in RP-408A.
    pub comparator_mode: ComparatorMode,
    /// Instance identifier.
    pub instance: String,
    /// Random seed.
    pub seed: u64,
    /// Population size at construction time.
    pub population_size: usize,
    /// Number of valid individuals produced by the constructor (Initial Feasibility Rate numerator).
    pub valid_count: usize,
    /// Number of invalid individuals produced by the constructor.
    pub invalid_count: usize,
    /// Initial Feasibility Rate = valid_count / population_size.
    pub initial_feasibility_rate: f64,
    /// Whether the constructor produced at least one valid individual.
    pub any_feasible: bool,
    // --- Reserved for deeper evaluator instrumentation (RP-412 Phase 2) ---
    /// Number of capacity constraint violations across all invalid individuals.
    /// Currently 0 — requires per-individual violation breakdown from evaluator.
    pub capacity_violation_count: u32,
    /// Number of segment-budget violations across all invalid individuals.
    /// Currently 0 — requires per-individual violation breakdown from evaluator.
    pub budget_violation_count: u32,
    /// Number of repair attempts made during construction.
    /// Currently 0 — repair is not yet a separate phase in this harness.
    pub repair_attempts: u32,
    /// Number of successful repair attempts.
    /// Currently 0 — repair is not yet a separate phase in this harness.
    pub repair_successes: u32,
}

// ---------------------------------------------------------------------------
// TelemetrySink trait
// ---------------------------------------------------------------------------

/// Sink for RP-410 telemetry records.
///
/// Implementations must be zero-overhead when disabled. The `NullTelemetrySink`
/// provides this guarantee; the compiler eliminates all calls to it.
pub trait TelemetrySink {
    fn emit_move(&mut self, record: &MoveRecord);
    fn emit_generation(&mut self, record: &GenerationRecord);
    /// Emit a construction diagnostics record (RP-412). Called once per run,
    /// immediately after the initial population is evaluated.
    fn emit_construction(&mut self, record: &ConstructionRecord);
    /// Emit a candidate pipeline record (RP-410B). Called once per generated
    /// child, before selection. High volume — enable only for RP-410B analysis.
    fn emit_candidate(&mut self, record: &CandidateRecord);
    fn flush(&mut self);
}

// ---------------------------------------------------------------------------
// NullTelemetrySink — zero-overhead no-op
// ---------------------------------------------------------------------------

/// A telemetry sink that discards all records. Used when telemetry is disabled.
/// The compiler inlines and eliminates all calls to this sink.
pub struct NullTelemetrySink;

impl TelemetrySink for NullTelemetrySink {
    #[inline(always)]
    fn emit_move(&mut self, _record: &MoveRecord) {}
    #[inline(always)]
    fn emit_generation(&mut self, _record: &GenerationRecord) {}
    #[inline(always)]
    fn emit_construction(&mut self, _record: &ConstructionRecord) {}
    #[inline(always)]
    fn emit_candidate(&mut self, _record: &CandidateRecord) {}
    #[inline(always)]
    fn flush(&mut self) {}
}

// ---------------------------------------------------------------------------
// JsonlTelemetrySink — writes JSONL to a Write target
// ---------------------------------------------------------------------------

/// A telemetry sink that serialises records as newline-delimited JSON.
///
/// Move records and generation records are written to separate sinks so they
/// can be stored in separate files for efficient post-hoc analysis.
///
/// # Example
///
/// ```rust,no_run
/// use std::fs::File;
/// use std::io::BufWriter;
/// use roadef::telemetry::JsonlTelemetrySink;
///
/// let moves_file = BufWriter::new(File::create("rp410_moves_setA-06_seed42.jsonl").unwrap());
/// let gens_file  = BufWriter::new(File::create("rp410_generations_setA-06_seed42.jsonl").unwrap());
/// let mut sink = JsonlTelemetrySink::new(moves_file, gens_file);
/// ```
pub struct JsonlTelemetrySink<W1: Write, W2: Write> {
    moves_sink: W1,
    generations_sink: W2,
    /// Number of records written (for diagnostics).
    pub moves_written: u64,
    pub generations_written: u64,
}

impl<W1: Write, W2: Write> JsonlTelemetrySink<W1, W2> {
    pub fn new(moves_sink: W1, generations_sink: W2) -> Self {
        Self {
            moves_sink,
            generations_sink,
            moves_written: 0,
            generations_written: 0,
        }
    }
}

impl<W1: Write, W2: Write> TelemetrySink for JsonlTelemetrySink<W1, W2> {
    fn emit_move(&mut self, record: &MoveRecord) {
        if let Ok(json) = serde_json::to_string(record) {
            let _ = writeln!(self.moves_sink, "{}", json);
            self.moves_written += 1;
        }
    }

    fn emit_generation(&mut self, record: &GenerationRecord) {
        if let Ok(json) = serde_json::to_string(record) {
            let _ = writeln!(self.generations_sink, "{}", json);
            self.generations_written += 1;
        }
    }

    fn emit_construction(&mut self, record: &ConstructionRecord) {
        // Construction records are written to the generations sink so they appear
        // in the same JSONL file as generation records, distinguished by record_type.
        if let Ok(json) = serde_json::to_string(record) {
            let _ = writeln!(self.generations_sink, "{}", json);
        }
    }

    fn emit_candidate(&mut self, record: &CandidateRecord) {
        // Candidate records are written to the moves sink so they appear in the
        // same JSONL file as move records, distinguished by record_type = "candidate".
        if let Ok(json) = serde_json::to_string(record) {
            let _ = writeln!(self.moves_sink, "{}", json);
        }
    }

    fn flush(&mut self) {
        let _ = self.moves_sink.flush();
        let _ = self.generations_sink.flush();
    }
}

// ---------------------------------------------------------------------------
// FourStreamTelemetrySink — RP-408B: one dedicated file per record type
// ---------------------------------------------------------------------------

/// A telemetry sink that routes each record type to its own dedicated writer.
///
/// - `W1` — candidates (CandidateRecord)
/// - `W2` — generations (GenerationRecord)
/// - `W3` — moves (MoveRecord)
/// - `W4` — construction (ConstructionRecord)
///
/// This is the preferred sink for RP-408B and later campaigns where per-stream
/// files are required for efficient post-hoc analysis.
pub struct FourStreamTelemetrySink<W1: Write, W2: Write, W3: Write, W4: Write> {
    candidates_sink: W1,
    generations_sink: W2,
    moves_sink: W3,
    construction_sink: W4,
    pub candidates_written: u64,
    pub generations_written: u64,
    pub moves_written: u64,
    pub construction_written: u64,
}

impl<W1: Write, W2: Write, W3: Write, W4: Write> FourStreamTelemetrySink<W1, W2, W3, W4> {
    /// Create a new four-stream sink.
    ///
    /// Argument order: `new_full(candidates, generations, moves, construction)`.
    pub fn new_full(
        candidates_sink: W1,
        generations_sink: W2,
        moves_sink: W3,
        construction_sink: W4,
    ) -> Self {
        Self {
            candidates_sink,
            generations_sink,
            moves_sink,
            construction_sink,
            candidates_written: 0,
            generations_written: 0,
            moves_written: 0,
            construction_written: 0,
        }
    }
}

impl<W1: Write, W2: Write, W3: Write, W4: Write> TelemetrySink
    for FourStreamTelemetrySink<W1, W2, W3, W4>
{
    fn emit_move(&mut self, record: &MoveRecord) {
        if let Ok(json) = serde_json::to_string(record) {
            let _ = writeln!(self.moves_sink, "{}", json);
            self.moves_written += 1;
        }
    }

    fn emit_generation(&mut self, record: &GenerationRecord) {
        if let Ok(json) = serde_json::to_string(record) {
            let _ = writeln!(self.generations_sink, "{}", json);
            self.generations_written += 1;
        }
    }

    fn emit_construction(&mut self, record: &ConstructionRecord) {
        if let Ok(json) = serde_json::to_string(record) {
            let _ = writeln!(self.construction_sink, "{}", json);
            self.construction_written += 1;
        }
    }

    fn emit_candidate(&mut self, record: &CandidateRecord) {
        if let Ok(json) = serde_json::to_string(record) {
            let _ = writeln!(self.candidates_sink, "{}", json);
            self.candidates_written += 1;
        }
    }

    fn flush(&mut self) {
        let _ = self.candidates_sink.flush();
        let _ = self.generations_sink.flush();
        let _ = self.moves_sink.flush();
        let _ = self.construction_sink.flush();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sorted_load_vector() {
        let sats = vec![0.3, 0.9, 0.1, 0.7];
        let v = sorted_load_vector(&sats);
        assert_eq!(v, vec![0.9, 0.7, 0.3, 0.1]);
    }

    #[test]
    fn test_compute_sdi_empty() {
        assert_eq!(compute_sdi(&[]), 0.0);
    }

    #[test]
    fn test_compute_sdi_short() {
        // Only rank 1 present — SDI should be 0 (ranks 2–20 are absent)
        let v = vec![0.9];
        assert_eq!(compute_sdi(&v), 0.0);
    }

    #[test]
    fn test_compute_sdi_full() {
        // 20 equal loads of 0.5 — SDI = Σ(i=2..20) (1/(i-1)) * 0.5
        let v = vec![0.5f64; 20];
        let expected: f64 = (2..=20usize).map(|i| 0.5 / (i - 1) as f64).sum();
        let got = compute_sdi(&v);
        assert!(
            (got - expected).abs() < 1e-10,
            "got={} expected={}",
            got,
            expected
        );
    }

    #[test]
    fn test_zone_deltas_pure_shoulder_improvement() {
        // Old: uniform 0.5 across 30 ranks. New: shoulder (ranks 2–20) drops to 0.3.
        let old: Vec<f64> = vec![0.5; 30];
        let mut new = old.clone();
        for i in 1..20 {
            new[i] = 0.3;
        } // ranks 2–20 (0-indexed 1..20)
        let d = ZoneDeltas::compute(&old, &new);
        assert_eq!(d.delta_rank1, 0.0);
        assert!(d.delta_2_20 < 0.0, "shoulder should improve");
        assert_eq!(d.delta_21_100, 0.0);
        assert_eq!(d.delta_tail, 0.0);
        assert_eq!(d.classify(1e-9), "shoulder");
    }

    #[test]
    fn test_zone_deltas_neutral() {
        let v = vec![0.5; 30];
        let d = ZoneDeltas::compute(&v, &v);
        assert_eq!(d.classify(1e-9), "neutral");
    }

    #[test]
    fn test_null_sink_compiles() {
        let mut sink = NullTelemetrySink;
        let record = MoveRecord {
            record_type: "move",
            run_uuid: "00000000-0000-0000-0000-000000000000".to_string(),
            comparator_mode: ComparatorMode::Scalar,
            instance: "setA-01".to_string(),
            seed: 42,
            generation: 0,
            operator: "evolution",
            deltas: ZoneDeltas {
                delta_rank1: 0.0,
                delta_2_20: -0.1,
                delta_21_100: 0.0,
                delta_tail: 0.0,
            },
            move_class: "shoulder".to_string(),
            new_obj: 1.0,
            prev_obj: 1.1,
            new_mlu: 0.8,
            new_sdi: 0.5,
        };
        sink.emit_move(&record);
        sink.flush();
    }

    #[test]
    fn test_jsonl_sink_writes_valid_json() {
        let mut moves_buf = Vec::new();
        let mut gens_buf = Vec::new();
        {
            let mut sink = JsonlTelemetrySink::new(&mut moves_buf, &mut gens_buf);
            let move_rec = MoveRecord {
                record_type: "move",
                run_uuid: "00000000-0000-0000-0000-000000000000".to_string(),
                comparator_mode: ComparatorMode::Scalar,
                instance: "setA-06".to_string(),
                seed: 99,
                generation: 5,
                operator: "evolution",
                deltas: ZoneDeltas {
                    delta_rank1: -0.05,
                    delta_2_20: -0.2,
                    delta_21_100: 0.0,
                    delta_tail: 0.0,
                },
                move_class: "shoulder".to_string(),
                new_obj: 0.9,
                prev_obj: 1.1,
                new_mlu: 0.7,
                new_sdi: 0.4,
            };
            sink.emit_move(&move_rec);
            let gen_rec = GenerationRecord {
                record_type: "generation",
                run_uuid: "00000000-0000-0000-0000-000000000000".to_string(),
                comparator_mode: ComparatorMode::Scalar,
                instance: "setA-06".to_string(),
                seed: 99,
                generation: 5,
                best_obj: 0.9,
                best_mlu: 0.7,
                best_sdi: 0.4,
                top20_prefix: vec![0.7, 0.6, 0.5],
                valid_count: 75,
                population_size: 80,
                unique_fitness_count: 60,
                stagnation: 0,
                moves_peak: 0,
                moves_shoulder: 1,
                moves_transition: 0,
                moves_tail: 0,
                moves_mixed: 0,
                moves_neutral: 0,
                crossover_count: 10,
                mutation_count: 3,
                generation0_valid_count: 0,
                // RP-411 timing fields
                eval_time_ms: 12.5,
                crossover_time_ms: 3.2,
                mutation_time_ms: 1.1,
                repair_time_ms: 0.0,
                selection_time_ms: 0.5,
                telemetry_time_ms: 0.1,
                other_time_ms: 0.3,
                total_gen_time_ms: 17.7,
                p10b_infeasible_entering_repair: 0,
                p10b_feasible_entering_repair: 0,
                p10b_repair_attempts: 0,
                p10b_repair_successes: 0,
                p10b_repair_failures: 0,
                p10b_repair_ms: 0.0,
                p10b_improve_ms: 0.0,
                p10b_repair_ms_per_infeasible: 0.0,
                p10c0_genome_changed_count: 0,
                p10c0_genome_unchanged_count: 0,
                p10c0_violation_count_improved: 0,
                p10c0_violation_count_unchanged: 0,
                p10c0_violation_count_worsened: 0,
                p10c0_sum_max_sat_before: 0.0,
                p10c0_sum_max_sat_after: 0.0,
            };
            sink.emit_generation(&gen_rec);
            assert_eq!(sink.moves_written, 1);
            assert_eq!(sink.generations_written, 1);
        }
        // Verify the output is valid JSON
        let moves_str = String::from_utf8(moves_buf).unwrap();
        let _: serde_json::Value = serde_json::from_str(moves_str.trim()).unwrap();
        let gens_str = String::from_utf8(gens_buf).unwrap();
        let _: serde_json::Value = serde_json::from_str(gens_str.trim()).unwrap();
    }
}
