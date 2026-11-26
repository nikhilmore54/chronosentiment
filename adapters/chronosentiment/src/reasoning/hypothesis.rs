use crate::metrics::concepts::Concept;
use crate::reasoning::evidence::EvidenceSet;

#[derive(Debug, Clone)]
pub struct CompetingHypotheses {
    pub hypotheses: Vec<EvaluatedHypothesis>,
}

#[derive(Debug, Clone)]
pub struct EvaluatedHypothesis {
    pub name: String,
    pub confidence: f64,
    pub supporting_evidence_count: usize,
    pub contradicting_evidence_count: usize,
}

pub trait Hypothesis {
    fn name(&self) -> String;
    fn supports(&self, evidence: &EvidenceSet) -> bool;
    fn contradicts(&self, evidence: &EvidenceSet) -> bool;
    fn confidence(&self, evidence: &EvidenceSet) -> f64;
    fn required_evidence(&self) -> Vec<Concept>;
}

pub struct TrendFollowingHypothesis;

impl Hypothesis for TrendFollowingHypothesis {
    fn name(&self) -> String {
        "Trend Continuation".to_string()
    }

    fn supports(&self, evidence: &EvidenceSet) -> bool {
        evidence
            .evidence
            .iter()
            .any(|e| e.concept == Concept::Trend && e.supports_continuation)
    }

    fn contradicts(&self, evidence: &EvidenceSet) -> bool {
        evidence
            .evidence
            .iter()
            .any(|e| e.concept == Concept::Trend && !e.supports_continuation)
    }

    fn confidence(&self, _evidence: &EvidenceSet) -> f64 {
        0.61 // Mocked complex logic
    }

    fn required_evidence(&self) -> Vec<Concept> {
        vec![Concept::Trend, Concept::Momentum]
    }
}

pub struct MeanReversionHypothesis;

impl Hypothesis for MeanReversionHypothesis {
    fn name(&self) -> String {
        "Mean Reversion".to_string()
    }

    fn supports(&self, evidence: &EvidenceSet) -> bool {
        evidence
            .evidence
            .iter()
            .any(|e| e.concept == Concept::Volatility && !e.supports_continuation)
    }

    fn contradicts(&self, evidence: &EvidenceSet) -> bool {
        evidence
            .evidence
            .iter()
            .any(|e| e.concept == Concept::Momentum && e.supports_continuation)
    }

    fn confidence(&self, _evidence: &EvidenceSet) -> f64 {
        0.24 // Mocked complex logic
    }

    fn required_evidence(&self) -> Vec<Concept> {
        vec![Concept::Volatility, Concept::Momentum]
    }
}

pub struct HypothesisEngine {
    hypotheses: Vec<Box<dyn Hypothesis>>,
}

impl HypothesisEngine {
    pub fn new() -> Self {
        Self {
            hypotheses: vec![
                Box::new(TrendFollowingHypothesis),
                Box::new(MeanReversionHypothesis),
            ],
        }
    }

    pub fn evaluate(&self, evidence: &EvidenceSet) -> CompetingHypotheses {
        let mut evaluated = Vec::new();

        for h in &self.hypotheses {
            let supports_count = if h.supports(evidence) { 1 } else { 0 };
            let contradicts_count = if h.contradicts(evidence) { 1 } else { 0 };

            evaluated.push(EvaluatedHypothesis {
                name: h.name(),
                confidence: h.confidence(evidence),
                supporting_evidence_count: supports_count,
                contradicting_evidence_count: contradicts_count,
            });
        }

        // Sort by confidence descending
        evaluated.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        CompetingHypotheses {
            hypotheses: evaluated,
        }
    }
}
