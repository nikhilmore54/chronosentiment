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

    fn analyze(&self, instance: &I) -> Result<(Self::Features, Self::Difficulty, Self::Policy), String>;
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
        
        let score = if count > 0 { score_sum / count as f64 } else { 0.0 };
        SimilarityScore { score, metrics }
    }
}
