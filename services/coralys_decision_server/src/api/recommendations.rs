//! MVP-009 — Recommendations API: `GET /recommendations/latest`
//!
//! Runs the Recommendation Engine against all certified decisions in the ledger
//! and returns a ranked [`RecommendationSnapshot`].
//!
//! The engine is stateless. The [`EvidenceStore`] is loaded once at server
//! startup from the frozen HDV-001 outcomes file and shared via [`AppState`].
//!
//! **Acceptance criteria:**
//! - AC-R1: Returns 200 + ranked recommendations for all decisions in the ledger.
//! - AC-R2: Recommendations are sorted by rank_score descending.
//! - AC-R3: Each recommendation includes evidence, action, geometry, and score components.
//! - AC-R4: No confidence, probability, or expected-return fields are added.
//! - AC-R5: The recommendation_policy_version is present in every record.
//! - AC-R6: The snapshot includes evaluated count and actionable count.

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use coralys_decision::recommendation::{
    RecommendationEngine, RecommendationRecord, engine::RecommendationAction,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

// ─── Response types ───────────────────────────────────────────────────────────

/// Response envelope for `GET /recommendations/latest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationSnapshot {
    /// Total decisions evaluated.
    pub evaluated: usize,
    /// Decisions with action BUY or WATCH.
    pub actionable: usize,
    /// All recommendations, sorted by rank_score descending.
    pub recommendations: Vec<RecommendationRecord>,
}

// ─── Handler ──────────────────────────────────────────────────────────────────

/// `GET /recommendations/latest`
///
/// Evaluates all certified decisions in the ledger through the Recommendation
/// Engine and returns a ranked snapshot.
pub async fn get_recommendations_latest(State(state): State<AppState>) -> impl IntoResponse {
    let evidence_store = match &state.evidence_store {
        Some(store) => store,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "Evidence store not loaded. Check HDV-001 outcomes file path."
                })),
            )
                .into_response();
        }
    };

    let ledger = state.ledger.read().await;
    let all_decisions = ledger.all_decisions();

    let engine = RecommendationEngine::new(evidence_store);

    let mut recommendations: Vec<RecommendationRecord> = all_decisions
        .iter()
        .map(|record| {
            let dir = match &record.decision.direction {
                coralys_decision::Direction::Long => "LONG",
                coralys_decision::Direction::Short => "SHORT",
                coralys_decision::Direction::NoTrade => "NO_TRADE",
            };
            engine.evaluate(
                &record.identity.decision_id,
                &record.identity.instrument,
                dir,
                &record.decision.trend,
                &record.decision.momentum,
                record.decision.reference_price,
                record.decision.atr_14,
                record.decision.effective_session.as_deref(),
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
    let actionable = recommendations
        .iter()
        .filter(|r| {
            r.action == RecommendationAction::Buy || r.action == RecommendationAction::Watch
        })
        .count();

    let snapshot = RecommendationSnapshot {
        evaluated,
        actionable,
        recommendations,
    };

    (StatusCode::OK, Json(snapshot)).into_response()
}
