use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

use crate::research::dataset::ResearchDataset;
use crate::research::experiment::{ResearchExperiment, ResearchRun};

use async_trait::async_trait;

#[async_trait]
pub trait ExperimentRepository: Send + Sync {
    async fn save_run(&self, run: ResearchRun);
    async fn get_runs(&self, experiment_id: Uuid) -> Vec<ResearchRun>;
}

pub struct ResearchLaboratory {
    experiments: HashMap<Uuid, Box<dyn ResearchExperiment>>,
    repository: Box<dyn ExperimentRepository>,
}

impl ResearchLaboratory {
    pub fn new(repository: Box<dyn ExperimentRepository>) -> Self {
        Self {
            experiments: HashMap::new(),
            repository,
        }
    }

    pub fn register_experiment(&mut self, experiment: Box<dyn ResearchExperiment>) {
        self.experiments.insert(experiment.id(), experiment);
    }

    pub async fn execute_experiment(
        &self,
        experiment_id: Uuid,
        dataset: &ResearchDataset,
    ) -> Result<Option<ResearchRun>, Box<dyn std::error::Error + Send + Sync>> {
        let exp = match self.experiments.get(&experiment_id) {
            Some(e) => e,
            None => return Ok(None),
        };

        let measurements = exp.execute(dataset).await?;

        let run = ResearchRun {
            run_id: Uuid::new_v4(),
            experiment_id: exp.id(),
            dataset_hash: dataset.content_hash.clone(),
            execution_time: Utc::now(),
            measurements,
        };

        self.repository.save_run(run.clone()).await;

        Ok(Some(run))
    }
}
