//! RP-608-ADAPTER-SELECT-15M-V1
//!
//! Clean-slate multi-lookback soft score on **completed** 15m closes.
//! Adapter-owned entry selection (D1=a filter on Coralys/Estimand candidate side).
//! Does not touch `coralys-*`. Does not change frozen traj15 / OF ±0.30 / stops.
//!
//! Frozen enter band (D2): `|S_side| ≥ 0.35` → ENTER; else STANDASIDE.
//! Insufficient history → INSUFFICIENT (fail-closed standaside).

use serde::{Deserialize, Serialize};

pub const PROTOCOL_ID: &str = "RP-608-ADAPTER-SELECT-15M-V1";
pub const SCHEMA_VERSION: &str = "1";
/// Frozen D2 enter band (absolute side-aligned score).
pub const ENTER_ABS: f64 = 0.35;
pub const LOOKBACKS: [usize; 3] = [3, 8, 20];
/// Soft-agreement weights for L3 / L8 / L20 (sum = 1.0).
pub const LOOKBACK_WEIGHTS: [f64; 3] = [0.45, 0.35, 0.20];
/// Prior completed bars used for |r_k| magnitude ranks.
pub const RANK_WINDOW: usize = 50;
/// Absolute-return saturation scale (~2% → ~0.5 of sat term).
pub const RET_SCALE: f64 = 0.02;
/// Blend: mag_rank vs absolute-return saturation inside each lookback term.
const MAG_RANK_BLEND: f64 = 0.35;
const PATH_BLEND: f64 = 0.15;
const SOFT_BLEND: f64 = 0.85;
const EPS: f64 = 1e-9;
const FLAT_EPS: f64 = 1e-12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CandidateSide {
    Long,
    Short,
    Flat,
}

