use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use chronosentiment_adapter::metrics::concepts::Concept;
use chronosentiment_adapter::reasoning::assessment::{
    AssessmentEngine, AssessmentProfile, Direction, DomainAssessment, Maturity, Persistence,
    Strength,
};
use chronosentiment_adapter::repository::knowledge::{
    ArtifactLineage, ArtifactMetadata, ArtifactRepository, ArtifactType,
};
use chronosentiment_adapter::repository::postgres_knowledge::PostgresKnowledgeRepository;
use coralys_moga::runtime::optimization::metric::MetricReport;

#[sqlx::test]
async fn test_assessment_persistence_and_immutability(
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Run migrations to create the tables in the isolated test DB
    sqlx::migrate!("./migrations").run(&pool).await?;

    let repo = PostgresKnowledgeRepository::new(pool.clone());
    let engine = AssessmentEngine;

    // 1. Setup mock metric report and concepts
    let metrics = MetricReport::default();
    let concepts = vec![Concept::Trend];
    let eval_time = Utc::now();
    let instrument_id = Uuid::new_v4();

    // Insert mock instrument into DB to satisfy foreign key constraints
    sqlx::query(
        "INSERT INTO instruments (id, exchange, display_symbol) VALUES ($1, 'TEST', 'MOCK_INST')",
    )
    .bind(instrument_id)
    .execute(&pool)
    .await?;

    // 2. Setup Reproducibility Metadata
    let metadata_a = ArtifactMetadata {
        artifact_id: Uuid::new_v4(),
        artifact_schema_version: "1.0".to_string(),
        artifact_type: ArtifactType::Assessment,
        created_at: Utc::now(),
        evaluation_timestamp: eval_time,
        engine_versions: json!({"assessment_engine": "1.0"}),
        lineage: ArtifactLineage {
            produced_by: "TimeMachine".to_string(),
            consumed_artifacts: vec![],
            parent_artifacts: vec![],
        },
        replay_context_hash: "hash_xyz".to_string(),
        knowledge_lake_version: "lake_1.0".to_string(),
        content_hash: "".to_string(), // will be filled by assess_with_metadata
    };

    // 3. Generate Assessment
    let profile_a =
        engine.assess_with_metadata(&metrics, &concepts, metadata_a.clone(), Some(instrument_id));

    // Store Assessment
    repo.store(&profile_a).await?;

    // Verify it exists
    let fetched: AssessmentProfile = repo
        .get(profile_a.metadata.artifact_id)
        .await?
        .expect("Assessment should be persisted");
    assert_eq!(fetched.metadata.artifact_id, profile_a.metadata.artifact_id);
    assert_eq!(
        fetched.metadata.content_hash,
        profile_a.metadata.content_hash
    );

    // 4. Verify Append-Only (Immutability)
    // Attempting to store the exact same artifact_id again should fail (unique constraint)
    let store_result = repo.store(&profile_a).await;
    assert!(
        store_result.is_err(),
        "Repository must reject overwriting an existing artifact"
    );

    // 5. Verify Deterministic Hashing
    let metadata_b = ArtifactMetadata {
        artifact_id: Uuid::new_v4(), // Different identity!
        artifact_schema_version: "1.0".to_string(),
        artifact_type: ArtifactType::Assessment,
        created_at: Utc::now(),          // Different creation time
        evaluation_timestamp: eval_time, // Same eval time
        engine_versions: json!({"assessment_engine": "1.0"}), // Same engine
        lineage: ArtifactLineage {
            produced_by: "TimeMachine".to_string(),
            consumed_artifacts: vec![],
            parent_artifacts: vec![],
        }, // Same lineage
        replay_context_hash: "hash_xyz".to_string(), // Same reproducible state
        knowledge_lake_version: "lake_1.0".to_string(),
        content_hash: "".to_string(),
    };

    let profile_b =
        engine.assess_with_metadata(&metrics, &concepts, metadata_b, Some(instrument_id));

    // They are physically different objects
    assert_ne!(
        profile_a.metadata.artifact_id,
        profile_b.metadata.artifact_id
    );

    // But they must have identical content hashes because they represent the identical reproducible reasoning!
    assert_eq!(
        profile_a.metadata.content_hash, profile_b.metadata.content_hash,
        "Identical reasoning runs with different artifact IDs must produce the same content_hash"
    );

    Ok(())
}
