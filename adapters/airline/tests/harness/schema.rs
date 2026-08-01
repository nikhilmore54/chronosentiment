//! WP2 + WP4 — Standardized Experiment Schema and Generation Metrics
//!
//! Every experiment in Section 3 uses these types. The schema is intentionally
//! minimal: only fields that are needed for Batch 1 (landscape analysis,
//! multi-seed validation, population scaling, generation scaling).
//!
//! Additional fields (operator acceptance rates, diversity metrics, etc.) are
//! added when the experiments that require them are implemented.

use serde::{Deserialize, Serialize};

// ── Experiment configuration (WP2) ────────────────────────────────────────────

/// Initialization strategy for the evolutionary algorithm.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InitStrategy {
    RoundRobin,
    Random,
    GreedySeeded,
}

impl InitStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            InitStrategy::RoundRobin => "round_robin",
            InitStrategy::Random => "random",
            InitStrategy::GreedySeeded => "greedy_seeded",
        }
    }
}

/// Operator configuration for the evolutionary algorithm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorConfig {
    /// Crossover probability (0.0–1.0).
    pub crossover_prob: f64,
    /// Mutation probability per gene (0.0–1.0; use 0.0 for "1/P" default).
    /// If 0.0, the harness uses 1.0 / n_pairings at runtime.
    pub mutation_prob_per_gene: f64,
    /// Tournament selection size.
    pub tournament_k: usize,
}

impl Default for OperatorConfig {
    fn default() -> Self {
        Self {
            crossover_prob: 0.80,
            mutation_prob_per_gene: 0.0, // 0.0 = use 1/P
            tournament_k: 3,
        }
    }
}

/// Full configuration for one experiment run.
///
/// This is the canonical source of truth for what was run. It is serialized
/// to `experiment.json` alongside the results so the run is reproducible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    /// Human-readable experiment identifier (e.g. "exp0_landscape_instance3").
    pub experiment_id: String,
    /// Section 3 experiment number (0–10).
    pub experiment_number: u8,
    /// Human-readable name (e.g. "Landscape Analysis").
    pub experiment_name: String,
    /// Benchmark suite (e.g. "gerad-g2014-22").
    pub benchmark: String,
    /// Instance identifier (e.g. "instance3").
    pub instance: String,
    /// Random seed for reproducibility.
    pub random_seed: u64,
    /// Population size.
    pub population: usize,
    /// Number of generations.
    pub generations: usize,
    /// Initialization strategy.
    pub init_strategy: InitStrategy,
    /// Operator configuration.
    pub operators: OperatorConfig,
    /// Layover rest threshold in hours.
    pub layover_rest_hours: f64,
    /// Number of independent repeat runs (for multi-seed experiments).
    pub n_repeats: usize,
    /// ISO 8601 timestamp when the experiment was started.
    pub timestamp_utc: String,
    /// Reproducibility metadata (filled in at runtime).
    pub reproducibility: Option<crate::harness::reproducibility::ReproducibilityInfo>,
}

impl ExperimentConfig {
    /// Create a config with sensible defaults matching the Section 2.17 baseline.
    pub fn baseline(instance: &str) -> Self {
        Self {
            experiment_id: format!("baseline_{instance}"),
            experiment_number: 0,
            experiment_name: "GERAD Coralys v1.0 Baseline".to_string(),
            benchmark: "gerad-g2014-22".to_string(),
            instance: instance.to_string(),
            random_seed: 42,
            population: 50,
            generations: 200,
            init_strategy: InitStrategy::RoundRobin,
            operators: OperatorConfig::default(),
            layover_rest_hours: 8.0,
            n_repeats: 1,
            timestamp_utc: String::new(), // filled at runtime
            reproducibility: None,
        }
    }
}

// ── Per-generation metrics (WP4) ──────────────────────────────────────────────

/// One record per generation, emitted by the evolutionary loop.
///
/// These become rows in `generations.csv`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRecord {
    /// Generation index (0-based).
    pub generation: usize,
    /// Best fitness in the population (lower is better).
    pub best_fitness: f64,
    /// Mean fitness across the population.
    pub mean_fitness: f64,
    /// Median fitness across the population.
    pub median_fitness: f64,
    /// Worst fitness in the population.
    pub worst_fitness: f64,
    /// Fraction of population with finite (feasible) fitness.
    pub feasible_fraction: f64,
    /// Number of repair operations applied this generation.
    pub repair_count: usize,
    /// Wall-clock milliseconds elapsed since experiment start.
    pub elapsed_ms: u128,
}

