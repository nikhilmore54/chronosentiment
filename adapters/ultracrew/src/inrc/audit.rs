//! Feasibility Distribution Audit — INRC-specific snapshot module.
//!
//! Answers: "Can Coralys reach feasibility?"
//!
//! At a target generation (typically Gen 5000), captures a full statistical
//! snapshot of the Pareto archive and reports:
//!
//!   - Archive size
//!   - Feasible count (all four HC == 0)
//!   - Near-feasible count (total HC violations ≤ 5)
//!   - Near-feasible count (total HC violations ≤ 10)
//!   - Per-member: HC_Coverage, HC_Skills, HC_Successions, SoftTotal, OfficialTotal
//!   - Best feasible official score
//!   - Best near-feasible official score
//!   - Best infeasible official score
//!
//! Architecture: INRC-specific. Lives in the adapter. Does NOT touch Coralys core.

use crate::inrc::optimization::InrcEvaluation;

/// Per-member snapshot at audit time.
#[derive(Clone, Debug)]
pub struct MemberSnapshot {
    /// Index within the archive at snapshot time.
    pub archive_index: usize,

    pub hc_coverage: usize,
    pub hc_skills: usize,
    pub hc_one_shift_per_day: usize,
    pub hc_forbidden_successions: usize,

    /// Sum of all four HC violation counts.
    pub total_hc_violations: usize,

    /// Sum of all soft constraint penalties.
    pub soft_total: i32,

    /// Official total score = total_hc_penalty + soft_total.
    /// Formula: hc_coverage + hc_skills + hc_one_shift_per_day
    ///          + hc_forbidden_successions + soft_total
    /// (HC components are already penalty-weighted ×1000 by the evaluator.)
    /// Canonical definition: inrc_observer::score_inrc_official()
    pub official_total: i64,

    /// Whether all four HC counts are zero.
    pub feasible: bool,

    /// Proxy objective vector (O1..On) at snapshot time.
    pub objective_vector: Vec<f64>,
}

impl MemberSnapshot {
    /// Construct from an [`InrcEvaluation`] and a proxy objective vector.
    pub fn from_evaluation(
        archive_index: usize,
        eval: &InrcEvaluation,
        objective_vector: Vec<f64>,
    ) -> Self {
        let total_hc = eval.hc_coverage
            + eval.hc_skills
            + eval.hc_one_shift_per_day
            + eval.hc_forbidden_successions;

        let soft_total = eval.soft_report.total_penalty;
        // Each hard constraint violation has a penalty weight of 1000.
        // Official total score = (total_hc * 1000) + soft_total.
        let official_total = (total_hc as i64 * 1000) + soft_total as i64;

        Self {
            archive_index,
            hc_coverage: eval.hc_coverage,
            hc_skills: eval.hc_skills,
            hc_one_shift_per_day: eval.hc_one_shift_per_day,
            hc_forbidden_successions: eval.hc_forbidden_successions,
            total_hc_violations: total_hc,
            soft_total,
            official_total,
            feasible: eval.is_feasible(),
            objective_vector,
        }
    }
}

/// Aggregated statistics for a subset of archive members.
#[derive(Clone, Debug)]
pub struct SubsetStats {
    pub count: usize,
    pub best_official_score: Option<i64>,
    pub worst_official_score: Option<i64>,
    pub mean_hc_violations: f64,
    pub mean_soft_total: f64,
}

impl SubsetStats {
    fn from_members(members: &[&MemberSnapshot]) -> Self {
        if members.is_empty() {
            return Self {
                count: 0,
                best_official_score: None,
                worst_official_score: None,
                mean_hc_violations: 0.0,
                mean_soft_total: 0.0,
            };
        }
        let count = members.len();
        let best = members.iter().map(|m| m.official_total).min();
        let worst = members.iter().map(|m| m.official_total).max();
        let mean_hc = members
            .iter()
            .map(|m| m.total_hc_violations as f64)
            .sum::<f64>()
            / count as f64;
        let mean_soft = members.iter().map(|m| m.soft_total as f64).sum::<f64>() / count as f64;
        Self {
            count,
            best_official_score: best,
            worst_official_score: worst,
            mean_hc_violations: mean_hc,
            mean_soft_total: mean_soft,
        }
    }
}

