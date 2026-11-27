//! RP-608-SCI-ADAPTER-REC-ML1D-001 — Dual-dimension ACT/PASS around sealed Coralys side.
//!
//! Reliability \(Q_t\) + economic \(U_t\) → deterministic ACT/PASS.
//! No side replacement · no sizing · no exit changes · Coralys crates untouched.
//! Hypotheses H0–H11 / experiments E0–E3 · extensions Ec2 / H9 / R3.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::sci_adapter_rec::{
    build_features, calibrate, fit_isotonic, fit_logistic_l2, raw_score, scorecard, wilson_lower,
    Band, FoldScorecard, IsotonicMap, JointCuts, LogisticModel, Side, FEATURE_SCHEMA_ID, L2_LAMBDA,
};

pub const PROTOCOL_ID: &str = "RP-608-SCI-ADAPTER-REC-ML1D-001";
pub const SCHEMA_VERSION: &str = "1.2";
pub const ML1D_FEATURE_SCHEMA_ID: &str = "ml1d_coralys_actpass_v1";

/// Predeclared Ec1 friction proxy (not a live venue table).
pub const FRICTION_BPS_FLOOR: f64 = 2.0;
pub const FRICTION_VOL_SCALE: f64 = 0.15;
/// Ec2: extra friction when volume rank is thin (1 − vol_rank20).
pub const FRICTION_LIQ_SCALE: f64 = 0.10;
/// Ec2: extra friction on |dlog_vol| shocks.
pub const FRICTION_DLOG_SCALE: f64 = 0.05;
/// H9: improper double-count weight mixing evidence margin back into U.
pub const H9_DOUBLE_COUNT_WEIGHT: f64 = 0.25;
/// R2 ACI target miscoverage (monitor, not Q̂ itself).
pub const R2_ALPHA_TARGET: f64 = 0.30;
/// R2 ACI step size (predeclared; Gibbs–Candès-style adaptation scale).
pub const R2_ACI_GAMMA: f64 = 0.05;
/// R2 static rolling window for Brier deterioration flag.
pub const R2_STATIC_WINDOW: usize = 200;
/// R2 flag when rolling/EWMA Brier exceeds validation baseline by this margin.
pub const R2_BRIER_DELTA: f64 = 0.03;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EconomicMode {
    /// Constant / vol_rel friction (Ec1).
    Ec1,
    /// Liquidity-conditioned friction via vol_rank20 + dlog_vol (Ec2).
    Ec2,
}

