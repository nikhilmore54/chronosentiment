use chrono::Utc;
use uuid::Uuid;
use coralys_moga::runtime::optimization::metric::MetricReport;
use crate::evidence::{EvidenceItem, EvidenceSourceType};
use crate::validation::context::MarketEvaluationContext as EvaluationContext;

pub struct ChronoEvidenceEngine;

impl ChronoEvidenceEngine {
    /// Generates EvidenceItems from the MetricReport. It does not draw conclusions (e.g., "Buy Reliance").
    /// It merely wraps raw metric data into semantic evidence blocks (e.g., "Bullish momentum").
    pub fn generate_evidence(&self, context: &EvaluationContext, report: &MetricReport) -> Vec<EvidenceItem> {
        let mut evidence = Vec::new();
        let now = Utc::now().timestamp() as u64; // In a strict setup, you might use context.evaluation_timestamp

        // 1. Momentum Evidence
        if let (Some(ma20), Some(ma50)) = (report.get_float("ma_20"), report.get_float("ma_50")) {
            let (title, content) = if ma20 > ma50 {
                ("Bullish momentum", format!("MA20 ({:.2}) is above MA50 ({:.2}).", ma20, ma50))
            } else {
                ("Bearish momentum", format!("MA20 ({:.2}) is below MA50 ({:.2}).", ma20, ma50))
            };

            let item = EvidenceItem::new(
                Uuid::new_v4().to_string(),
                context.research_session_id.clone(),
                title,
                EvidenceSourceType::FinancialData,
                content,
                now,
            ).with_source_name("ChronoMetricEngine");
            
            evidence.push(item);
        }

        // 2. Risk/Volatility Evidence
        if let Some(volatility) = report.get_float("volatility_20d") {
            let (title, content) = if volatility > 0.3 {
                ("High Volatility Environment", format!("Annualized 20d volatility is {:.1}%", volatility * 100.0))
            } else {
                ("Stable Volatility Environment", format!("Annualized 20d volatility is {:.1}%", volatility * 100.0))
            };

            let item = EvidenceItem::new(
                Uuid::new_v4().to_string(),
                context.research_session_id.clone(),
                title,
                EvidenceSourceType::FinancialData,
                content,
                now,
            ).with_source_name("ChronoMetricEngine");

            evidence.push(item);
        }

        evidence
    }
}
