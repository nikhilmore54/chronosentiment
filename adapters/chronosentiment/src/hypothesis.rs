// ChronoSentiment — Investment Thesis (versioned Hypothesis)
//
// The Investment Thesis is the core Hypothesis primitive for ChronoSentiment.
// Theses are versioned — each revision creates a new version (v1, v2, v3...).
// All versions are preserved; none are deleted or modified.
//
// Platform primitive: Hypothesis
// ChronoSentiment realisation: InvestmentThesis

use serde::{Deserialize, Serialize};

// ── Thesis Status ─────────────────────────────────────────────────────────────

/// Lifecycle status of an Investment Thesis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThesisStatus {
    /// Thesis is being drafted; not yet active.
    Draft,
    /// Thesis is active — the investor's current belief.
    Active,
    /// Thesis is under review (quarterly review or committee review).
    UnderReview,
    /// Thesis has been revised — superseded by a newer version.
    Revised,
    /// Thesis has been invalidated by evidence.
    Invalidated,
    /// Thesis has been closed (position exited; outcome recorded).
    Closed,
}

// ── Investment Thesis ─────────────────────────────────────────────────────────

/// A versioned investment thesis — the investor's structured belief about
/// an investment opportunity.
///
/// Platform invariant: thesis versions are immutable once created. A revision
/// creates a new version; the previous version is preserved unchanged.
///
/// Versioning:
///   - `version` is set by the Workspace when the thesis is added (1, 2, 3...).
///   - `version_notes` explains what changed in this version and why.
///   - `evidence_ids` links to the evidence items that support this version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentThesis {
    pub thesis_id: String,
    pub workspace_id: String,
    /// Version number — set by the Workspace (1-indexed).
    pub version: u32,
    /// The core investment belief.
    pub thesis_statement: String,
    /// Explicit assumptions the thesis depends on.
    pub assumptions: Vec<String>,
    /// Risks that could invalidate the thesis.
    pub risks: Vec<String>,
    /// IDs of evidence items that support this thesis version.
    pub evidence_ids: Vec<String>,
    /// What changed in this version and why (required for v2+).
    pub version_notes: Option<String>,
    pub status: ThesisStatus,
    pub created_at: u64,
}

impl InvestmentThesis {
    /// Create a new thesis (version will be set by the Workspace).
    pub fn new(
        thesis_id: impl Into<String>,
        workspace_id: impl Into<String>,
        thesis_statement: impl Into<String>,
        assumptions: Vec<String>,
        risks: Vec<String>,
        version_notes: Option<impl Into<String>>,
        created_at: u64,
    ) -> Self {
        Self {
            thesis_id: thesis_id.into(),
            workspace_id: workspace_id.into(),
            version: 0, // set by Workspace.add_thesis_version()
            thesis_statement: thesis_statement.into(),
            assumptions,
            risks,
            evidence_ids: Vec::new(),
            version_notes: version_notes.map(|n| n.into()),
            status: ThesisStatus::Draft,
            created_at,
        }
    }

    /// Builder: link evidence items to this thesis version.
    pub fn with_evidence(mut self, evidence_ids: Vec<String>) -> Self {
        self.evidence_ids = evidence_ids;
        self
    }

    /// Activate this thesis (mark as the investor's current belief).
    pub fn activate(&mut self) {
        self.status = ThesisStatus::Active;
    }

    /// Mark this thesis as revised (superseded by a newer version).
    pub fn mark_revised(&mut self) {
        self.status = ThesisStatus::Revised;
    }

    /// Mark this thesis as invalidated by evidence.
    pub fn invalidate(&mut self) {
        self.status = ThesisStatus::Invalidated;
    }

    /// Whether this thesis version is the active (current) belief.
    pub fn is_active(&self) -> bool {
        self.status == ThesisStatus::Active
    }

