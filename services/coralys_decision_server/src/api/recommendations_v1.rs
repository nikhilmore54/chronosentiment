//! MVP-010 — Recommendations v1 API
//!
//! Two endpoints:
//!
//! ## `GET /recommendations/v1/latest`
//!
//! Returns **one recommendation per ticker** — the most recent decision for
//! each instrument, evaluated through RecommendationEngineV1.
//!
//! Deduplication: group all decisions by ticker → sort by decision_timestamp
//! descending → take the newest → evaluate → rank.
//!
//! This is the **operational screen**: what Coralys thinks *now* about each
//! ticker. Older observations are not shown here.
//!
//! ## `GET /recommendations/v1/history`
//!
//! Returns **all** recommendation observations — every decision ever evaluated,
//! sorted by decision_timestamp descending. Used by the History/Replay screen.
//!
//! Evidence source: REC-001-H (Rec001hStore, 101 tickers, 121,805 records).
//! Policy version: "v1".
//!
//! **Acceptance criteria:**
//! - AC-V1-R1: `/latest` returns at most one record per ticker (newest wins).
//! - AC-V1-R2: Recommendations are sorted by rank_score descending.
//! - AC-V1-R3: Each recommendation includes adaptive_target, adaptive_risk, adaptive_rr,
//!             adaptive_horizon_sessions, degradation_level, target_rate, sample_size.
//! - AC-V1-R4: No confidence, probability, or expected-return fields are added.
//! - AC-V1-R5: recommendation_policy_version == "v1" in every record.
//! - AC-V1-R6: Snapshot includes evaluated, actionable, buy, watch, no_trade counts.
//! - AC-V1-R7: Returns 503 when Rec001hStore is not loaded.
//! - AC-V1-R8: `/history` returns all observations (no deduplication).

use std::collections::HashMap;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use coralys_decision::recommendation::{
    RecommendationEngineV1, RecommendationRecordV1, engine::RecommendationAction,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

// ─── Response types ───────────────────────────────────────────────────────────

/// Response envelope for `GET /recommendations/v1/latest` and `/history`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationSnapshotV1 {
    /// Total decisions evaluated (unique tickers for /latest, all for /history).
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
    /// Recommendations, sorted by rank_score descending.
    pub recommendations: Vec<RecommendationRecordV1>,
}

// ─── Shared evaluation helper ─────────────────────────────────────────────────

/// Evaluate a slice of decision records through RecommendationEngineV1 and
/// return a ranked [`RecommendationSnapshotV1`].
fn evaluate_and_rank(
    decisions: Vec<&coralys_decision::DecisionRecord>,
    engine: &RecommendationEngineV1,
) -> Vec<RecommendationRecordV1> {
    let mut recommendations: Vec<RecommendationRecordV1> = decisions
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

    recommendations
}

