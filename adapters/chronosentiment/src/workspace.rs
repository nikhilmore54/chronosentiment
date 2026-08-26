// ChronoSentiment — Research/Decision Workspace
//
// The Workspace is the transaction boundary for a single investment research
// or decision cycle. Everything in the lifecycle happens inside exactly one
// Workspace. It is the unit of provenance, archival, and access control.
//
// This module provides the shared Workspace foundation used by both
// ChronoSentiment Personal (ResearchWorkspace) and Enterprise (DecisionWorkspace).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::evidence::EvidenceItem;
use crate::hypothesis::InvestmentThesis;
use crate::timeline::{TimelineEvent, TimelineEventKind};

// ── Workspace Status ──────────────────────────────────────────────────────────

/// Lifecycle state of a Workspace.
///
/// Transitions:
///   Active → UnderReview → Decided → Monitoring → Closed
///   Active → Closed (if research is abandoned)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceStatus {
    /// Research is in progress; evidence and thesis are being developed.
    Active,
    /// Thesis is under review (committee review or personal review).
    UnderReview,
    /// A decision has been made; position may be open.
    Decided,
    /// Position is open; thesis is being monitored against new evidence.
    Monitoring,
    /// Research cycle is complete; workspace is archived.
    Closed,
}

// ── Workspace ─────────────────────────────────────────────────────────────────

