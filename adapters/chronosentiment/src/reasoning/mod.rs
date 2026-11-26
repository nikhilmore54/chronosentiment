//! Reasoning artifacts.
//!
//! Product assessments live in `assessment`.
//! Knowledge Lake `Decision` / `OpportunityStrategy` types remain here so B3/B4
//! dumps can be read. The *generators* (`DecisionEngine`, `StrategyEngine`) are
//! compiled only with `--features legacy-lake`.
//!
//! `intraday_classification` contains the frozen Intelligence Contract v1
//! implementation: entry-time classifier, H120 reassessment, and action mapping.
//! See `docs/INTELLIGENCE_CONTRACT_V1.md`.

pub mod assessment;
pub mod decision;
pub mod evidence;
pub mod historical_reasoning;
pub mod hypothesis;
pub mod intraday_classification;
pub mod policy_engine;
pub mod scenario;
pub mod strategy;
