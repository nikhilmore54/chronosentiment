/// decision_memory.rs — UltraRoster P2: Decision Memory
///
/// Governance: P2 is a new capability. No changes to optimization.rs,
/// decision_support.rs, or any existing module. Coralys core frozen.
/// P1 finding frozen: alternatives-generation hypothesis failed.
/// P1.1 (meaningful alternatives) deferred until after P2.
///
/// P2 objective:
///   Build a memory model that captures the full decision lifecycle:
///
///   Situation → Candidate alternatives → Recommendation → Planner choice
///   → Planner modifications → Approved roster → Observed outcome
///
/// This is not merely "previous roster = X". It is a structured record
/// of what was known, what was recommended, what was chosen, how it was
/// modified, and what actually happened — so that future decisions can
/// be informed by past experience.
///
/// Design principles:
///   1. Append-only: memory records are never modified after creation.
///   2. Serializable: all records can be persisted to JSON.
///   3. Queryable: records can be retrieved by situation fingerprint.
///   4. No optimizer dependency: memory is captured at the presentation
///      layer, not inside the optimization engine.
///
/// Lifecycle stages:
///   Stage 1 — PRESENTED: system generated alternatives and recommendation.
///   Stage 2 — DECIDED: planner selected an alternative (or rejected all).
///   Stage 3 — MODIFIED: planner modified the selected roster.
///   Stage 4 — APPROVED: final roster approved for execution.
///   Stage 5 — OBSERVED: outcome recorded after execution.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Situation fingerprint — describes the planning context
// ---------------------------------------------------------------------------

/// A compact description of the planning situation at decision time.
/// Used to retrieve similar past decisions when making new ones.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SituationFingerprint {
    /// Number of workers available
    pub worker_count: usize,
    /// Number of shifts to cover
    pub shift_count: usize,
    /// Planning horizon in hours
    pub horizon_hours: f64,
    /// Weekend ratio (0.0–1.0) of shifts
    pub weekend_ratio: f64,
    /// Scenario identifier (e.g. "family_c_0.6")
    pub scenario_id: String,
    /// Optional: any locked assignments at decision time
    pub locked_assignment_count: usize,
}

impl SituationFingerprint {
    /// Compute a simple similarity score (0.0–1.0) against another fingerprint.
    /// 1.0 = identical situation, 0.0 = completely different.
    pub fn similarity(&self, other: &SituationFingerprint) -> f64 {
        if self.scenario_id != other.scenario_id { return 0.0; }
        let worker_sim = 1.0 - ((self.worker_count as f64 - other.worker_count as f64).abs()
            / self.worker_count.max(1) as f64).min(1.0);
        let shift_sim = 1.0 - ((self.shift_count as f64 - other.shift_count as f64).abs()
            / self.shift_count.max(1) as f64).min(1.0);
        let weekend_sim = 1.0 - (self.weekend_ratio - other.weekend_ratio).abs();
        (worker_sim + shift_sim + weekend_sim) / 3.0
    }
}

// ---------------------------------------------------------------------------
// Alternative snapshot — what was presented to the planner
// ---------------------------------------------------------------------------

/// A snapshot of one alternative as presented to the planner.
/// Captures the metrics visible at decision time, not the full assignment map
/// (which can be large). The assignment map is stored in the approved roster.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeSnapshot {
    pub id: String,
    pub coverage: f64,
    pub fairness_penalty: f64,
    pub utilization: f64,
    pub cost: f64,
    /// Number of assignments that differ from the recommended alternative
    pub diff_from_recommended: usize,
}

// ---------------------------------------------------------------------------
// Planner decision — what the planner chose and why
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlannerChoice {
    /// Planner accepted one of the presented alternatives
    AcceptedAlternative { alternative_id: String },
    /// Planner rejected all alternatives and provided their own
    RejectedAll { reason: Option<String> },
    /// Planner accepted the recommendation without review
    AcceptedRecommendation,
}

// ---------------------------------------------------------------------------
// Roster modification — what the planner changed
// ---------------------------------------------------------------------------

