use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::reasoning::strategy::Horizon;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactPopulation {
    pub artifact_types: Vec<String>,
    pub population_rules: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchDataset {
    pub dataset_id: Uuid,
    pub name: String,
    pub knowledge_lake_version: String,

    pub universe: Value,
    pub date_range: DateRange,
    pub horizons: Vec<Horizon>,

    pub inclusion_rules: Value,
    pub exclusion_rules: Value,

    pub artifact_population: ArtifactPopulation,

    pub content_hash: String,
}

impl ResearchDataset {
    pub fn new(
        name: String,
        knowledge_lake_version: String,
        universe: Value,
        date_range: DateRange,
        horizons: Vec<Horizon>,
        inclusion_rules: Value,
        exclusion_rules: Value,
        artifact_population: ArtifactPopulation,
    ) -> Self {
        let mut dataset = Self {
            dataset_id: Uuid::new_v4(),
            name,
            knowledge_lake_version,
            universe,
            date_range,
            horizons,
            inclusion_rules,
            exclusion_rules,
            artifact_population,
            content_hash: String::new(),
        };
        
        dataset.content_hash = dataset.calculate_hash();
        dataset
    }
    
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        
        // E1 & E3 - Hashing specific fields (exclude dataset_id and name for identity independence)
        
        // 1. Knowledge Lake Version
        hasher.update(self.knowledge_lake_version.as_bytes());
        
        // 2. Universe (Serialize to canonical JSON string)
        let universe_str = serde_json::to_string(&self.universe).unwrap_or_default();
        hasher.update(universe_str.as_bytes());
        
        // 3. Date Range
        hasher.update(self.date_range.start.timestamp().to_be_bytes());
        hasher.update(self.date_range.end.timestamp().to_be_bytes());
        
        // 4. Horizons (Canonical ordering)
        let mut sorted_horizons = self.horizons.clone();
        sorted_horizons.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
        for horizon in &sorted_horizons {
            hasher.update(format!("{:?}", horizon).as_bytes());
        }
        
        // 5. Inclusion / Exclusion Rules
        let inc_str = serde_json::to_string(&self.inclusion_rules).unwrap_or_default();
        hasher.update(inc_str.as_bytes());
        
        let exc_str = serde_json::to_string(&self.exclusion_rules).unwrap_or_default();
        hasher.update(exc_str.as_bytes());
        
        // 6. Artifact Population
        // Sort artifact types for canonical ordering
        let mut sorted_types = self.artifact_population.artifact_types.clone();
        sorted_types.sort();
        
        for t in &sorted_types {
            hasher.update(t.as_bytes());
        }
        
        let pop_rules_str = serde_json::to_string(&self.artifact_population.population_rules).unwrap_or_default();
        hasher.update(pop_rules_str.as_bytes());
        
        format!("{:x}", hasher.finalize())
    }
}
