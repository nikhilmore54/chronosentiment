use chronosentiment_optimization::{ScenarioContext, BehavioralArchetype};
use std::f64;

pub fn classify_direction_bias(direction_bias: u8) -> BehavioralArchetype {
    match direction_bias {
        0..=25 => BehavioralArchetype::ShortSpecialist,
        75..=100 => BehavioralArchetype::LongSpecialist,
        _ => BehavioralArchetype::DualCore,
    }
}

#[inline]
pub fn regime_multiplier(regime: ScenarioContext, bias: BehavioralArchetype) -> f64 {
    match regime {
        ScenarioContext::BullTrend => match bias {
            BehavioralArchetype::LongSpecialist => 2.0,
            BehavioralArchetype::ShortSpecialist => 0.3, // Skeptic weight
            BehavioralArchetype::DualCore => 1.0,
        },
        ScenarioContext::BearTrend => match bias {
            BehavioralArchetype::ShortSpecialist => 2.0,
            BehavioralArchetype::LongSpecialist => 0.3, // Skeptic weight
            BehavioralArchetype::DualCore => 1.0,
        },
        ScenarioContext::MeanReversion => match bias {
            BehavioralArchetype::DualCore => 1.3,
            BehavioralArchetype::LongSpecialist => 0.7,
            BehavioralArchetype::ShortSpecialist => 0.7,
        },
        ScenarioContext::HighVolatilityNoise => 0.7, // Soft penalty for all
    }
}

/// Institutional Regime Detector (Phase D.1.24)
#[inline]
pub fn detect_market_regime(price: f64, sma20: f64, momentum: f64, norm_vol: f64) -> ScenarioContext {
    const HIGH_VOL_THRESHOLD: f64 = 0.005;
    const MOMENTUM_STRONG: f64 = 0.6;
    const TREND_STRENGTH_MIN: f64 = 0.55;
    const NEAR_SMA_EPS: f64 = 0.0015;

    if norm_vol > HIGH_VOL_THRESHOLD {
        return ScenarioContext::HighVolatilityNoise;
    }

    let dist = if sma20.abs() > f64::EPSILON {
        (price - sma20) / sma20
    } else {
        0.0
    };
    let trend_strength = 0.5 * momentum + 0.5 * dist.abs().min(1.0);

    if momentum > MOMENTUM_STRONG && trend_strength > TREND_STRENGTH_MIN {
        if price > sma20 {
            return ScenarioContext::BullTrend;
        } else if price < sma20 {
            return ScenarioContext::BearTrend;
        }
    }

    if dist.abs() < NEAR_SMA_EPS {
        return ScenarioContext::MeanReversion;
    }

    ScenarioContext::MeanReversion
}

pub fn calculate_capture_efficiency(realized: f64, expected: f64) -> f64 {
    let eps = 1e-6;
    let denom = if expected.abs() < eps {
        expected.signum() * eps
    } else {
        expected
    };
    (realized / denom).clamp(-2.0, 2.0)
}

pub fn classify_efficiency(e: f64) -> &'static str {
    if e > 1.0 {
        "OUTPERFORM"
    } else if e > 0.7 {
        "GOOD"
    } else if e > 0.3 {
        "DECAY"
    } else {
        "FAILED"
    }
}


pub fn initialize_ga_delegates() {
    chronosentiment_optimization::init_domain_delegates(chronosentiment_optimization::DomainDelegates {
        detect_scenario: detect_market_regime,
        scenario_multiplier: regime_multiplier,
        classify_behavior: classify_direction_bias,
        calculate_efficiency: calculate_capture_efficiency,
        classify_efficiency: classify_efficiency,
    });
}
