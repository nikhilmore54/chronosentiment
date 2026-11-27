//! RP-608-SCI-ADAPTER-REC-001 — P1 offline scientific recommendation kernel.
//!
//! Adapter-owned. Coralys no-touch. Deterministic. Books never pooled.
//! Price + volume features on completed 15m bars; calibrated GLM + isotonic;
//! joint ENTER on p̂ + Wilson CI lower bound (S1=b · S2 · S3 · S10).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const PROTOCOL_ID: &str = "RP-608-SCI-ADAPTER-REC-001";
pub const SCHEMA_VERSION: &str = "1";
pub const FEATURE_SCHEMA_ID: &str = "price_volume_15m_v1";
pub const LOOKBACKS: [usize; 3] = [3, 8, 20];
pub const RANK_WINDOW: usize = 50;
pub const VOL_MEAN_WINDOW: usize = 20;
pub const MAX_FEATURES: usize = 13;
pub const LABEL_HORIZON_BARS: usize = 10;
pub const PURGE_BARS: usize = 30; // lookback 20 + label horizon 10
pub const L2_LAMBDA: f64 = 1.0;
pub const ELASTIC_L1: f64 = 0.05;
pub const ELASTIC_L2: f64 = 1.0;
pub const EPS: f64 = 1e-12;
pub const FLAT_EPS: f64 = 1e-12;