impl GenerationRecord {
    /// Compute a GenerationRecord from a fitness vector.
    pub fn from_fitnesses(
        generation: usize,
        fitnesses: &[f64],
        repair_count: usize,
        elapsed_ms: u128,
    ) -> Self {
        let n = fitnesses.len();
        let finite: Vec<f64> = fitnesses.iter().cloned().filter(|f| f.is_finite()).collect();
        let feasible_fraction = finite.len() as f64 / n as f64;

        let best_fitness = finite.iter().cloned().fold(f64::INFINITY, f64::min);
        let worst_fitness = finite.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let mean_fitness = if finite.is_empty() {
            f64::INFINITY
        } else {
            finite.iter().sum::<f64>() / finite.len() as f64
        };
        let median_fitness = if finite.is_empty() {
            f64::INFINITY
        } else {
            let mut sorted = finite.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mid = sorted.len() / 2;
            if sorted.len() % 2 == 0 {
                (sorted[mid - 1] + sorted[mid]) / 2.0
            } else {
                sorted[mid]
            }
        };

        Self {
            generation,
            best_fitness,
            mean_fitness,
            median_fitness,
            worst_fitness,
            feasible_fraction,
            repair_count,
            elapsed_ms,
        }
    }
}

// ── Per-run summary (WP3) ─────────────────────────────────────────────────────

/// Summary statistics for one complete experiment run.
///
/// Serialized to `summary.json` and appended as a row in `metrics.csv`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    /// Experiment configuration.
    pub config: ExperimentConfig,
    /// Best fitness achieved across all generations.
    pub best_fitness: f64,
    /// Generation at which best fitness was first achieved.
    pub best_generation: usize,
    /// Final generation best fitness (may differ from best_fitness if regression).
    pub final_fitness: f64,
    /// Total wall-clock milliseconds for the run.
    pub total_ms: u128,
    /// Number of pairings in the instance.
    pub n_pairings: usize,
    /// Number of rotations (crew members) in the instance.
    pub n_rotations: usize,
    /// Greedy baseline score for comparison (None if not run).
    pub greedy_baseline: Option<f64>,
    /// Greedy baseline runtime in milliseconds (None if not run).
    pub greedy_baseline_ms: Option<u128>,
    /// Per-generation records (populated when generation logging is enabled).
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub generations: Vec<GenerationRecord>,
}

// ── Multi-run aggregate (for multi-seed experiments) ─────────────────────────

/// Aggregate statistics across multiple independent runs of the same config.
///
/// Used by Experiment 4 (multi-seed validation).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiRunAggregate {
    /// Experiment configuration (shared across all runs; seed varies).
    pub config: ExperimentConfig,
    /// Number of runs completed.
    pub n_runs: usize,
    /// Seeds used.
    pub seeds: Vec<u64>,
    /// Best fitness per run.
    pub best_fitnesses: Vec<f64>,
    /// Mean of best fitnesses.
    pub mean_best: f64,
    /// Median of best fitnesses.
    pub median_best: f64,
    /// Standard deviation of best fitnesses.
    pub std_best: f64,
    /// Minimum best fitness (best run).
    pub min_best: f64,
    /// Maximum best fitness (worst run).
    pub max_best: f64,
    /// 95% confidence interval half-width (1.96 * std / sqrt(n)).
    pub ci95_half_width: f64,
}

impl MultiRunAggregate {
    /// Compute aggregate from a list of (seed, best_fitness) pairs.
    pub fn from_runs(config: ExperimentConfig, runs: &[(u64, f64)]) -> Self {
        let n = runs.len();
        let seeds: Vec<u64> = runs.iter().map(|(s, _)| *s).collect();
        let best_fitnesses: Vec<f64> = runs.iter().map(|(_, f)| *f).collect();

        let mean_best = best_fitnesses.iter().sum::<f64>() / n as f64;
        let variance = best_fitnesses.iter()
            .map(|f| (f - mean_best).powi(2))
            .sum::<f64>() / (n as f64 - 1.0).max(1.0);
        let std_best = variance.sqrt();

        let mut sorted = best_fitnesses.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = sorted.len() / 2;
        let median_best = if sorted.len() % 2 == 0 {
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[mid]
        };

        let min_best = sorted.first().cloned().unwrap_or(f64::INFINITY);
        let max_best = sorted.last().cloned().unwrap_or(f64::INFINITY);
        let ci95_half_width = 1.96 * std_best / (n as f64).sqrt();

        Self {
            config,
            n_runs: n,
            seeds,
            best_fitnesses,
            mean_best,
            median_best,
            std_best,
            min_best,
            max_best,
            ci95_half_width,
        }
    }
}

// ── Experiment result (top-level output) ─────────────────────────────────────

/// Top-level result written to `experiment.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResult {
    /// The run summary (single-seed) or None for multi-seed experiments.
    pub run: Option<RunSummary>,
    /// Multi-run aggregate (multi-seed experiments only).
    pub aggregate: Option<MultiRunAggregate>,
}