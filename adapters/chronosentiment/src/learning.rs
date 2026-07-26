// ChronoSentiment — Personal Investment Learning Loop
//
// The Learning Loop analyses completed investment outcomes and extracts
// patterns that improve future research quality. It is the mechanism by
// which the investor's knowledge evolves over time.
//
// Platform primitive: Learning (computes) + Pattern (stores)
// ChronoSentiment realisation: PersonalInvestmentLearningLoop
//
// The Learning Loop:
//   1. Receives completed InvestmentOutcomes from closed Workspaces.
//   2. Analyses outcomes against the thesis that drove the decision.
//   3. Extracts InvestmentPatterns (recurring behaviours, biases, strengths).
//   4. Promotes mature patterns to InvestmentInsights.
//   5. Generates a QuarterlyReviewReport summarising the learning cycle.

use serde::{Deserialize, Serialize};
use crate::workspace::InvestmentOutcome;

// ── Investment Pattern ────────────────────────────────────────────────────────

/// A recurring pattern extracted from multiple investment outcomes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentPattern {
    pub pattern_id: String,
    pub description: String,
    pub pattern_type: InvestmentPatternType,
    /// Confidence in [0.0, 1.0] — increases as more outcomes confirm the pattern.
    pub confidence: f64,
    /// Number of outcomes that contributed to this pattern.
    pub supporting_outcomes: usize,
    /// Maturity level — advances as confidence and supporting_outcomes grow.
    pub maturity: PatternMaturity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestmentPatternType {
    /// Recurring thesis assumption that proved incorrect.
    AssumptionBias,
    /// Recurring risk that was underestimated.
    RiskUnderestimation,
    /// Recurring strength — thesis assumptions that consistently proved correct.
    ThesisStrength,
    /// Recurring timing error (entry/exit too early or too late).
    TimingError,
    /// Recurring evidence gap — important evidence type consistently missing.
    EvidenceGap,
    /// Positive pattern — consistently profitable thesis type.
    HighConvictionSuccess,
}

/// Pattern maturity — advances as evidence accumulates.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PatternMaturity {
    /// Observed in 1–2 outcomes; not yet reliable.
    Candidate,
    /// Observed in 3–5 outcomes; emerging pattern.
    Observed,
    /// Observed in 6–10 outcomes; reliable pattern.
    Repeated,
    /// Observed in 10+ outcomes with high confidence; actionable.
    Validated,
}

impl PatternMaturity {
    fn from_outcomes(count: usize, confidence: f64) -> Self {
        if count >= 10 && confidence >= 0.8 {
            PatternMaturity::Validated
        } else if count >= 6 {
            PatternMaturity::Repeated
        } else if count >= 3 {
            PatternMaturity::Observed
        } else {
            PatternMaturity::Candidate
        }
    }
}

// ── Investment Insight ────────────────────────────────────────────────────────

/// A validated insight added to the Personal Investment Knowledge Graph.
///
/// Insights are promoted from mature patterns. They represent actionable
/// knowledge that should influence future research behaviour.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentInsight {
    pub insight_id: String,
    pub description: String,
    pub source_pattern_id: String,
    pub recommendation: String,
    pub confidence: f64,
    pub timestamp: u64,
}

// ── Quarterly Review Report ───────────────────────────────────────────────────

/// A structured quarterly review report — produced at the end of each
/// learning loop run.
///
/// The quarterly review is the formal mechanism by which the investor
/// reviews their active theses and captures learnings from closed positions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarterlyReviewReport {
    pub report_id: String,
    pub quarter: String,
    pub outcomes_reviewed: usize,
    pub profitable_outcomes: usize,
    pub loss_outcomes: usize,
    pub thesis_validation_rate: f64,
    pub patterns_identified: usize,
    pub patterns_validated: usize,
    pub insights_added: usize,
    pub mean_return_pct: Option<f64>,
    pub summary: Vec<String>,
    pub timestamp: u64,
}

// ── Personal Investment Learning Loop ────────────────────────────────────────

