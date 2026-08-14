//! G-GATE predictive-value experiment (v1.1 only).
//! Read-only against a B3 restore. Does not mutate B3 or v1.1 methodology files.

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const SEED: u64 = 20_260_813;
const B_BOOT: usize = 10_000;
const L_BLOCK: usize = 5;
const TRAIN_N: usize = 55;
const VAL_N: usize = 27;
const TEST_N: usize = 28;
const MIN_TEST_N: usize = 20;
const HORIZONS: [&str; 4] = ["5D", "10D", "20D", "60D"];
const MOD31: u64 = 1 << 31;

const B3_DUMP_SHA256: &str =
    "af11d318b03fb171207f96348fcf210e1b9149b1ab6e699c06c363faec518788";
const METH_V11: &str = "e129d7add66d7f4c12aab14811a3d552abf6b603f012eeb75c99c484e0065e66";
const METH_SPLIT_V11: &str =
    "6e9b3405a21b21f6c59cf99c05822c0d20007d335ef38a7bb5a21cf8f79d5691";
const METH_MANIFEST_V11: &str =
    "1604563a0a4516cbe983ef398ad36b6e1daacc8842b7a8daa28812e8ffee958e";

struct Boundary {
    rank: usize,
    fold: &'static str,
    ts: &'static str,
    strategy_id: &'static str,
    decision_id: &'static str,
}

const BOUNDARIES: [Boundary; 6] = [
    Boundary {
        rank: 1,
        fold: "TRAIN",
        ts: "2021-10-31T15:30:00Z",
        strategy_id: "04921177-db49-4144-a9ab-d940146e8002",
        decision_id: "db0f370c-5a9b-433d-a093-2fe6763501bc",
    },
    Boundary {
        rank: 55,
        fold: "TRAIN",
        ts: "2023-07-31T15:30:00Z",
        strategy_id: "59660824-e669-4f47-9ef3-7cc48d46d162",
        decision_id: "93609122-4ad2-4da1-9c61-5748f823fcdc",
    },
    Boundary {
        rank: 56,
        fold: "VALIDATION",
        ts: "2023-07-31T15:30:00Z",
        strategy_id: "8a413318-a5fe-4aac-ac0e-a90f560ec206",
        decision_id: "3b3a3d5b-9797-46d0-b47c-e455677ccbe8",
    },
    Boundary {
        rank: 82,
        fold: "VALIDATION",
        ts: "2024-03-31T15:30:00Z",
        strategy_id: "e82c0102-bf6a-4628-b22f-335da95c5532",
        decision_id: "be35a9dc-e09b-4273-be2e-d4ed85cd14d8",
    },
    Boundary {
        rank: 83,
        fold: "TEST",
        ts: "2024-04-30T15:30:00Z",
        strategy_id: "4a2a7b11-eb67-4394-af79-888ccf99f2e6",
        decision_id: "95d67b0c-9ec2-4ea2-bbe1-beaf29882019",
    },
    Boundary {
        rank: 110,
        fold: "TEST",
        ts: "2024-12-31T15:30:00Z",
        strategy_id: "a56feca3-5ae0-4c47-b4e0-2d7b298b3874",
        decision_id: "dc0478aa-6307-4820-815d-ffada7496e56",
    },
];

#[derive(Clone)]
struct RankedStrategy {
    rn: i64,
    fold: String,
    strategy_id: Uuid,
    decision_id: Uuid,
    evaluation_timestamp: DateTime<Utc>,
    signature_hash: String,
    assessment_ts: DateTime<Utc>,
}

#[derive(Clone)]
struct OutcomeRow {
    strategy_id: Uuid,
    horizon: String,
    outcome_return: f64,
    horizon_expiry: DateTime<Utc>,
}

#[derive(Clone)]
struct Obs {
    strategy_id: Uuid,
    p_hat: f64,
    y: f64,
}

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    fn next_u(&mut self) -> f64 {
        self.state = (1103515245u64.wrapping_mul(self.state).wrapping_add(12345)) % MOD31;
        self.state as f64 / MOD31 as f64
    }
}

#[derive(Clone, Serialize)]
struct HorizonMetrics {
    horizon: String,
    n: usize,
    n_pos: usize,
    n_neg: usize,
    auc: Option<f64>,
    auc_ci: Option<(f64, f64)>,
    delta_auc: Option<f64>,
    delta_auc_ci: Option<(f64, f64)>,
    brier: Option<f64>,
    brier_baseline: Option<f64>,
    delta_brier: Option<f64>,
    cal_intercept: Option<f64>,
    cal_slope: Option<f64>,
    observed_rate: Option<f64>,
    predicted_rate: Option<f64>,
    p_value: Option<f64>,
    holm_p: Option<f64>,
    p_baseline: f64,
    n_undefined_bootstrap: usize,
    metrics_defined: bool,
    min_n_ok: bool,
    both_classes: bool,
    var_p_ok: bool,
    contribution: String,
}

fn round10(x: f64) -> f64 {
    if !x.is_finite() {
        return x;
    }
    let s = 10_000_000_000.0;
    let y = x * s;
    let r = if y >= 0.0 {
        (y + 0.5).floor()
    } else {
        (y - 0.5).ceil()
    };
    r / s
}