/// A single assignment change made by the planner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentChange {
    pub shift_id: u64,
    pub original_worker_id: Option<u64>,
    pub new_worker_id: Option<u64>,
    pub reason: Option<String>,
}

// ---------------------------------------------------------------------------
// Observed outcome — what actually happened after execution
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutcomeQuality {
    /// All shifts covered, no violations reported
    Successful,
    /// Minor issues (late arrivals, minor constraint violations)
    MinorIssues { description: String },
    /// Major issues (uncovered shifts, significant violations)
    MajorIssues { description: String },
    /// Not yet observed (outcome pending)
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservedOutcome {
    pub quality: OutcomeQuality,
    /// Actual coverage achieved (may differ from planned)
    pub actual_coverage: Option<f64>,
    /// Any constraint violations observed during execution
    pub violations_observed: u32,
    /// Free-text notes from the planner or system
    pub notes: Option<String>,
    pub observed_at_unix_ms: u64,
}

// ---------------------------------------------------------------------------
// Decision record — the full lifecycle
// ---------------------------------------------------------------------------

/// The complete record of one planning decision, from presentation to outcome.
/// Records are append-only: each stage adds fields without modifying earlier ones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    /// Unique identifier for this decision
    pub decision_id: String,
    /// When this decision was initiated (Unix ms)
    pub created_at_unix_ms: u64,

    // Stage 1: PRESENTED
    pub situation: SituationFingerprint,
    pub alternatives_presented: Vec<AlternativeSnapshot>,
    pub system_recommendation: Option<String>,
    pub recommendation_reasons: Vec<String>,
    pub presentation_stage_complete: bool,

    // Stage 2: DECIDED
    pub planner_choice: Option<PlannerChoice>,
    pub decided_at_unix_ms: Option<u64>,

    // Stage 3: MODIFIED
    pub planner_modifications: Vec<AssignmentChange>,
    pub modification_count: usize,

    // Stage 4: APPROVED
    pub approved_roster: Option<HashMap<u64, u64>>, // ShiftID -> WorkerID
    pub approved_at_unix_ms: Option<u64>,

    // Stage 5: OBSERVED
    pub outcome: Option<ObservedOutcome>,
}

impl DecisionRecord {
    /// Create a new record at Stage 1 (PRESENTED).
    pub fn new_presented(
        decision_id: String,
        situation: SituationFingerprint,
        alternatives: Vec<AlternativeSnapshot>,
        system_recommendation: Option<String>,
        recommendation_reasons: Vec<String>,
    ) -> Self {
        Self {
            decision_id,
            created_at_unix_ms: now_unix_ms(),
            situation,
            alternatives_presented: alternatives,
            system_recommendation,
            recommendation_reasons,
            presentation_stage_complete: true,
            planner_choice: None,
            decided_at_unix_ms: None,
            planner_modifications: Vec::new(),
            modification_count: 0,
            approved_roster: None,
            approved_at_unix_ms: None,
            outcome: None,
        }
    }

    /// Advance to Stage 2 (DECIDED).
    pub fn record_decision(&mut self, choice: PlannerChoice) {
        self.planner_choice = Some(choice);
        self.decided_at_unix_ms = Some(now_unix_ms());
    }

    /// Advance to Stage 3 (MODIFIED). May be called with an empty list if no changes.
    pub fn record_modifications(&mut self, changes: Vec<AssignmentChange>) {
        self.modification_count = changes.len();
        self.planner_modifications = changes;
    }

    /// Advance to Stage 4 (APPROVED).
    pub fn record_approval(&mut self, roster: HashMap<u64, u64>) {
        self.approved_roster = Some(roster);
        self.approved_at_unix_ms = Some(now_unix_ms());
    }

    /// Advance to Stage 5 (OBSERVED).
    pub fn record_outcome(&mut self, outcome: ObservedOutcome) {
        self.outcome = Some(outcome);
    }

