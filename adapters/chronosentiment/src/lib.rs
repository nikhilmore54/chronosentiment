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
pub mod ingestion;
pub mod instrument;
pub mod learning;
pub mod metrics;
pub mod observation;
pub mod policy;
pub mod portfolio;
pub mod reasoning;
pub mod repository;
pub mod timeline;
pub mod validation;
pub mod workspace;

pub use evidence::{EvidenceDossier, EvidenceItem, EvidenceSourceType};
pub use hypothesis::{InvestmentThesis, ReviewVerdict, ThesisReview, ThesisStatus};
pub use learning::{
    InvestmentInsight, InvestmentPattern, InvestmentPatternType, PatternMaturity,
    PersonalInvestmentLearningLoop, QuarterlyReviewReport,
};
pub use timeline::{TimelineEvent, TimelineEventKind, TimelineView};
pub use workspace::{InvestmentOutcome, InvestmentWorkspace, OutcomeResult, WorkspaceStatus};
pub mod decision_support;
pub mod product;
pub mod time_machine;
pub mod universe;

#[cfg(feature = "research")]
#[path = "../research/src/mod.rs"]
pub mod research;
