use chronosentiment_optimization::CandidateEvaluation;
use std::collections::HashMap;

/// A lightweight semantic mapping of a pure mechanical candidate evaluation.
/// This prevents runtime state and granular simulation traces from bloating
/// the orchestrator logic.
#[derive(Debug, Clone)]
pub struct SemanticEvaluationReport {
    pub fitness: f64,
    pub avg_pnl: f64,
    pub regime: Option<String>,
    pub classification: String,
    pub metrics: HashMap<String, f64>,
}

impl From<CandidateEvaluation> for SemanticEvaluationReport {
    fn from(eval: CandidateEvaluation) -> Self {
        let mut report_metrics = HashMap::new();
        report_metrics.insert("total_pnl".to_string(), eval.total_pnl);
        report_metrics.insert("win_rate".to_string(), eval.win_rate);
        report_metrics.insert("trade_count".to_string(), eval.trade_count as f64);
        report_metrics.insert("max_drawdown".to_string(), eval.max_drawdown);
        
        Self {
            fitness: eval.fitness,
            avg_pnl: eval.avg_pnl,
            regime: None,
            classification: "TBD".to_string(),
            metrics: report_metrics,
        }
    }
}
