//! Portfolio Historical Replay v0.1 — Same-Period Longitudinal Comparison.
//!
//! Runs two independent simulated portfolios over exactly the P.E.2 historical
//! period (certified T = 2026-07-15T03:45:00Z), using the same C3-002 decisions
//! and market data, but different execution contracts:
//!
//! ## P.E.2 Portfolio
//!   - Target: fixed +5%
//!   - Stop: none
//!   - Max hold: 20 sessions
//!   - Uses: seal_execution_intent() + first_exit()
//!
//! ## Coralys v0 Portfolio
//!   - Target: ATR/TMV (coralys-exec-v0)
//!   - Stop: Coralys risk_boundary (enforced)
//!   - Max hold: 20 sessions
//!   - Uses: seal_coralys_execution_intent() + first_exit_with_optional_stop(stop_authorized=true)
//!
//! ## Capital Model (frozen for v0.1)
//!   - Rs.5,000 initial capital, Day 0 only
//!   - No recurring contributions
//!   - Cash recycled after positions close
//!   - Both portfolios start with identical Rs.5,000 cash
//!   - Allocation: equal split across eligible instruments at T
//!
//! ## Methodological invariants
//!   - P.E.3-B archive is NOT touched or regenerated
//!   - C3-002 direction is NOT modified
//!   - coralys-exec-v0 multipliers are NOT modified (frozen artifact)
//!   - P.E.2 arm uses its original contract exactly (no stop added)
//!   - Divergence between portfolios is the product effect being measured

use std::collections::BTreeMap;

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::ingestion::yahoo::YahooHistoricalBar;

use super::coralys_execution_model::{
    seal_coralys_execution_intent, CoralysExecutionResult, CORALYS_EXEC_ARTIFACT_HASH,
    MAXIMUM_HOLD_SESSIONS,
};
use super::csp006_protocol::{RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_UNIVERSE};
use super::observatory_execution::{
    entry_close, first_exit, first_exit_with_optional_stop, seal_execution_intent,
    ExitReason, SealedExecutionIntent, EXECUTION_TARGET_PCT,
};
use super::observatory_historical::{decision_time_bars, generate_historical_replay_decision};
use super::observatory_live_execution_pe3::atr_14_at_t;
use super::observatory_prospective::latest_session_at_or_before;
use super::policy_artifact::PolicyArtifact;
use super::DecisionAction;

// ─── Experiment constants ─────────────────────────────────────────────────────

pub const PORTFOLIO_REPLAY_REQUESTED_CLOCK: &str = "2026-07-15T03:45:00+00:00";
pub const INITIAL_CAPITAL_INR: f64 = 5000.0;
pub const PORTFOLIO_REPLAY_PATH_KIND: &str = "portfolio_replay_v0_1";
pub const PORTFOLIO_REPLAY_EXPERIMENT_ID: &str =
    "portfolio_comparison_pe2_vs_coralys_v0_2026-08-16";
pub const PE2_ARM_CONTRACT: &str = "targeted_execution_v0_fixed_5pct_20_sessions";
pub const CORALYS_ARM_CONTRACT: &str = "coralys_exec_v0_atr_tmv_stop_enforced_20_sessions";

// ─── Position status ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PositionStatus {
    Open,
    ClosedTarget,
    ClosedStop,
    ClosedHorizon,
    ClosedAmbiguous,
}

impl PositionStatus {
    pub fn is_closed(&self) -> bool {
        !matches!(self, PositionStatus::Open)
    }
}

// ─── Portfolio position ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioPosition {
    pub instrument: String,
    pub decision_id: String,
    pub direction: String,
    pub entry_price: f64,
    pub capital_allocated_inr: f64,
    pub units: f64,
    pub target_pct: f64,
    pub target_price: f64,
    pub stop_pct: Option<f64>,
    pub stop_price: Option<f64>,
    pub entry_time: String,
    pub exit_time: Option<String>,
    pub exit_price: Option<f64>,
    pub exit_reason: Option<ExitReason>,
    pub holding_sessions: Option<u32>,
    pub realized_pnl_inr: Option<f64>,
    pub realized_return_pct: Option<f64>,
    pub status: PositionStatus,
}