impl CandidateSide {
    pub fn as_str(self) -> &'static str {
        match self {
            CandidateSide::Long => "LONG",
            CandidateSide::Short => "SHORT",
            CandidateSide::Flat => "FLAT",
        }
    }

    pub fn parse(side: &str) -> Self {
        match side.trim().to_ascii_uppercase().as_str() {
            "LONG" => CandidateSide::Long,
            "SHORT" => CandidateSide::Short,
            _ => CandidateSide::Flat,
        }
    }

    fn sign(self) -> Option<f64> {
        match self {
            CandidateSide::Long => Some(1.0),
            CandidateSide::Short => Some(-1.0),
            CandidateSide::Flat => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecommendationBand {
    Enter,
    Standaside,
    Insufficient,
}

impl RecommendationBand {
    pub fn as_str(self) -> &'static str {
        match self {
            RecommendationBand::Enter => "ENTER",
            RecommendationBand::Standaside => "STANDASIDE",
            RecommendationBand::Insufficient => "INSUFFICIENT",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LookbackFeature {
    pub lookback: usize,
    pub ret: Option<f64>,
    pub mag_rank: Option<f64>,
    pub signed_mag: Option<f64>,
    pub eligible: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecommendationStamp {
    pub protocol_id: String,
    pub schema_version: String,
    pub candidate_side: String,
    pub score: f64,
    pub side_score: f64,
    pub enter_abs: f64,
    pub band: RecommendationBand,
    pub blocks_entry: bool,
    pub block_reason: Option<String>,
    pub path_consistency: Option<f64>,
    pub features: Vec<LookbackFeature>,
    pub n_closes: usize,
}

fn signed_return(closes: &[f64], lookback: usize) -> Option<f64> {
    if lookback == 0 || closes.len() < lookback {
        return None;
    }
    let c1 = closes[closes.len() - 1];
    let c0 = closes[closes.len() - lookback];
    Some((c1 - c0) / (c0.abs() + EPS))
}

/// Rank of `|cur|` among up to `RANK_WINDOW` prior completed lookback returns
/// ending at bars before the decision bar. Returns mid-rank 0.5 when no priors.
fn magnitude_rank(closes: &[f64], lookback: usize, cur_abs: f64) -> f64 {
    // Prior end indices: decision end = n-1; prior ends = n-2, n-3, ... while
    // lookback fits: end+1 >= lookback ⇒ end >= lookback-1.
    let n = closes.len();
    if n < lookback + 1 {
        return 0.5;
    }
    let mut priors: Vec<f64> = Vec::with_capacity(RANK_WINDOW);
    let mut end = n - 2; // one bar before decision
    while priors.len() < RANK_WINDOW {
        if end + 1 < lookback {
            break;
        }
        let c1 = closes[end];
        let c0 = closes[end + 1 - lookback];
        let r = (c1 - c0) / (c0.abs() + EPS);
        priors.push(r.abs());
        if end == 0 {
            break;
        }
        end -= 1;
    }
    if priors.is_empty() {
        return 0.5;
    }
    let below = priors.iter().filter(|&&p| p < cur_abs).count() as f64;
    below / (priors.len() as f64)
}

fn path_consistency(closes: &[f64], r20: f64) -> f64 {
    // Fraction of consecutive close steps over the L20 window that move with r20.
    if closes.len() < 20 || r20.abs() <= FLAT_EPS {
        return 0.5;
    }
    let want_up = r20 > 0.0;
    let start = closes.len() - 20;
    let mut agree = 0u32;
    let mut total = 0u32;
    for i in start..(closes.len() - 1) {
        let step = closes[i + 1] - closes[i];
        if step.abs() <= FLAT_EPS {
            total += 1;
            continue;
        }
        total += 1;
        if (step > 0.0) == want_up {
            agree += 1;
        }
    }
    if total == 0 {
        return 0.5;
    }
    agree as f64 / total as f64
}

fn clamp_unit(x: f64) -> f64 {
    if x > 1.0 {
        1.0
    } else if x < -1.0 {
        -1.0
    } else {
        x
    }
}

/// Pure scorer: completed 15m closes + Coralys/Estimand candidate side → stamp.
pub fn recommend(closes: &[f64], candidate_side: &str) -> RecommendationStamp {
    let side = CandidateSide::parse(candidate_side);
    let n_closes = closes.len();

    // Required lookbacks must all be eligible → else INSUFFICIENT.
    let max_lb = *LOOKBACKS.iter().max().unwrap_or(&20);
    if n_closes < max_lb {
        return RecommendationStamp {
            protocol_id: PROTOCOL_ID.to_string(),
            schema_version: SCHEMA_VERSION.to_string(),
            candidate_side: side.as_str().to_string(),
            score: 0.0,
            side_score: 0.0,
            enter_abs: ENTER_ABS,
            band: RecommendationBand::Insufficient,
            blocks_entry: true,
            block_reason: Some("INSUFFICIENT_LOOKBACK_HISTORY".to_string()),
            path_consistency: None,
            features: LOOKBACKS
                .iter()
                .map(|&k| LookbackFeature {
                    lookback: k,
                    ret: None,
                    mag_rank: None,
                    signed_mag: None,
                    eligible: false,
                })
                .collect(),
            n_closes,
        };
    }

    if side.sign().is_none() {
        return RecommendationStamp {
            protocol_id: PROTOCOL_ID.to_string(),
            schema_version: SCHEMA_VERSION.to_string(),
            candidate_side: side.as_str().to_string(),
            score: 0.0,
            side_score: 0.0,
            enter_abs: ENTER_ABS,
            band: RecommendationBand::Standaside,
            blocks_entry: true,
            block_reason: Some("CANDIDATE_SIDE_NOT_TRADEABLE".to_string()),
            path_consistency: None,
            features: LOOKBACKS
                .iter()
                .map(|&k| {
                    let ret = signed_return(closes, k);
                    LookbackFeature {
                        lookback: k,
                        ret,
                        mag_rank: None,
                        signed_mag: None,
                        eligible: ret.is_some(),
                    }
                })
                .collect(),
            n_closes,
        };
    }

    let mut features = Vec::with_capacity(LOOKBACKS.len());
    let mut soft = 0.0;
    for (i, &k) in LOOKBACKS.iter().enumerate() {
        let ret = signed_return(closes, k).expect("history gated above");
        let mag = magnitude_rank(closes, k, ret.abs());
        // Absolute-return saturation keeps mature trends from collapsing when
        // fractional |r_k| ranks low vs earlier (cheaper) bars in a drift.
        let sat = ret.abs() / (ret.abs() + RET_SCALE);
        let strength = MAG_RANK_BLEND * mag + (1.0 - MAG_RANK_BLEND) * sat;
        let sign = if ret > FLAT_EPS {
            1.0
        } else if ret < -FLAT_EPS {
            -1.0
        } else {
            0.0
        };
        let signed_mag = sign * strength;
        soft += LOOKBACK_WEIGHTS[i] * signed_mag;
        features.push(LookbackFeature {
            lookback: k,
            ret: Some(ret),
            mag_rank: Some(mag),
            signed_mag: Some(signed_mag),
            eligible: true,
        });
    }

    let r20 = features
        .iter()
        .find(|f| f.lookback == 20)
        .and_then(|f| f.ret)
        .unwrap_or(0.0);
    let path = path_consistency(closes, r20);
    let path_signed = if r20 > FLAT_EPS {
        path
    } else if r20 < -FLAT_EPS {
        -path
    } else {
        0.0
    };
    let score = clamp_unit(SOFT_BLEND * soft + PATH_BLEND * path_signed);
    let side_sign = side.sign().unwrap();
    let side_score = score * side_sign;

    let (band, blocks, reason) = if side_score >= ENTER_ABS {
        (RecommendationBand::Enter, false, None)
    } else {
        (
            RecommendationBand::Standaside,
            true,
            Some("SIDE_SCORE_BELOW_ENTER_BAND".to_string()),
        )
    };

    RecommendationStamp {
        protocol_id: PROTOCOL_ID.to_string(),
        schema_version: SCHEMA_VERSION.to_string(),
        candidate_side: side.as_str().to_string(),
        score,
        side_score,
        enter_abs: ENTER_ABS,
        band,
        blocks_entry: blocks,
        block_reason: reason,
        path_consistency: Some(path),
        features,
        n_closes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rising(n: usize) -> Vec<f64> {
        (0..n).map(|i| 100.0 + i as f64).collect()
    }

    fn falling(n: usize) -> Vec<f64> {
        (0..n).map(|i| 200.0 - i as f64).collect()
    }

    #[test]
    fn insufficient_history() {
        let stamp = recommend(&[100.0, 101.0], "LONG");
        assert!(stamp.blocks_entry);
        assert_eq!(stamp.band, RecommendationBand::Insufficient);
        assert_eq!(
            stamp.block_reason.as_deref(),
            Some("INSUFFICIENT_LOOKBACK_HISTORY")
        );
    }

    #[test]
    fn rising_allows_long() {
        let stamp = recommend(&rising(80), "LONG");
        assert!(!stamp.blocks_entry, "{stamp:?}");
        assert_eq!(stamp.band, RecommendationBand::Enter);
        assert!(stamp.side_score >= ENTER_ABS, "side={}", stamp.side_score);
        assert!(stamp.score > 0.0);
    }

    #[test]
    fn rising_blocks_short() {
        let stamp = recommend(&rising(80), "SHORT");
        assert!(stamp.blocks_entry);
        assert_eq!(stamp.band, RecommendationBand::Standaside);
        assert!(stamp.side_score < ENTER_ABS);
        assert_eq!(
            stamp.block_reason.as_deref(),
            Some("SIDE_SCORE_BELOW_ENTER_BAND")
        );
    }

    #[test]
    fn falling_allows_short() {
        let stamp = recommend(&falling(80), "SHORT");
        assert!(!stamp.blocks_entry, "{stamp:?}");
        assert_eq!(stamp.band, RecommendationBand::Enter);
        assert!(stamp.side_score >= ENTER_ABS);
        assert!(stamp.score < 0.0);
    }

    #[test]
    fn flat_candidate_standaside() {
        let stamp = recommend(&rising(80), "NO_TRADE");
        assert!(stamp.blocks_entry);
        assert_eq!(
            stamp.block_reason.as_deref(),
            Some("CANDIDATE_SIDE_NOT_TRADEABLE")
        );
    }

    #[test]
    fn deterministic_same_input() {
        let closes = rising(80);
        let a = recommend(&closes, "LONG");
        let b = recommend(&closes, "LONG");
        assert_eq!(a.score, b.score);
        assert_eq!(a.side_score, b.side_score);
        assert_eq!(a.band, b.band);
        assert_eq!(a.features, b.features);
    }

    #[test]
    fn no_partial_bar_leakage() {
        let completed = rising(40);
        let mut with_forming = completed.clone();
        with_forming.push(0.0);
        let a = recommend(&completed, "LONG");
        let b = recommend(&with_forming[..with_forming.len() - 1], "LONG");
        assert_eq!(a.score, b.score);
        assert_eq!(a.band, b.band);
    }

    #[test]
    fn enter_abs_frozen() {
        assert!((ENTER_ABS - 0.35).abs() < 1e-15);
    }

    #[test]
    fn noisy_flat_standaside() {
        // Oscillating closes → weak |S|; should not ENTER.
        let mut closes = Vec::new();
        for i in 0..80 {
            closes.push(100.0 + if i % 2 == 0 { 0.01 } else { -0.01 });
        }
        let stamp = recommend(&closes, "LONG");
        assert!(stamp.blocks_entry, "{stamp:?}");
        assert_ne!(stamp.band, RecommendationBand::Enter);
    }
}
