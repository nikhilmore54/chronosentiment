use chrono::Utc;
use uuid::Uuid;
use crate::evidence::EvidenceItem;
use crate::hypothesis::InvestmentThesis;
use crate::validation::context::MarketEvaluationContext as EvaluationContext;

pub struct ChronoHypothesisEngine;

impl ChronoHypothesisEngine {
    /// Generates Hypotheses from Evidence. It begins reasoning.
    /// E.g. "Reliance will outperform Nifty over 3 months."
    pub fn generate_hypotheses(&self, context: &EvaluationContext, evidence: &[EvidenceItem]) -> Vec<InvestmentThesis> {
        let mut hypotheses = Vec::new();
        let now = Utc::now().timestamp() as u64;

        // Naive logic: If we have Bullish momentum evidence, form a bullish hypothesis.
        let has_bullish = evidence.iter().any(|e| e.title == "Bullish momentum");
        let has_bearish = evidence.iter().any(|e| e.title == "Bearish momentum");
        
        let supporting: Vec<String> = evidence.iter().map(|e| e.title.clone()).collect();
        let contradicting = vec![]; // Simplified

        if has_bullish {
            let thesis = InvestmentThesis::new(
                Uuid::new_v4().to_string(),
                context.research_session_id.clone(),
                "Asset will outperform over the next 3 months due to bullish technical momentum.",
                supporting.clone(),
                contradicting.clone(),
                Some("Assumes macroeconomic stability."),
                now,
            );
            hypotheses.push(thesis);
        } else if has_bearish {
            let thesis = InvestmentThesis::new(
                Uuid::new_v4().to_string(),
                context.research_session_id.clone(),
                "Asset will underperform over the next 3 months due to bearish technical breakdown.",
                supporting.clone(),
                contradicting.clone(),
                Some("Assumes no immediate positive catalysts."),
                now,
            );
            hypotheses.push(thesis);
        } else {
            // Neutral
            let thesis = InvestmentThesis::new(
                Uuid::new_v4().to_string(),
                context.research_session_id.clone(),
                "Asset will trade sideways.",
                supporting,
                contradicting,
                None::<String>,
                now,
            );
            hypotheses.push(thesis);
        }

        hypotheses
    }
}
