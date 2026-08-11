use crate::instrument::Instrument;
use crate::metrics::concepts::Concept;

/// Determines how an instrument should be evaluated, defining its reasoning intent.
pub trait EvaluationProfile {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    
    /// The high-level ontological concepts this profile cares about.
    fn active_concepts(&self) -> Vec<Concept>;
    
    /// Given a concept, returns the names of the specific metric models 
    /// that should be used as evidence providers for this concept.
    fn metrics_for_concept(&self, concept: &Concept) -> Vec<&'static str>;
}

/// A default profile suitable for generic large-cap equities.
pub struct LargeCapCoreProfile;

impl EvaluationProfile for LargeCapCoreProfile {
    fn name(&self) -> &str {
        "LargeCapCore"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }

    fn active_concepts(&self) -> Vec<Concept> {
        vec![
            Concept::Trend,
            Concept::Momentum,
            Concept::Volatility,
            Concept::Liquidity,
        ]
    }

    fn metrics_for_concept(&self, concept: &Concept) -> Vec<&'static str> {
        match concept {
            Concept::Trend => vec!["ma_20", "ma_50"],
            Concept::Momentum => vec!["roc_20"],
            Concept::Volatility => vec!["atr_14"],
            Concept::Liquidity => vec!["volume_20d"],
            _ => vec![],
        }
    }
}

/// A factory to assign profiles based on instrument classification.
pub struct ProfileAssigner;

impl ProfileAssigner {
    pub fn assign(instrument: &Instrument) -> Box<dyn EvaluationProfile> {
        // For now, default to LargeCapCore. Later this would inspect instrument sector/cap.
        let _ = instrument;
        Box::new(LargeCapCoreProfile)
    }
}
