use chrono::{TimeZone, Utc};
use serde_json::json;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use chronosentiment_adapter::reasoning::strategy::Horizon;
use chronosentiment_adapter::research::dataset::{ArtifactPopulation, DateRange, ResearchDataset};
use chronosentiment_adapter::research::experiment::{
    ExperimentMeasurements, ResearchExperiment, ResearchRun,
};
use chronosentiment_adapter::research::laboratory::{ExperimentRepository, ResearchLaboratory};

// A simple mock repository that stores runs in memory.
#[derive(Clone)]
struct MockRepository {
    runs: Arc<Mutex<Vec<ResearchRun>>>,
}

impl MockRepository {
    fn new() -> Self {
        Self {
            runs: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

use async_trait::async_trait;

#[async_trait]
impl ExperimentRepository for MockRepository {
    async fn save_run(&self, run: ResearchRun) {
        self.runs.lock().unwrap().push(run);
    }

    async fn get_runs(&self, experiment_id: Uuid) -> Vec<ResearchRun> {
        let runs = self.runs.lock().unwrap();
        runs.iter()
            .filter(|r| r.experiment_id == experiment_id)
            .cloned()
            .collect()
    }
}

// A mock experiment that does no real work but returns basic measurements.
struct MockExperiment {
    id: Uuid,
}

impl MockExperiment {
    fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }
}

#[async_trait]
impl ResearchExperiment for MockExperiment {
    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "Mock Experiment"
    }

    fn description(&self) -> &str {
        "Used for testing ResearchLaboratory infrastructure."
    }

    async fn execute(
        &self,
        _dataset: &ResearchDataset,
    ) -> Result<ExperimentMeasurements, Box<dyn std::error::Error + Send + Sync>> {
        Ok(ExperimentMeasurements {
            metadata: json!({"status": "completed"}),
            findings: vec![json!({"insight": "Test passed"})],
        })
    }
}

fn create_test_dataset() -> ResearchDataset {
    ResearchDataset::new(
        "Lab Test Dataset".to_string(),
        "v1.0".to_string(),
        json!({"index": "Nifty50"}),
        DateRange {
            start: Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap(),
        },
        vec![Horizon::Swing],
        json!({}),
        json!({}),
        ArtifactPopulation {
            artifact_types: vec!["Assessment".to_string()],
            population_rules: json!({}),
        },
    )
}

#[tokio::test]
async fn test_laboratory_executes_and_persists_run() {
    let repo = MockRepository::new();
    let mut lab = ResearchLaboratory::new(Box::new(repo.clone()));

    let experiment = MockExperiment::new();
    let experiment_id = experiment.id();

    lab.register_experiment(Box::new(experiment));

    let dataset = create_test_dataset();
    let dataset_hash = dataset.content_hash.clone();

    let run = lab
        .execute_experiment(experiment_id, &dataset)
        .await
        .expect("Execution should succeed")
        .expect("Experiment should exist and execute");

    // Verify the run contains the correct dataset hash
    assert_eq!(run.dataset_hash, dataset_hash);
    assert_eq!(run.experiment_id, experiment_id);
    assert_eq!(run.measurements.findings.len(), 1);

    // Verify it was persisted to the repository
    let stored_runs = repo.get_runs(experiment_id).await;
    assert_eq!(stored_runs.len(), 1);
    assert_eq!(stored_runs[0].run_id, run.run_id);
}