impl PortfolioPosition {
    pub fn unrealized_pnl_inr(&self, mark_price: f64) -> f64 {
        if self.entry_price <= 0.0 || mark_price <= 0.0 {
            return 0.0;
        }
        let ret = match self.direction.as_str() {
            "LONG" => (mark_price - self.entry_price) / self.entry_price,
            "SHORT" => (self.entry_price - mark_price) / self.entry_price,
            _ => 0.0,
        };
        self.capital_allocated_inr * ret
    }

    pub fn market_value_inr(&self, mark_price: f64) -> f64 {
        self.capital_allocated_inr + self.unrealized_pnl_inr(mark_price)
    }
}

// ─── Portfolio arm ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioArm {
    pub arm_id: String,
    pub execution_contract: String,
    pub initial_capital_inr: f64,
    pub cash_inr: f64,
    pub positions: Vec<PortfolioPosition>,
    pub total_contributions_inr: f64,
    pub total_realized_pnl_inr: f64,
    pub n_positions_opened: usize,
    pub n_target: usize,
    pub n_stop: usize,
    pub n_horizon: usize,
    pub n_ambiguous: usize,
    pub holding_sessions_list: Vec<u32>,
    pub peak_portfolio_value_inr: f64,
    pub max_drawdown_inr: f64,
    pub max_drawdown_pct: f64,
}

impl PortfolioArm {
    pub fn new(arm_id: &str, execution_contract: &str, initial_capital: f64) -> Self {
        PortfolioArm {
            arm_id: arm_id.to_string(),
            execution_contract: execution_contract.to_string(),
            initial_capital_inr: initial_capital,
            cash_inr: initial_capital,
            positions: Vec::new(),
            total_contributions_inr: initial_capital,
            total_realized_pnl_inr: 0.0,
            n_positions_opened: 0,
            n_target: 0,
            n_stop: 0,
            n_horizon: 0,
            n_ambiguous: 0,
            holding_sessions_list: Vec::new(),
            peak_portfolio_value_inr: initial_capital,
            max_drawdown_inr: 0.0,
            max_drawdown_pct: 0.0,
        }
    }

    pub fn total_value_inr(&self, mark_prices: &BTreeMap<String, f64>) -> f64 {
        let invested: f64 = self
            .positions
            .iter()
            .filter(|p| !p.status.is_closed())
            .map(|p| {
                let mark = mark_prices.get(&p.instrument).copied().unwrap_or(p.entry_price);
                p.market_value_inr(mark)
            })
            .sum();
        self.cash_inr + invested
    }

    pub fn invested_inr(&self) -> f64 {
        self.positions
            .iter()
            .filter(|p| !p.status.is_closed())
            .map(|p| p.capital_allocated_inr)
            .sum()
    }

    pub fn unrealized_pnl_inr(&self, mark_prices: &BTreeMap<String, f64>) -> f64 {
        self.positions
            .iter()
            .filter(|p| !p.status.is_closed())
            .map(|p| {
                let mark = mark_prices.get(&p.instrument).copied().unwrap_or(p.entry_price);
                p.unrealized_pnl_inr(mark)
            })
            .sum()
    }

    pub fn update_drawdown(&mut self, current_value: f64) {
        if current_value > self.peak_portfolio_value_inr {
            self.peak_portfolio_value_inr = current_value;
        }
        let drawdown = self.peak_portfolio_value_inr - current_value;
        if drawdown > self.max_drawdown_inr {
            self.max_drawdown_inr = drawdown;
            if self.peak_portfolio_value_inr > 0.0 {
                self.max_drawdown_pct = drawdown / self.peak_portfolio_value_inr;
            }
        }
    }