    /// Current lifecycle stage name.
    pub fn stage(&self) -> &'static str {
        if self.outcome.is_some() { return "OBSERVED"; }
        if self.approved_roster.is_some() { return "APPROVED"; }
        if self.modification_count > 0 { return "MODIFIED"; }
        if self.planner_choice.is_some() { return "DECIDED"; }
        "PRESENTED"
    }

    /// Whether the planner accepted the system recommendation without modification.
    pub fn accepted_recommendation_unchanged(&self) -> bool {
        matches!(&self.planner_choice, Some(PlannerChoice::AcceptedRecommendation))
            && self.modification_count == 0
    }

    /// Whether the planner overrode the recommendation.
    pub fn overrode_recommendation(&self) -> bool {
        match &self.planner_choice {
            Some(PlannerChoice::AcceptedAlternative { alternative_id }) => {
                self.system_recommendation.as_deref() != Some(alternative_id.as_str())
            }
            Some(PlannerChoice::RejectedAll { .. }) => true,
            _ => false,
        }
    }
}

// ---------------------------------------------------------------------------
// Decision memory store — in-memory collection with persistence support
// ---------------------------------------------------------------------------

/// In-memory store of all decision records.
/// Supports append, retrieval by ID, and similarity-based lookup.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DecisionMemory {
    records: Vec<DecisionRecord>,
}

impl DecisionMemory {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    /// Append a new record. Records are append-only.
    pub fn append(&mut self, record: DecisionRecord) {
        self.records.push(record);
    }

    /// Retrieve a record by decision_id.
    pub fn get(&self, decision_id: &str) -> Option<&DecisionRecord> {
        self.records.iter().find(|r| r.decision_id == decision_id)
    }

    /// Retrieve a mutable record by decision_id (for stage advancement).
    pub fn get_mut(&mut self, decision_id: &str) -> Option<&mut DecisionRecord> {
        self.records.iter_mut().find(|r| r.decision_id == decision_id)
    }

    /// Find the N most similar past decisions to a given situation.
    /// Returns records sorted by similarity descending, with their similarity score.
    pub fn find_similar(
        &self,
        situation: &SituationFingerprint,
        n: usize,
        min_similarity: f64,
    ) -> Vec<(&DecisionRecord, f64)> {
        let mut scored: Vec<(&DecisionRecord, f64)> = self.records.iter()
            .map(|r| (r, r.situation.similarity(situation)))
            .filter(|(_, sim)| *sim >= min_similarity)
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(n);
        scored
    }

    /// All records with a completed outcome.
    pub fn completed_decisions(&self) -> Vec<&DecisionRecord> {
        self.records.iter().filter(|r| r.outcome.is_some()).collect()
    }

    /// Summary statistics across all completed decisions.
    pub fn summary(&self) -> MemorySummary {
        let completed = self.completed_decisions();
        let total = self.records.len();
        let n_completed = completed.len();
        let n_accepted_rec = completed.iter()
            .filter(|r| r.accepted_recommendation_unchanged()).count();
        let n_overrode = completed.iter()
            .filter(|r| r.overrode_recommendation()).count();
        let n_successful = completed.iter()
            .filter(|r| matches!(r.outcome.as_ref().map(|o| &o.quality),
                Some(OutcomeQuality::Successful))).count();
        let mean_modifications = if n_completed > 0 {
            completed.iter().map(|r| r.modification_count).sum::<usize>() as f64
                / n_completed as f64
        } else { 0.0 };

        MemorySummary {
            total_decisions: total,
            completed_decisions: n_completed,
            accepted_recommendation_unchanged: n_accepted_rec,
            overrode_recommendation: n_overrode,
            successful_outcomes: n_successful,
            mean_planner_modifications: mean_modifications,
        }
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Total number of records.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Summary statistics
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySummary {
    pub total_decisions: usize,
    pub completed_decisions: usize,
    pub accepted_recommendation_unchanged: usize,
    pub overrode_recommendation: usize,
    pub successful_outcomes: usize,
    pub mean_planner_modifications: f64,
}

// ---------------------------------------------------------------------------
// Utility
// ---------------------------------------------------------------------------

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Generate a simple decision ID from a counter and timestamp.
pub fn make_decision_id(counter: usize) -> String {
    format!("DEC-{:04}-{}", counter, now_unix_ms())
}