impl EconomicMode {
    pub fn as_str(self) -> &'static str {
        match self {
            EconomicMode::Ec1 => "EC1",
            EconomicMode::Ec2 => "EC2",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CombineMode {
    /// Proper dual: evidence_ok ∧ economic_ok (Q enters U once).
    Dual,
    /// Improper fused: ACT on U + λ·evidence_margin (double-counts Q) — H9.
    FusedDoubleCount,
}

impl CombineMode {
    pub fn as_str(self) -> &'static str {
        match self {
            CombineMode::Dual => "DUAL",
            CombineMode::FusedDoubleCount => "FUSED_DOUBLE_COUNT",
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
    pub economic_mode: String,
    pub combine_mode: String,
    pub coralys_side: String,
    pub q_hat: f64,
    pub q_lb_wilson: f64,
    pub u_hat: f64,
    pub fused_score: f64,
    pub friction_proxy: f64,
    pub evidence_ok: bool,
    pub economic_ok: bool,
    pub act: bool,
    pub band: Band,
    pub blocks_entry: bool,
    pub block_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionAwareDiag {
    pub fold: String,
    pub act_n: usize,
    pub act_empirical_wr: f64,
    pub act_mean_q: f64,
    pub act_mean_q_lb: f64,
    pub act_brier: f64,
    pub q_minus_wr: f64,
    pub note: String,
}

pub fn ml1d_schema_hash() -> String {
    let mut h = Sha256::new();
    h.update(ML1D_FEATURE_SCHEMA_ID.as_bytes());
    h.update(SCHEMA_VERSION.as_bytes());
    h.update(b"reuses:");
    h.update(FEATURE_SCHEMA_ID.as_bytes());
    h.update(b"|Qt+Ut|ACT_PASS|Ec2|H9|R3|R2");
    format!("{:x}", h.finalize())
}

/// Binary nonconformity for outcome vs calibrated Q (win → 1−q, loss → q).
pub fn binary_nonconformity(y: f64, q: f64) -> f64 {
    let q = q.clamp(0.0, 1.0);
    if y >= 0.5 {
        1.0 - q
    } else {
        q
    }
}

/// ACI-style adaptive threshold on nonconformity (uncertainty monitor for H5).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AciMonitor {
    pub alpha_target: f64,
    pub gamma: f64,
    /// Adaptive quantile level on nonconformity (higher → stricter PASS).
    pub level: f64,
    pub updates: usize,
}

impl AciMonitor {
    pub fn new(alpha_target: f64, gamma: f64) -> Self {
        Self {
            alpha_target,
            gamma,
            level: alpha_target.clamp(0.05, 0.95),
            updates: 0,
        }
    }

    pub fn default_r2() -> Self {
        Self::new(R2_ALPHA_TARGET, R2_ACI_GAMMA)
    }

    /// Update after outcome is revealed. `err=1` if nonconformity exceeded level.
    pub fn update(&mut self, y: f64, q: f64) {
        let s = binary_nonconformity(y, q);
        let err = if s > self.level { 1.0 } else { 0.0 };
        self.level = (self.level + self.gamma * (err - self.alpha_target)).clamp(0.05, 0.95);
        self.updates += 1;
    }

    /// Evidence still OK under adaptive nonconformity gate (stricter when drifting).
    pub fn accept_q(&self, q: f64) -> bool {
        // For prospective ACT: treat predicted win path nonconformity as 1−q.
        // Accept when 1−q ≤ level ⇔ q ≥ 1−level.
        q >= (1.0 - self.level)
    }
}

/// Static rolling Brier deterioration flag (comparator for H5 detection delay).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticBrierMonitor {
    pub window: usize,
    pub baseline_brier: f64,
    pub delta: f64,
    buf: Vec<f64>,
}

impl StaticBrierMonitor {
    pub fn new(baseline_brier: f64) -> Self {
        Self {
            window: R2_STATIC_WINDOW,
            baseline_brier,
            delta: R2_BRIER_DELTA,
            buf: Vec::with_capacity(R2_STATIC_WINDOW),
        }
    }

    pub fn update(&mut self, y: f64, q: f64) {
        let d = y - q;
        self.buf.push(d * d);
        if self.buf.len() > self.window {
            self.buf.remove(0);
        }
    }

    pub fn rolling_brier(&self) -> Option<f64> {
        if self.buf.len() < self.window / 2 {
            return None;
        }
        Some(self.buf.iter().sum::<f64>() / self.buf.len() as f64)
    }

    pub fn deteriorated(&self) -> bool {
        match self.rolling_brier() {
            Some(b) => b > self.baseline_brier + self.delta,
            None => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R2DriftReport {
    pub validation_baseline_brier: f64,
    pub holdout_static_flag_index: Option<usize>,
    pub holdout_aci_flag_index: Option<usize>,
    pub aci_detection_earlier: bool,
    pub holdout_high_vol_brier: f64,
    pub holdout_low_vol_brier: f64,
    pub e1_static: FoldScorecard,
    pub e1_r2_aci: FoldScorecard,
    pub h5_status: String,
    pub note: String,
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

/// Ec2: Ec1 base + thin-liquidity and volume-shock adders (decision-time only).
pub fn friction_proxy_ec2_pct(features: &[f64], risk_distance_pct: f64) -> f64 {
    let base = friction_proxy_pct(features, risk_distance_pct);
    let vol_rank = features.get(9).copied().unwrap_or(0.5).clamp(0.0, 1.0);
    let dlog = features.get(8).copied().unwrap_or(0.0).abs();
    let risk = risk_distance_pct.max(0.0);
    let liq_add = FRICTION_LIQ_SCALE * (1.0 - vol_rank) * risk;
    let shock_add = FRICTION_DLOG_SCALE * dlog * risk;
    (base + liq_add + shock_add).max(0.0)
}

pub fn friction_for_mode(
    mode: EconomicMode,
    features: &[f64],
    risk_distance_pct: f64,
) -> f64 {
    match mode {
        EconomicMode::Ec1 => friction_proxy_pct(features, risk_distance_pct),
        EconomicMode::Ec2 => friction_proxy_ec2_pct(features, risk_distance_pct),
    }
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

/// Improper H9 fused score: U already embeds Q; add λ·evidence margin again.
pub fn fused_double_count_score(q: f64, q_lb: f64, u: f64, cuts: &Ml1dCuts) -> f64 {
    let margin_q = q - cuts.reliability.p_min;
    let margin_lb = q_lb - cuts.reliability.p_lb;
    let evidence_margin = margin_q.min(margin_lb);
    u + H9_DOUBLE_COUNT_WEIGHT * evidence_margin
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
    decide_ml1d_ex(
        experiment,
        EconomicMode::Ec1,
        CombineMode::Dual,
        coralys_side,
        closes,
        volumes,
        long_pack,
        short_pack,
        cuts,
        target_distance_pct,
        risk_distance_pct,
    )
}

pub fn decide_ml1d_ex(
    experiment: Experiment,
    economic_mode: EconomicMode,
    combine_mode: CombineMode,
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
            economic_mode: economic_mode.as_str().into(),
            combine_mode: combine_mode.as_str().into(),
            coralys_side: coralys_side.as_str().into(),
            q_hat: 0.0,
            q_lb_wilson: 0.0,
            u_hat: 0.0,
            fused_score: 0.0,
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
    let friction = friction_for_mode(economic_mode, &feat.values, risk_distance_pct);
    let u = expected_net_utility(q, target_distance_pct, risk_distance_pct, friction);
    let fused = fused_double_count_score(q, q_lb, u, cuts);
    let evidence_ok = evidence_sufficient(q, q_lb, &cuts.reliability);
    let economic_ok = match combine_mode {
        CombineMode::Dual => u > cuts.u_min,
        CombineMode::FusedDoubleCount => fused > cuts.u_min,
    };

    let (act, reason) = match (experiment, combine_mode) {
        (Experiment::E0, _) => (true, None),
        (Experiment::E1, _) => {
            if evidence_ok {
                (true, None)
            } else {
                (false, Some("RELIABILITY_EVIDENCE_INSUFFICIENT".into()))
            }
        }
        (Experiment::E2, _) => {
            if economic_ok {
                (true, None)
            } else {
                (false, Some("ECONOMIC_UTILITY_BELOW_CUT".into()))
            }
        }
        (Experiment::E3, CombineMode::Dual) => {
            if evidence_ok && economic_ok {
                (true, None)
            } else if !evidence_ok {
                (false, Some("RELIABILITY_EVIDENCE_INSUFFICIENT".into()))
            } else {
                (false, Some("ECONOMIC_UTILITY_BELOW_CUT".into()))
            }
        }
        (Experiment::E3, CombineMode::FusedDoubleCount) => {
            // H9: single fused threshold only (no separate evidence ∧ U gate).
            if economic_ok {
                (true, None)
            } else {
                (false, Some("FUSED_SCORE_BELOW_CUT".into()))
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
        economic_mode: economic_mode.as_str().into(),
        combine_mode: combine_mode.as_str().into(),
        coralys_side: coralys_side.as_str().into(),
        q_hat: q,
        q_lb_wilson: q_lb,
        u_hat: u,
        fused_score: fused,
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

/// R3: selection-aware diagnostics on the ACT subset (naive vs selected).
pub fn selection_aware_diag(
    fold: &str,
    y: &[f64],
    q_hat: &[f64],
    q_lb: &[f64],
    act_mask: &[bool],
) -> SelectionAwareDiag {
    let mut n = 0usize;
    let mut wins = 0usize;
    let mut sum_q = 0.0;
    let mut sum_lb = 0.0;
    let mut brier = 0.0;
    for i in 0..y.len() {
        if !act_mask[i] {
            continue;
        }
        n += 1;
        let yi = y[i];
        let qi = q_hat.get(i).copied().unwrap_or(0.0);
        let lbi = q_lb.get(i).copied().unwrap_or(0.0);
        if yi >= 0.5 {
            wins += 1;
        }
        sum_q += qi;
        sum_lb += lbi;
        let d = yi - qi;
        brier += d * d;
    }
    let wr = if n == 0 { 0.0 } else { wins as f64 / n as f64 };
    let mean_q = if n == 0 { 0.0 } else { sum_q / n as f64 };
    let mean_lb = if n == 0 { 0.0 } else { sum_lb / n as f64 };
    let brier = if n == 0 { 0.0 } else { brier / n as f64 };
    SelectionAwareDiag {
        fold: fold.into(),
        act_n: n,
        act_empirical_wr: wr,
        act_mean_q: mean_q,
        act_mean_q_lb: mean_lb,
        act_brier: brier,
        q_minus_wr: mean_q - wr,
        note: "Naive ACT-subset mean Q vs empirical WR; large positive gap ⇒ overconfident selected statements (H6)."
            .into(),
    }
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
        assert_eq!(st.economic_mode, "EC1");
    }

    #[test]
    fn ec2_friction_ge_ec1() {
        let feat = vec![0.0; 13];
        // thin liquidity: vol_rank20 = 0
        let mut thin = feat.clone();
        thin[9] = 0.0;
        thin[7] = 1.0;
        let ec1 = friction_proxy_pct(&thin, 1.0);
        let ec2 = friction_proxy_ec2_pct(&thin, 1.0);
        assert!(ec2 >= ec1);
    }

    #[test]
    fn fused_double_count_moves_with_evidence() {
        let cuts = Ml1dCuts::default();
        let low = fused_double_count_score(0.20, 0.10, -0.01, &cuts);
        let high = fused_double_count_score(0.50, 0.40, -0.01, &cuts);
        assert!(high > low);
    }

    #[test]
    fn aci_tightens_on_persistent_errors() {
        let mut aci = AciMonitor::default_r2();
        let level0 = aci.level;
        for _ in 0..50 {
            // Always loss while q high → large nonconformity → err often 1
            aci.update(0.0, 0.8);
        }
        assert!(aci.level > level0);
    }

    #[test]
    fn static_brier_flags_when_worse_than_baseline() {
        let mut m = StaticBrierMonitor::new(0.10);
        for _ in 0..R2_STATIC_WINDOW {
            m.update(1.0, 0.0); // brier contrib 1.0
        }
        assert!(m.deteriorated());
    }
}
