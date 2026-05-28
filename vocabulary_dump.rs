pub enum ScenarioContext {
    #[default]
    MeanReversion,
    BullTrend,
    BearTrend,
    HighVolatilityNoise,
}

impl std::fmt::Display for ScenarioContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ScenarioContext::MeanReversion => "MeanReversion",
            ScenarioContext::BullTrend => "BullTrend",
            ScenarioContext::BearTrend => "BearTrend",
            ScenarioContext::HighVolatilityNoise => "HighVolatilityNoise",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehavioralArchetype {
    LongSpecialist,
    ShortSpecialist,
    DualCore,
}

#[inline]
pub fn classify_direction_bias(direction_bias: u8) -> BehavioralArchetype {
    let funcs = crate::DOMAIN_DELEGATES.get().expect("Domain delegates uninitialized");
    (funcs.classify_behavior)(direction_bias)
}

#[inline]
pub fn regime_multiplier(regime: ScenarioContext, bias: BehavioralArchetype) -> f64 {
    let funcs = crate::DOMAIN_DELEGATES.get().expect("Domain delegates uninitialized");
    (funcs.scenario_multiplier)(regime, bias)
}

/// Institutional Regime Detector (Phase D.1.24)
#[inline]
pub fn detect_market_regime(price: f64, sma20: f64, momentum: f64, norm_vol: f64) -> ScenarioContext {
--
pub struct SignalAlphaReport {
    pub signal_idx: usize,
    pub asset: String,
    pub support_count: usize,
    pub support_ratio: f64,
    pub avg_score: f64,
    pub archetype_diversity: f64,
    pub alpha_score: f64,
    pub conviction: f64,
    pub archetypes: Vec<u8>,
    pub consensus_label: String, // "HIGH", "CROWDED", "NICHE"
    pub disagreement_entropy: f64,
    pub feature_diversity: f64,
    pub realized_edge_factor: f64,
    // Phase D.1.13 Temporal Layer
    pub signal_timestamp: u64,
    pub temporal_stability: f64,
    pub persistence_count: usize,
    pub alignment_factor: f64,
}

--
pub struct SignalAlpha {
    pub ts: usize,
    pub price: f64,
    pub archetype: u8,
    pub direction: i8,
    pub strength: f64,
    pub source: SignalSource,
    pub conviction: ConvictionOutcome,
    pub is_probe: bool,
}

fn extract_weak_signals(
    window_data: &Vec<(usize, ConvictionOutcome)>,
    candidate: &Candidate,
) -> Vec<(
    usize,
    ConvictionOutcome,
    f64,
    &'static str,
    f64,
    SignalSource,
