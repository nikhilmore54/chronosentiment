//! RP-608-SCI-ADAPTER-REC-ML1D-001 — Dual-dimension ACT/PASS around sealed Coralys side.
//!
//! Reliability \(Q_t\) + economic \(U_t\) → deterministic ACT/PASS.
//! No side replacement · no sizing · no exit changes · Coralys crates untouched.
//! Hypotheses H0–H11 / experiments E0–E3.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::sci_adapter_rec::{
    build_features, calibrate, fit_isotonic, fit_logistic_l2, raw_score, scorecard, wilson_lower,
    Band, FoldScorecard, IsotonicMap, JointCuts, LogisticModel, Side, FEATURE_SCHEMA_ID, L2_LAMBDA,
};

pub const PROTOCOL_ID: &str = "RP-608-SCI-ADAPTER-REC-ML1D-001";
pub const SCHEMA_VERSION: &str = "1";
pub const ML1D_FEATURE_SCHEMA_ID: &str = "ml1d_coralys_actpass_v1";

/// Predeclared Ec1 friction proxy (not a live venue table).
pub const FRICTION_BPS_FLOOR: f64 = 2.0;
pub const FRICTION_VOL_SCALE: f64 = 0.15;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Experiment {
    E0,
    E1,
    E2,
    E3,
}

