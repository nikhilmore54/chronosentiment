//! WP5 — Structured Logging API
//!
//! Replaces ad hoc `eprintln!` calls with typed events. Events are written to
//! stderr (always visible in `cargo test`) and optionally to a JSON-lines file.
//!
//! # Usage
//!
//! ```rust,ignore
//! let logger = EventLogger::new(Some("/tmp/experiment.jsonl"));
//! logger.emit(ExperimentEvent::ExperimentStart { experiment_id: "exp0".into() });
//! logger.info("Starting generation loop");
//! ```

use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};

// ── Log level ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

// ── Typed events ──────────────────────────────────────────────────────────────

/// Structured event emitted by the experiment harness.
///
/// Each variant corresponds to a meaningful lifecycle event. The `#[serde(tag)]`
/// attribute produces `{"event": "ExperimentStart", ...}` in JSON output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "PascalCase")]
pub enum ExperimentEvent {
    ExperimentStart {
        experiment_id: String,
        instance: String,
        population: usize,
        generations: usize,
        seed: u64,
    },
    RunStart {
        run_index: usize,
        seed: u64,
    },
    GenerationEnd {
        generation: usize,
        best_fitness: f64,
        mean_fitness: f64,
        feasible_fraction: f64,
        repair_count: usize,
        elapsed_ms: u128,
    },
    RepairApplied {
        generation: usize,
        genome_index: usize,
        empty_rotation: usize,
        donor_rotation: usize,
    },
    RunComplete {
        run_index: usize,
        best_fitness: f64,
        best_generation: usize,
        total_ms: u128,
    },
    RunAborted {
        run_index: usize,
        reason: String,
    },
    ExperimentComplete {
        experiment_id: String,
        n_runs: usize,
        total_ms: u128,
    },
    Message {
        level: LogLevel,
        text: String,
    },
}

impl ExperimentEvent {
    /// Short human-readable prefix for stderr output.
    fn prefix(&self) -> &'static str {
        match self {
            ExperimentEvent::ExperimentStart { .. } => "[harness] experiment_start",
            ExperimentEvent::RunStart { .. }        => "[harness] run_start",
            ExperimentEvent::GenerationEnd { .. }   => "[harness] generation",
            ExperimentEvent::RepairApplied { .. }   => "[harness] repair",
            ExperimentEvent::RunComplete { .. }     => "[harness] run_complete",
            ExperimentEvent::RunAborted { .. }      => "[harness] run_aborted",
            ExperimentEvent::ExperimentComplete { .. } => "[harness] experiment_complete",
            ExperimentEvent::Message { level, .. }  => match level {
                LogLevel::Debug => "[harness] DEBUG",
                LogLevel::Info  => "[harness] INFO",
                LogLevel::Warn  => "[harness] WARN",
                LogLevel::Error => "[harness] ERROR",
            },
        }
    }

    /// One-line human-readable summary for stderr.
    fn summary(&self) -> String {
        match self {
            ExperimentEvent::ExperimentStart { experiment_id, instance, population, generations, seed } =>
                format!("id={experiment_id} instance={instance} pop={population} gen={generations} seed={seed}"),
            ExperimentEvent::RunStart { run_index, seed } =>
                format!("run={run_index} seed={seed}"),
            ExperimentEvent::GenerationEnd { generation, best_fitness, mean_fitness, feasible_fraction, repair_count, elapsed_ms } =>
                format!("gen={generation} best={best_fitness:.4} mean={mean_fitness:.4} feasible={feasible_fraction:.2} repairs={repair_count} ms={elapsed_ms}"),
            ExperimentEvent::RepairApplied { generation, genome_index, empty_rotation, donor_rotation } =>
                format!("gen={generation} genome={genome_index} empty_rot={empty_rotation} donor_rot={donor_rotation}"),
            ExperimentEvent::RunComplete { run_index, best_fitness, best_generation, total_ms } =>
                format!("run={run_index} best={best_fitness:.4} best_gen={best_generation} ms={total_ms}"),
            ExperimentEvent::RunAborted { run_index, reason } =>
                format!("run={run_index} reason={reason}"),
            ExperimentEvent::ExperimentComplete { experiment_id, n_runs, total_ms } =>
                format!("id={experiment_id} runs={n_runs} ms={total_ms}"),
            ExperimentEvent::Message { text, .. } =>
                text.clone(),
        }
    }
}

// ── Logger ────────────────────────────────────────────────────────────────────

/// Thread-safe event logger.
///
/// Writes to stderr (always) and optionally to a JSON-lines sink file.
/// Clone the `Arc` to share across threads.
#[derive(Clone)]
pub struct EventLogger {
    inner: Arc<Mutex<LoggerInner>>,
}

struct LoggerInner {
    sink: Option<std::fs::File>,
    events: Vec<ExperimentEvent>,
    min_level: LogLevel,
}

impl EventLogger {
    /// Create a new logger. If `sink_path` is Some, events are also written as
    /// JSON-lines to that file (appended if it exists).
    pub fn new(sink_path: Option<&str>) -> Self {
        let sink = sink_path.map(|p| {
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(p)
                .expect("Cannot open log sink file")
        });
        Self {
            inner: Arc::new(Mutex::new(LoggerInner {
                sink,
                events: Vec::new(),
                min_level: LogLevel::Info,
            })),
        }
    }

    /// Emit a structured event.
    pub fn emit(&self, event: ExperimentEvent) {
        let mut inner = self.inner.lock().unwrap();
        // Write to stderr
        eprintln!("  {} {}", event.prefix(), event.summary());
        // Write to JSON-lines sink if configured
        if let Some(ref mut file) = inner.sink {
            if let Ok(json) = serde_json::to_string(&event) {
                let _ = writeln!(file, "{json}");
            }
        }
        inner.events.push(event);
    }

    /// Convenience: emit an Info message.
    pub fn info(&self, text: impl Into<String>) {
        self.emit(ExperimentEvent::Message { level: LogLevel::Info, text: text.into() });
    }

    /// Convenience: emit a Warn message.
    pub fn warn(&self, text: impl Into<String>) {
        self.emit(ExperimentEvent::Message { level: LogLevel::Warn, text: text.into() });
    }

    /// Convenience: emit a Debug message (only if min_level <= Debug).
    pub fn debug(&self, text: impl Into<String>) {
        let inner = self.inner.lock().unwrap();
        if inner.min_level <= LogLevel::Debug {
            drop(inner);
            self.emit(ExperimentEvent::Message { level: LogLevel::Debug, text: text.into() });
        }
    }

    /// Return a snapshot of all events emitted so far.
    pub fn events(&self) -> Vec<ExperimentEvent> {
        self.inner.lock().unwrap().events.clone()
    }
}