/// Full end-to-end Personal Investment Learning Loop.
///
/// Workflow:
///   1. `record_outcome` — record a completed investment outcome
///   2. `identify_patterns` — extract patterns from accumulated outcomes
///   3. `add_insight` — add validated insights to the Knowledge Graph
///   4. `auto_promote_insights` — automatically promote mature patterns
///   5. `generate_quarterly_report` — produce a structured quarterly review
pub struct PersonalInvestmentLearningLoop {
    pub outcomes: Vec<InvestmentOutcome>,
    pub patterns: Vec<InvestmentPattern>,
    pub insights: Vec<InvestmentInsight>,
    pattern_counter: u32,
    insight_counter: u32,
}

impl PersonalInvestmentLearningLoop {
    pub fn new() -> Self {
        Self {
            outcomes: Vec::new(),
            patterns: Vec::new(),
            insights: Vec::new(),
            pattern_counter: 0,
            insight_counter: 0,
        }
    }

    /// Step 1 — Record a completed investment outcome.
    pub fn record_outcome(&mut self, outcome: InvestmentOutcome) {
        self.outcomes.push(outcome);
    }

    /// Step 2 — Identify patterns from accumulated outcomes.
    pub fn identify_patterns(&mut self) {
        if self.outcomes.is_empty() {
            return;
        }
        let n = self.outcomes.len() as f64;

        // Pattern: thesis validation rate.
        let validated_count = self.outcomes.iter().filter(|o| o.thesis_validated).count();
        let validation_rate = validated_count as f64 / n;
        if validation_rate < 0.5 && n >= 3.0 {
            self.upsert_pattern(
                "low-thesis-validation",
                format!(
                    "Thesis assumptions proved incorrect in {:.0}% of completed investments",
                    (1.0 - validation_rate) * 100.0
                ),
                InvestmentPatternType::AssumptionBias,
                (1.0 - validation_rate).min(1.0),
                (n as usize) - validated_count,
            );
        }

        // Pattern: high conviction success.
        let profitable_count = self.outcomes.iter()
            .filter(|o| {
                matches!(o.result, crate::workspace::OutcomeResult::Profitable)
            })
            .count();
        let profit_rate = profitable_count as f64 / n;
        if profit_rate >= 0.6 && n >= 3.0 {
            self.upsert_pattern(
                "high-conviction-success",
                format!(
                    "Profitable outcomes achieved in {:.0}% of completed investments",
                    profit_rate * 100.0
                ),
                InvestmentPatternType::HighConvictionSuccess,
                profit_rate.min(1.0),
                profitable_count,
            );
        }

        // Pattern: loss rate.
        let loss_count = self.outcomes.iter()
            .filter(|o| matches!(o.result, crate::workspace::OutcomeResult::Loss))
            .count();
        let loss_rate = loss_count as f64 / n;
        if loss_rate >= 0.3 && n >= 3.0 {
            self.upsert_pattern(
                "recurring-losses",
                format!(
                    "Loss outcomes in {:.0}% of completed investments — review risk assessment process",
                    loss_rate * 100.0
                ),
                InvestmentPatternType::RiskUnderestimation,
                loss_rate.min(1.0),
                loss_count,
            );
        }

        // Pattern: evidence gap (no key learnings recorded).
        let no_learnings_count = self.outcomes.iter()
            .filter(|o| o.key_learnings.is_empty())
            .count();
        let no_learnings_rate = no_learnings_count as f64 / n;
        if no_learnings_rate >= 0.4 && n >= 3.0 {
            self.upsert_pattern(
                "missing-post-outcome-learnings",
                format!(
                    "Post-outcome learnings not recorded in {:.0}% of investments — knowledge is being lost",
                    no_learnings_rate * 100.0
                ),
                InvestmentPatternType::EvidenceGap,
                no_learnings_rate.min(1.0),
                no_learnings_count,
            );
        }
    }

    fn upsert_pattern(
        &mut self,
        key: &str,
        description: String,
        pattern_type: InvestmentPatternType,
        confidence: f64,
        supporting_outcomes: usize,
    ) {
        if let Some(existing) = self.patterns.iter_mut().find(|p| p.pattern_id == key) {
            existing.confidence = confidence;
            existing.supporting_outcomes = supporting_outcomes;
            existing.maturity = PatternMaturity::from_outcomes(supporting_outcomes, confidence);
            existing.description = description;
        } else {
            self.pattern_counter += 1;
            self.patterns.push(InvestmentPattern {
                pattern_id: key.to_string(),
                description,
                pattern_type,
                confidence,
                supporting_outcomes,
                maturity: PatternMaturity::from_outcomes(supporting_outcomes, confidence),
            });
        }
    }

