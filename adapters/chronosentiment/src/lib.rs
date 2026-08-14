// ChronoSentiment Adapter — Coralys Platform
//
// Shared foundation for ChronoSentiment Enterprise and Personal products.
//
// Module structure:
//   evidence   — EvidenceItem (immutable research sources)
//   hypothesis — InvestmentThesis (versioned hypothesis)
//   timeline   — TimelineEvent, TimelineView (research/decision timeline)
//   workspace  — InvestmentWorkspace (transaction boundary)
//   learning   — PersonalInvestmentLearningLoop (learning loop + patterns)
//
// Platform primitives realised:
//   Workspace  → InvestmentWorkspace
//   Evidence   → EvidenceItem (immutable, append-only)
//   Hypothesis → InvestmentThesis (versioned)
//   Review     → ThesisReview
//   Timeline   → TimelineEvent, TimelineView
//   Outcome    → InvestmentOutcome
//   Learning   → PersonalInvestmentLearningLoop
//   Pattern    → InvestmentPattern

pub mod evidence;
pub mod hypothesis;
pub mod timeline;
pub mod workspace;
pub mod learning;
pub mod observation;
pub mod instrument;
pub mod ingestion;
pub mod validation;
pub mod portfolio;
pub mod policy;
pub mod repository;
pub mod metrics;
pub mod reasoning;

// Re-export the most commonly used types for convenience.
pub use evidence::{EvidenceItem, EvidenceSourceType, EvidenceDossier};
pub use hypothesis::{InvestmentThesis, ThesisStatus, ThesisReview, ReviewVerdict};
pub use timeline::{TimelineEvent, TimelineEventKind, TimelineView};
pub use workspace::{InvestmentWorkspace, WorkspaceStatus, InvestmentOutcome, OutcomeResult};
pub use learning::{
    PersonalInvestmentLearningLoop,
    InvestmentPattern,
    InvestmentPatternType,
    PatternMaturity,
    InvestmentInsight,
    QuarterlyReviewReport,
};
pub mod universe;
pub mod decision_support;
pub mod research;
