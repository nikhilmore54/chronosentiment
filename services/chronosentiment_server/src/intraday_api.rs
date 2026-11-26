//! GET /api/v1/intraday/decisions — thin HTTP adapter over adapter DecisionBrief.
//!
//! ## Responsibility
//!
//! Domain ownership lives in `chronosentiment_adapter::product::intraday_decision`.
//! This module serializes those briefs and joins frozen reassessment v0.2 CSVs.
//! It does not classify, invent prices, or alter IC v1.
//!
//! ## Architecture
//!
//! ```text
//! datasets/p4_opportunity_dataset.json   (golden historical fixture)
//!                    │
//!                    ▼
//!     adapters/chronosentiment product::intraday_decision
//!         load_intraday_briefs() + frozen IC v1
//!                    │
//!                    ▼
//!            DecisionBrief DTO
//!                    │
//!                    ▼
//!          GET /api/v1/intraday/decisions
//!          GET /api/v1/intraday/decisions/search?ticker=...
//!          GET /api/v1/intraday/decisions/:id
//! ```
//!
//! ## Endpoints
//!
//! - `GET /api/v1/intraday/decisions`
//!   Returns all 459 enriched decisions.
//!
//! - `GET /api/v1/intraday/decisions/search?ticker=IDEA&direction=SHORT&state=ENTER`
//!   Returns filtered decisions. All query params optional.
//!
//! - `GET /api/v1/intraday/decisions/:id`
//!   Returns a single decision by decision_id.
//!
//! ## Data source
//!
//! v1: loads from `INTRADAY_DATASET_PATH` env var
//!     (default: `datasets/p4_opportunity_dataset.json`).
//! v2: will load from the live execution ledger / Postgres decision store.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chronosentiment_adapter::product::{
    DecisionBrief, ExecutionFacts, PaperBlotter, assemble_paper_blotter, attach_decision_links,
    load_intraday_briefs, load_paper_positions,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// ── In-memory decision store (loaded once at startup) ─────────────────────────

pub type IntradayStore = Arc<Vec<DecisionBrief>>;