    pub fn close_position(
        &mut self,
        instrument: &str,
        exit_price: f64,
        exit_time: &str,
        exit_reason: ExitReason,
        holding_sessions: u32,
    ) {
        for pos in self.positions.iter_mut() {
            if pos.instrument == instrument && !pos.status.is_closed() {
                let ret = match pos.direction.as_str() {
                    "LONG" => (exit_price - pos.entry_price) / pos.entry_price,
                    "SHORT" => (pos.entry_price - exit_price) / pos.entry_price,
                    _ => 0.0,
                };
                let pnl = pos.capital_allocated_inr * ret;
                let returned_cash = pos.capital_allocated_inr + pnl;

                pos.exit_price = Some(exit_price);
                pos.exit_time = Some(exit_time.to_string());
                pos.exit_reason = Some(exit_reason);
                pos.holding_sessions = Some(holding_sessions);
                pos.realized_pnl_inr = Some(pnl);
                pos.realized_return_pct = Some(ret);
                pos.status = match exit_reason {
                    ExitReason::Target => PositionStatus::ClosedTarget,
                    ExitReason::Stop => PositionStatus::ClosedStop,
                    ExitReason::Horizon => PositionStatus::ClosedHorizon,
                    ExitReason::Ambiguous => PositionStatus::ClosedAmbiguous,
                    _ => PositionStatus::ClosedHorizon,
                };

                self.cash_inr += returned_cash;
                self.total_realized_pnl_inr += pnl;
                self.holding_sessions_list.push(holding_sessions);

                match exit_reason {
                    ExitReason::Target => self.n_target += 1,
                    ExitReason::Stop => self.n_stop += 1,
                    ExitReason::Horizon => self.n_horizon += 1,
                    ExitReason::Ambiguous => self.n_ambiguous += 1,
                    _ => {}
                }
                break;
            }
        }
    }

    pub fn avg_holding_sessions(&self) -> Option<f64> {
        if self.holding_sessions_list.is_empty() {
            return None;
        }
        let sum: u32 = self.holding_sessions_list.iter().sum();
        Some(sum as f64 / self.holding_sessions_list.len() as f64)
    }

    pub fn median_holding_sessions(&self) -> Option<f64> {
        if self.holding_sessions_list.is_empty() {
            return None;
        }
        let mut sorted = self.holding_sessions_list.clone();
        sorted.sort_unstable();
        let n = sorted.len();
        if n % 2 == 0 {
            Some((sorted[n / 2 - 1] + sorted[n / 2]) as f64 / 2.0)
        } else {
            Some(sorted[n / 2] as f64)
        }
    }

    pub fn total_return_pct(&self, mark_prices: &BTreeMap<String, f64>) -> f64 {
        if self.initial_capital_inr <= 0.0 {
            return 0.0;
        }
        (self.total_value_inr(mark_prices) - self.initial_capital_inr) / self.initial_capital_inr
    }

    pub fn capital_utilization_pct(&self) -> f64 {
        if self.initial_capital_inr <= 0.0 {
            return 0.0;
        }
        self.invested_inr() / self.initial_capital_inr
    }
}