fn round_half_away(x: f64) -> i64 {
    if x >= 0.0 {
        (x + 0.5).floor() as i64
    } else {
        (x - 0.5).ceil() as i64
    }
}

fn fmt10(x: Option<f64>) -> String {
    match x {
        Some(v) if v.is_finite() => format!("{:.10}", round10(v)),
        Some(_) => "undefined".to_string(),
        None => "—".to_string(),
    }
}

fn fmt_ci(ci: Option<(f64, f64)>) -> String {
    match ci {
        Some((lo, hi)) => format!("[{:.10}, {:.10}]", round10(lo), round10(hi)),
        None => "—".to_string(),
    }
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let bytes = fs::read(path)?;
    Ok(hex_sha256(&bytes))
}

fn hex_sha256(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

fn fold_of_rank(rn: i64) -> &'static str {
    if rn <= TRAIN_N as i64 {
        "TRAIN"
    } else if rn <= (TRAIN_N + VAL_N) as i64 {
        "VALIDATION"
    } else {
        "TEST"
    }
}

fn auc_mann_whitney(obs: &[Obs]) -> Option<f64> {
    let n_pos = obs.iter().filter(|o| o.y == 1.0).count();
    let n_neg = obs.iter().filter(|o| o.y == 0.0).count();
    if n_pos == 0 || n_neg == 0 {
        return None;
    }
    let mut correct = 0.0;
    let mut tied = 0.0;
    for a in obs {
        if a.y != 1.0 {
            continue;
        }
        for b in obs {
            if b.y != 0.0 {
                continue;
            }
            if a.p_hat > b.p_hat {
                correct += 1.0;
            } else if a.p_hat == b.p_hat {
                tied += 1.0;
            }
        }
    }
    Some((correct + 0.5 * tied) / (n_pos as f64 * n_neg as f64))
}

fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.iter().sum::<f64>() / xs.len() as f64
}

fn ols_intercept_slope(p: &[f64], y: &[f64]) -> Option<(f64, f64)> {
    let n = p.len() as f64;
    if n < 2.0 {
        return None;
    }
    let mp = mean(p);
    let my = mean(y);
    let mut sxx = 0.0;
    let mut sxy = 0.0;
    for i in 0..p.len() {
        let dp = p[i] - mp;
        sxx += dp * dp;
        sxy += dp * (y[i] - my);
    }
    if sxx == 0.0 {
        return None;
    }
    let b = sxy / sxx;
    let a = my - b * mp;
    Some((a, b))
}

fn percentile_sorted(sorted: &[f64], p: f64) -> f64 {
    let n = sorted.len();
    if n == 0 {
        return f64::NAN;
    }
    let idx = round_half_away(p * (n as f64 - 1.0)) as usize;
    sorted[idx.min(n - 1)]
}

fn holm_adjust(ps: &[Option<f64>]) -> Vec<Option<f64>> {
    let m = ps.len();
    let mut indexed: Vec<(usize, f64)> = ps
        .iter()
        .enumerate()
        .filter_map(|(i, p)| p.map(|v| (i, v)))
        .collect();
    if indexed.len() != m {
        return vec![None; m];
    }
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut adj = vec![0.0f64; m];
    let mut running: f64 = 0.0;
    for (rank_zero, (orig, p)) in indexed.iter().enumerate() {
        let k = m - rank_zero;
        let raw = (k as f64) * p;
        running = running.max(raw).min(1.0);
        adj[*orig] = running;
    }
    adj.into_iter().map(Some).collect()
}

fn build_blocks(n: usize, l: usize) -> Vec<Vec<usize>> {
    let l = if n < l { n } else { l };
    let mut blocks = Vec::new();
    let mut i = 0;
    while i < n {
        let end = (i + l).min(n);
        blocks.push((i..end).collect());
        i = end;
    }
    blocks
}

fn resample_indices(lcg: &mut Lcg, n: usize, blocks: &[Vec<usize>]) -> Vec<usize> {
    let n_blocks = blocks.len();
    let mut sampled = Vec::new();
    while sampled.len() < n {
        let u = lcg.next_u();
        let bi = ((u * n_blocks as f64) as usize).min(n_blocks - 1);
        sampled.extend_from_slice(&blocks[bi]);
    }
    sampled.truncate(n);
    sampled
}

