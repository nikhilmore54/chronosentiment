use std::collections::HashMap;

pub trait InstanceFeatures: Send + Sync + std::fmt::Debug {}

pub trait DifficultyAssessment: Send + Sync + std::fmt::Debug {
    fn difficulty_label(&self) -> &'static str;
}

pub trait ConfigurationPolicy: Send + Sync + std::fmt::Debug {}

pub trait InstanceAnalyzer<I>: Send + Sync {
    type Features: InstanceFeatures;
    type Difficulty: DifficultyAssessment;
    type Policy: ConfigurationPolicy;

    fn analyze(
        &self,
        instance: &I,
    ) -> Result<(Self::Features, Self::Difficulty, Self::Policy), String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EvidenceLevel {
    Low,
    Medium,
    High,
    Verified,
}

impl std::fmt::Display for EvidenceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkConfiguration {
    pub population_size: usize,
    pub generation_limit: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub local_search_intensity: usize,
    pub repair_intensity: usize,
    pub diversity_preservation: bool,
    pub route_preserving_crossover: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkExecutionMetrics {
    pub final_objective: f64,
    pub percentage_gap: f64,
    pub feasibility: bool,
    pub runtime_sec: f64,
    pub best_generation: usize,
    pub convergence_speed: f64, // Improvement rate per second
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkTelemetry {
    pub total_invocations: usize,
    pub successful_repairs: usize,
    pub failed_repairs: usize,
    pub violations_encountered: HashMap<String, usize>,
    pub heuristic_attempts: HashMap<String, usize>,
    pub heuristic_successes: HashMap<String, usize>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkRun {
    pub instance_name: String,
    pub family: String,
    pub customer_count: usize,
    pub difficulty_label: String,
    pub configuration: BenchmarkConfiguration,
    pub metrics: BenchmarkExecutionMetrics,
    pub telemetry: BenchmarkTelemetry,
    pub evidence_level: EvidenceLevel,
    pub rationales: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkComparison {
    pub name: String,
    pub base_metrics: BenchmarkExecutionMetrics,
    pub comp_metrics: BenchmarkExecutionMetrics,
    pub objective_improvement_pct: f64,
    pub runtime_improvement_pct: f64,
    pub feasibility_improved: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkCampaign {
    pub campaign_name: String,
    pub runs: Vec<BenchmarkRun>,
    pub comparisons: Vec<BenchmarkComparison>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SimilarityScore {
    pub score: f64, // 0.0 to 1.0 (1.0 = identical)
    pub metrics: HashMap<String, f64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SimilarityGroup {
    pub group_name: String,
    pub member_instances: Vec<String>,
}

pub struct SimilarityAnalyzer;

impl SimilarityAnalyzer {
    pub fn compute_similarity(
        f1: &HashMap<String, f64>,
        f2: &HashMap<String, f64>,
    ) -> SimilarityScore {
        let mut score_sum = 0.0;
        let mut count = 0;
        let mut metrics = HashMap::new();

        for (k, &v1) in f1 {
            if let Some(&v2) = f2.get(k) {
                let diff = (v1 - v2).abs();
                let denom = v1.abs().max(v2.abs()).max(1e-9);
                let similarity = 1.0 - (diff / denom).min(1.0);
                metrics.insert(k.clone(), similarity);
                score_sum += similarity;
                count += 1;
            }
        }

        let score = if count > 0 {
            score_sum / count as f64
        } else {
            0.0
        };
        SimilarityScore { score, metrics }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ObservatoryDatabase {
    pub runs: Vec<BenchmarkRun>,
    pub features_cache: HashMap<String, HashMap<String, f64>>,
}

impl ObservatoryDatabase {
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        if !std::path::Path::new(path).exists() {
            return Ok(Self::default());
        }
        let data = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&data).map_err(|e| e.to_string())
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let dir = std::path::Path::new(path).parent();
        if let Some(parent) = dir {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let data = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, data).map_err(|e| e.to_string())
    }

    pub fn record_run(&mut self, run: BenchmarkRun, features: HashMap<String, f64>) {
        self.features_cache
            .insert(run.instance_name.clone(), features);
        self.runs.push(run);
    }

    pub fn best_config_for(
        &self,
        target_features: &HashMap<String, f64>,
        similarity_threshold: f64,
    ) -> Option<(BenchmarkConfiguration, EvidenceLevel, Vec<String>)> {
        let mut best_run: Option<&BenchmarkRun> = None;
        let mut highest_similarity = 0.0;

        for run in &self.runs {
            if let Some(cached_feats) = self.features_cache.get(&run.instance_name) {
                let score = SimilarityAnalyzer::compute_similarity(target_features, cached_feats);
                if score.score >= similarity_threshold && score.score > highest_similarity {
                    highest_similarity = score.score;
                    best_run = Some(run);
                }
            }
        }

        best_run.map(|run| {
            let rationale = vec![
                format!(
                    "Found historically similar instance '{}' (Similarity: {:.2}%)",
                    run.instance_name,
                    highest_similarity * 100.0
                ),
                format!(
                    "Historical run yielded objective value {:.2} in {:.2}s (Feasible: {})",
                    run.metrics.final_objective, run.metrics.runtime_sec, run.metrics.feasibility
                ),
            ];
            (
                run.configuration.clone(),
                EvidenceLevel::Verified,
                rationale,
            )
        })
    }
}
