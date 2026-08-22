import sys

with open("services/ultracrew_server/src/main.rs", "r") as f:
    content = f.read()

# Replace ConstraintEngine
content = content.replace("let constraint_engine = ultracrew::constraint_engine::ConstraintEngine::new(context);", "let constraint_engine = ultracrew::constraint_engine::InrcConstraintEvaluator::new(context);")

# Replace Scenario in ScheduleAnalysisRequest
content = content.replace("    scenario: Option<Scenario>,", "    scenario: Option<ultracrew::public_contracts::InrcScenario>,")

# Replace pairings_handler
import re
pattern = re.compile(r"async fn pairings_handler\([\s\S]*?^}\n", re.MULTILINE)
content = pattern.sub('''async fn pairings_handler(
    axum::Json(_req): axum::Json<ScheduleAnalysisRequest>,
) -> Result<axum::Json<PairingsResponse>, (axum::http::StatusCode, String)> {
    Err((axum::http::StatusCode::UNPROCESSABLE_ENTITY, "DOMAIN_CONCEPT_NOT_SUPPORTED\\ndomain=inrc\\nconcept=pairing".to_string()))
}
''', content)

# Replace duties_handler
pattern2 = re.compile(r"async fn duties_handler\([\s\S]*?^}\n", re.MULTILINE)
content = pattern2.sub('''async fn duties_handler(
    axum::Json(_req): axum::Json<ScheduleAnalysisRequest>,
) -> Result<axum::Json<DutiesResponse>, (axum::http::StatusCode, String)> {
    Err((axum::http::StatusCode::UNPROCESSABLE_ENTITY, "DOMAIN_CONCEPT_NOT_SUPPORTED\\ndomain=inrc\\nconcept=duty".to_string()))
}
''', content)

with open("services/ultracrew_server/src/main.rs", "w") as f:
    f.write(content)

print("Done")