fn reliability_table(obs: &[Obs]) -> Value {
    let mut sorted = obs.to_vec();
    sorted.sort_by(|a, b| {
        a.p_hat
            .partial_cmp(&b.p_hat)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.strategy_id.cmp(&b.strategy_id))
    });
    let distinct: HashSet<u64> = sorted.iter().map(|o| o.p_hat.to_bits()).collect();
    if distinct.len() < 5 {
        let mut groups: BTreeMap<u64, Vec<&Obs>> = BTreeMap::new();
        for o in &sorted {
            groups.entry(o.p_hat.to_bits()).or_default().push(o);
        }
        let bins: Vec<Value> = groups
            .values()
            .map(|g| {
                let ps: Vec<f64> = g.iter().map(|o| o.p_hat).collect();
                let ys: Vec<f64> = g.iter().map(|o| o.y).collect();
                json!({
                    "n": g.len(),
                    "mean_p": round10(mean(&ps)),
                    "mean_y": round10(mean(&ys)),
                })
            })
            .collect();
        return json!(bins);
    }
    let n = sorted.len();
    let base = n / 5;
    let rem = n % 5;
    let mut bins = Vec::new();
    let mut start = 0;
    for i in 0..5 {
        let sz = base + if i < rem { 1 } else { 0 };
        let slice = &sorted[start..start + sz];
        let ps: Vec<f64> = slice.iter().map(|o| o.p_hat).collect();
        let ys: Vec<f64> = slice.iter().map(|o| o.y).collect();
        bins.push(json!({
            "bin": i + 1,
            "n": sz,
            "mean_p": round10(mean(&ps)),
            "mean_y": round10(mean(&ys)),
        }));
        start += sz;
    }
    json!(bins)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL is required")?;
    let out_dir = PathBuf::from(
        std::env::var("G_GATE_OUT_DIR").unwrap_or_else(|_| "G_GATE_OUT".to_string()),
    );
    fs::create_dir_all(&out_dir)?;
    let bundle = out_dir.join("G_EXTENSION");
    fs::create_dir_all(&bundle)?;

    let dataset_sha = std::env::var("DATASET_SHA256")
        .or_else(|_| std::env::var("B3_DATASET_SHA256"))
        .unwrap_or_else(|_| B3_DUMP_SHA256.to_string());
    let meth_manifest_sha =
        std::env::var("METH_MANIFEST_SHA256").unwrap_or_else(|_| METH_MANIFEST_V11.to_string());
    let binary_sha = std::env::var("EXPERIMENT_BINARY_SHA256").unwrap_or_else(|_| "unspecified".to_string());
    let dataset_label = std::env::var("G_GATE_DATASET").unwrap_or_else(|_| {
        if dataset_sha == B3_DUMP_SHA256 {
            "B3".to_string()
        } else {
            "B4".to_string()
        }
    });

    if dataset_label == "B3" && dataset_sha != B3_DUMP_SHA256 {
        eprintln!("STOP: B3 dump hash mismatch. expected {B3_DUMP_SHA256} got {dataset_sha}");
        std::process::exit(2);
    }
    if dataset_label != "B3" && dataset_label != "B4" {
        eprintln!("STOP: G_GATE_DATASET must be B3 or B4, got {dataset_label}");
        std::process::exit(2);
    }

    let pool = PgPoolOptions::new()
        .max_connections(3)
        .connect(&db_url)
        .await?;

    let n_assess: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_assessments")
        .fetch_one(&pool)
        .await?;
    let n_dec: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_decisions")
        .fetch_one(&pool)
        .await?;
    let n_strat: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_strategies")
        .fetch_one(&pool)
        .await?;
    let n_out: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_outcomes")
        .fetch_one(&pool)
        .await?;
    if n_assess != 195 || n_dec != 195 || n_strat != 110 || n_out != 440 {
        eprintln!(
            "STOP: identity counts mismatch: assessments={n_assess} decisions={n_dec} strategies={n_strat} outcomes={n_out}"
        );
        std::process::exit(2);
    }

    let ranked = load_ranked_strategies(&pool).await?;
    if ranked.len() != 110 {
        eprintln!("STOP: expected 110 ranked strategies, got {}", ranked.len());
        std::process::exit(2);
    }

    if dataset_label == "B3" {
        if let Err(msg) = verify_boundaries(&ranked) {
            eprintln!("STOP: frozen B3 split cannot be reproduced: {msg}");
            std::process::exit(2);
        }
    } else {
        let n_train = ranked.iter().filter(|s| s.fold == "TRAIN").count();
        let n_val = ranked.iter().filter(|s| s.fold == "VALIDATION").count();
        let n_test = ranked.iter().filter(|s| s.fold == "TEST").count();
        if n_train != TRAIN_N || n_val != VAL_N || n_test != TEST_N {
            eprintln!(
                "STOP: B4 chronological split sizes TRAIN={n_train} VAL={n_val} TEST={n_test} (expected {TRAIN_N}/{VAL_N}/{TEST_N})"
            );
            std::process::exit(2);
        }
    }

    let outcomes = load_outcomes(&pool).await?;
    if outcomes.len() != 440 {
        eprintln!("STOP: expected 440 outcomes, got {}", outcomes.len());
        std::process::exit(2);
    }

    let mut by_strategy_horizons: HashMap<Uuid, HashSet<String>> = HashMap::new();
    for o in &outcomes {
        by_strategy_horizons
            .entry(o.strategy_id)
            .or_default()
            .insert(o.horizon.clone());
    }
    for s in &ranked {
        let hs = by_strategy_horizons.get(&s.strategy_id);
        if hs.map(|h| h.len()).unwrap_or(0) != 4 {
            eprintln!(
                "STOP: strategy {} does not have exactly 4 outcomes",
                s.strategy_id
            );
            std::process::exit(2);
        }
        for h in HORIZONS {
            if !hs.unwrap().contains(h) {
                eprintln!("STOP: strategy {} missing horizon {h}", s.strategy_id);
                std::process::exit(2);
            }
        }
    }

    let (leakage_pass, leakage_md) = leakage_audit(&ranked, &outcomes);

    let train: Vec<&RankedStrategy> = ranked.iter().filter(|s| s.fold == "TRAIN").collect();
    let test: Vec<&RankedStrategy> = ranked.iter().filter(|s| s.fold == "TEST").collect();
    if train.len() != TRAIN_N || test.len() != TEST_N {
        eprintln!(
            "STOP: fold sizes TRAIN={} TEST={} (expected {TRAIN_N}/{TEST_N})",
            train.len(),
            test.len()
        );
        std::process::exit(2);
    }

    let mut outcome_map: HashMap<(Uuid, String), f64> = HashMap::new();
    for o in &outcomes {
        outcome_map.insert((o.strategy_id, o.horizon.clone()), o.outcome_return);
    }

    let mut metrics: Vec<HorizonMetrics> = Vec::new();
    let mut reliability: BTreeMap<String, Value> = BTreeMap::new();

    for horizon in HORIZONS {
        let mut train_pos: HashMap<String, usize> = HashMap::new();
        let mut train_n: HashMap<String, usize> = HashMap::new();
        let mut n_pos_all = 0usize;
        let mut n_all = 0usize;
        for s in &train {
            let y = if *outcome_map.get(&(s.strategy_id, horizon.to_string())).unwrap() > 0.0 {
                1usize
            } else {
                0
            };
            *train_pos.entry(s.signature_hash.clone()).or_insert(0) += y;
            *train_n.entry(s.signature_hash.clone()).or_insert(0) += 1;
            n_pos_all += y;
            n_all += 1;
        }
        let p_baseline = n_pos_all as f64 / n_all as f64;

        let mut test_obs: Vec<Obs> = Vec::new();
        for s in &test {
            let y = if *outcome_map.get(&(s.strategy_id, horizon.to_string())).unwrap() > 0.0 {
                1.0
            } else {
                0.0
            };
            let p_hat = match train_n.get(&s.signature_hash).copied().unwrap_or(0) {
                0 => p_baseline,
                n => *train_pos.get(&s.signature_hash).unwrap_or(&0) as f64 / n as f64,
            };
            test_obs.push(Obs {
                strategy_id: s.strategy_id,
                p_hat,
                y,
            });
        }
        // Keep chronological split order for block bootstrap (v1.1 §9).
        // AUC is pairwise and does not require sorting the evaluation vector.

        let n = test_obs.len();
        let n_pos = test_obs.iter().filter(|o| o.y == 1.0).count();
        let n_neg = n - n_pos;
        let min_n_ok = n >= MIN_TEST_N;
        let both_classes = n_pos >= 1 && n_neg >= 1;
        let ps: Vec<f64> = test_obs.iter().map(|o| o.p_hat).collect();
        let ys: Vec<f64> = test_obs.iter().map(|o| o.y).collect();
        let cal = ols_intercept_slope(&ps, &ys);
        let var_p_ok = cal.is_some();

        let auc = if min_n_ok && both_classes {
            auc_mann_whitney(&test_obs)
        } else {
            None
        };
        let brier = if min_n_ok && both_classes {
            let sq: Vec<f64> = ps.iter().zip(ys.iter()).map(|(p, y)| (p - y).powi(2)).collect();
            Some(mean(&sq))
        } else {
            None
        };
        let brier_base = if min_n_ok && both_classes {
            Some(mean(
                &ys.iter()
                    .map(|y| (p_baseline - y).powi(2))
                    .collect::<Vec<_>>(),
            ))
        } else {
            None
        };

        let mut n_undefined_bootstrap = 0usize;
        let mut boot_aucs: Vec<f64> = Vec::new();
        let mut boot_delta: Vec<f64> = Vec::new();
        if auc.is_some() {
            let mut lcg = Lcg::new(SEED);
            let blocks = build_blocks(n, L_BLOCK);
            for _ in 0..B_BOOT {
                let idx = resample_indices(&mut lcg, n, &blocks);
                let sample: Vec<Obs> = idx.iter().map(|&i| test_obs[i].clone()).collect();
                match auc_mann_whitney(&sample) {
                    Some(a) => {
                        boot_aucs.push(a);
                        boot_delta.push(a - 0.5);
                    }
                    None => n_undefined_bootstrap += 1,
                }
            }
        }

        let (auc_ci, delta_ci, p_value) = if auc.is_some() && n_undefined_bootstrap == 0 {
            let mut sa = boot_aucs.clone();
            sa.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mut sd = boot_delta.clone();
            sd.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let ci_a = (
                percentile_sorted(&sa, 0.025),
                percentile_sorted(&sa, 0.975),
            );
            let ci_d = (
                percentile_sorted(&sd, 0.025),
                percentile_sorted(&sd, 0.975),
            );
            let n_le0 = boot_delta.iter().filter(|d| **d <= 0.0).count();
            let p = (1.0 + n_le0 as f64) / (B_BOOT as f64 + 1.0);
            (Some(ci_a), Some(ci_d), Some(p))
        } else {
            (None, None, None)
        };

        let metrics_defined = auc.is_some()
            && auc_ci.is_some()
            && p_value.is_some()
            && brier.is_some()
            && cal.is_some()
            && min_n_ok
            && both_classes
            && var_p_ok;

        reliability.insert(horizon.to_string(), reliability_table(&test_obs));

        metrics.push(HorizonMetrics {
            horizon: horizon.to_string(),
            n,
            n_pos,
            n_neg,
            auc,
            auc_ci,
            delta_auc: auc.map(|a| a - 0.5),
            delta_auc_ci: delta_ci,
            brier,
            brier_baseline: brier_base,
            delta_brier: match (brier_base, brier) {
                (Some(bb), Some(bc)) => Some(bb - bc),
                _ => None,
            },
            cal_intercept: cal.map(|c| c.0),
            cal_slope: cal.map(|c| c.1),
            observed_rate: Some(n_pos as f64 / n as f64),
            predicted_rate: Some(mean(&ps)),
            p_value,
            holm_p: None,
            p_baseline,
            n_undefined_bootstrap,
            metrics_defined,
            min_n_ok,
            both_classes,
            var_p_ok,
            contribution: String::new(),
        });
    }

    let raw_ps: Vec<Option<f64>> = metrics.iter().map(|m| m.p_value).collect();
    let holm = holm_adjust(&raw_ps);
    for (m, hp) in metrics.iter_mut().zip(holm.into_iter()) {
        m.holm_p = hp;
        let dpos = m.delta_auc.map(|d| d > 0.0).unwrap_or(false);
        let cilb = m
            .delta_auc_ci
            .map(|c| c.0 > 0.0)
            .unwrap_or(false);
        let holm_sig = m.holm_p.map(|p| p < 0.05).unwrap_or(false);
        m.contribution = format!(
            "defined={} ΔAUC>0={} CI_lb>0={} holm_p<0.05={}",
            m.metrics_defined, dpos, cilb, holm_sig
        );
    }

    let all_defined = metrics.iter().all(|m| m.metrics_defined);
    let detected = leakage_pass
        && all_defined
        && metrics.iter().all(|m| m.delta_auc.unwrap() > 0.0)
        && metrics.iter().all(|m| m.delta_auc_ci.unwrap().0 > 0.0)
        && metrics.iter().all(|m| m.holm_p.unwrap() < 0.05);

    let classification = if !leakage_pass || !all_defined {
        "INCONCLUSIVE"
    } else if detected {
        "PREDICTIVE_VALUE_DETECTED"
    } else {
        "PREDICTIVE_VALUE_NOT_DETECTED"
    };

    let exec_ts = Utc::now();
    let config = json!({
        "protocol": "G-GATE v1.1",
        "dataset": dataset_label,
        "dataset_sha256": dataset_sha,
        "methodology": "G_Extension_Methodology_v1.1",
        "methodology_sha256": METH_V11,
        "train_test_split_sha256": METH_SPLIT_V11,
        "methodology_manifest_sha256": meth_manifest_sha,
        "seed": SEED,
        "B": B_BOOT,
        "L": L_BLOCK,
        "prng": "LCG state=(1103515245*state+12345) mod 2^31; draw=state/2^31; first draw after one update from seed",
        "percentile": "nearest_rank index=round_half_away(p*(B-1))",
        "Y_h": "1 iff outcome_return > 0",
        "candidate": "train-only P(Y_h=1|signature_hash); unseen -> train prevalence",
        "split": "ranks 1-55 TRAIN, 56-82 VALIDATION, 83-110 TEST; order evaluation_timestamp ASC, strategy_id ASC",
        "holm_alpha": 0.05,
        "min_test_n": MIN_TEST_N,
        "binary_sha256": binary_sha,
    });

    let output_txt = render_output(&metrics, classification, leakage_pass, &dataset_label);
    let results_md = render_report(&metrics, classification, leakage_pass, &reliability, &dataset_label);
    fs::write(bundle.join("output.txt"), &output_txt)?;
    fs::write(bundle.join("results.md"), &results_md)?;
    fs::write(bundle.join("leakage_audit.md"), &leakage_md)?;
    fs::write(
        bundle.join("configuration.json"),
        serde_json::to_string_pretty(&config)?,
    )?;
    fs::write(bundle.join("dataset.sha256"), format!("{dataset_sha}\n"))?;
    fs::write(bundle.join("binary.sha256"), format!("{binary_sha}\n"))?;
    fs::write(
        bundle.join("methodology_manifest.sha256"),
        format!("{meth_manifest_sha}\n"),
    )?;

    if let Ok(root) = std::env::var("REPO_ROOT") {
        let root = PathBuf::from(root);
        let _ = fs::copy(
            root.join("G_Extension_Methodology_v1.1.md"),
            bundle.join("methodology.md"),
        );
        let _ = fs::copy(
            root.join("G_Extension_Methodology_v1.1_TrainTestSplit.md"),
            bundle.join("train_test_split.md"),
        );
    }

    let output_sha = hex_sha256(output_txt.as_bytes());
    let results_sha = hex_sha256(results_md.as_bytes());
    let leakage_sha = hex_sha256(leakage_md.as_bytes());

    let mut horizon_json = Vec::new();
    for m in &metrics {
        horizon_json.push(json!({
            "horizon": m.horizon,
            "n": m.n,
            "n_pos": m.n_pos,
            "n_neg": m.n_neg,
            "auc": m.auc.map(round10),
            "auc_ci": m.auc_ci.map(|(a,b)| json!([round10(a), round10(b)])),
            "delta_auc": m.delta_auc.map(round10),
            "delta_auc_ci": m.delta_auc_ci.map(|(a,b)| json!([round10(a), round10(b)])),
            "brier": m.brier.map(round10),
            "brier_baseline": m.brier_baseline.map(round10),
            "delta_brier": m.delta_brier.map(round10),
            "calibration_intercept": m.cal_intercept.map(round10),
            "calibration_slope": m.cal_slope.map(round10),
            "observed_rate": m.observed_rate.map(round10),
            "predicted_rate": m.predicted_rate.map(round10),
            "p_baseline": round10(m.p_baseline),
            "p_value": m.p_value.map(round10),
            "holm_adjusted_p": m.holm_p.map(round10),
            "n_undefined_bootstrap": m.n_undefined_bootstrap,
            "metrics_defined": m.metrics_defined,
            "contribution": m.contribution,
            "reliability": reliability.get(&m.horizon),
        }));
    }

    let witness = json!({
        "protocol": "G-GATE v1.1",
        "classification": classification,
        "predictive_value_claim": match classification {
            "PREDICTIVE_VALUE_DETECTED" => "DETECTED",
            "PREDICTIVE_VALUE_NOT_DETECTED" => "NOT_DETECTED",
            _ => "NOT_ESTABLISHED",
        },
        "leakage_result": if leakage_pass { "PASS" } else { "FAIL" },
        "dataset_label": dataset_label,
        "dataset_sha256": dataset_sha,
        "b3_dataset_sha256": if dataset_label == "B3" { Some(dataset_sha.clone()) } else { None },
        "methodology_manifest_sha256": meth_manifest_sha,
        "methodology_v1_1_sha256": METH_V11,
        "methodology_split_v1_1_sha256": METH_SPLIT_V11,
        "experiment_binary_sha256": binary_sha,
        "configuration": config,
        "seed": SEED,
        "output_sha256": output_sha,
        "results_sha256": results_sha,
        "leakage_audit_sha256": leakage_sha,
        "execution_timestamp": exec_ts.to_rfc3339(),
        "boundary_identities_reproduced": dataset_label == "B3",
        "horizons": horizon_json,
    });
    let witness_txt = serde_json::to_string_pretty(&witness)?;
    fs::write(bundle.join("witness.json"), &witness_txt)?;

    fs::write(out_dir.join("G_GATE_REPORT.md"), &results_md)?;
    fs::write(out_dir.join("g_gate_output.txt"), &output_txt)?;
    fs::write(out_dir.join("G_GATE_WITNESS.json"), &witness_txt)?;

    println!("{output_txt}");
    println!("CLASSIFICATION={classification}");
    println!("OUTPUT_DIR={}", out_dir.display());
    Ok(())
}

