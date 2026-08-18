//! MVP-010 — Recommendations v1 API: `GET /recommendations/v1/latest`
//!
//! Runs RecommendationEngineV1 against all certified decisions in the ledger
//! and returns a ranked [`RecommendationSnapshotV1`].
//!
//! Evidence source: REC-001-H (Rec001hStore, 101 tickers, 121,805 records).
//! Policy version: "v1".
//!
//! Key differences from v0 (`/recommendations/latest`):
//! - Ticker-specific analogue population (not aggregate HDV-001 pool)
//! - Adaptive geometry from MFE/MAE percentiles (not fixed ATR multipliers)
//! - First-exit semantics: target from winning analogues, risk from losing analogues
//! - Graceful degradation: Exact → RelaxVolume → RelaxBoth → StateOnly → NO_TRADE
//! - Volume regime conditioning from relative_volume_20 (defaults to 1.0 = Normal)
//!
//! **Acceptance criteria:**
//! - AC-V1-R1: Returns 200 + ranked recommendations for all decisions in the ledger.
//! - AC-V1-R2: Recommendations are sorted by rank_score descending.
//! - AC-V1-R3: Each recommendation includes adaptive_target, adaptive_risk, adaptive_rr,
//!             adaptive_horizon_sessions, degradation_level, target_rate, sample_size.
//! - AC-V1-R4: No confidence, probability, or expected-return fields are added.
//! - AC-V1-R5: recommendation_policy_version == "v1" in every record.
//! - AC-V1-R6: Snapshot includes evaluated, actionable, buy, watch, no_trade counts.
//! - AC-V1-R7: Returns 503 when Rec001hStore is not loaded.

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use coralys_decision::recommendation::{
    RecommendationEngineV1, RecommendationRecordV1,
    engine::RecommendationAction,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

// ─── Response types ───────────────────────────────────────────────────────────

/// Response envelope for `GET /recommendations/v1/latest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationSnapshotV1 {
    /// Total decisions evaluated.
    pub evaluated: usize,
    /// Decisions with action BUY or WATCH.
    pub actionable: usize,
    /// Decisions with action BUY.
    pub buy: usize,
    /// Decisions with action WATCH.
    pub watch: usize,
    /// Decisions with action NO_TRADE.
    pub no_trade: usize,
    /// Policy version — always "v1".
    pub policy_version: String,
    /// All recommendations, sorted by rank_score descending.
    pub recommendations: Vec<RecommendationRecordV1>,
}

// ─── Handler ──────────────────────────────────────────────────────────────────

/// `GET /recommendations/v1/latest`
///
/// Evaluates all certified decisions in the ledger through RecommendationEngineV1
/// and returns a ranked snapshot.
///
/// The engine is stateless at query time — all state is in the Rec001hStore
/// loaded at startup. No external I/O occurs during request handling.
pub async fn get_recommendations_v1_latest(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let rec001h_store = match &state.rec001h_store {
        Some(store) => store,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "REC-001-H evidence store not loaded. Check REC001H_DIR env var.",
                    "hint": "Expected at datasets/recommendation/historical/*.jsonl"
                })),
            )
                .into_response();
        }
    };

    let ledger = state.ledger.read().await;
    let all_decisions = ledger.all_decisions();

    let engine = RecommendationEngineV1::new(rec001h_store);

    let mut recommendations: Vec<RecommendationRecordV1> = all_decisions
        .iter()
        .map(|record| {
            let dir = match &record.decision.direction {
                coralys_decision::Direction::Long => "LONG",
                coralys_decision::Direction::Short => "SHORT",
                coralys_decision::Direction::NoTrade => "NO_TRADE",
            };

            // Normalise instrument to TICKER_NS format expected by Rec001hStore.
            // The ledger stores "ADANIENT.NS"; the store indexes as "ADANIENT_NS".
            let instrument_key = record.identity.instrument.replace('.', "_");

            // relative_volume_20 is not stored in DecisionCore (it's a market data
            // field not yet propagated to the decision record). Default to 1.0
            // (Normal volume regime) — the engine will use RelaxVolume or RelaxBoth
            // as needed via graceful degradation.
            let relative_volume_20 = 1.0_f64;

            engine.evaluate(
                &record.identity.decision_id,
                &instrument_key,
                dir,
                &record.decision.trend,
                &record.decision.momentum,
                record.decision.reference_price,
                &record.decision.volatility,
                relative_volume_20,
            )
        })
        .collect();

    // Sort by rank_score descending
    recommendations.sort_by(|a, b| {
        b.rank_score
            .partial_cmp(&a.rank_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let evaluated = recommendations.len();
    let buy = recommendations
        .iter()
        .filter(|r| r.action == RecommendationAction::Buy)
        .count();
    let watch = recommendations
        .iter()
        .filter(|r| r.action == RecommendationAction::Watch)
        .count();
    let no_trade = recommendations
        .iter()
        .filter(|r| r.action == RecommendationAction::NoTrade)
        .count();
    let actionable = buy + watch;

    let snapshot = RecommendationSnapshotV1 {
        evaluated,
        actionable,
        buy,
        watch,
        no_trade,
        policy_version: "v1".to_string(),
        recommendations,
    };

    (StatusCode::OK, Json(snapshot)).into_response()
}