/// A single investment research or decision workspace.
///
/// The Workspace accumulates evidence, manages thesis versions, and records
/// the complete timeline of how the investment thinking evolved.
///
/// Platform invariants enforced here:
///   - Evidence is immutable once added (append-only).
///   - Every Workspace has exactly one active Intent (research objective).
///   - Every Outcome belongs to exactly one Workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentWorkspace {
    pub workspace_id: String,
    /// The company or asset being researched (Subject).
    pub subject: String,
    /// The portfolio or mandate context (Context).
    pub portfolio: String,
    /// The research objective — what the investor is trying to determine (Intent).
    pub research_objective: String,
    /// Immutable evidence items — append-only.
    evidence: Vec<EvidenceItem>,
    /// All thesis versions created in this workspace.
    thesis_versions: Vec<InvestmentThesis>,
    /// Chronological timeline of all events.
    timeline: Vec<TimelineEvent>,
    /// Current workspace status.
    pub status: WorkspaceStatus,
    /// Recorded investment outcome (if any).
    pub outcome: Option<InvestmentOutcome>,
    /// Arbitrary metadata (tags, notes, etc.).
    pub metadata: HashMap<String, String>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl InvestmentWorkspace {
    /// Create a new workspace with a research objective.
    pub fn new(
        workspace_id: impl Into<String>,
        subject: impl Into<String>,
        portfolio: impl Into<String>,
        research_objective: impl Into<String>,
        created_at: u64,
    ) -> Self {
        let workspace_id = workspace_id.into();
        let subject = subject.into();
        let portfolio = portfolio.into();
        let research_objective = research_objective.into();

        let open_event = TimelineEvent {
            event_id: format!("{}-open", workspace_id),
            workspace_id: workspace_id.clone(),
            kind: TimelineEventKind::WorkspaceOpened,
            description: format!("Research workspace opened: {}", subject),
            timestamp: created_at,
        };

        Self {
            workspace_id,
            subject,
            portfolio,
            research_objective,
            evidence: Vec::new(),
            thesis_versions: Vec::new(),
            timeline: vec![open_event],
            status: WorkspaceStatus::Active,
            outcome: None,
            metadata: HashMap::new(),
            created_at,
            updated_at: created_at,
        }
    }

    // ── Evidence (immutable, append-only) ────────────────────────────────────

    /// Add an evidence item. Evidence is immutable once added.
    ///
    /// Platform invariant: evidence cannot be removed or modified after addition.
    pub fn add_evidence(&mut self, item: EvidenceItem, now: u64) {
        let event = TimelineEvent {
            event_id: format!("{}-ev-{}", self.workspace_id, self.evidence.len()),
            workspace_id: self.workspace_id.clone(),
            kind: TimelineEventKind::EvidenceAdded,
            description: format!("[{}] {}", format!("{:?}", item.source_type), item.title),
            timestamp: now,
        };
        self.evidence.push(item);
        self.timeline.push(event);
        self.updated_at = now;
    }

    /// Return all evidence items (immutable slice).
    pub fn evidence(&self) -> &[EvidenceItem] {
        &self.evidence
    }

    // ── Thesis (versioned) ────────────────────────────────────────────────────

    /// Add a new thesis version.
    ///
    /// Each call creates a new version (v1, v2, v3...). The previous version
    /// is not modified — all versions are preserved.
    pub fn add_thesis_version(&mut self, mut thesis: InvestmentThesis, now: u64) {
        let version = self.thesis_versions.len() + 1;
        thesis.version = version as u32;
        thesis.workspace_id = self.workspace_id.clone();

        let kind = if version == 1 {
            TimelineEventKind::ThesisCreated
        } else {
            TimelineEventKind::ThesisRevised
        };

        let event = TimelineEvent {
            event_id: format!("{}-th-{}", self.workspace_id, version),
            workspace_id: self.workspace_id.clone(),
            kind,
            description: format!(
                "Thesis v{}: {}",
                version,
                thesis.version_notes.as_deref().unwrap_or("(no notes)")
            ),
            timestamp: now,
        };

        self.thesis_versions.push(thesis);
        self.timeline.push(event);
        self.updated_at = now;
    }

    /// Return the active (most recent) thesis version, if any.
    pub fn active_thesis(&self) -> Option<&InvestmentThesis> {
        self.thesis_versions.last()
    }

    /// Return all thesis versions.
    pub fn thesis_versions(&self) -> &[InvestmentThesis] {
        &self.thesis_versions
    }

    // ── Status transitions ────────────────────────────────────────────────────

    /// Transition the workspace to a new status.
    pub fn transition(&mut self, new_status: WorkspaceStatus, now: u64) {
        let kind = match &new_status {
            WorkspaceStatus::UnderReview => TimelineEventKind::ReviewScheduled,
            WorkspaceStatus::Decided => TimelineEventKind::DecisionMade,
            WorkspaceStatus::Monitoring => TimelineEventKind::PositionOpened,
            WorkspaceStatus::Closed => TimelineEventKind::WorkspaceClosed,
            WorkspaceStatus::Active => TimelineEventKind::WorkspaceOpened,
        };
        let event = TimelineEvent {
            event_id: format!("{}-st-{}", self.workspace_id, now),
            workspace_id: self.workspace_id.clone(),
            kind,
            description: format!("Status → {:?}", new_status),
            timestamp: now,
        };
        self.status = new_status;
        self.timeline.push(event);
        self.updated_at = now;
    }

    // ── Outcome ───────────────────────────────────────────────────────────────

    /// Record the investment outcome. Can only be set once.
    ///
    /// Platform invariant: every Outcome belongs to exactly one Workspace.
    pub fn record_outcome(
        &mut self,
        outcome: InvestmentOutcome,
        now: u64,
    ) -> Result<(), &'static str> {
        if self.outcome.is_some() {
            return Err("Outcome already recorded for this workspace");
        }
        let event = TimelineEvent {
            event_id: format!("{}-out", self.workspace_id),
            workspace_id: self.workspace_id.clone(),
            kind: TimelineEventKind::OutcomeRecorded,
            description: format!(
                "Outcome recorded: {} ({:?})",
                outcome.summary, outcome.result
            ),
            timestamp: now,
        };
        self.outcome = Some(outcome);
        self.timeline.push(event);
        self.updated_at = now;
        Ok(())
    }

    // ── Timeline ──────────────────────────────────────────────────────────────

    /// Return the complete timeline (chronological).
    pub fn timeline(&self) -> &[TimelineEvent] {
        &self.timeline
    }

    /// Return timeline events of a specific kind.
    pub fn timeline_by_kind(&self, kind: &TimelineEventKind) -> Vec<&TimelineEvent> {
        self.timeline.iter().filter(|e| &e.kind == kind).collect()
    }
}