    /// Step 3 — Add a validated insight to the Personal Investment Knowledge Graph.
    ///
    /// Only patterns at `Repeated` or `Validated` maturity should produce insights.
    pub fn add_insight(
        &mut self,
        source_pattern_id: &str,
        recommendation: impl Into<String>,
        timestamp: u64,
    ) -> Option<String> {
        let pattern = self.patterns.iter().find(|p| p.pattern_id == source_pattern_id)?;
        if pattern.maturity < PatternMaturity::Repeated {
            return None;
        }
        self.insight_counter += 1;
        let insight_id = format!("insight-{:04}", self.insight_counter);
        let insight = InvestmentInsight {
            insight_id: insight_id.clone(),
            description: pattern.description.clone(),
            source_pattern_id: source_pattern_id.to_string(),
            recommendation: recommendation.into(),
            confidence: pattern.confidence,
            timestamp,
        };
        self.insights.push(insight);
        Some(insight_id)
    }

    /// Step 4 — Automatically promote all mature patterns to insights.
    pub fn auto_promote_insights(&mut self, timestamp: u64) {
        let promotable: Vec<(String, String)> = self.patterns.iter()
            .filter(|p| p.maturity >= PatternMaturity::Repeated)
            .filter(|p| !self.insights.iter().any(|i| i.source_pattern_id == p.pattern_id))
            .map(|p| {
                let rec = match p.pattern_type {
                    InvestmentPatternType::AssumptionBias =>
                        "Review assumption-setting process; stress-test each assumption against bear-case scenarios before committing.".to_string(),
                    InvestmentPatternType::RiskUnderestimation =>
                        "Add a dedicated risk review step before each investment decision; consider position sizing reduction.".to_string(),
                    InvestmentPatternType::ThesisStrength =>
                        "Document the characteristics of successful theses as a reference template for future research.".to_string(),
                    InvestmentPatternType::TimingError =>
                        "Review entry/exit criteria; consider staged entry to reduce timing risk.".to_string(),
                    InvestmentPatternType::EvidenceGap =>
                        "Add a post-outcome review step to every closed workspace; record at least three key learnings.".to_string(),
                    InvestmentPatternType::HighConvictionSuccess =>
                        "Document the characteristics of successful high-conviction investments as a reference for future thesis development.".to_string(),
                };
                (p.pattern_id.clone(), rec)
            })
            .collect();

        for (pattern_id, recommendation) in promotable {
            self.add_insight(&pattern_id, recommendation, timestamp);
        }
    }

    /// Step 5 — Generate a structured quarterly review report.
    pub fn generate_quarterly_report(&self, quarter: impl Into<String>, timestamp: u64) -> QuarterlyReviewReport {
        let n = self.outcomes.len();
        let profitable = self.outcomes.iter()
            .filter(|o| matches!(o.result, crate::workspace::OutcomeResult::Profitable))
            .count();
        let losses = self.outcomes.iter()
            .filter(|o| matches!(o.result, crate::workspace::OutcomeResult::Loss))
            .count();
        let validated = self.outcomes.iter().filter(|o| o.thesis_validated).count();
        let thesis_validation_rate = if n > 0 { validated as f64 / n as f64 } else { 0.0 };

        let returns: Vec<f64> = self.outcomes.iter()
            .filter_map(|o| o.return_pct)
            .collect();
        let mean_return_pct = if returns.is_empty() {
            None
        } else {
            Some(returns.iter().sum::<f64>() / returns.len() as f64)
        };

        let validated_patterns = self.patterns.iter()
            .filter(|p| p.maturity == PatternMaturity::Validated)
            .count();

        let mut summary = Vec::new();
        summary.push(format!("Outcomes reviewed: {}", n));
        summary.push(format!("Profitable: {} | Losses: {}", profitable, losses));
        summary.push(format!("Thesis validation rate: {:.0}%", thesis_validation_rate * 100.0));
        if let Some(ret) = mean_return_pct {
            summary.push(format!("Mean return: {:.1}%", ret));
        }
        summary.push(format!("Patterns identified: {}", self.patterns.len()));
        summary.push(format!("Patterns validated: {}", validated_patterns));
        summary.push(format!("Insights in Knowledge Graph: {}", self.insights.len()));

        for pattern in &self.patterns {
            if pattern.maturity >= PatternMaturity::Repeated {
                summary.push(format!(
                    "[{:?}] {} (confidence: {:.0}%)",
                    pattern.maturity,
                    pattern.description,
                    pattern.confidence * 100.0,
                ));
            }
        }

        QuarterlyReviewReport {
            report_id: format!("qr-{}", timestamp),
            quarter: quarter.into(),
            outcomes_reviewed: n,
            profitable_outcomes: profitable,
            loss_outcomes: losses,
            thesis_validation_rate,
            patterns_identified: self.patterns.len(),
            patterns_validated: validated_patterns,
            insights_added: self.insights.len(),
            mean_return_pct,
            summary,
            timestamp,
        }
    }