/// Load and enrich all records from the golden dataset.
/// Called once at server startup; result is shared via Arc.
pub fn load_intraday_store(dataset_path: &str) -> Result<IntradayStore, String> {
    let briefs = load_intraday_briefs(dataset_path)?;
    eprintln!(
        "[intraday_api] Loaded {} decisions from {}",
        briefs.len(),
        dataset_path
    );
    Ok(Arc::new(briefs))
}

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub ticker: Option<String>,
    pub direction: Option<String>,
    pub entry_state: Option<String>,
    pub h120_state: Option<String>,
    pub action: Option<String>,
    pub date: Option<String>,
}

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct DecisionListResponse {
    pub total: usize,
    pub decisions: Vec<DecisionBrief>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/v1/intraday/decisions
/// Returns all enriched decisions.
pub async fn get_all_decisions(
    State(store): State<IntradayStore>,
) -> Json<DecisionListResponse> {
    let decisions = (*store).clone();
    Json(DecisionListResponse {
        total: decisions.len(),
        decisions,
    })
}

/// GET /api/v1/intraday/decisions/search?ticker=IDEA&direction=SHORT&...
/// Returns filtered decisions.
pub async fn search_decisions(
    State(store): State<IntradayStore>,
    Query(params): Query<SearchParams>,
) -> Json<DecisionListResponse> {
    let decisions: Vec<DecisionBrief> = store
        .iter()
        .filter(|d| {
            if let Some(ref t) = params.ticker {
                if !d.ticker.to_uppercase().contains(&t.to_uppercase()) {
                    return false;
                }
            }
            if let Some(ref dir) = params.direction {
                if d.direction.to_uppercase() != dir.to_uppercase() {
                    return false;
                }
            }
            if let Some(ref es) = params.entry_state {
                if d.entry_state != *es {
                    return false;
                }
            }
            if let Some(ref hs) = params.h120_state {
                if d.h120_state != *hs {
                    return false;
                }
            }
            if let Some(ref act) = params.action {
                if d.entry_action != *act {
                    return false;
                }
            }
            if let Some(ref date) = params.date {
                if d.date != *date {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    Json(DecisionListResponse {
        total: decisions.len(),
        decisions,
    })
}

/// GET /api/v1/intraday/decisions/:id
/// Returns a single decision by decision_id.
pub async fn get_decision_by_id(
    State(store): State<IntradayStore>,
    Path(id): Path<String>,
) -> Result<Json<DecisionBrief>, StatusCode> {
    match store.iter().find(|d| d.id == id) {
        Some(d) => Ok(Json(d.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// ── Portfolio context ─────────────────────────────────────────────────────────

/// Portfolio context for a single decision.
/// Mirrors the Backtest v2 OQS-ranked selection logic exactly.
#[derive(Debug, Serialize, Clone)]
pub struct DecisionPortfolioContext {
    pub id: String,
    pub ticker: String,
    pub date: String,
    pub direction: String,
    pub oqs: u32,
    pub entry_state: String,
    pub entry_action: String,
    /// 1-based rank within the day's ACT candidates (sorted by OQS desc)
    pub oqs_rank: usize,
    /// Total ACT candidates on this date
    pub candidates_on_date: usize,
    /// Whether this decision is in the top MAX_POSITIONS for the day
    pub selected: bool,
    /// Position size as a percentage of portfolio capital (0.0 if not selected)
    pub position_size_pct: f64,
}

#[derive(Debug, Serialize)]
pub struct PortfolioContextResponse {
    pub total_candidates: usize,
    pub total_selected: usize,
    pub max_positions: usize,
    pub position_fraction: f64,
    pub decisions: Vec<DecisionPortfolioContext>,
}

/// Portfolio simulation parameters (mirrors Backtest v2 — do not change).
const MAX_POSITIONS: usize = 5;
const POSITION_FRACTION: f64 = 0.10;

/// ACT states — mirrors Backtest v2 ACT_STATES exactly.
fn is_act_state(action: &str) -> bool {
    matches!(action, "ACT")
}

/// GET /api/v1/intraday/portfolio-context
///
/// Returns all IC v1 ACT decisions enriched with portfolio context:
/// OQS rank within the day, selected (top-5 by OQS), and position size.
///
/// This mirrors the Backtest v2 OQS-ranked selection logic.
/// IC v1 classification is NOT modified.
pub async fn get_portfolio_context(
    State(store): State<IntradayStore>,
) -> Json<PortfolioContextResponse> {
    use std::collections::HashMap;

    // 1. Filter to ACT decisions only (entry_action == "ACT")
    let act_decisions: Vec<&DecisionBrief> = store
        .iter()
        .filter(|d| is_act_state(&d.entry_action))
        .collect();

    // 2. Group by date
    let mut by_date: HashMap<&str, Vec<&DecisionBrief>> = HashMap::new();
    for d in &act_decisions {
        by_date.entry(d.date.as_str()).or_default().push(d);
    }

    // 3. For each date, sort by OQS descending and assign rank + selected
    let mut result: Vec<DecisionPortfolioContext> = Vec::new();

    let mut sorted_dates: Vec<&str> = by_date.keys().copied().collect();
    sorted_dates.sort();

    for date in sorted_dates {
        let mut day = by_date[date].clone();
        // Sort by OQS descending (highest quality first)
        day.sort_by(|a, b| b.oqs.cmp(&a.oqs));

        let n = day.len();
        for (rank_0, d) in day.iter().enumerate() {
            let rank = rank_0 + 1; // 1-based
            let selected = rank <= MAX_POSITIONS;
            let position_size_pct = if selected { POSITION_FRACTION * 100.0 } else { 0.0 };

            result.push(DecisionPortfolioContext {
                id: d.id.clone(),
                ticker: d.ticker.clone(),
                date: d.date.clone(),
                direction: d.direction.clone(),
                oqs: d.oqs,
                entry_state: d.entry_state.clone(),
                entry_action: d.entry_action.clone(),
                oqs_rank: rank,
                candidates_on_date: n,
                selected,
                position_size_pct,
            });
        }
    }

    let total_selected = result.iter().filter(|d| d.selected).count();

    Json(PortfolioContextResponse {
        total_candidates: result.len(),
        total_selected,
        max_positions: MAX_POSITIONS,
        position_fraction: POSITION_FRACTION,
        decisions: result,
    })
}

// ── Decision Feed / Recommendations ──────────────────────────────────────────

/// A single recommendation card for the Decision Feed.
/// Derived entirely from the existing IntradayStore — no new data loading.
#[derive(Debug, Serialize, Clone)]
pub struct RecommendationCard {
    /// Decision ID — links to the full DecisionBrief detail view.
    pub id: String,
    pub ticker: String,
    pub date: String,
    pub direction: String,
    pub oqs: u32,
    /// 1-based rank within the day's ACT candidates (sorted by OQS desc)
    pub oqs_rank: usize,
    /// Total ACT candidates on this date
    pub candidates_on_date: usize,
    /// Entry state from IC v1
    pub entry_state: String,
    /// Copied from the adapter DecisionBrief. Never recalculated.
    pub entry_action: String,
    /// Copied from the adapter DecisionBrief. Never recalculated.
    pub reference_price: Option<f64>,
    /// Copied from the adapter DecisionBrief. Never recalculated.
    pub entry_price: Option<f64>,
    /// Copied from the adapter DecisionBrief. Never recalculated.
    pub execution: ExecutionFacts,
    /// Historical win rate for this state (frozen from v0.9)
    pub hist_win: Option<f64>,
    /// Historical profit factor for this state
    pub hist_pf: Option<f64>,
    /// Historical median return for this state
    pub hist_med: Option<f64>,
    /// Observed H300 return (null for live/future decisions)
    pub h300_ret: Option<f64>,
    /// Observed outcome (null for live/future decisions)
    pub outcome: Option<String>,
    /// Status: NEW (no position yet), ACTIVE (position open), CLOSED (exited)
    /// v0.1: always "NEW" — position management is a future capability.
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct RecommendationFeedResponse {
    pub total: usize,
    pub selected_per_day: usize,
    pub recommendations: Vec<RecommendationCard>,
}

/// GET /api/v1/intraday/recommendations
///
/// Returns the top-N selected ACT decisions per date as recommendation cards,
/// sorted by date descending then OQS rank ascending (best first).
///
/// This is the primary Decision Feed endpoint for the Cockpit UI.
/// It is derived entirely from the existing IntradayStore — no new data loading.
/// IC v1 classification is NOT modified.
pub async fn get_recommendations(
    State(store): State<IntradayStore>,
) -> Json<RecommendationFeedResponse> {
    use std::collections::HashMap;

    // 1. Filter to ACT decisions only
    let act_decisions: Vec<&DecisionBrief> = store
        .iter()
        .filter(|d| is_act_state(&d.entry_action))
        .collect();

    // 2. Group by date
    let mut by_date: HashMap<&str, Vec<&DecisionBrief>> = HashMap::new();
    for d in &act_decisions {
        by_date.entry(d.date.as_str()).or_default().push(d);
    }

    // 3. For each date, sort by OQS desc, take top MAX_POSITIONS
    let mut cards: Vec<RecommendationCard> = Vec::new();

    let mut sorted_dates: Vec<&str> = by_date.keys().copied().collect();
    // Descending date order — most recent first in the feed
    sorted_dates.sort_by(|a, b| b.cmp(a));

    for date in sorted_dates {
        let mut day = by_date[date].clone();
        day.sort_by(|a, b| b.oqs.cmp(&a.oqs));

        let n = day.len();
        for (rank_0, d) in day.iter().enumerate().take(MAX_POSITIONS) {
            let rank = rank_0 + 1;
            cards.push(RecommendationCard {
                id: d.id.clone(),
                ticker: d.ticker.clone(),
                date: d.date.clone(),
                direction: d.direction.clone(),
                oqs: d.oqs,
                oqs_rank: rank,
                candidates_on_date: n,
                entry_state: d.entry_state.clone(),
                entry_action: d.entry_action.clone(),
                reference_price: d.reference_price,
                entry_price: d.entry_price,
                execution: d.execution.clone(),
                hist_win: d.hist_win,
                hist_pf: d.hist_pf,
                hist_med: d.hist_med,
                h300_ret: d.h300_ret,
                outcome: d.outcome.clone(),
                status: "NEW".to_string(),
            });
        }
    }

    let total = cards.len();
    Json(RecommendationFeedResponse {
        total,
        selected_per_day: MAX_POSITIONS,
        recommendations: cards,
    })
}

// ── Position Lifecycle ────────────────────────────────────────────────────────

/// Decision evidence — the authoritative facts that support the recommendation.
/// The frontend renders these facts; it never manufactures explanations.
#[derive(Debug, Serialize, Clone)]
pub struct DecisionEvidence {
    // Historical performance facts (frozen from v0.9)
    pub hist_win: Option<f64>,
    pub hist_pf: Option<f64>,
    pub hist_med: Option<f64>,

    /// Copied from the adapter DecisionBrief. Never recalculated.
    pub reference_price: Option<f64>,
    /// Copied from the adapter DecisionBrief. Never recalculated.
    pub entry_price: Option<f64>,
    /// Copied from the adapter DecisionBrief. Never recalculated.
    pub execution: ExecutionFacts,
    /// Exit price from reassessment v0.2 CSV (null if NEW).
    pub exit_price: Option<f64>,

    // Opportunity facts
    pub oqs: u32,
    pub oqs_rank: usize,
    pub candidates_on_date: usize,
    pub entry_state: String,
    pub entry_why: String,
    pub entry_risk: String,
    pub entry_horizon: String,

    // Observed path facts (null for live/future decisions)
    pub h60_ret: Option<f64>,
    pub h120_ret: Option<f64>,
    pub h300_ret: Option<f64>,
    pub mfe_h60: Option<f64>,

    // Reassessment facts (null if not reassessed)
    pub h120_state: String,
    pub h120_why: String,
}

/// Lifecycle-enriched recommendation card.
/// Extends RecommendationCard with position state from the reassessment v0.2 replay.
#[derive(Debug, Serialize, Clone)]
pub struct LifecycleCard {
    // Identity
    pub id: String,
    pub ticker: String,
    pub date: String,
    pub direction: String,
    pub oqs: u32,
    pub oqs_rank: usize,
    pub candidates_on_date: usize,
    pub entry_state: String,
    /// Copied from the adapter DecisionBrief. Never recalculated.
    pub entry_action: String,
    /// Copied from the adapter DecisionBrief. Never recalculated.
    pub reference_price: Option<f64>,
    /// Copied from the adapter DecisionBrief. Never recalculated.
    pub entry_price: Option<f64>,
    /// Copied from the adapter DecisionBrief. Never recalculated.
    pub execution: ExecutionFacts,
    /// Exit price from reassessment v0.2 CSV (null if NEW)
    pub exit_price: Option<f64>,
    pub hist_win: Option<f64>,
    pub hist_pf: Option<f64>,
    pub hist_med: Option<f64>,
    pub h300_ret: Option<f64>,
    pub outcome: Option<String>,
    /// NEW | REASSESS_EXIT | CLOSED
    pub status: String,
    // Lifecycle fields (null for NEW)
    /// Realized return at exit (null if NEW)
    pub realized_ret: Option<f64>,
    /// Exit reason: STOP | TARGET | HORIZON | REASSESS_EXIT (null if NEW)
    pub exit_reason: Option<String>,
    /// IST time of reassessment exit (null if not reassessed)
    pub reassess_time_ist: Option<String>,
    /// H300 counterfactual return (what would have happened without early exit)
    pub h300_counterfactual_ret: Option<f64>,
    /// Authoritative facts supporting the recommendation — rendered by the UI, never manufactured.
    pub evidence: DecisionEvidence,
}

#[derive(Debug, Serialize)]
pub struct LifecycleFeedResponse {
    pub total: usize,
    pub selected_per_day: usize,
    pub cards: Vec<LifecycleCard>,
}

/// Minimal reassessment row parsed from reassessment_v2_YYYYMMDD.csv
struct ReassessRow {
    exit_reason: String,
    realized_return: f64,
    reassess_triggered: bool,
    reassess_time_ist: Option<String>,
    h300_counterfactual_ret: Option<f64>,
    exit_price: Option<f64>,
}

/// Load all reassessment_v2_*.csv files from the datasets directory.
/// Returns a map of (ticker, date_str) → ReassessRow.
fn load_reassessment_index(datasets_dir: &str) -> HashMap<(String, String), ReassessRow> {
    let mut index: HashMap<(String, String), ReassessRow> = HashMap::new();

    let dir = match std::fs::read_dir(datasets_dir) {
        Ok(d) => d,
        Err(_) => return index,
    };

    for entry in dir.flatten() {
        let path = entry.path();
        let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !fname.starts_with("reassessment_v2_") || !fname.ends_with(".csv") {
            continue;
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let mut lines = content.lines();
        // Parse header to find column indices
        let header = match lines.next() {
            Some(h) => h,
            None => continue,
        };
        let cols: Vec<&str> = header.split(',').collect();
        let idx = |name: &str| cols.iter().position(|&c| c == name);

        let i_ticker   = idx("ticker");
        let i_date     = idx("date");
        let i_exit_r   = idx("exit_reason");
        let i_realized = idx("realized_return");
        let i_reassess = idx("reassess_triggered");
        let i_rtime    = idx("reassess_time_ist");
        let i_cf       = idx("h300_counterfactual_ret");
        let i_exit_px  = idx("exit_price");

        for line in lines {
            let fields: Vec<&str> = line.split(',').collect();
            let get = |i: Option<usize>| i.and_then(|n| fields.get(n).copied()).unwrap_or("");

            let ticker   = get(i_ticker).to_string();
            let date     = {
                let raw = get(i_date);
                // Reassessment CSVs use YYYYMMDD; the adapter brief uses YYYY-MM-DD.
                if raw.len() == 8 && raw.chars().all(|c| c.is_ascii_digit()) {
                    format!("{}-{}-{}", &raw[0..4], &raw[4..6], &raw[6..8])
                } else {
                    raw.to_string()
                }
            };
            let exit_r   = get(i_exit_r).to_string();
            let realized = get(i_realized).parse::<f64>().unwrap_or(0.0);
            let reassess = get(i_reassess) == "True";
            let rtime    = {
                let v = get(i_rtime);
                if v.is_empty() || v == "None" { None } else { Some(v.to_string()) }
            };
            let cf = get(i_cf).parse::<f64>().ok();
            let exit_px = get(i_exit_px).parse::<f64>().ok();

            if !ticker.is_empty() && !date.is_empty() {
                index.insert((ticker, date), ReassessRow {
                    exit_reason: exit_r,
                    realized_return: realized,
                    reassess_triggered: reassess,
                    reassess_time_ist: rtime,
                    h300_counterfactual_ret: cf,
                    exit_price: exit_px,
                });
            }
        }
    }

    index
}

/// GET /api/v1/intraday/position-lifecycle
///
/// Returns the top-N selected ACT decisions per date enriched with lifecycle
/// status from the reassessment v0.2 replay CSVs.
///
/// Status values:
///   NEW           — no replay data (live/future decision)
///   REASSESS_EXIT — reassessment engine triggered early exit
///   CLOSED        — position closed via STOP, TARGET, or HORIZON
///
/// IC v1, Backtest v2, Stop Engine v0.2, and Reassessment v0.2 are NOT modified.
pub async fn get_position_lifecycle(
    State(store): State<IntradayStore>,
) -> Json<LifecycleFeedResponse> {
    // Load reassessment index from datasets/
    let datasets_dir = std::env::var("INTRADAY_DATASET_PATH")
        .unwrap_or_else(|_| "datasets/p4_opportunity_dataset.json".to_string());
    let dir = std::path::Path::new(&datasets_dir)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("datasets");
    let reassess_index = load_reassessment_index(dir);

    // Build recommendation cards (same logic as get_recommendations)
    let act_decisions: Vec<&DecisionBrief> = store
        .iter()
        .filter(|d| is_act_state(&d.entry_action))
        .collect();

    let mut by_date: HashMap<&str, Vec<&DecisionBrief>> = HashMap::new();
    for d in &act_decisions {
        by_date.entry(d.date.as_str()).or_default().push(d);
    }

    let mut cards: Vec<LifecycleCard> = Vec::new();
    let mut sorted_dates: Vec<&str> = by_date.keys().copied().collect();
    sorted_dates.sort_by(|a, b| b.cmp(a)); // most recent first

    for date in sorted_dates {
        let mut day = by_date[date].clone();
        day.sort_by(|a, b| b.oqs.cmp(&a.oqs));

        let n = day.len();
        for (rank_0, d) in day.iter().enumerate().take(MAX_POSITIONS) {
            let rank = rank_0 + 1;

            // Join with reassessment index
            let key = (d.ticker.clone(), d.date.clone());
            let (status, realized_ret, exit_reason, reassess_time_ist, h300_cf, exit_price) =
                if let Some(row) = reassess_index.get(&key) {
                    let status = if row.reassess_triggered {
                        "REASSESS_EXIT".to_string()
                    } else {
                        "CLOSED".to_string()
                    };
                    (
                        status,
                        Some(row.realized_return),
                        Some(row.exit_reason.clone()),
                        row.reassess_time_ist.clone(),
                        row.h300_counterfactual_ret,
                        row.exit_price,
                    )
                } else {
                    ("NEW".to_string(), None, None, None, None, None)
                };

            cards.push(LifecycleCard {
                id: d.id.clone(),
                ticker: d.ticker.clone(),
                date: d.date.clone(),
                direction: d.direction.clone(),
                oqs: d.oqs,
                oqs_rank: rank,
                candidates_on_date: n,
                entry_state: d.entry_state.clone(),
                entry_action: d.entry_action.clone(),
                reference_price: d.reference_price,
                entry_price: d.entry_price,
                execution: d.execution.clone(),
                exit_price,
                hist_win: d.hist_win,
                hist_pf: d.hist_pf,
                hist_med: d.hist_med,
                h300_ret: d.h300_ret,
                outcome: d.outcome.clone(),
                status,
                realized_ret,
                exit_reason,
                reassess_time_ist,
                h300_counterfactual_ret: h300_cf,
                evidence: DecisionEvidence {
                    hist_win: d.hist_win,
                    hist_pf: d.hist_pf,
                    hist_med: d.hist_med,
                    reference_price: d.reference_price,
                    entry_price: d.entry_price,
                    execution: d.execution.clone(),
                    exit_price,
                    oqs: d.oqs,
                    oqs_rank: rank,
                    candidates_on_date: n,
                    entry_state: d.entry_state.clone(),
                    entry_why: d.entry_why.clone(),
                    entry_risk: d.entry_risk.clone(),
                    entry_horizon: d.entry_horizon.clone(),
                    h60_ret: d.h60_ret,
                    h120_ret: d.h120_ret,
                    h300_ret: d.h300_ret,
                    mfe_h60: d.mfe_h60,
                    h120_state: d.h120_state.clone(),
                    h120_why: d.h120_why.clone(),
                },
            });
        }
    }

    let total = cards.len();
    Json(LifecycleFeedResponse {
        total,
        selected_per_day: MAX_POSITIONS,
        cards,
    })
}

fn datasets_dir() -> String {
    let dataset_path = std::env::var("INTRADAY_DATASET_PATH")
        .unwrap_or_else(|_| "datasets/p4_opportunity_dataset.json".to_string());
    std::path::Path::new(&dataset_path)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("datasets")
        .to_string()
}

/// GET /api/v1/intraday/paper-trades
///
/// Frozen Paper Trader v0.2 replay. Does not re-walk bars or alter IC v1.
/// Paper fills are not substituted for DecisionBrief entry_price / reference_price.
pub async fn get_paper_trades(State(store): State<IntradayStore>) -> Json<PaperBlotter> {
    let mut positions = load_paper_positions(&datasets_dir());
    attach_decision_links(&mut positions, store.as_ref());
    Json(assemble_paper_blotter(positions))
}