impl Experiment {
    pub fn as_str(self) -> &'static str {
        match self {
            Experiment::E0 => "E0",
            Experiment::E1 => "E1",
            Experiment::E2 => "E2",
            Experiment::E3 => "E3",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityPack {
    pub side: String,
    pub glm: LogisticModel,
    pub isotonic: IsotonicMap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ml1dCuts {
    pub reliability: JointCuts,
    pub u_min: f64,
}

impl Default for Ml1dCuts {
    fn default() -> Self {
        Self {
            reliability: JointCuts {
                p_min: 0.30,
                p_lb: 0.18,
            },
            u_min: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ml1dFrozenPack {
    pub protocol_id: String,
    pub schema_version: String,
    pub feature_schema_id: String,
    pub schema_hash: String,
    pub long: ReliabilityPack,
    pub short: ReliabilityPack,
    pub cuts: Ml1dCuts,
    #[serde(default)]
    pub ladder_stage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ml1dStamp {
    pub protocol_id: String,
    pub schema_version: String,
    pub feature_schema_id: String,
    pub schema_hash: String,
    pub experiment: String,
    pub coralys_side: String,
    pub q_hat: f64,
    pub q_lb_wilson: f64,
    pub u_hat: f64,
    pub friction_proxy: f64,
    pub evidence_ok: bool,
    pub economic_ok: bool,
    pub act: bool,
    pub band: Band,
    pub blocks_entry: bool,
    pub block_reason: Option<String>,
}

pub fn ml1d_schema_hash() -> String {
    let mut h = Sha256::new();
    h.update(ML1D_FEATURE_SCHEMA_ID.as_bytes());
    h.update(SCHEMA_VERSION.as_bytes());
    h.update(b"reuses:");
    h.update(FEATURE_SCHEMA_ID.as_bytes());
    h.update(b"|Qt+Ut|ACT_PASS");
    format!("{:x}", h.finalize())
}

pub fn fit_reliability_pack(side: Side, xs: &[Vec<f64>], ys: &[f64]) -> ReliabilityPack {
    let glm = fit_logistic_l2(xs, ys, L2_LAMBDA);
    let scores: Vec<f64> = xs.iter().map(|x| raw_score(&glm, x)).collect();
    let isotonic = fit_isotonic(&scores, ys);
    ReliabilityPack {
        side: side.as_str().into(),
        glm,
        isotonic,
    }
}

pub fn refit_isotonic(pack: &ReliabilityPack, xs: &[Vec<f64>], ys: &[f64]) -> ReliabilityPack {
    let scores: Vec<f64> = xs.iter().map(|x| raw_score(&pack.glm, x)).collect();
    let isotonic = fit_isotonic(&scores, ys);
    ReliabilityPack {
        side: pack.side.clone(),
        glm: pack.glm.clone(),
        isotonic,
    }
}

pub fn predict_q(pack: &ReliabilityPack, features: &[f64]) -> (f64, f64) {
    let s = raw_score(&pack.glm, features);
    calibrate(&pack.isotonic, s)
}

pub fn friction_proxy_pct(features: &[f64], risk_distance_pct: f64) -> f64 {
    let vol_rel = features.get(7).copied().unwrap_or(0.0).abs();
    let floor = FRICTION_BPS_FLOOR / 100.0;
    let vol_add = FRICTION_VOL_SCALE * vol_rel * risk_distance_pct.max(0.0);
    (floor + vol_add).max(0.0)
}

pub fn expected_net_utility(
    q: f64,
    target_distance_pct: f64,
    risk_distance_pct: f64,
    friction_pct: f64,
) -> f64 {
    let q = q.clamp(0.0, 1.0);
    q * target_distance_pct.max(0.0) - (1.0 - q) * risk_distance_pct.max(0.0) - friction_pct
}

pub fn evidence_sufficient(q: f64, q_lb: f64, cuts: &JointCuts) -> bool {
    q >= cuts.p_min && q_lb >= cuts.p_lb
}

pub fn decide_ml1d(
    experiment: Experiment,
    coralys_side: Side,
    closes: &[f64],
    volumes: &[f64],
    long_pack: &ReliabilityPack,
    short_pack: &ReliabilityPack,
    cuts: &Ml1dCuts,
    target_distance_pct: f64,
    risk_distance_pct: f64,
) -> Ml1dStamp {
    let schema = ml1d_schema_hash();
    let feat = build_features(closes, volumes, coralys_side);
    if !feat.eligible {
        return Ml1dStamp {
            protocol_id: PROTOCOL_ID.into(),
            schema_version: SCHEMA_VERSION.into(),
            feature_schema_id: ML1D_FEATURE_SCHEMA_ID.into(),
            schema_hash: schema,
            experiment: experiment.as_str().into(),
            coralys_side: coralys_side.as_str().into(),
            q_hat: 0.0,
            q_lb_wilson: 0.0,
            u_hat: 0.0,
            friction_proxy: 0.0,
            evidence_ok: false,
            economic_ok: false,
            act: false,
            band: Band::Insufficient,
            blocks_entry: true,
            block_reason: Some("INSUFFICIENT_PRICE_VOLUME_HISTORY".into()),
        };
    }
    let pack = match coralys_side {
        Side::Long => long_pack,
        Side::Short => short_pack,
    };
    let (q, q_lb) = predict_q(pack, &feat.values);
    let friction = friction_proxy_pct(&feat.values, risk_distance_pct);
    let u = expected_net_utility(q, target_distance_pct, risk_distance_pct, friction);
    let evidence_ok = evidence_sufficient(q, q_lb, &cuts.reliability);
    let economic_ok = u > cuts.u_min;

    let (act, reason) = match experiment {
        Experiment::E0 => (true, None),
        Experiment::E1 => {
            if evidence_ok {
                (true, None)
            } else {
                (false, Some("RELIABILITY_EVIDENCE_INSUFFICIENT".into()))
            }
        }
        Experiment::E2 => {
            if economic_ok {
                (true, None)
            } else {
                (false, Some("ECONOMIC_UTILITY_NONPOSITIVE".into()))
            }
        }
        Experiment::E3 => {
            if evidence_ok && economic_ok {
                (true, None)
            } else if !evidence_ok {
                (false, Some("RELIABILITY_EVIDENCE_INSUFFICIENT".into()))
            } else {
                (false, Some("ECONOMIC_UTILITY_NONPOSITIVE".into()))
            }
        }
    };

    let band = if act { Band::Enter } else { Band::Standaside };
    Ml1dStamp {
        protocol_id: PROTOCOL_ID.into(),
        schema_version: SCHEMA_VERSION.into(),
        feature_schema_id: ML1D_FEATURE_SCHEMA_ID.into(),
        schema_hash: schema,
        experiment: experiment.as_str().into(),
        coralys_side: coralys_side.as_str().into(),
        q_hat: q,
        q_lb_wilson: q_lb,
        u_hat: u,
        friction_proxy: friction,
        evidence_ok,
        economic_ok,
        act,
        band,
        blocks_entry: !act,
        block_reason: reason,
    }
}

pub fn scorecard_experiment(
    fold: &str,
    y: &[f64],
    act_mask: &[bool],
    net_proxy: &[f64],
) -> FoldScorecard {
    scorecard(fold, y, act_mask, net_proxy)
}

pub fn wilson_wr_lb(wins: usize, n: usize) -> f64 {
    wilson_lower(wins as u64, n as u64, 0.95)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utility_positive_when_high_q() {
        let u = expected_net_utility(0.8, 1.0, 0.5, 0.05);
        assert!(u > 0.0);
    }

    #[test]
    fn utility_negative_when_low_q() {
        let u = expected_net_utility(0.2, 1.0, 0.5, 0.05);
        assert!(u < 0.0);
    }

    #[test]
    fn e0_always_acts_when_features_ok() {
        let closes: Vec<f64> = (0..80).map(|i| 100.0 + (i as f64) * 0.1).collect();
        let vols: Vec<f64> = (0..80).map(|i| 10.0 + (i % 5) as f64).collect();
        let pack = ReliabilityPack {
            side: "LONG".into(),
            glm: fit_logistic_l2(&[vec![0.0; 13], vec![1.0; 13]], &[0.0, 1.0], 1.0),
            isotonic: fit_isotonic(&[0.0, 1.0], &[0.0, 1.0]),
        };
        let cuts = Ml1dCuts::default();
        let st = decide_ml1d(
            Experiment::E0,
            Side::Long,
            &closes,
            &vols,
            &pack,
            &pack,
            &cuts,
            1.0,
            0.5,
        );
        assert!(st.act);
        assert_eq!(st.coralys_side, "LONG");
    }
}
