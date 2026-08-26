//! Reasoning artifacts.
//!
//! Product assessments live in `assessment`.
//! Knowledge Lake `Decision` / `OpportunityStrategy` types remain here so B3/B4
//! dumps can be read. The *generators* (`DecisionEngine`, `StrategyEngine`) are
//! compiled only with `--features legacy-lake`.

pub mod assessment;
pub mod decision;
pub mod evidence;
pub mod historical_reasoning;
pub mod hypothesis;
pub mod policy_engine;
pub mod scenario;
pub mod strategy;
