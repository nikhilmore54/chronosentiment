//! WP6 — Automatic Report Generator
//!
//! Generates a Markdown summary table from experiment results. The output is
//! designed to be pasted directly into Section 3 research documents without
//! manual transcription.
//!
//! # Usage
//!
//! ```rust,ignore
//! let report = ReportGenerator::from_summaries(&summaries);
//! println!("{}", report.markdown_table());
//! std::fs::write("results/exp0/report.md", report.markdown_table()).unwrap();
//! ```

use crate::harness::schema::{MultiRunAggregate, RunSummary};

/// Generates reports from experiment results.
pub struct ReportGenerator;

impl ReportGenerator {
    /// Generate a Markdown comparison table from a list of single-run summaries.
    ///
    /// Columns: Instance | Pairings | Rotations | Greedy | EA Best | Best Gen | Time (ms) | vs Greedy
    pub fn markdown_table(summaries: &[RunSummary]) -> String {
        let mut out = String::new();

        out.push_str("| Instance | Pairings | Rotations | Greedy | EA Best | Best Gen | Time (ms) | vs Greedy |\n");
        out.push_str("|----------|----------|-----------|--------|---------|----------|-----------|-----------|\n");

        for s in summaries {
            let greedy_str = s
                .greedy_baseline
                .map(|v| format!("{v:.4}"))
                .unwrap_or_else(|| "—".to_string());

            let vs_greedy = match (s.greedy_baseline, s.best_fitness.is_finite()) {
                (Some(g), true) if g > 0.0 => {
                    let pct = (s.best_fitness - g) / g * 100.0;
                    if pct < -0.001 {
                        format!("{pct:+.1}% ✓")
                    } else if pct > 0.001 {
                        format!("{pct:+.1}% ✗")
                    } else {
                        "=".to_string()
                    }
                }
                _ => "—".to_string(),
            };

            let best_str = if s.best_fitness.is_finite() {
                format!("{:.4}", s.best_fitness)
            } else {
                "inf".to_string()
            };

            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
                s.config.instance,
                s.n_pairings,
                s.n_rotations,
                greedy_str,
                best_str,
                s.best_generation,
                s.total_ms,
                vs_greedy,
            ));
        }

        out
    }

    /// Generate a Markdown statistics table from a multi-run aggregate.
    ///
    /// Columns: Instance | N | Mean | Median | Std | Min | Max | CI95 ±
    pub fn markdown_aggregate_table(aggregates: &[MultiRunAggregate]) -> String {
        let mut out = String::new();

        out.push_str("| Instance | N | Mean | Median | Std | Min | Max | CI95 ± |\n");
        out.push_str("|----------|---|------|--------|-----|-----|-----|--------|\n");

        for a in aggregates {
            out.push_str(&format!(
                "| {} | {} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} |\n",
                a.config.instance,
                a.n_runs,
                a.mean_best,
                a.median_best,
                a.std_best,
                a.min_best,
                a.max_best,
                a.ci95_half_width,
            ));
        }

        out
    }

    /// Generate a Markdown convergence summary from generation records embedded
    /// in a RunSummary. Shows generation at which each 10% improvement threshold
    /// was first crossed.
    pub fn markdown_convergence_summary(summary: &RunSummary) -> String {
        if summary.generations.is_empty() {
            return "No generation records available.\n".to_string();
        }

        let initial = summary.generations[0].best_fitness;
        let final_best = summary.best_fitness;
        let total_improvement = if initial.is_finite() && initial > 0.0 {
            (initial - final_best) / initial * 100.0
        } else {
            0.0
        };

        let mut out = String::new();
        out.push_str(&format!(
            "**Instance:** {}  **Seed:** {}  **Total improvement:** {:.1}%\n\n",
            summary.config.instance, summary.config.random_seed, total_improvement
        ));
        out.push_str("| Generation | Best Fitness | Mean Fitness | Feasible % | Repairs |\n");
        out.push_str("|------------|-------------|--------------|------------|---------|\n");

        // Emit every 10th generation plus the final one
        let step = (summary.generations.len() / 10).max(1);
        for (i, r) in summary.generations.iter().enumerate() {
            if i % step == 0 || i == summary.generations.len() - 1 {
                out.push_str(&format!(
                    "| {} | {:.4} | {:.4} | {:.1}% | {} |\n",
                    r.generation,
                    r.best_fitness,
                    r.mean_fitness,
                    r.feasible_fraction * 100.0,
                    r.repair_count,
                ));
            }
        }

        out
    }

    /// Write a full experiment report to a Markdown file.
    pub fn write_report(
        path: &std::path::Path,
        experiment_id: &str,
        summaries: &[RunSummary],
        aggregates: &[MultiRunAggregate],
    ) -> std::io::Result<()> {
        let mut content = String::new();
        content.push_str(&format!("# Experiment Report: {experiment_id}\n\n"));
        content.push_str(&format!(
            "Generated: {}\n\n",
            chrono::Utc::now().to_rfc3339()
        ));

        if !summaries.is_empty() {
            content.push_str("## Results\n\n");
            content.push_str(&Self::markdown_table(summaries));
            content.push('\n');
        }

        if !aggregates.is_empty() {
            content.push_str("## Multi-seed Statistics\n\n");
            content.push_str(&Self::markdown_aggregate_table(aggregates));
            content.push('\n');
        }

        if !summaries.is_empty() {
            content.push_str("## Convergence Profiles\n\n");
            for s in summaries {
                if !s.generations.is_empty() {
                    content.push_str(&format!("### Instance: {}\n\n", s.config.instance));
                    content.push_str(&Self::markdown_convergence_summary(s));
                    content.push('\n');
                }
            }
        }

        std::fs::write(path, content)?;
        Ok(())
    }
}