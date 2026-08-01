//! WP3 — Machine-readable Results Persistence
//!
//! Writes experiment outputs to a structured directory:
//!
//! ```text
//! results/
//!   <experiment_id>/
//!     experiment.json        — full ExperimentConfig
//!     reproducibility.json   — ReproducibilityInfo
//!     instance<N>/
//!       run<M>/
//!         summary.json       — RunSummary (without generations)
//!         generations.csv    — one row per GenerationRecord
//!         best_genome.json   — best genome found
//!     metrics.csv            — one row per run (appended)
//!     experiment_result.json — ExperimentResult (final)
//! ```
//!
//! All writes are append-safe: `metrics.csv` is appended so partial runs
//! are not lost if the process is interrupted.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::harness::schema::{ExperimentConfig, ExperimentResult, GenerationRecord, RunSummary};

/// Manages result persistence for one experiment.
pub struct ResultPersistence {
    /// Root output directory for this experiment.
    pub root: PathBuf,
}

impl ResultPersistence {
    /// Create the output directory structure and write `experiment.json`.
    pub fn initialise(config: &ExperimentConfig, results_base: &Path) -> std::io::Result<Self> {
        let root = results_base.join(&config.experiment_id);
        fs::create_dir_all(&root)?;

        // Write experiment.json
        let config_json = serde_json::to_string_pretty(config)
            .expect("ExperimentConfig must be serializable");
        fs::write(root.join("experiment.json"), config_json)?;

        // Write metrics.csv header if file does not exist
        let metrics_path = root.join("metrics.csv");
        if !metrics_path.exists() {
            let mut f = fs::File::create(&metrics_path)?;
            writeln!(
                f,
                "instance,run_index,seed,n_pairings,n_rotations,\
                 best_fitness,best_generation,final_fitness,total_ms,\
                 greedy_baseline,greedy_baseline_ms"
            )?;
        }

        Ok(Self { root })
    }

    /// Create the directory for one run and return its path.
    pub fn prepare_run_dir(&self, instance: &str, run_index: usize) -> std::io::Result<PathBuf> {
        let dir = self
            .root
            .join(format!("instance_{instance}"))
            .join(format!("run_{run_index:03}"));
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    /// Write `generations.csv` for one run.
    pub fn write_generations(
        run_dir: &Path,
        records: &[GenerationRecord],
    ) -> std::io::Result<()> {
        let path = run_dir.join("generations.csv");
        let mut f = fs::File::create(path)?;
        writeln!(
            f,
            "generation,best_fitness,mean_fitness,median_fitness,\
             worst_fitness,feasible_fraction,repair_count,elapsed_ms"
        )?;
        for r in records {
            writeln!(
                f,
                "{},{:.6},{:.6},{:.6},{:.6},{:.4},{},{}",
                r.generation,
                r.best_fitness,
                r.mean_fitness,
                r.median_fitness,
                r.worst_fitness,
                r.feasible_fraction,
                r.repair_count,
                r.elapsed_ms,
            )?;
        }
        Ok(())
    }

    /// Write `summary.json` for one run (without generation records to keep it compact).
    pub fn write_run_summary(run_dir: &Path, summary: &RunSummary) -> std::io::Result<()> {
        let mut compact = summary.clone();
        compact.generations.clear();
        let json = serde_json::to_string_pretty(&compact)
            .expect("RunSummary must be serializable");
        fs::write(run_dir.join("summary.json"), json)?;
        Ok(())
    }

    /// Append one row to `metrics.csv`.
    pub fn append_metrics_row(
        &self,
        instance: &str,
        run_index: usize,
        summary: &RunSummary,
    ) -> std::io::Result<()> {
        let path = self.root.join("metrics.csv");
        let mut f = fs::OpenOptions::new().append(true).open(path)?;
        writeln!(
            f,
            "{},{},{},{},{},{:.6},{},{:.6},{},{},{}",
            instance,
            run_index,
            summary.config.random_seed,
            summary.n_pairings,
            summary.n_rotations,
            summary.best_fitness,
            summary.best_generation,
            summary.final_fitness,
            summary.total_ms,
            summary
                .greedy_baseline
                .map(|v| format!("{v:.6}"))
                .unwrap_or_default(),
            summary
                .greedy_baseline_ms
                .map(|v| v.to_string())
                .unwrap_or_default(),
        )?;
        Ok(())
    }

    /// Write the best genome to `best_genome.json` in the run directory.
    pub fn write_best_genome(run_dir: &Path, genome: &[usize]) -> std::io::Result<()> {
        let json = serde_json::to_string(genome).expect("genome must be serializable");
        fs::write(run_dir.join("best_genome.json"), json)?;
        Ok(())
    }

    /// Write the final `experiment_result.json`.
    pub fn write_experiment_result(&self, result: &ExperimentResult) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(result)
            .expect("ExperimentResult must be serializable");
        fs::write(self.root.join("experiment_result.json"), json)?;
        Ok(())
    }
}