pub const FEATURE_NAMES: [&str; MAX_FEATURES] = [
    "side_r3",
    "side_r8",
    "side_r20",
    "mag_rank3",
    "mag_rank8",
    "mag_rank20",
    "side_path20",
    "vol_rel20",
    "dlog_vol",
    "vol_rank20",
    "abs_r3",
    "abs_r8",
    "abs_r20",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Side {
    Long,
    Short,
}

impl Side {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_uppercase().as_str() {
            "LONG" => Some(Side::Long),
            "SHORT" => Some(Side::Short),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Side::Long => "LONG",
            Side::Short => "SHORT",
        }
    }

    pub fn sign(self) -> f64 {
        match self {
            Side::Long => 1.0,
            Side::Short => -1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Band {
    Enter,
    Standaside,
    Insufficient,
}

impl Band {
    pub fn as_str(self) -> &'static str {
        match self {
            Band::Enter => "ENTER",
            Band::Standaside => "STANDASIDE",
            Band::Insufficient => "INSUFFICIENT",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVector {
    pub values: [f64; MAX_FEATURES],
    pub eligible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogisticModel {
    pub intercept: f64,
    pub weights: Vec<f64>,
    pub feature_means: Vec<f64>,
    pub feature_stds: Vec<f64>,
    pub ladder_stage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsotonicMap {
    /// Sorted unique raw scores (x) and calibrated probabilities (y).
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    /// Per-bin counts for Wilson CI (aligned to x after collapse).
    pub bin_wins: Vec<u64>,
    pub bin_n: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointCuts {
    pub p_min: f64,
    pub p_lb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideModelPack {
    pub side: String,
    pub glm: LogisticModel,
    pub isotonic: IsotonicMap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SciRecStamp {
    pub protocol_id: String,
    pub schema_version: String,
    pub feature_schema_id: String,
    pub schema_hash: String,
    pub ladder_stage: String,
    pub side: String,
    pub p_hat: f64,
    pub p_lb_wilson: f64,
    pub p_long: f64,
    pub p_short: f64,
    pub p_lb_long: f64,
    pub p_lb_short: f64,
    pub band: Band,
    pub blocks_entry: bool,
    pub block_reason: Option<String>,
    pub p_min: f64,
    pub p_lb_cut: f64,
}

fn signed_return(closes: &[f64], lookback: usize) -> Option<f64> {
    if lookback == 0 || closes.len() < lookback {
        return None;
    }
    let c1 = closes[closes.len() - 1];
    let c0 = closes[closes.len() - lookback];
    Some((c1 - c0) / (c0.abs() + EPS))
}

fn magnitude_rank(closes: &[f64], lookback: usize, cur_abs: f64) -> f64 {
    let n = closes.len();
    if n < lookback + 1 {
        return 0.5;
    }
    let mut priors = Vec::with_capacity(RANK_WINDOW);
    let mut end = n - 2;
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
    below / priors.len() as f64
}

fn path_consistency(closes: &[f64], r20: f64) -> f64 {
    if closes.len() < 20 || r20.abs() <= FLAT_EPS {
        return 0.5;
    }
    let want_up = r20 > 0.0;
    let start = closes.len() - 20;
    let mut agree = 0u32;
    let mut total = 0u32;
    for i in start..(closes.len() - 1) {
        let step = closes[i + 1] - closes[i];
        total += 1;
        if step.abs() <= FLAT_EPS {
            continue;
        }
        if (step > 0.0) == want_up {
            agree += 1;
        }
    }
    if total == 0 {
        0.5
    } else {
        agree as f64 / total as f64
    }
}

fn volume_features(volumes: &[f64]) -> Option<(f64, f64, f64)> {
    if volumes.len() < VOL_MEAN_WINDOW {
        return None;
    }
    let v1 = volumes[volumes.len() - 1];
    let window = &volumes[volumes.len() - VOL_MEAN_WINDOW..];
    let mean = window.iter().sum::<f64>() / VOL_MEAN_WINDOW as f64;
    let vol_rel = v1 / (mean + EPS) - 1.0;
    let v0 = volumes[volumes.len() - 2];
    let dlog = (v1 + EPS).ln() - (v0 + EPS).ln();
    // rank of current volume vs prior 50 completed volumes (excluding current)
    let mut priors = Vec::with_capacity(RANK_WINDOW);
    let mut i = volumes.len().saturating_sub(2);
    while priors.len() < RANK_WINDOW {
        priors.push(volumes[i]);
        if i == 0 {
            break;
        }
        i -= 1;
    }
    let below = priors.iter().filter(|&&p| p < v1).count() as f64;
    let vol_rank = if priors.is_empty() {
        0.5
    } else {
        below / priors.len() as f64
    };
    Some((vol_rel, dlog, vol_rank))
}

/// Decision-time price+volume features aligned to `side` (S1=b / S10).
pub fn build_features(closes: &[f64], volumes: &[f64], side: Side) -> FeatureVector {
    let max_lb = *LOOKBACKS.iter().max().unwrap_or(&20);
    if closes.len() < max_lb || volumes.len() < VOL_MEAN_WINDOW || closes.len() != volumes.len() {
        return FeatureVector {
            values: [0.0; MAX_FEATURES],
            eligible: false,
        };
    }
    let Some((vol_rel, dlog, vol_rank)) = volume_features(volumes) else {
        return FeatureVector {
            values: [0.0; MAX_FEATURES],
            eligible: false,
        };
    };
    let s = side.sign();
    let mut values = [0.0; MAX_FEATURES];
    for (i, &k) in LOOKBACKS.iter().enumerate() {
        let r = signed_return(closes, k).unwrap_or(0.0);
        values[i] = r * s;
        values[3 + i] = magnitude_rank(closes, k, r.abs());
        values[10 + i] = r.abs();
    }
    let r20 = signed_return(closes, 20).unwrap_or(0.0);
    let path = path_consistency(closes, r20);
    let path_signed = if r20.abs() <= FLAT_EPS {
        0.0
    } else if r20 > 0.0 {
        path
    } else {
        -path
    };
    values[6] = path_signed * s;
    values[7] = vol_rel;
    values[8] = dlog;
    values[9] = vol_rank;
    FeatureVector {
        values,
        eligible: true,
    }
}

pub fn schema_hash() -> String {
    let mut h = Sha256::new();
    h.update(FEATURE_SCHEMA_ID.as_bytes());
    h.update(SCHEMA_VERSION.as_bytes());
    for n in FEATURE_NAMES {
        h.update(n.as_bytes());
    }
    format!("{:x}", h.finalize())
}

fn sigmoid(z: f64) -> f64 {
    if z >= 0.0 {
        let e = (-z).exp();
        1.0 / (1.0 + e)
    } else {
        let e = z.exp();
        e / (1.0 + e)
    }
}

fn standardize_fit(xs: &[Vec<f64>]) -> (Vec<f64>, Vec<f64>) {
    let d = xs[0].len();
    let n = xs.len() as f64;
    let mut means = vec![0.0; d];
    let mut stds = vec![0.0; d];
    for row in xs {
        for (j, &v) in row.iter().enumerate() {
            means[j] += v;
        }
    }
    for m in &mut means {
        *m /= n;
    }
    for row in xs {
        for (j, &v) in row.iter().enumerate() {
            let dlt = v - means[j];
            stds[j] += dlt * dlt;
        }
    }
    for s in &mut stds {
        *s = (*s / n).sqrt().max(1e-8);
    }
    (means, stds)
}

fn apply_standardize(x: &[f64], means: &[f64], stds: &[f64]) -> Vec<f64> {
    x.iter()
        .enumerate()
        .map(|(j, &v)| (v - means[j]) / stds[j])
        .collect()
}

/// L2-regularized logistic regression via IRLS (deterministic).
pub fn fit_logistic_l2(xs: &[Vec<f64>], ys: &[f64], l2: f64) -> LogisticModel {
    assert!(!xs.is_empty());
    assert_eq!(xs.len(), ys.len());
    let (means, stds) = standardize_fit(xs);
    let d = xs[0].len();
    let mut w = vec![0.0; d];
    let mut b = 0.0;
    let n = xs.len();
    for _ in 0..80 {
        let mut grad_w = vec![0.0; d];
        let mut grad_b = 0.0;
        let mut hess_w = vec![0.0; d];
        let mut hess_b = 0.0;
        for i in 0..n {
            let zrow = apply_standardize(&xs[i], &means, &stds);
            let mut z = b;
            for j in 0..d {
                z += w[j] * zrow[j];
            }
            let p = sigmoid(z);
            let r = p - ys[i];
            let s = p * (1.0 - p);
            grad_b += r;
            hess_b += s;
            for j in 0..d {
                grad_w[j] += r * zrow[j];
                hess_w[j] += s * zrow[j] * zrow[j];
            }
        }
        // L2 on weights only
        for j in 0..d {
            grad_w[j] += l2 * w[j];
            hess_w[j] += l2;
        }
        b -= grad_b / (hess_b + 1e-8);
        for j in 0..d {
            w[j] -= grad_w[j] / (hess_w[j] + 1e-8);
        }
    }
    LogisticModel {
        intercept: b,
        weights: w,
        feature_means: means,
        feature_stds: stds,
        ladder_stage: "A".into(),
    }
}

/// Elastic-net-ish logistic via proximal IRLS (Stage B ladder).
pub fn fit_logistic_elastic(xs: &[Vec<f64>], ys: &[f64], l1: f64, l2: f64) -> LogisticModel {
    let mut m = fit_logistic_l2(xs, ys, l2);
    // soft-threshold weights
    for w in &mut m.weights {
        let sign = if *w >= 0.0 { 1.0 } else { -1.0 };
        let a = w.abs() - l1;
        *w = if a > 0.0 { sign * a } else { 0.0 };
    }
    // refit L2 with fixed support mask (one more IRLS pass set)
    let d = m.weights.len();
    let support: Vec<bool> = m.weights.iter().map(|w| w.abs() > 1e-12).collect();
    let (means, stds) = (m.feature_means.clone(), m.feature_stds.clone());
    let mut w = m.weights.clone();
    let mut b = m.intercept;
    let n = xs.len();
    for _ in 0..40 {
        let mut grad_w = vec![0.0; d];
        let mut grad_b = 0.0;
        let mut hess_w = vec![0.0; d];
        let mut hess_b = 0.0;
        for i in 0..n {
            let zrow = apply_standardize(&xs[i], &means, &stds);
            let mut z = b;
            for j in 0..d {
                if support[j] {
                    z += w[j] * zrow[j];
                }
            }
            let p = sigmoid(z);
            let r = p - ys[i];
            let s = p * (1.0 - p);
            grad_b += r;
            hess_b += s;
            for j in 0..d {
                if !support[j] {
                    continue;
                }
                grad_w[j] += r * zrow[j];
                hess_w[j] += s * zrow[j] * zrow[j];
            }
        }
        for j in 0..d {
            if !support[j] {
                w[j] = 0.0;
                continue;
            }
            grad_w[j] += l2 * w[j];
            hess_w[j] += l2;
            w[j] -= grad_w[j] / (hess_w[j] + 1e-8);
        }
        b -= grad_b / (hess_b + 1e-8);
    }
    LogisticModel {
        intercept: b,
        weights: w,
        feature_means: means,
        feature_stds: stds,
        ladder_stage: "B".into(),
    }
}

pub fn raw_score(model: &LogisticModel, x: &[f64]) -> f64 {
    let zrow = apply_standardize(x, &model.feature_means, &model.feature_stds);
    let mut z = model.intercept;
    for (j, &v) in zrow.iter().enumerate() {
        z += model.weights[j] * v;
    }
    z
}

pub fn predict_proba_raw(model: &LogisticModel, x: &[f64]) -> f64 {
    sigmoid(raw_score(model, x))
}

/// Pool Adjacent Violators isotonic regression (non-decreasing) on sorted pairs.
pub fn fit_isotonic(scores: &[f64], ys: &[f64]) -> IsotonicMap {
    assert_eq!(scores.len(), ys.len());
    let mut idx: Vec<usize> = (0..scores.len()).collect();
    idx.sort_by(|&a, &b| {
        scores[a]
            .partial_cmp(&scores[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut blocks: Vec<(f64, f64, u64, u64)> = Vec::new(); // sum_y, sum_n-weight as n, wins, n
    for &i in &idx {
        let y = ys[i];
        blocks.push((y, 1.0, if y >= 0.5 { 1 } else { 0 }, 1));
        while blocks.len() >= 2 {
            let n = blocks.len();
            let mean_prev = blocks[n - 2].0 / blocks[n - 2].1;
            let mean_cur = blocks[n - 1].0 / blocks[n - 1].1;
            if mean_prev <= mean_cur + 1e-15 {
                break;
            }
            let b2 = blocks.pop().unwrap();
            let b1 = blocks.pop().unwrap();
            blocks.push((b1.0 + b2.0, b1.1 + b2.1, b1.2 + b2.2, b1.3 + b2.3));
        }
    }
    // Expand to per-point then collapse unique x
    let mut x = Vec::new();
    let mut y = Vec::new();
    let mut bin_wins = Vec::new();
    let mut bin_n = Vec::new();
    let mut cursor = 0usize;
    for block in &blocks {
        let mean = block.0 / block.1;
        let count = block.1 as usize;
        let start = cursor;
        let end = cursor + count;
        let mut sx = 0.0;
        for &i in &idx[start..end] {
            sx += scores[i];
        }
        let mx = sx / count as f64;
        x.push(mx);
        y.push(mean.clamp(1e-6, 1.0 - 1e-6));
        bin_wins.push(block.2);
        bin_n.push(block.3);
        cursor = end;
    }
    // Predeclared reliability merge: fold bins with n < 25 into neighbors (deterministic L→R).
    const MIN_BIN_N: u64 = 25;
    let mut mx = x;
    let mut my = y;
    let mut mw = bin_wins;
    let mut mn = bin_n;
    let mut i = 0usize;
    while i < mn.len() {
        if mn[i] >= MIN_BIN_N || mn.len() == 1 {
            i += 1;
            continue;
        }
        if i + 1 < mn.len() {
            let n1 = mn[i] as f64;
            let n2 = mn[i + 1] as f64;
            mx[i + 1] = (mx[i] * n1 + mx[i + 1] * n2) / (n1 + n2);
            let w = mw[i] + mw[i + 1];
            let n = mn[i] + mn[i + 1];
            my[i + 1] = (w as f64 / n as f64).clamp(1e-6, 1.0 - 1e-6);
            mw[i + 1] = w;
            mn[i + 1] = n;
            mx.remove(i);
            my.remove(i);
            mw.remove(i);
            mn.remove(i);
        } else if i > 0 {
            let n1 = mn[i - 1] as f64;
            let n2 = mn[i] as f64;
            mx[i - 1] = (mx[i - 1] * n1 + mx[i] * n2) / (n1 + n2);
            let w = mw[i - 1] + mw[i];
            let n = mn[i - 1] + mn[i];
            my[i - 1] = (w as f64 / n as f64).clamp(1e-6, 1.0 - 1e-6);
            mw[i - 1] = w;
            mn[i - 1] = n;
            mx.remove(i);
            my.remove(i);
            mw.remove(i);
            mn.remove(i);
            i -= 1;
        } else {
            i += 1;
        }
    }
    IsotonicMap {
        x: mx,
        y: my,
        bin_wins: mw,
        bin_n: mn,
    }
}

pub fn calibrate(iso: &IsotonicMap, score: f64) -> (f64, f64) {
    if iso.x.is_empty() {
        return (0.5, 0.0);
    }
    // find rightmost x <= score
    let mut k = 0usize;
    for i in 0..iso.x.len() {
        if iso.x[i] <= score {
            k = i;
        } else {
            break;
        }
    }
    let p = iso.y[k];
    let lb = wilson_lower(iso.bin_wins[k], iso.bin_n[k], 0.95);
    (p, lb)
}

/// Wilson score interval lower bound (z≈1.959964 for 95%).
pub fn wilson_lower(wins: u64, n: u64, _confidence: f64) -> f64 {
    if n == 0 {
        return 0.0;
    }
    let z = 1.959_963_984_540_054_f64;
    let nf = n as f64;
    let phat = wins as f64 / nf;
    let z2 = z * z;
    let denom = 1.0 + z2 / nf;
    let centre = phat + z2 / (2.0 * nf);
    let margin = z * ((phat * (1.0 - phat) + z2 / (4.0 * nf)) / nf).sqrt();
    ((centre - margin) / denom).clamp(0.0, 1.0)
}

pub fn decide_generator(
    long_pack: &SideModelPack,
    short_pack: &SideModelPack,
    closes: &[f64],
    volumes: &[f64],
    cuts: &JointCuts,
) -> SciRecStamp {
    let schema = schema_hash();
    let f_l = build_features(closes, volumes, Side::Long);
    let f_s = build_features(closes, volumes, Side::Short);
    if !f_l.eligible || !f_s.eligible {
        return SciRecStamp {
            protocol_id: PROTOCOL_ID.into(),
            schema_version: SCHEMA_VERSION.into(),
            feature_schema_id: FEATURE_SCHEMA_ID.into(),
            schema_hash: schema,
            ladder_stage: long_pack.glm.ladder_stage.clone(),
            side: "FLAT".into(),
            p_hat: 0.0,
            p_lb_wilson: 0.0,
            p_long: 0.0,
            p_short: 0.0,
            p_lb_long: 0.0,
            p_lb_short: 0.0,
            band: Band::Insufficient,
            blocks_entry: true,
            block_reason: Some("INSUFFICIENT_PRICE_VOLUME_HISTORY".into()),
            p_min: cuts.p_min,
            p_lb_cut: cuts.p_lb,
        };
    }
    let s_l = raw_score(&long_pack.glm, &f_l.values);
    let s_s = raw_score(&short_pack.glm, &f_s.values);
    let (p_l, lb_l) = calibrate(&long_pack.isotonic, s_l);
    let (p_s, lb_s) = calibrate(&short_pack.isotonic, s_s);
    let (side, p_hat, p_lb) = if p_l >= p_s {
        ("LONG", p_l, lb_l)
    } else {
        ("SHORT", p_s, lb_s)
    };
    let (band, blocks, reason) = if p_hat >= cuts.p_min && p_lb >= cuts.p_lb {
        (Band::Enter, false, None)
    } else {
        (
            Band::Standaside,
            true,
            Some("JOINT_P_OR_CI_BELOW_CUT".into()),
        )
    };
    SciRecStamp {
        protocol_id: PROTOCOL_ID.into(),
        schema_version: SCHEMA_VERSION.into(),
        feature_schema_id: FEATURE_SCHEMA_ID.into(),
        schema_hash: schema,
        ladder_stage: long_pack.glm.ladder_stage.clone(),
        side: side.into(),
        p_hat,
        p_lb_wilson: p_lb,
        p_long: p_l,
        p_short: p_s,
        p_lb_long: lb_l,
        p_lb_short: lb_s,
        band,
        blocks_entry: blocks,
        block_reason: reason,
        p_min: cuts.p_min,
        p_lb_cut: cuts.p_lb,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoldScorecard {
    pub fold: String,
    pub baseline_n: usize,
    pub baseline_wins: usize,
    pub baseline_wr: f64,
    pub enter_n: usize,
    pub enter_wins: usize,
    pub enter_wr: f64,
    pub coverage: f64,
    pub payoff_asym: Option<f64>,
    pub mean_net_proxy: Option<f64>,
}

pub fn scorecard(
    fold: &str,
    baseline_y: &[f64],
    enter_mask: &[bool],
    payoff_proxy: &[f64],
) -> FoldScorecard {
    let baseline_n = baseline_y.len();
    let baseline_wins = baseline_y.iter().filter(|&&y| y >= 0.5).count();
    let mut enter_n = 0usize;
    let mut enter_wins = 0usize;
    let mut win_pay = Vec::new();
    let mut loss_pay = Vec::new();
    let mut nets = Vec::new();
    for i in 0..baseline_y.len() {
        if !enter_mask[i] {
            continue;
        }
        enter_n += 1;
        let y = baseline_y[i];
        let pay = payoff_proxy.get(i).copied().unwrap_or(0.0);
        nets.push(pay);
        if y >= 0.5 {
            enter_wins += 1;
            win_pay.push(pay.abs());
        } else {
            loss_pay.push(pay.abs());
        }
    }
    let avg_w = if win_pay.is_empty() {
        None
    } else {
        Some(win_pay.iter().sum::<f64>() / win_pay.len() as f64)
    };
    let avg_l = if loss_pay.is_empty() {
        None
    } else {
        Some(loss_pay.iter().sum::<f64>() / loss_pay.len() as f64)
    };
    let asym = match (avg_w, avg_l) {
        (Some(w), Some(l)) if l > EPS => Some(w / l),
        _ => None,
    };
    let mean_net = if nets.is_empty() {
        None
    } else {
        Some(nets.iter().sum::<f64>() / nets.len() as f64)
    };
    FoldScorecard {
        fold: fold.into(),
        baseline_n,
        baseline_wins,
        baseline_wr: if baseline_n == 0 {
            0.0
        } else {
            baseline_wins as f64 / baseline_n as f64
        },
        enter_n,
        enter_wins,
        enter_wr: if enter_n == 0 {
            0.0
        } else {
            enter_wins as f64 / enter_n as f64
        },
        coverage: if baseline_n == 0 {
            0.0
        } else {
            enter_n as f64 / baseline_n as f64
        },
        payoff_asym: asym,
        mean_net_proxy: mean_net,
    }
}

pub fn coverage_floor_ok(sc: &FoldScorecard) -> bool {
    sc.enter_n >= 80 || sc.coverage + 1e-12 >= 0.40
}

pub fn wl_improves(sc: &FoldScorecard) -> bool {
    sc.enter_n > 0 && sc.enter_wr > sc.baseline_wr + 1e-12
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn features_insufficient_short_history() {
        let f = build_features(&[1.0, 2.0], &[1.0, 2.0], Side::Long);
        assert!(!f.eligible);
    }

    #[test]
    fn features_deterministic() {
        let closes: Vec<f64> = (0..80).map(|i| 100.0 + i as f64).collect();
        let vols: Vec<f64> = (0..80).map(|i| 10.0 + (i % 7) as f64).collect();
        let a = build_features(&closes, &vols, Side::Long);
        let b = build_features(&closes, &vols, Side::Long);
        assert!(a.eligible);
        assert_eq!(a.values, b.values);
    }

    #[test]
    fn wilson_bounds() {
        let lb = wilson_lower(50, 100, 0.95);
        assert!(lb > 0.3 && lb < 0.5);
    }

    #[test]
    fn logistic_separates() {
        let mut xs = Vec::new();
        let mut ys = Vec::new();
        for i in 0..200 {
            let x0 = if i < 100 { -1.0 } else { 1.0 };
            xs.push(vec![x0, 0.1 * (i as f64 % 5.0)]);
            ys.push(if i < 100 { 0.0 } else { 1.0 });
        }
        let m = fit_logistic_l2(&xs, &ys, 1.0);
        let p_lo = predict_proba_raw(&m, &[-1.0, 0.0]);
        let p_hi = predict_proba_raw(&m, &[1.0, 0.0]);
        assert!(p_hi > p_lo);
    }
}
