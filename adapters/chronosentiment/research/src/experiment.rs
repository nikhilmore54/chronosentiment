use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::research::dataset::ResearchDataset;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentMeasurements {
    pub metadata: Value,
    pub findings: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchRun {
    pub run_id: Uuid,
    pub experiment_id: Uuid,
    pub dataset_hash: String,
    pub execution_time: DateTime<Utc>,
    pub measurements: ExperimentMeasurements,
}

use async_trait::async_trait;

#[async_trait]
pub trait ResearchExperiment: Send + Sync {
    fn id(&self) -> Uuid;
    fn name(&self) -> &str;
    fn description(&self) -> &str;

    /// Execute the experiment against a defined population.
    /// This keeps the laboratory generic; the experiment itself handles the specific research domain (e.g. Phase G).
    async fn execute(
        &self,
        dataset: &ResearchDataset,
    ) -> Result<ExperimentMeasurements, Box<dyn std::error::Error + Send + Sync>>;
}
