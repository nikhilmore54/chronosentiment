use crate::ga::MarketRegime;
use serde::{Deserialize, Serialize};

/// ChronoSentiment — Temporal Elasticity Timing Brain
/// Dynamically estimates the natural propagation half-life of an edge
/// based on the physical state and microstructure quality of the market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeHalfLifeEstimator {
    pub base_half_life_minutes: f64,
}

impl Default for EdgeHalfLifeEstimator {
    fn default() -> Self {
        Self {
            base_half_life_minutes: 75.0,
        }
    }
}

impl EdgeHalfLifeEstimator {
    /// Dynamically computes the expected lifespan of a trade in minutes
    /// based on the multi-dimensional structure of the entry signal.
    pub fn estimate_half_life(
        &self,
        path_coherence: f64,
        vol_persistence: f64,
        drift_toxicity: f64,
        regime: MarketRegime,
        execution_feasibility: f64,
    ) -> f64 {
        let mut half_life = self.base_half_life_minutes;

        // 1. Directional Coherence Multiplier
        // High coherence indicates strong persistent flow -> allow the trade to breathe.
        // Low coherence indicates choppy mean-reverting noise -> truncate trade lifespan.
        let coherence_mult = (path_coherence / 2.0).clamp(0.4, 2.5);
        half_life *= coherence_mult;

        // 2. Volatility Kinetic Persistence
        // Volatility persistence measures short-term vs long-term volatility expansion.
        // Expanding volatility supports ongoing drift -> increase breathing room.
        let vol_mult = if vol_persistence.is_finite() && vol_persistence > 0.0 {
            vol_persistence.clamp(0.5, 1.8)
        } else {
            1.0
        };
        half_life *= vol_mult;

        // 3. Drift Toxicity Penalty
        // Adversarial adverse price movement ratio. A toxic bleed kills the edge quickly.
        let toxicity_penalty = (1.2 - drift_toxicity).clamp(0.2, 1.2);
        half_life *= toxicity_penalty;

        // 4. Regime Honesty Multiplier
        // HighVolatilityNoise (divergence = 0.0) is structurally honest -> boost breathing room.
        // MeanReversion (deceptive chop) -> aggressive horizon contraction.
        let regime_mult = match regime {
            MarketRegime::HighVolatilityNoise => 1.3,
            MarketRegime::MeanReversion => 0.6,
            _ => 1.0,
        };
        half_life *= regime_mult;

        // 5. Execution Feasibility Support
        // Clean micro-execution environment validates strong participation.
        let feas_mult = (execution_feasibility * 1.5).clamp(0.7, 1.3);
        half_life *= feas_mult;

        // Absolute boundaries: clamp trade lifespans between 10 minutes and 4 hours
        half_life.clamp(10.0, 240.0)
    }
}