/// Full feasibility distribution snapshot at one generation.
pub struct FeasibilitySnapshot {
    /// Generation at which this snapshot was taken.
    pub generation: u64,

    pub feasible_count: usize,
    pub near_feasible_5: usize,
    pub near_feasible_10: usize,
    pub infeasible_count: usize,

    pub best_feasible_score: Option<i64>,
    pub best_near_feasible_score: Option<i64>,
    pub best_infeasible_score: Option<i64>,

    /// All archive member snapshots.
    pub members: Vec<MemberSnapshot>,
}

impl FeasibilitySnapshot {
    pub fn new(generation: u64) -> Self {
        Self {
            generation,
            feasible_count: 0,
            near_feasible_5: 0,
            near_feasible_10: 0,
            infeasible_count: 0,
            best_feasible_score: None,
            best_near_feasible_score: None,
            best_infeasible_score: None,
            members: Vec::new(),
        }
    }

    /// Add a member and update aggregate counts.
    pub fn add_member(&mut self, snapshot: MemberSnapshot) {
        let hc = snapshot.total_hc_violations;
        let score = snapshot.official_total;

        if snapshot.feasible {
            self.feasible_count += 1;
            self.best_feasible_score = Some(match self.best_feasible_score {
                Some(prev) => prev.min(score),
                None => score,
            });
        } else {
            self.infeasible_count += 1;
            self.best_infeasible_score = Some(match self.best_infeasible_score {
                Some(prev) => prev.min(score),
                None => score,
            });
        }

        if hc <= 5 {
            self.near_feasible_5 += 1;
        }
        if hc <= 10 {
            self.near_feasible_10 += 1;
        }

        if hc <= 10 {
            self.best_near_feasible_score = Some(match self.best_near_feasible_score {
                Some(prev) => prev.min(score),
                None => score,
            });
        }

        self.members.push(snapshot);
    }

    pub fn archive_size(&self) -> usize {
        self.members.len()
    }

    pub fn feasible_stats(&self) -> SubsetStats {
        let subset: Vec<&MemberSnapshot> = self.members.iter().filter(|m| m.feasible).collect();
        SubsetStats::from_members(&subset)
    }

    pub fn infeasible_stats(&self) -> SubsetStats {
        let subset: Vec<&MemberSnapshot> = self.members.iter().filter(|m| !m.feasible).collect();
        SubsetStats::from_members(&subset)
    }

    pub fn near_feasible_stats(&self, max_violations: usize) -> SubsetStats {
        let subset: Vec<&MemberSnapshot> = self
            .members
            .iter()
            .filter(|m| m.total_hc_violations <= max_violations)
            .collect();
        SubsetStats::from_members(&subset)
    }

    /// HC violation histogram: (violation_count, member_count).
    pub fn hc_violation_histogram(&self) -> Vec<(usize, usize)> {
        let mut counts: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
        for m in &self.members {
            *counts.entry(m.total_hc_violations).or_insert(0) += 1;
        }
        let mut hist: Vec<(usize, usize)> = counts.into_iter().collect();
        hist.sort_by_key(|(k, _)| *k);
        hist
    }