// ─── Comparison report ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmSummary {
    pub arm_id: String,
    pub execution_contract: String,
    pub initial_capital_inr: f64,
    pub final_cash_inr: f64,
    pub final_invested_inr: f64,
    pub final_portfolio_value_inr: f64,
    pub total_return_pct: f64,
    pub total_realized_pnl_inr: f64,
    pub total_unrealized_pnl_inr: f64,
    pub max_drawdown_inr: f64,
    pub max_drawdown_pct: f64,
    pub n_positions_opened: usize,
    pub n_target: usize,
    pub n_stop: usize,
    pub n_horizon: usize,
    pub n_ambiguous: usize,
    pub avg_holding_sessions: Option<f64>,
    pub median_holding_sessions: Option<f64>,
    pub capital_utilization_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioComparisonReport {
    pub path_kind: String,
    pub experiment_id: String,
    pub requested_clock: String,
    pub certified_t: String,
    pub initial_capital_inr: f64,
    pub coralys_artifact_hash: String,
    pub c3_002_artifact_hash: String,
    pub pe2_arm: ArmSummary,
    pub coralys_arm: ArmSummary,
    pub exploratory_note: String,
    pub methodology_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioReplayLedger {
    pub path_kind: String,
    pub experiment_id: String,
    pub requested_clock: String,
    pub certified_t: String,
    pub initial_capital_inr: f64,
    pub coralys_artifact_hash: String,
    pub c3_002_artifact_hash: String,
    pub pe2_arm: PortfolioArm,
    pub coralys_arm: PortfolioArm,
    pub comparison: PortfolioComparisonReport,
    pub integrity_note: String,
}

// ─── Output guard ─────────────────────────────────────────────────────────────

pub fn refuse_portfolio_replay_output(path: &str) -> Result<(), String> {
    for forbidden in [
        "pe2_control_2026-08-16",
        "pe3_coralys_v0_2026-08-16",
        "observatory/prospective",
        "historical_replay_v0",
        "historical_replay_v1",
        "selected_policy.json",
    ] {
        if path.contains(forbidden) {
            return Err(format!(
                "portfolio replay v0.1 refuses to write to protected path: {forbidden}"
            ));
        }
    }
    Ok(())
}

// ─── Helper ───────────────────────────────────────────────────────────────────

fn build_arm_summary(arm: &PortfolioArm, mark_prices: &BTreeMap<String, f64>) -> ArmSummary {
    ArmSummary {
        arm_id: arm.arm_id.clone(),
        execution_contract: arm.execution_contract.clone(),
        initial_capital_inr: arm.initial_capital_inr,
        final_cash_inr: arm.cash_inr,
        final_invested_inr: arm.invested_inr(),
        final_portfolio_value_inr: arm.total_value_inr(mark_prices),
        total_return_pct: arm.total_return_pct(mark_prices),
        total_realized_pnl_inr: arm.total_realized_pnl_inr,
        total_unrealized_pnl_inr: arm.unrealized_pnl_inr(mark_prices),
        max_drawdown_inr: arm.max_drawdown_inr,
        max_drawdown_pct: arm.max_drawdown_pct,
        n_positions_opened: arm.n_positions_opened,
        n_target: arm.n_target,
        n_stop: arm.n_stop,
        n_horizon: arm.n_horizon,
        n_ambiguous: arm.n_ambiguous,
        avg_holding_sessions: arm.avg_holding_sessions(),
        median_holding_sessions: arm.median_holding_sessions(),
        capital_utilization_pct: arm.capital_utilization_pct(),
    }
}

// ─── Main replay ─────────────────────────────────────────────────────────────

/// Run the Portfolio Historical Replay v0.1.
///
/// Two portfolios, same period, same decisions, different execution contracts:
///   - P.E.2 arm: fixed +5% target, no stop
///   - Coralys v0 arm: ATR/TMV target + enforced risk_boundary stop
pub fn run_portfolio_replay(
    artifact: &PolicyArtifact,
    cache: &BTreeMap<String, Vec<YahooHistoricalBar>>,
) -> Result<PortfolioReplayLedger, String> {
    // Identity gates
    if artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("portfolio replay v0.1 identity-gates C3-002".into());
    }
    if CORALYS_EXEC_ARTIFACT_HASH
        != "3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f"
    {
        return Err(format!(
            "portfolio replay v0.1 coralys artifact hash mismatch: {CORALYS_EXEC_ARTIFACT_HASH}"
        ));
    }

    // Resolve certified T
    let requested = DateTime::parse_from_rfc3339(PORTFOLIO_REPLAY_REQUESTED_CLOCK)
        .map(|t| t.with_timezone(&Utc))
        .map_err(|e| format!("clock parse error: {e}"))?;

    let first_instrument = RESEARCH_UNIVERSE.first().ok_or("RESEARCH_UNIVERSE is empty")?;
    let first_bars = cache
        .get(*first_instrument)
        .ok_or_else(|| format!("cache missing {first_instrument}"))?;
    let certified_t = latest_session_at_or_before(first_bars, requested)
        .ok_or_else(|| format!("no certified session for {first_instrument}"))?;

    for instrument in RESEARCH_UNIVERSE {
        let bars = cache
            .get(instrument)
            .ok_or_else(|| format!("cache missing {instrument}"))?;
        let t = latest_session_at_or_before(bars, requested)
            .ok_or_else(|| format!("no certified session for {instrument}"))?;
        if t != certified_t {
            return Err(format!(
                "{instrument} certified T {t} != cohort T {certified_t}"
            ));
        }
    }

    let mut pe2_arm = PortfolioArm::new("pe2", PE2_ARM_CONTRACT, INITIAL_CAPITAL_INR);
    let mut coralys_arm =
        PortfolioArm::new("coralys_v0", CORALYS_ARM_CONTRACT, INITIAL_CAPITAL_INR);

    // Phase 1: collect decisions
    struct Plan {
        instrument: String,
        bars: Vec<YahooHistoricalBar>,
        decision: super::observatory_slice::SealedDecisionRecord,
        entry_price: f64,
        direction_str: String,
        atr: Option<f64>,
    }

    let mut plans: Vec<Plan> = Vec::new();

    for instrument in RESEARCH_UNIVERSE {
        let bars = cache
            .get(instrument)
            .ok_or_else(|| format!("cache missing {instrument}"))?
            .clone();
        let known = decision_time_bars(&bars, certified_t);
        let decision =
            generate_historical_replay_decision(artifact, instrument, &bars, certified_t)?;
        if decision.action == DecisionAction::NoTrade {
            continue;
        }
        let entry = entry_close(&bars, certified_t)
            .ok_or_else(|| format!("no entry close for {instrument}"))?;
        let atr = atr_14_at_t(&known, certified_t);
        let direction_str = match decision.action {
            DecisionAction::Long => "LONG",
            DecisionAction::Short => "SHORT",
            DecisionAction::NoTrade => unreachable!(),
        }
        .to_string();
        plans.push(Plan {
            instrument: instrument.to_string(),
            bars,
            decision,
            entry_price: entry,
            direction_str,
            atr,
        });
    }

    if plans.is_empty() {
        return Err("no eligible instruments — all NO_TRADE".into());
    }

    let n_eligible = plans.len();
    let alloc = INITIAL_CAPITAL_INR / n_eligible as f64;

    // Phase 2: open positions
    for plan in &plans {
        // P.E.2 arm
        let pe2_intent =
            seal_execution_intent(&plan.decision, plan.entry_price, EXECUTION_TARGET_PCT)?;
        pe2_arm.positions.push(PortfolioPosition {
            instrument: plan.instrument.clone(),
            decision_id: plan.decision.decision_id.clone(),
            direction: plan.direction_str.clone(),
            entry_price: plan.entry_price,
            capital_allocated_inr: alloc,
            units: alloc / plan.entry_price,
            target_pct: pe2_intent.target_pct,
            target_price: pe2_intent.target_price,
            stop_pct: None,
            stop_price: None,
            entry_time: certified_t.to_rfc3339(),
            exit_time: None,
            exit_price: None,
            exit_reason: None,
            holding_sessions: None,
            realized_pnl_inr: None,
            realized_return_pct: None,
            status: PositionStatus::Open,
        });
        pe2_arm.n_positions_opened += 1;
        pe2_arm.cash_inr -= alloc;

        // Coralys arm
        let coralys_result = seal_coralys_execution_intent(
            &plan.instrument,
            &plan.decision.decision_time,
            &plan.decision.decision_time,
            &plan.direction_str,
            plan.entry_price,
            plan.atr,
            &plan.decision.state.trend,
            &plan.decision.state.momentum,
            &plan.decision.state.state_hash,
        )?;

        match coralys_result {
            CoralysExecutionResult::Intent(ci) => {
                coralys_arm.positions.push(PortfolioPosition {
                    instrument: plan.instrument.clone(),
                    decision_id: plan.decision.decision_id.clone(),
                    direction: plan.direction_str.clone(),
                    entry_price: plan.entry_price,
                    capital_allocated_inr: alloc,
                    units: alloc / plan.entry_price,
                    target_pct: ci.target_pct,
                    target_price: ci.target_price,
                    stop_pct: Some(ci.risk_pct),
                    stop_price: Some(ci.risk_boundary),
                    entry_time: certified_t.to_rfc3339(),
                    exit_time: None,
                    exit_price: None,
                    exit_reason: None,
                    holding_sessions: None,
                    realized_pnl_inr: None,
                    realized_return_pct: None,
                    status: PositionStatus::Open,
                });
                coralys_arm.n_positions_opened += 1;
                coralys_arm.cash_inr -= alloc;
            }
            CoralysExecutionResult::Invalid { reason, .. } => {
                eprintln!(
                    "Coralys arm: {} excluded (ATR unavailable): {}",
                    plan.instrument, reason
                );
            }
        }
    }

    // Phase 3: replay exits
    for plan in &plans {
        let bars = &plan.bars;

        // P.E.2 exit
        if let Some(pos) = pe2_arm
            .positions
            .iter()
            .find(|p| p.instrument == plan.instrument && !p.status.is_closed())
            .cloned()
        {
            let intent = SealedExecutionIntent {
                decision_id: plan.decision.decision_id.clone(),
                instrument: plan.instrument.clone(),
                decision_time: plan.decision.decision_time.clone(),
                action: plan.direction_str.clone(),
                entry_price: pos.entry_price,
                target_pct: pos.target_pct,
                target_price: pos.target_price,
                stop_pct: None,
                stop_price: None,
                max_holding_sessions: MAXIMUM_HOLD_SESSIONS,
                target_source: "deterministic_policy_parameter".to_string(),
                execution_contract: PE2_ARM_CONTRACT.to_string(),
                sealed_at_t: true,
                intent_hash: String::new(),
            };
            let exit = first_exit(&plan.decision, &intent, bars)?;
            if let (Some(ep), Some(et), Some(hs)) =
                (exit.exit_price, exit.exit_time.as_deref(), exit.holding_sessions)
            {
                pe2_arm.close_position(&plan.instrument, ep, et, exit.exit_reason, hs);
                let marks = BTreeMap::new();
                let val = pe2_arm.total_value_inr(&marks);
                pe2_arm.update_drawdown(val);
            }
        }

        // Coralys exit
        if let Some(pos) = coralys_arm
            .positions
            .iter()
            .find(|p| p.instrument == plan.instrument && !p.status.is_closed())
            .cloned()
        {
            let stop_price = pos.stop_price;
            let intent = SealedExecutionIntent {
                decision_id: plan.decision.decision_id.clone(),
                instrument: plan.instrument.clone(),
                decision_time: plan.decision.decision_time.clone(),
                action: plan.direction_str.clone(),
                entry_price: pos.entry_price,
                target_pct: pos.target_pct,
                target_price: pos.target_price,
                stop_pct: pos.stop_pct,
                stop_price,
                max_holding_sessions: MAXIMUM_HOLD_SESSIONS,
                target_source: "coralys_exec_v0_atr_tmv".to_string(),
                execution_contract: CORALYS_ARM_CONTRACT.to_string(),
                sealed_at_t: false,
                intent_hash: String::new(),
            };
            let exit = first_exit_with_optional_stop(
                &plan.decision,
                &intent,
                bars,
                stop_price,
                true,
            )?;
            if let (Some(ep), Some(et), Some(hs)) =
                (exit.exit_price, exit.exit_time.as_deref(), exit.holding_sessions)
            {
                coralys_arm.close_position(&plan.instrument, ep, et, exit.exit_reason, hs);
                let marks = BTreeMap::new();
                let val = coralys_arm.total_value_inr(&marks);
                coralys_arm.update_drawdown(val);
            }
        }
    }

    // Phase 4: final mark prices
    let mut final_marks: BTreeMap<String, f64> = BTreeMap::new();
    for plan in &plans {
        if let Some(last_close) = plan
            .bars
            .iter()
            .filter_map(|b| {
                let ts = chrono::Utc.timestamp_opt(b.timestamp, 0).single()?;
                if b.adj_close > 0.0 && b.adj_close.is_finite() {
                    Some((ts, b.adj_close))
                } else {
                    None
                }
            })
            .max_by_key(|(ts, _)| *ts)
            .map(|(_, c)| c)
        {
            final_marks.insert(plan.instrument.clone(), last_close);
        }
    }

    let pe2_final = pe2_arm.total_value_inr(&final_marks);
    pe2_arm.update_drawdown(pe2_final);
    let coralys_final = coralys_arm.total_value_inr(&final_marks);
    coralys_arm.update_drawdown(coralys_final);

    let pe2_summary = build_arm_summary(&pe2_arm, &final_marks);
    let coralys_summary = build_arm_summary(&coralys_arm, &final_marks);

    let comparison = PortfolioComparisonReport {
        path_kind: PORTFOLIO_REPLAY_PATH_KIND.to_string(),
        experiment_id: PORTFOLIO_REPLAY_EXPERIMENT_ID.to_string(),
        requested_clock: PORTFOLIO_REPLAY_REQUESTED_CLOCK.to_string(),
        certified_t: certified_t.to_rfc3339(),
        initial_capital_inr: INITIAL_CAPITAL_INR,
        coralys_artifact_hash: CORALYS_EXEC_ARTIFACT_HASH.to_string(),
        c3_002_artifact_hash: RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH.to_string(),
        pe2_arm: pe2_summary,
        coralys_arm: coralys_summary,
        exploratory_note: format!(
            "Exploratory n={n_eligible}. Not a statistical strategy backtest. \
             Results are evidence, not proof. Both frozen artifacts unchanged.",
        ),
        methodology_note: "P.E.2 arm: fixed +5% target, no stop. \
             Coralys v0 arm: ATR/TMV target + enforced risk_boundary stop. \
             Same C3-002 decisions, same market data, same period. \
             Divergence is the product effect being measured."
            .to_string(),
    };

    Ok(PortfolioReplayLedger {
        path_kind: PORTFOLIO_REPLAY_PATH_KIND.to_string(),
        experiment_id: PORTFOLIO_REPLAY_EXPERIMENT_ID.to_string(),
        requested_clock: PORTFOLIO_REPLAY_REQUESTED_CLOCK.to_string(),
        certified_t: certified_t.to_rfc3339(),
        initial_capital_inr: INITIAL_CAPITAL_INR,
        coralys_artifact_hash: CORALYS_EXEC_ARTIFACT_HASH.to_string(),
        c3_002_artifact_hash: RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH.to_string(),
        pe2_arm,
        coralys_arm,
        comparison,
        integrity_note: format!(
            "Portfolio Replay v0.1. C3-002: {c3}. Coralys: {coralys}. \
             P.E.3-B archive untouched. P.E.2 archive untouched.",
            c3 = RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH,
            coralys = CORALYS_EXEC_ARTIFACT_HASH,
        ),
    })
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_guard_blocks_pe2_archive() {
        assert!(refuse_portfolio_replay_output(
            "historical_runs/pe2_control_2026-08-16/ledger.json"
        )
        .is_err());
    }

    #[test]
    fn output_guard_blocks_pe3_archive() {
        assert!(refuse_portfolio_replay_output(
            "historical_runs/pe3_coralys_v0_2026-08-16/ledger.json"
        )
        .is_err());
    }

    #[test]
    fn output_guard_allows_portfolio_comparison_path() {
        assert!(refuse_portfolio_replay_output(
            "historical_runs/portfolio_comparison_pe2_vs_pe3_2026-08-16/ledger.json"
        )
        .is_ok());
    }

    #[test]
    fn arm_new_initializes_correctly() {
        let arm = PortfolioArm::new("test", "contract_v0", 5000.0);
        assert_eq!(arm.cash_inr, 5000.0);
        assert_eq!(arm.initial_capital_inr, 5000.0);
        assert_eq!(arm.n_positions_opened, 0);
        assert!(arm.positions.is_empty());
    }

    #[test]
    fn arm_close_position_updates_cash_and_pnl() {
        let mut arm = PortfolioArm::new("test", "contract_v0", 5000.0);
        arm.positions.push(PortfolioPosition {
            instrument: "INFY.NS".to_string(),
            decision_id: "dec-001".to_string(),
            direction: "LONG".to_string(),
            entry_price: 1000.0,
            capital_allocated_inr: 1000.0,
            units: 1.0,
            target_pct: 0.05,
            target_price: 1050.0,
            stop_pct: None,
            stop_price: None,
            entry_time: "2026-07-15T03:45:00Z".to_string(),
            exit_time: None,
            exit_price: None,
            exit_reason: None,
            holding_sessions: None,
            realized_pnl_inr: None,
            realized_return_pct: None,
            status: PositionStatus::Open,
        });
        arm.n_positions_opened = 1;
        arm.cash_inr = 4000.0;

        arm.close_position("INFY.NS", 1050.0, "2026-07-25T03:45:00Z", ExitReason::Target, 8);

        assert_eq!(arm.n_target, 1);
        assert!((arm.cash_inr - 5050.0).abs() < 0.01, "cash={}", arm.cash_inr);
        assert!((arm.total_realized_pnl_inr - 50.0).abs() < 0.01);
        assert_eq!(arm.positions[0].status, PositionStatus::ClosedTarget);
    }

    #[test]
    fn arm_close_stop_records_stop_exit() {
        let mut arm = PortfolioArm::new("test", "contract_v0", 5000.0);
        arm.positions.push(PortfolioPosition {
            instrument: "TCS.NS".to_string(),
            decision_id: "dec-002".to_string(),
            direction: "LONG".to_string(),
            entry_price: 2000.0,
            capital_allocated_inr: 1000.0,
            units: 0.5,
            target_pct: 0.08,
            target_price: 2160.0,
            stop_pct: Some(0.04),
            stop_price: Some(1920.0),
            entry_time: "2026-07-15T03:45:00Z".to_string(),
            exit_time: None,
            exit_price: None,
            exit_reason: None,
            holding_sessions: None,
            realized_pnl_inr: None,
            realized_return_pct: None,
            status: PositionStatus::Open,
        });
        arm.cash_inr = 4000.0;

        arm.close_position("TCS.NS", 1920.0, "2026-07-18T03:45:00Z", ExitReason::Stop, 3);

        assert_eq!(arm.n_stop, 1);
        assert!((arm.total_realized_pnl_inr - (-40.0)).abs() < 0.01);
        assert_eq!(arm.positions[0].status, PositionStatus::ClosedStop);
    }

    #[test]
    fn pe2_arm_contract_distinct_from_coralys() {
        assert_ne!(PE2_ARM_CONTRACT, CORALYS_ARM_CONTRACT);
        assert_eq!(PE2_ARM_CONTRACT, "targeted_execution_v0_fixed_5pct_20_sessions");
        assert_eq!(
            CORALYS_ARM_CONTRACT,
            "coralys_exec_v0_atr_tmv_stop_enforced_20_sessions"
        );
    }

    #[test]
    fn initial_capital_is_5000() {
        assert!((INITIAL_CAPITAL_INR - 5000.0).abs() < 1e-9);
    }

    #[test]
    fn requested_clock_matches_pe2_period() {
        assert_eq!(PORTFOLIO_REPLAY_REQUESTED_CLOCK, "2026-07-15T03:45:00+00:00");
    }
}