    pub fn patterns(&self) -> &[InvestmentPattern] {
        &self.patterns
    }

    pub fn insights(&self) -> &[InvestmentInsight] {
        &self.insights
    }
}

impl Default for PersonalInvestmentLearningLoop {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::{InvestmentOutcome, OutcomeResult};

    fn make_outcome(id: &str, result: OutcomeResult, validated: bool, return_pct: Option<f64>) -> InvestmentOutcome {
        InvestmentOutcome {
            outcome_id: id.to_string(),
            workspace_id: "ws-001".to_string(),
            result,
            summary: "Test outcome".to_string(),
            return_pct,
            holding_period_days: Some(180),
            thesis_validated: validated,
            key_learnings: if validated { vec!["Thesis confirmed.".to_string()] } else { vec![] },
            recorded_at: 1000,
        }
    }

    #[test]
    fn identifies_assumption_bias_pattern() {
        let mut loop_ = PersonalInvestmentLearningLoop::new();
        // 4 outcomes, 3 with thesis not validated.
        loop_.record_outcome(make_outcome("o1", OutcomeResult::Loss, false, Some(-5.0)));
        loop_.record_outcome(make_outcome("o2", OutcomeResult::Loss, false, Some(-8.0)));
        loop_.record_outcome(make_outcome("o3", OutcomeResult::Profitable, true, Some(12.0)));
        loop_.record_outcome(make_outcome("o4", OutcomeResult::Loss, false, Some(-3.0)));
        loop_.identify_patterns();
        assert!(loop_.patterns().iter().any(|p| p.pattern_type == InvestmentPatternType::AssumptionBias));
    }

    #[test]
    fn identifies_high_conviction_success_pattern() {
        let mut loop_ = PersonalInvestmentLearningLoop::new();
        for i in 0..5 {
            loop_.record_outcome(make_outcome(
                &format!("o{}", i),
                OutcomeResult::Profitable,
                true,
                Some(15.0),
            ));
        }
        loop_.identify_patterns();
        assert!(loop_.patterns().iter().any(|p| p.pattern_type == InvestmentPatternType::HighConvictionSuccess));
    }

    #[test]
    fn quarterly_report_calculates_mean_return() {
        let mut loop_ = PersonalInvestmentLearningLoop::new();
        loop_.record_outcome(make_outcome("o1", OutcomeResult::Profitable, true, Some(10.0)));
        loop_.record_outcome(make_outcome("o2", OutcomeResult::Profitable, true, Some(20.0)));
        loop_.record_outcome(make_outcome("o3", OutcomeResult::Loss, false, Some(-5.0)));
        let report = loop_.generate_quarterly_report("Q3 2026", 9999);
        assert_eq!(report.outcomes_reviewed, 3);
        assert!((report.mean_return_pct.unwrap() - 8.333).abs() < 0.01);
    }

    #[test]
    fn auto_promote_insights_from_repeated_patterns() {
        let mut loop_ = PersonalInvestmentLearningLoop::new();
        // 6 outcomes with no learnings → EvidenceGap pattern at Repeated maturity.
        for i in 0..6 {
            let mut outcome = make_outcome(&format!("o{}", i), OutcomeResult::Profitable, true, Some(5.0));
            outcome.key_learnings = vec![]; // no learnings recorded
            loop_.record_outcome(outcome);
        }
        loop_.identify_patterns();
        loop_.auto_promote_insights(1000);
        assert!(!loop_.insights().is_empty());
    }
}