    /// Print a human-readable report to stdout.
    pub fn print_report(&self) {
        println!(
            "=== Feasibility Distribution Audit (Gen {}) ===",
            self.generation
        );
        println!("  Archive Size        : {}", self.archive_size());
        println!("  Feasible            : {}", self.feasible_count);
        println!("  Near-Feasible (≤5)  : {}", self.near_feasible_5);
        println!("  Near-Feasible (≤10) : {}", self.near_feasible_10);
        println!("  Infeasible          : {}", self.infeasible_count);
        println!();

        println!("  Best Feasible Score     : {:?}", self.best_feasible_score);
        println!(
            "  Best Near-Feasible Score: {:?}",
            self.best_near_feasible_score
        );
        println!(
            "  Best Infeasible Score   : {:?}",
            self.best_infeasible_score
        );
        println!();

        let fs = self.feasible_stats();
        println!("  Feasible Subset:");
        if fs.count > 0 {
            println!("    Count             : {}", fs.count);
            println!("    Best Score        : {:?}", fs.best_official_score);
            println!("    Worst Score       : {:?}", fs.worst_official_score);
            println!("    Mean Soft Total   : {:.1}", fs.mean_soft_total);
        } else {
            println!("    (none)");
        }
        println!();

        let ifs = self.infeasible_stats();
        println!("  Infeasible Subset:");
        if ifs.count > 0 {
            println!("    Count             : {}", ifs.count);
            println!("    Best Score        : {:?}", ifs.best_official_score);
            println!("    Mean HC Violations: {:.2}", ifs.mean_hc_violations);
            println!("    Mean Soft Total   : {:.1}", ifs.mean_soft_total);
        } else {
            println!("    (none)");
        }
        println!();

        let nf5 = self.near_feasible_stats(5);
        println!("  Near-Feasible (≤5 HC) Subset:");
        if nf5.count > 0 {
            println!("    Count             : {}", nf5.count);
            println!("    Best Score        : {:?}", nf5.best_official_score);
            println!("    Mean HC Violations: {:.2}", nf5.mean_hc_violations);
        } else {
            println!("    (none)");
        }
        println!();

        println!("  HC Violation Histogram:");
        for (violations, count) in self.hc_violation_histogram() {
            let bar: String = "#".repeat(count.min(40));
            println!("    HC={:>4} : {:>4} members  {}", violations, count, bar);
        }
        println!();

        println!("  Per-Member Detail:");
        println!(
            "    {:>4}  {:>6}  {:>6}  {:>6}  {:>6}  {:>8}  {:>10}  {}",
            "Idx", "HC_Cov", "HC_Skl", "HC_Suc", "HC_1Sh", "SoftTot", "Official", "Feasible"
        );
        for m in &self.members {
            println!(
                "    {:>4}  {:>6}  {:>6}  {:>6}  {:>6}  {:>8}  {:>10}  {}",
                m.archive_index,
                m.hc_coverage,
                m.hc_skills,
                m.hc_forbidden_successions,
                m.hc_one_shift_per_day,
                m.soft_total,
                m.official_total,
                if m.feasible { "YES" } else { "no" },
            );
        }
    }

    /// Serialize to JSON lines for logging/export.
    pub fn to_json_lines(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!(
            r#"{{"type":"fda_summary","gen":{},"archive_size":{},"feasible":{},"near_feasible_5":{},"near_feasible_10":{},"infeasible":{},"best_feasible":{:?},"best_nf5":{:?},"best_infeasible":{:?}}}"#,
            self.generation,
            self.archive_size(),
            self.feasible_count,
            self.near_feasible_5,
            self.near_feasible_10,
            self.infeasible_count,
            self.best_feasible_score,
            self.best_near_feasible_score,
            self.best_infeasible_score,
        ));
        for m in &self.members {
            let proxy: Vec<String> = m
                .objective_vector
                .iter()
                .map(|v| format!("{:.4}", v))
                .collect();
            lines.push(format!(
                r#"{{"type":"fda_member","idx":{},"hc_cov":{},"hc_skl":{},"hc_suc":{},"hc_1sh":{},"hc_total":{},"soft_total":{},"official":{},"feasible":{},"proxy":[{}]}}"#,
                m.archive_index,
                m.hc_coverage,
                m.hc_skills,
                m.hc_forbidden_successions,
                m.hc_one_shift_per_day,
                m.total_hc_violations,
                m.soft_total,
                m.official_total,
                m.feasible,
                proxy.join(","),
            ));
        }
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inrc::optimization::{InrcEvaluation, InrcGenome, SoftConstraintReport};

