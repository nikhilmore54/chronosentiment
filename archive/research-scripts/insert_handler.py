import sys

def main():
    file_path = "services/ultracrew_server/src/main.rs"
    with open(file_path, "r") as f:
        content = f.read()

    handler_code = """
async fn disruption_recovery_handler(
    State(_state): State<Arc<Mutex<AppState>>>,
    Json(req): Json<ultracrew::public_contracts::DisruptionRecoveryRequest>,
) -> Result<Json<ultracrew::public_contracts::DisruptionRecoveryResult>, (axum::http::StatusCode, String)> {
    let result = ultracrew::recovery_engine::RecoveryEngine::evaluate_recovery(&req);
    Ok(Json(result))
}

"""
    
    # insert before async fn validate_handler
    new_content = content.replace("async fn validate_handler", handler_code + "async fn validate_handler")
    
    # also add to the router
    router_line = ".route(\"/api/reschedule\", post(reschedule_handler))"
    new_router_line = router_line + "\n            .route(\"/api/disruption_recovery\", post(disruption_recovery_handler))"
    new_content = new_content.replace(router_line, new_router_line)
    
    with open(file_path, "w") as f:
        f.write(new_content)

if __name__ == "__main__":
    main()