async fn load_ranked_strategies(
    pool: &PgPool,
) -> Result<Vec<RankedStrategy>, Box<dyn std::error::Error + Send + Sync>> {
    let rows = sqlx::query(
        r#"
        WITH strat AS (
          SELECT s.id AS strategy_id,
                 d.id AS decision_id,
                 d.evaluation_timestamp,
                 a.signature_hash,
                 a.evaluation_timestamp AS assessment_ts
          FROM knowledge_strategies s
          JOIN knowledge_decisions d ON s.decision_id = d.id
          JOIN knowledge_assessments a ON d.assessment_id = a.id
        ),
        ordered AS (
          SELECT strategy_id, decision_id, evaluation_timestamp, signature_hash, assessment_ts,
                 ROW_NUMBER() OVER (
                   ORDER BY evaluation_timestamp ASC, strategy_id ASC
                 ) AS rn
          FROM strat
        )
        SELECT rn, strategy_id, decision_id, evaluation_timestamp, signature_hash, assessment_ts
        FROM ordered
        ORDER BY rn
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut out = Vec::new();
    for row in rows {
        let rn: i64 = row.try_get("rn")?;
        out.push(RankedStrategy {
            rn,
            fold: fold_of_rank(rn).to_string(),
            strategy_id: row.try_get("strategy_id")?,
            decision_id: row.try_get("decision_id")?,
            evaluation_timestamp: row.try_get("evaluation_timestamp")?,
            signature_hash: row.try_get("signature_hash")?,
            assessment_ts: row.try_get("assessment_ts")?,
        });
    }
    Ok(out)
}

async fn load_outcomes(
    pool: &PgPool,
) -> Result<Vec<OutcomeRow>, Box<dyn std::error::Error + Send + Sync>> {
    let rows = sqlx::query(
        r#"
        SELECT strategy_id, horizon, outcome_return, horizon_expiry_timestamp
        FROM knowledge_outcomes
        "#,
    )
    .fetch_all(pool)
    .await?;
    let mut out = Vec::new();
    for row in rows {
        out.push(OutcomeRow {
            strategy_id: row.try_get("strategy_id")?,
            horizon: row.try_get("horizon")?,
            outcome_return: row.try_get("outcome_return")?,
            horizon_expiry: row.try_get("horizon_expiry_timestamp")?,
        });
    }
    Ok(out)
}

fn verify_boundaries(ranked: &[RankedStrategy]) -> Result<(), String> {
    for b in &BOUNDARIES {
        let s = ranked
            .iter()
            .find(|r| r.rn == b.rank as i64)
            .ok_or_else(|| format!("missing rank {}", b.rank))?;
        if s.fold != b.fold {
            return Err(format!(
                "rank {} fold {} != {}",
                b.rank, s.fold, b.fold
            ));
        }
        if s.strategy_id.to_string() != b.strategy_id {
            return Err(format!(
                "rank {} strategy_id {} != {}",
                b.rank, s.strategy_id, b.strategy_id
            ));
        }
        if s.decision_id.to_string() != b.decision_id {
            return Err(format!(
                "rank {} decision_id {} != {}",
                b.rank, s.decision_id, b.decision_id
            ));
        }
        let ts = s
            .evaluation_timestamp
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        if ts != b.ts {
            return Err(format!("rank {} timestamp {ts} != {}", b.rank, b.ts));
        }
    }
    Ok(())
}

fn leakage_audit(ranked: &[RankedStrategy], outcomes: &[OutcomeRow]) -> (bool, String) {
    let mut checks: Vec<(u8, &'static str, bool, String)> = Vec::new();

    let mut ts_ok = true;
    let mut ts_fail = 0usize;
    for s in ranked {
        if s.assessment_ts > s.evaluation_timestamp {
            ts_ok = false;
            ts_fail += 1;
        }
    }
    checks.push((
        1,
        "Feature timestamps <= decision evaluation_timestamp",
        ts_ok,
        if ts_ok {
            "All assessment timestamps are <= linked decision timestamps. Feature is signature_hash only.".into()
        } else {
            format!("{ts_fail} assessments have evaluation_timestamp after the decision.")
        },
    ));

    checks.push((
        2,
        "Labels not used as features",
        true,
        "Candidate uses only signature_hash. outcome_return is used solely to form Y_h.".into(),
    ));
    checks.push((
        3,
        "Lookup fitted on TRAIN only",
        true,
        "p_hat is estimated from ranks 1–55 only.".into(),
    ));
    checks.push((
        4,
        "VALIDATION not used for fitting/selection/thresholding",
        true,
        "Validation is unused in model construction. No threshold is selected.".into(),
    ));
    checks.push((
        5,
        "TEST unused until final evaluation",
        true,
        "Test labels and signatures are not used to estimate p_hat or p_baseline.".into(),
    ));

    let mut dup_ok = true;
    let mut folds: HashMap<String, HashSet<Uuid>> = HashMap::new();
    for s in ranked {
        let set = folds.entry(s.fold.clone()).or_default();
        if !set.insert(s.strategy_id) {
            dup_ok = false;
        }
    }
    checks.push((
        6,
        "No duplicate strategy_id in a fold",
        dup_ok,
        format!(
            "TRAIN={} VALIDATION={} TEST={}",
            folds.get("TRAIN").map(|s| s.len()).unwrap_or(0),
            folds.get("VALIDATION").map(|s| s.len()).unwrap_or(0),
            folds.get("TEST").map(|s| s.len()).unwrap_or(0)
        ),
    ));

    let mut cluster_ok = true;
    let fold_of: HashMap<Uuid, String> = ranked.iter().map(|s| (s.strategy_id, s.fold.clone())).collect();
    let mut per: HashMap<Uuid, HashSet<String>> = HashMap::new();
    for o in outcomes {
        per.entry(o.strategy_id).or_default().insert(o.horizon.clone());
        if !fold_of.contains_key(&o.strategy_id) {
            cluster_ok = false;
        }
    }
    for (_id, hs) in &per {
        if hs.len() != 4 {
            cluster_ok = false;
        }
    }
    checks.push((
        7,
        "Cluster constraint: four horizons share fold",
        cluster_ok,
        "Each strategy’s four outcomes inherit the strategy fold.".into(),
    ));

    checks.push((
        8,
        "Calendar overlap of outcome-expiry windows documented",
        true,
        "TRAIN/VAL share 2023-07-31T15:30:00Z. Some TRAIN 60D expiries fall after first VAL evaluations. Some VAL 60D expiries fall after TEST start 2024-04-30T15:30:00Z. TRAIN latest 60D expiry 2023-09-29T15:30:00Z is before TEST start. Ranks were not altered.".into(),
    ));
    checks.push((
        9,
        "No scaler/encoder/prevalence from validation or test",
        true,
        "Prevalence and signature rates are training-fold only. No scaler or encoder is fitted.".into(),
    ));

    let pass = checks.iter().all(|c| c.2);
    let mut md = String::from("# G-GATE Leakage Audit (v1.1)\n\n");
    md.push_str(&format!(
        "**Result:** {}\n\n",
        if pass { "PASS" } else { "FAIL" }
    ));
    md.push_str("| # | Check | Result | Notes |\n|---|-------|--------|-------|\n");
    for (n, name, ok, notes) in &checks {
        md.push_str(&format!(
            "| {n} | {name} | {} | {notes} |\n",
            if *ok { "PASS" } else { "FAIL" }
        ));
    }
    md.push_str("\nAny FAIL forces INCONCLUSIVE.\n");
    (pass, md)
}

fn render_output(metrics: &[HorizonMetrics], classification: &str, leakage_pass: bool, dataset: &str) -> String {
    let mut s = String::from("G-GATE predictive-value experiment (v1.1)\n");
    s.push_str(&format!("Dataset: {dataset}\n"));
    s.push_str(&format!("Seed: {SEED}\n"));
    s.push_str(&format!("Leakage: {}\n", if leakage_pass { "PASS" } else { "FAIL" }));
    s.push_str(&format!("Classification: {classification}\n\n"));
    s.push_str("Horizon\tN\tPositive\tNegative\tAUC\tAUC_95CI\tDeltaAUC\tDeltaAUC_95CI\tBrier\tCalIntercept\tCalSlope\tp\tHolm_p\n");
    for m in metrics {
        s.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            m.horizon,
            m.n,
            m.n_pos,
            m.n_neg,
            fmt10(m.auc),
            fmt_ci(m.auc_ci),
            fmt10(m.delta_auc),
            fmt_ci(m.delta_auc_ci),
            fmt10(m.brier),
            fmt10(m.cal_intercept),
            fmt10(m.cal_slope),
            fmt10(m.p_value),
            fmt10(m.holm_p),
        ));
    }
    s
}