fn snapshot_from_recs(recommendations: Vec<RecommendationRecordV1>) -> RecommendationSnapshotV1 {
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
    RecommendationSnapshotV1 {
        evaluated,
        actionable,
        buy,
        watch,
        no_trade,
        policy_version: "v1".to_string(),
        recommendations,
    }
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

/// `GET /recommendations/v1/latest`
///
/// Returns **one recommendation per ticker** — the most recent decision for
/// each instrument. Older observations are excluded (use `/history` for those).
///
/// Deduplication: group by ticker → sort by decision_timestamp desc → take newest.
pub async fn get_recommendations_v1_latest(State(state): State<AppState>) -> impl IntoResponse {
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

    // ── Deduplication: keep only the newest decision per ticker ──────────────
    // Group by instrument, then pick the record with the latest decision_timestamp.
    let mut newest_by_ticker: HashMap<String, &coralys_decision::DecisionRecord> = HashMap::new();
    for record in &all_decisions {
        let ticker = record.identity.instrument.clone();
        match newest_by_ticker.get(&ticker) {
            None => {
                newest_by_ticker.insert(ticker, record);
            }
            Some(existing) => {
                if record.identity.decision_timestamp > existing.identity.decision_timestamp {
                    newest_by_ticker.insert(ticker, record);
                }
            }
        }
    }

    // Collect deduplicated decisions (order doesn't matter — engine will rank)
    let deduplicated: Vec<&coralys_decision::DecisionRecord> =
        newest_by_ticker.into_values().collect();

    let engine = RecommendationEngineV1::new(rec001h_store);
    let recommendations = evaluate_and_rank(deduplicated, &engine);
    let snapshot = snapshot_from_recs(recommendations);

    (StatusCode::OK, Json(snapshot)).into_response()
}

/// `GET /recommendations/v1/history`
///
/// Returns **all** recommendation observations — every decision ever evaluated,
/// sorted by rank_score descending. Used by the History/Replay screen.
///
/// No deduplication is applied. Use `/latest` for the operational live view.
pub async fn get_recommendations_v1_history(State(state): State<AppState>) -> impl IntoResponse {
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
    let recommendations = evaluate_and_rank(all_decisions, &engine);
    let snapshot = snapshot_from_recs(recommendations);

    (StatusCode::OK, Json(snapshot)).into_response()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use coralys_decision::{C3_002_POLICY_ARTIFACT_HASH, CORALYS_EXEC_ARTIFACT_HASH};
    use serde_json::{Value, json};

    /// Helper: POST a decision to the test server.
    async fn post_decision(server: &TestServer, decision_id: &str, instrument: &str, ts: &str) {
        let body = json!({
            "decision_id": decision_id,
            "instrument": instrument,
            "decision_timestamp": ts,
            "direction": "LONG",
            "trend": "Bearish",
            "momentum": "Positive",
            "volatility": "present",
            "target_price": null,
            "policy_artifact_hash": C3_002_POLICY_ARTIFACT_HASH,
            "execution_artifact_hash": CORALYS_EXEC_ARTIFACT_HASH,
            "decision_pipeline": "C3-002",
            "data_snapshot_id": format!("snap-{ts}"),
            "certified_timestamp": ts,
            "reference_risk_boundary_price": null,
            "reference_risk_boundary_type": "CORALYS_V0_ATR_TMV",
            "reference_price": 9000.0
        });
        server.post("/decisions").json(&body).await;
    }

    /// AC-V1-R1: 3 decisions for POLYCAB → /latest → exactly 1 POLYCAB record.
    ///
    /// Uses `make_app_with_rec001h()` which loads the real REC-001-H evidence
    /// store so the endpoint returns 200 (not 503).
    #[tokio::test]
    async fn latest_deduplicates_to_one_per_ticker() {
        let server = TestServer::new(crate::test_helpers::make_app_with_rec001h().await);

        // Post 3 decisions for POLYCAB at different timestamps
        post_decision(&server, "polycab-001", "POLYCAB.NS", "2026-08-16T10:15:00Z").await;
        post_decision(&server, "polycab-002", "POLYCAB.NS", "2026-08-17T10:15:00Z").await;
        post_decision(&server, "polycab-003", "POLYCAB.NS", "2026-08-18T10:15:00Z").await;

        let resp = server.get("/recommendations/v1/latest").await;
        // If REC-001-H store is not present in CI, skip rather than fail.
        if resp.status_code() == StatusCode::SERVICE_UNAVAILABLE {
            eprintln!("SKIP: REC-001-H store not available in this environment");
            return;
        }
        resp.assert_status_ok();
        let body: Value = resp.json();

        let recs = body["recommendations"].as_array().unwrap();
        let polycab_recs: Vec<&Value> = recs
            .iter()
            .filter(|r| r["instrument"].as_str().unwrap_or("") == "POLYCAB_NS")
            .collect();

        assert_eq!(
            polycab_recs.len(),
            1,
            "expected exactly 1 POLYCAB record in /latest, got {}",
            polycab_recs.len()
        );
        // The newest decision (polycab-003, 2026-08-18) should be the one returned
        assert_eq!(polycab_recs[0]["decision_id"], "polycab-003");
    }

    /// AC-V1-R8: /history returns all observations (no deduplication).
    ///
    /// Uses `make_app_with_rec001h()` which loads the real REC-001-H evidence
    /// store so the endpoint returns 200 (not 503).
    #[tokio::test]
    async fn history_returns_all_observations() {
        let server = TestServer::new(crate::test_helpers::make_app_with_rec001h().await);

        post_decision(&server, "polycab-h1", "POLYCAB.NS", "2026-08-16T10:15:00Z").await;
        post_decision(&server, "polycab-h2", "POLYCAB.NS", "2026-08-17T10:15:00Z").await;
        post_decision(&server, "polycab-h3", "POLYCAB.NS", "2026-08-18T10:15:00Z").await;

        let resp = server.get("/recommendations/v1/history").await;
        // If REC-001-H store is not present in CI, skip rather than fail.
        if resp.status_code() == StatusCode::SERVICE_UNAVAILABLE {
            eprintln!("SKIP: REC-001-H store not available in this environment");
            return;
        }
        resp.assert_status_ok();
        let body: Value = resp.json();

        let recs = body["recommendations"].as_array().unwrap();
        let polycab_recs: Vec<&Value> = recs
            .iter()
            .filter(|r| r["instrument"].as_str().unwrap_or("") == "POLYCAB_NS")
            .collect();

        assert_eq!(
            polycab_recs.len(),
            3,
            "expected 3 POLYCAB records in /history, got {}",
            polycab_recs.len()
        );
    }
}
