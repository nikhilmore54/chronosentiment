//! ChronoSentiment adapter — trading-domain decision and evaluation.
//!
//! ChronoSentiment owns assessments, `TradingDecision`, `DecisionPolicy`,
//! replay, forward observation, ledger, outcomes, and performance.
//! Coralys owns policy discovery / MOGA / ecology search. This crate must not
//! contain a competing policy optimizer on the default product path.
//!
//! Default features compile the product adapter only.
//! `--features legacy-lake` restores B3/B4 Knowledge Lake generators.
//! `--features research` restores the G-GATE laboratory (implies `legacy-lake`).

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
pub mod product;

#[cfg(feature = "research")]
#[path = "../research/src/mod.rs"]
pub mod research;