    fn make_eval(
        hc_cov: usize,
        hc_skl: usize,
        hc_suc: usize,
        hc_1sh: usize,
        soft: i32,
    ) -> InrcEvaluation {
        let total_hc = hc_cov + hc_skl + hc_suc + hc_1sh;
        let soft_report = SoftConstraintReport {
            assignment_penalty: soft,
            work_streak_penalty: 0,
            day_off_penalty: 0,
            weekend_penalty: 0,
            preferences_penalty: 0,
            optimal_coverage_penalty: 0,
            total_penalty: soft,
        };
        let fitness = 100_000.0 - (total_hc as f64 * 1000.0) - soft as f64;
        let objectives = vec![
            hc_cov as f64,
            hc_skl as f64,
            hc_1sh as f64,
            hc_suc as f64,
            soft as f64,
        ];
        let platform_result = coralys_core::EvaluationResult {
            objectives,
            hard_constraint_violations: Vec::new(),
            soft_constraint_violations: Vec::new(),
            metrics: std::collections::HashMap::new(),
        };
        InrcEvaluation {
            genome: InrcGenome { bits: vec![] },
            fitness,
            hc_coverage: hc_cov,
            hc_skills: hc_skl,
            hc_one_shift_per_day: hc_1sh,
            hc_forbidden_successions: hc_suc,
            soft_report,
            platform_result,
        }
    }

    #[test]
    fn test_feasibility_counts() {
        let mut snap = FeasibilitySnapshot::new(5000);

        // Fully feasible
        snap.add_member(MemberSnapshot::from_evaluation(
            0,
            &make_eval(0, 0, 0, 0, 1000),
            vec![0.9, 0.1],
        ));
        // Near-feasible (3 violations)
        snap.add_member(MemberSnapshot::from_evaluation(
            1,
            &make_eval(2, 1, 0, 0, 800),
            vec![0.7, 0.3],
        ));
        // Infeasible (12 violations)
        snap.add_member(MemberSnapshot::from_evaluation(
            2,
            &make_eval(5, 4, 2, 1, 500),
            vec![0.5, 0.5],
        ));

        assert_eq!(snap.archive_size(), 3);
        assert_eq!(snap.feasible_count, 1);
        assert_eq!(snap.near_feasible_5, 2); // 0 and 3 violations
        assert_eq!(snap.near_feasible_10, 2); // 0 and 3 violations
        assert_eq!(snap.infeasible_count, 2);
    }

    #[test]
    fn test_best_scores_by_category() {
        let mut snap = FeasibilitySnapshot::new(5000);

        // Feasible, soft=1000 → official = 0*1000 + 1000 = 1000
        snap.add_member(MemberSnapshot::from_evaluation(
            0,
            &make_eval(0, 0, 0, 0, 1000),
            vec![0.9, 0.1],
        ));
        // Feasible, soft=800 → official = 800
        snap.add_member(MemberSnapshot::from_evaluation(
            1,
            &make_eval(0, 0, 0, 0, 800),
            vec![0.8, 0.2],
        ));
        // Infeasible, 2 HC, soft=500 → official = 2000 + 500 = 2500
        snap.add_member(MemberSnapshot::from_evaluation(
            2,
            &make_eval(1, 1, 0, 0, 500),
            vec![0.5, 0.5],
        ));

        assert_eq!(snap.best_feasible_score, Some(800));
        assert_eq!(snap.best_infeasible_score, Some(2500));
    }

    #[test]
    fn test_hc_histogram() {
        let mut snap = FeasibilitySnapshot::new(5000);
        snap.add_member(MemberSnapshot::from_evaluation(
            0,
            &make_eval(0, 0, 0, 0, 100),
            vec![],
        ));
        snap.add_member(MemberSnapshot::from_evaluation(
            1,
            &make_eval(0, 0, 0, 0, 200),
            vec![],
        ));
        snap.add_member(MemberSnapshot::from_evaluation(
            2,
            &make_eval(1, 0, 0, 0, 100),
            vec![],
        ));
        snap.add_member(MemberSnapshot::from_evaluation(
            3,
            &make_eval(3, 0, 0, 0, 100),
            vec![],
        ));

        let hist = snap.hc_violation_histogram();
        // HC=0: 2 members, HC=1: 1 member, HC=3: 1 member
        assert_eq!(hist[0], (0, 2));
        assert_eq!(hist[1], (1, 1));
        assert_eq!(hist[2], (3, 1));
    }

    #[test]
    fn test_official_total_formula() {
        let eval = make_eval(2, 1, 0, 0, 500);
        let snap = MemberSnapshot::from_evaluation(0, &eval, vec![]);
        // total_hc = 3, official = 3*1000 + 500 = 3500
        assert_eq!(snap.total_hc_violations, 3);
        assert_eq!(snap.official_total, 3500);
        assert!(!snap.feasible);
    }
}