    /// Generate a human-readable summary of this thesis version.
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!(
            "Thesis v{}: {}",
            self.version, self.thesis_statement
        ));
        if !self.assumptions.is_empty() {
            lines.push(format!(
                "Assumptions ({}): {}",
                self.assumptions.len(),
                self.assumptions.join("; ")
            ));
        }
        if !self.risks.is_empty() {
            lines.push(format!(
                "Risks ({}): {}",
                self.risks.len(),
                self.risks.join("; ")
            ));
        }
        if let Some(notes) = &self.version_notes {
            lines.push(format!("Version notes: {}", notes));
        }
        lines.join("\n")
    }
}

// ── Thesis Review ─────────────────────────────────────────────────────────────

/// A structured review of an Investment Thesis against new evidence.
///
/// Reviews are created during quarterly research reviews (Personal) or
/// committee reviews (Enterprise). Each review produces a verdict and
/// may trigger a thesis revision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThesisReview {
    pub review_id: String,
    pub workspace_id: String,
    pub thesis_version_reviewed: u32,
    /// New evidence items reviewed in this session.
    pub new_evidence_ids: Vec<String>,
    /// Assessment of each assumption: (assumption text, still_valid, notes).
    pub assumption_assessments: Vec<AssumptionAssessment>,
    pub verdict: ReviewVerdict,
    pub reviewer_notes: String,
    pub reviewed_at: u64,
}

/// Assessment of a single thesis assumption against new evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssumptionAssessment {
    pub assumption: String,
    pub still_valid: bool,
    pub notes: String,
}

/// The verdict of a thesis review.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewVerdict {
    /// Thesis is confirmed — no revision needed.
    Confirmed,
    /// Thesis needs revision — one or more assumptions have changed.
    RevisionRequired,
    /// Thesis is invalidated — the core belief is no longer supportable.
    Invalidated,
}

impl ThesisReview {
    pub fn new(
        review_id: impl Into<String>,
        workspace_id: impl Into<String>,
        thesis_version_reviewed: u32,
        verdict: ReviewVerdict,
        reviewer_notes: impl Into<String>,
        reviewed_at: u64,
    ) -> Self {
        Self {
            review_id: review_id.into(),
            workspace_id: workspace_id.into(),
            thesis_version_reviewed,
            new_evidence_ids: Vec::new(),
            assumption_assessments: Vec::new(),
            verdict,
            reviewer_notes: reviewer_notes.into(),
            reviewed_at,
        }
    }

    pub fn with_evidence(mut self, evidence_ids: Vec<String>) -> Self {
        self.new_evidence_ids = evidence_ids;
        self
    }

    pub fn with_assessments(mut self, assessments: Vec<AssumptionAssessment>) -> Self {
        self.assumption_assessments = assessments;
        self
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_thesis(id: &str) -> InvestmentThesis {
        InvestmentThesis::new(
            id,
            "ws-001",
            "Reliance Industries is undervalued relative to long-term earnings power",
            vec!["Revenue growth continues at 12% CAGR".to_string()],
            vec!["Regulatory risk in telecom".to_string()],
            None::<String>,
            1000,
        )
    }

    #[test]
    fn new_thesis_is_draft() {
        let thesis = make_thesis("th-001");
        assert_eq!(thesis.status, ThesisStatus::Draft);
        assert_eq!(thesis.version, 0); // set by Workspace
    }

    #[test]
    fn thesis_can_be_activated() {
        let mut thesis = make_thesis("th-001");
        thesis.activate();
        assert!(thesis.is_active());
    }

    #[test]
    fn thesis_summary_includes_all_fields() {
        let mut thesis = make_thesis("th-001");
        thesis.version = 1;
        let summary = thesis.summary();
        assert!(summary.contains("Thesis v1"));
        assert!(summary.contains("Revenue growth"));
        assert!(summary.contains("Regulatory risk"));
    }

    #[test]
    fn review_verdict_confirmed() {
        let review = ThesisReview::new(
            "rev-001",
            "ws-001",
            1,
            ReviewVerdict::Confirmed,
            "Q2 results confirmed revenue growth thesis.",
            2000,
        );
        assert_eq!(review.verdict, ReviewVerdict::Confirmed);
    }
}