// ── Investment Outcome ────────────────────────────────────────────────────────

/// The recorded outcome of an investment decision.
///
/// Platform invariant: immutable once recorded; belongs to exactly one Workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentOutcome {
    pub outcome_id: String,
    pub workspace_id: String,
    pub result: OutcomeResult,
    pub summary: String,
    pub return_pct: Option<f64>,
    pub holding_period_days: Option<u32>,
    pub thesis_validated: bool,
    pub key_learnings: Vec<String>,
    pub recorded_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutcomeResult {
    Profitable,
    Breakeven,
    Loss,
    PositionStillOpen,
    DecisionNotExecuted,
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::{EvidenceItem, EvidenceSourceType};
    use crate::hypothesis::InvestmentThesis;

    fn make_workspace() -> InvestmentWorkspace {
        InvestmentWorkspace::new(
            "ws-001",
            "Reliance Industries",
            "ISA — Long-term growth",
            "Evaluate Reliance Industries as a long-term holding",
            1000,
        )
    }

    #[test]
    fn new_workspace_is_active() {
        let ws = make_workspace();
        assert_eq!(ws.status, WorkspaceStatus::Active);
        assert_eq!(ws.evidence().len(), 0);
        assert!(ws.active_thesis().is_none());
    }

    #[test]
    fn evidence_is_append_only() {
        let mut ws = make_workspace();
        let item = EvidenceItem::new(
            "ev-001",
            "ws-001",
            "Reliance FY2025 Annual Report",
            EvidenceSourceType::AnnualReport,
            "Strong revenue growth; diversified business model.",
            1001,
        );
        ws.add_evidence(item, 1001);
        assert_eq!(ws.evidence().len(), 1);
        // Timeline should have 2 events: WorkspaceOpened + EvidenceAdded.
        assert_eq!(ws.timeline().len(), 2);
    }

    #[test]
    fn thesis_versioning_increments() {
        let mut ws = make_workspace();
        let t1 = InvestmentThesis::new(
            "th-001",
            "ws-001",
            "Reliance is undervalued",
            vec!["Revenue growth continues".to_string()],
            vec!["Regulatory risk".to_string()],
            None::<String>,
            1002,
        );
        ws.add_thesis_version(t1, 1002);
        assert_eq!(ws.active_thesis().unwrap().version, 1);

        let t2 = InvestmentThesis::new(
            "th-002",
            "ws-001",
            "Reliance is undervalued — revised after Q2 results",
            vec!["Revenue growth confirmed".to_string()],
            vec!["Regulatory risk".to_string(), "Margin pressure".to_string()],
            Some("Q2 results confirmed revenue growth; added margin pressure risk."),
            1003,
        );
        ws.add_thesis_version(t2, 1003);
        assert_eq!(ws.active_thesis().unwrap().version, 2);
        assert_eq!(ws.thesis_versions().len(), 2);
    }

    #[test]
    fn outcome_can_only_be_recorded_once() {
        let mut ws = make_workspace();
        let outcome = InvestmentOutcome {
            outcome_id: "out-001".to_string(),
            workspace_id: "ws-001".to_string(),
            result: OutcomeResult::Profitable,
            summary: "Position closed at +18% over 14 months.".to_string(),
            return_pct: Some(18.0),
            holding_period_days: Some(420),
            thesis_validated: true,
            key_learnings: vec!["Revenue growth thesis was correct.".to_string()],
            recorded_at: 2000,
        };
        assert!(ws.record_outcome(outcome.clone(), 2000).is_ok());
        assert!(ws.record_outcome(outcome, 2001).is_err());
    }

    #[test]
    fn status_transition_recorded_in_timeline() {
        let mut ws = make_workspace();
        ws.transition(WorkspaceStatus::Monitoring, 1500);
        assert_eq!(ws.status, WorkspaceStatus::Monitoring);
        let monitoring_events = ws.timeline_by_kind(&TimelineEventKind::PositionOpened);
        assert_eq!(monitoring_events.len(), 1);
    }
}