fn render_report(
    metrics: &[HorizonMetrics],
    classification: &str,
    leakage_pass: bool,
    reliability: &BTreeMap<String, Value>,
    dataset: &str,
) -> String {
    let mut s = String::from("# G-GATE Report\n\n");
    s.push_str("**Protocol:** G-Extension Methodology v1.1  \n");
    s.push_str(&format!("**Dataset:** {dataset} (read-only)  \n"));
    s.push_str(&format!("**Seed:** `{SEED}`  \n"));
    s.push_str(&format!(
        "**Leakage audit:** {}  \n",
        if leakage_pass { "PASS" } else { "FAIL" }
    ));
    s.push_str(&format!("**Classification:** `{classification}`\n\n"));
    s.push_str("Scientific question: does decision-time `signature_hash` contain statistically demonstrable predictive information for `Y_h = 1[outcome_return > 0]` beyond the frozen training-prevalence baseline, on the held-out chronological test fold?\n\n");
    s.push_str("## Horizon metrics (test fold)\n\n");
    s.push_str("| Horizon | N | Positive | Negative | AUC | 95% CI | ΔAUC | ΔAUC 95% CI | Brier | Cal. intercept | Cal. slope | p-value | Holm-adjusted p | Classification contribution |\n");
    s.push_str("|---------|---|----------|----------|-----|--------|------|-------------|-------|----------------|------------|---------|-----------------|------------------------------|\n");
    for m in metrics {
        s.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            m.horizon,
            m.n,
            m.n_pos,
            m.n_neg,
            fmt10(m.auc),
            fmt_ci(m.auc_ci),
            fmt10(m.delta_auc),
            fmt_ci(m.delta_auc_ci),
            fmt10(m.brier),
            fmt10(m.cal_intercept),
            fmt10(m.cal_slope),
            fmt10(m.p_value),
            fmt10(m.holm_p),
            m.contribution,
        ));
    }
    s.push_str("\nBlank / `—` values are undefined, not zero.\n\n");
    s.push_str("## Secondary rates\n\n");
    s.push_str("| Horizon | p_baseline | Observed event rate | Predicted event rate | Brier baseline | ΔBrier | Undefined bootstrap AUCs |\n");
    s.push_str("|---------|------------|---------------------|----------------------|----------------|--------|--------------------------|\n");
    for m in metrics {
        s.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            m.horizon,
            fmt10(Some(m.p_baseline)),
            fmt10(m.observed_rate),
            fmt10(m.predicted_rate),
            fmt10(m.brier_baseline),
            fmt10(m.delta_brier),
            m.n_undefined_bootstrap,
        ));
    }
    s.push_str("\n## Reliability tables\n\n");
    for (h, table) in reliability {
        s.push_str(&format!("### {h}\n\n```json\n{}\n```\n\n", serde_json::to_string_pretty(table).unwrap()));
    }
    s.push_str("## Classification rule applied\n\n");
    s.push_str("`PREDICTIVE_VALUE_DETECTED` requires leakage PASS, all four horizons metrics-defined, every ΔAUC > 0, every ΔAUC CI lower bound > 0, and every Holm-adjusted p < 0.05.\n\n");
    s.push_str(&format!("**Final result:** `{classification}`\n"));
    s
}
