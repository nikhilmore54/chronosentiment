use axum::{routing::{get, post}, Router};

use crate::{
    handlers::strategy_handlers::{compare_strategies_handler, evaluate_strategy_handler, inspect_strategy_handler, test_determinism_handler, run_ga_handler, timeline_handler, events_handler, replay_handler, order_inspection_handler, health_handler, latest_signals_handler, trade_suggestions_handler, replay_suggestions_handler},
    services::evaluation_service::EvaluationService,
};

pub fn strategy_routes() -> Router<EvaluationService> {
    Router::new()
        .route("/health", get(health_handler))
        .route("/evaluate_strategy", post(evaluate_strategy_handler))
        .route("/compare_strategies", post(compare_strategies_handler))
        .route("/inspect_strategy", post(inspect_strategy_handler))
        .route("/test_determinism", post(test_determinism_handler))
        .route("/run_ga", get(run_ga_handler))
        .route("/timeline", get(timeline_handler))
        .route("/events", get(events_handler))
        .route("/replay/:id", get(replay_handler))
        .route("/order/:id", get(order_inspection_handler))
        .route("/ga/global-ranking", get(crate::handlers::strategy_handlers::get_global_ranking_handler))
        .route("/signals/latest", get(latest_signals_handler))
        .route("/signals/trade-suggestions", get(trade_suggestions_handler))
        .route("/signals/replay-suggestions", get(replay_suggestions_handler))
}
