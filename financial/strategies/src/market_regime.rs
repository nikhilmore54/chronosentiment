use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ScenarioContext {
    #[default]
    MeanReversion,
    BullTrend,
    BearTrend,
    HighVolatilityNoise,
    Unknown,
}

impl std::fmt::Display for ScenarioContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ScenarioContext::MeanReversion => "MeanReversion",
            ScenarioContext::BullTrend => "BullTrend",
            ScenarioContext::BearTrend => "BearTrend",
            ScenarioContext::HighVolatilityNoise => "HighVolatilityNoise",
            ScenarioContext::Unknown => "Unknown",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BehavioralArchetype {
    LongSpecialist,
    ShortSpecialist,
    DualCore,
    VolatilitySurfer,
}

#[inline]
pub fn classify_direction_bias(direction_bias: u8) -> BehavioralArchetype {
    if direction_bias > 70 {
        BehavioralArchetype::LongSpecialist
    } else if direction_bias < 30 {
        BehavioralArchetype::ShortSpecialist
    } else {
        BehavioralArchetype::DualCore
    }
}
