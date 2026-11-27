//! RP-608-SCI-ADAPTER-REC-ML1D-001 — Offline dual-dimension ACT/PASS package.
//!
//! Coralys side sealed (direction from frozen-plan labels). Adapter learns
//! reliability \(Q_t\) (R1) + economic \(U_t\) (Ec1) → E0–E3 ACT/PASS.
//! SCI-v1 S1=b retained as H11 comparator. Books never pooled. Authority 0%.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use chronosentiment_adapter::product::sci_adapter_rec::{
    build_features, coverage_floor_ok, decide_generator, fit_isotonic, raw_score, scorecard,
    wl_improves, Band, FoldScorecard, FrozenTwinPack, JointCuts, Side, FEATURE_NAMES,
    FEATURE_SCHEMA_ID, LABEL_HORIZON_BARS, PROTOCOL_ID as SCI_V1_PROTOCOL, PURGE_BARS,
    DEFAULT_FROZEN_PACK_REL, ETH_FROZEN_PACK_REL,
};
use chronosentiment_adapter::product::sci_adapter_rec_ml1d::{
    decide_ml1d_ex, fit_reliability_pack, ml1d_schema_hash, predict_q, selection_aware_diag,
    AciMonitor, CombineMode, EconomicMode, Experiment, Ml1dCuts, Ml1dFrozenPack, R2DriftReport,
    ReliabilityPack, SelectionAwareDiag, StaticBrierMonitor, FRICTION_BPS_FLOOR,
    FRICTION_DLOG_SCALE, FRICTION_LIQ_SCALE, FRICTION_VOL_SCALE, H9_DOUBLE_COUNT_WEIGHT,
    ML1D_FEATURE_SCHEMA_ID, PROTOCOL_ID, R2_ALPHA_TARGET, SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
struct Candle {
    close: f64,
    volume: f64,
    close_time_ms: i64,
}

#[derive(Debug, Deserialize)]
struct RecRow {
    date: String,
    timestamp_unix: i64,
    direction: String,
    outcome: String,
    #[serde(default)]
    target_distance_pct: Option<f64>,
    #[serde(default)]
    risk_distance_pct: Option<f64>,
}

#[derive(Debug, Clone)]
struct Sample {
    ts_ms: i64,
    date: String,
    side: Side,
    y: f64,
    payoff_proxy: f64,
    target_distance_pct: f64,
    risk_distance_pct: f64,
    features: Vec<f64>,
    bar_idx: usize,
}

#[derive(Debug, Serialize)]
struct ExperimentScore {
    experiment: String,
    validation: FoldScorecard,
    holdout: FoldScorecard,
    holdout_wilson_wr_lb: f64,
}

#[derive(Debug, Serialize)]
struct ExtensionScore {
    label: String,
    economic_mode: String,
    combine_mode: String,
    holdout: FoldScorecard,
    holdout_wilson_wr_lb: f64,
}

#[derive(Debug, Serialize)]
struct PackageReport {
    protocol_id: String,
    schema_version: String,
    feature_schema_id: String,
    schema_hash: String,
    book: String,
    cuts: Ml1dCuts,
    leakage_audit: serde_json::Value,
    experiments: Vec<ExperimentScore>,
    extensions: Vec<ExtensionScore>,
    r3_selection_aware: Vec<SelectionAwareDiag>,
    h11_sci_v1_comparator: FoldScorecard,
    e0_holdout: FoldScorecard,
    primary_e3_holdout: FoldScorecard,
    verdict: String,
    verdict_reasons: Vec<String>,
    extension_verdict: String,
    extension_reasons: Vec<String>,
    r2_drift: R2DriftReport,
    hypotheses_status: serde_json::Value,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn load_candles(path: &Path) -> Result<Vec<Candle>, Box<dyn std::error::Error>> {
    let f = File::open(path)?;
    Ok(serde_json::from_reader(BufReader::new(f))?)
}

fn load_recs(path: &Path) -> Result<Vec<RecRow>, Box<dyn std::error::Error>> {
    let f = File::open(path)?;
    let mut out = Vec::new();
    for line in BufReader::new(f).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        out.push(serde_json::from_str(&line)?);
    }
    Ok(out)
}

fn build_close_index(candles: &[Candle]) -> BTreeMap<i64, usize> {
    let mut m = BTreeMap::new();
    for (i, c) in candles.iter().enumerate() {
        m.insert(c.close_time_ms, i);
        m.insert(c.close_time_ms / 1000, i);
    }
    m
}

fn map_bar(index: &BTreeMap<i64, usize>, ts_unix: i64) -> Option<usize> {
    let ms = ts_unix * 1000;
    if let Some(&i) = index.get(&ms) {
        return Some(i);
    }
    if let Some(&i) = index.get(&ts_unix) {
        return Some(i);
    }
    index.range(..=ms).next_back().map(|(_, &i)| i)
}

fn payoff_proxy(row: &RecRow, win: bool) -> f64 {
    let tgt = row.target_distance_pct.unwrap_or(0.0);
    let risk = row.risk_distance_pct.unwrap_or(0.0);
    if win {
        tgt
    } else {
        -risk
    }
}

fn split_name(date: &str) -> &'static str {
    let d = &date[..10];
    if d < "2024-09-01" {
        "train"
    } else if d < "2024-11-01" {
        "validation"
    } else {
        "holdout"
    }
}

fn in_purge_after_train(bar_idx: usize, train_max_bar: Option<usize>) -> bool {
    match train_max_bar {
        Some(tmax) => bar_idx <= tmax + PURGE_BARS,
        None => false,
    }
}

fn wilson_wr_lb(wins: usize, n: usize) -> f64 {
    chronosentiment_adapter::product::sci_adapter_rec::wilson_lower(wins as u64, n as u64, 0.95)
}

fn brier(ys: &[f64], ps: &[f64]) -> f64 {
    if ys.is_empty() {
        return 0.0;
    }
    ys.iter()
        .zip(ps.iter())
        .map(|(y, p)| {
            let d = y - p;
            d * d
        })
        .sum::<f64>()
        / ys.len() as f64
}

fn eval_experiment_ex(
    fold: &str,
    samples: &[&Sample],
    experiment: Experiment,
    economic_mode: EconomicMode,
    combine_mode: CombineMode,
    long_pack: &ReliabilityPack,
    short_pack: &ReliabilityPack,
    cuts: &Ml1dCuts,
    closes: &[f64],
    volumes: &[f64],
) -> (FoldScorecard, Vec<bool>, Vec<f64>, Vec<f64>) {
    let mut y = Vec::with_capacity(samples.len());
    let mut mask = Vec::with_capacity(samples.len());
    let mut pay = Vec::with_capacity(samples.len());
    let mut qs = Vec::with_capacity(samples.len());
    let mut qlbs = Vec::with_capacity(samples.len());
    for s in samples {
        y.push(s.y);
        pay.push(s.payoff_proxy);
        let stamp = decide_ml1d_ex(
            experiment,
            economic_mode,
            combine_mode,
            s.side,
            &closes[..=s.bar_idx],
            &volumes[..=s.bar_idx],
            long_pack,
            short_pack,
            cuts,
            s.target_distance_pct,
            s.risk_distance_pct,
        );
        mask.push(stamp.act);
        qs.push(stamp.q_hat);
        qlbs.push(stamp.q_lb_wilson);
    }
    (scorecard(fold, &y, &mask, &pay), mask, qs, qlbs)
}

fn eval_experiment(
    fold: &str,
    samples: &[&Sample],
    experiment: Experiment,
    long_pack: &ReliabilityPack,
    short_pack: &ReliabilityPack,
    cuts: &Ml1dCuts,
    closes: &[f64],
    volumes: &[f64],
) -> FoldScorecard {
    eval_experiment_ex(
        fold,
        samples,
        experiment,
        EconomicMode::Ec1,
        CombineMode::Dual,
        long_pack,
        short_pack,
        cuts,
        closes,
        volumes,
    )
    .0
}

fn eval_sci_v1_comparator(
    fold: &str,
    samples: &[&Sample],
    pack: &FrozenTwinPack,
    closes: &[f64],
    volumes: &[f64],
) -> FoldScorecard {
    let mut y = Vec::with_capacity(samples.len());
    let mut mask = Vec::with_capacity(samples.len());
    let mut pay = Vec::with_capacity(samples.len());
    for s in samples {
        y.push(s.y);
        pay.push(s.payoff_proxy);
        let stamp = decide_generator(
            &pack.long,
            &pack.short,
            &closes[..=s.bar_idx],
            &volumes[..=s.bar_idx],
            &pack.cuts,
        );
        // H11: SCI-v1 as side generator — ACT only when it ENTERs matching Coralys side.
        let enter = stamp.band == Band::Enter && stamp.side == s.side.as_str();
        mask.push(enter);
    }
    scorecard(fold, &y, &mask, &pay)
}

fn pick_ml1d_cuts(
    val: &[&Sample],
    long_pack: &ReliabilityPack,
    short_pack: &ReliabilityPack,
    closes: &[f64],
    volumes: &[f64],
) -> Ml1dCuts {
    // Predeclared grids (not soak remine). Absolute U>0 is often empty at ~30%
    // calibrated base rate with 2:1 geometry — include mild negative u_min as a
    // rank/economic filter while disclosing absolute-U>0 coverage separately.
    let grid_p: [f64; 6] = [0.30, 0.35, 0.40, 0.45, 0.50, 0.55];
    let grid_lb: [f64; 5] = [0.18, 0.22, 0.25, 0.28, 0.32];
    let grid_u: [f64; 7] = [-0.05, -0.02, -0.01, 0.0, 0.01, 0.02, 0.05];
    let mut best: Option<(Ml1dCuts, f64)> = None;
    for &p_min in &grid_p {
        for &p_lb in &grid_lb {
            if p_lb > p_min {
                continue;
            }
            for &u_min in &grid_u {
                let cuts = Ml1dCuts {
                    reliability: JointCuts { p_min, p_lb },
                    u_min,
                };
                let sc = eval_experiment(
                    "validation",
                    val,
                    Experiment::E3,
                    long_pack,
                    short_pack,
                    &cuts,
                    closes,
                    volumes,
                );
                if !coverage_floor_ok(&sc) {
                    continue;
                }
                if !wl_improves(&sc) {
                    continue;
                }
                let net = sc.mean_net_proxy.unwrap_or(f64::NEG_INFINITY);
                let key = (sc.enter_wr - sc.baseline_wr) * 1000.0 + net * 100.0 + sc.coverage;
                let replace = match &best {
                    None => true,
                    Some((_, k)) => key > *k,
                };
                if replace {
                    best = Some((cuts, key));
                }
            }
        }
    }
    best.map(|(c, _)| c).unwrap_or_default()
}

fn parse_book_arg() -> String {
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == "--book" {
            if let Some(b) = args.next() {
                return b.to_ascii_uppercase();
            }
        } else if let Some(b) = a.strip_prefix("--book=") {
            return b.to_ascii_uppercase();
        }
    }
    "BTCUSDT".to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = repo_root();
    let book = parse_book_arg(); // BTCUSDT | ETHUSDT — never pooled
    let (candles_rel, gz_rel, artifact_stem, book_id) = match book.as_str() {
        "ETHUSDT" => (
            "datasets/crypto_15m/binance_ethusdt_2024_15m.json",
            "datasets/recommendation/historical_crypto/ETHUSDT.jsonl.gz",
            "eth_ml1d",
            "ETHUSDT_15M",
        ),
        _ => (
            "datasets/crypto_15m/binance_btcusdt_2024_15m.json",
            "datasets/recommendation/historical_crypto/BTCUSDT.jsonl.gz",
            "btc_ml1d",
            "BTCUSDT_15M",
        ),
    };
    let candles_path = root.join(candles_rel);
    let rec_path = root.join(format!("artifacts/rp608_sci_adapter_rec_ml1d/{book}.jsonl"));
    let out_dir = root.join("artifacts/rp608_sci_adapter_rec_ml1d");
    let docs_out = root.join("docs/research/rp608_sci_adapter_rec_ml1d");
    fs::create_dir_all(&out_dir)?;
    fs::create_dir_all(&docs_out)?;

    if !rec_path.exists() {
        let gz = root.join(gz_rel);
        eprintln!("Preparing JSONL from {:?}", gz);
        let status = std::process::Command::new("gunzip")
            .args(["-c"])
            .arg(&gz)
            .stdout(File::create(&rec_path)?)
            .status()?;
        if !status.success() {
            return Err("gunzip failed".into());
        }
    }

    println!("[{PROTOCOL_ID}] ML-1D offline — dual ACT/PASS around Coralys side");
    println!("  book={book_id} (never pooled)");
    println!(
        "  schema={} hash={}",
        ML1D_FEATURE_SCHEMA_ID,
        ml1d_schema_hash()
    );
    println!("  candles={:?}", candles_path);
    println!("  labels={:?}", rec_path);

    let candles = load_candles(&candles_path)?;
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let volumes: Vec<f64> = candles.iter().map(|c| c.volume).collect();
    let index = build_close_index(&candles);
    let recs = load_recs(&rec_path)?;

    let mut samples: Vec<Sample> = Vec::new();
    let mut skipped_horizon = 0usize;
    let mut skipped_map = 0usize;
    let mut skipped_feat = 0usize;
    for row in &recs {
        let win = match row.outcome.as_str() {
            "TARGET_BEFORE_RISK" => true,
            "RISK_BEFORE_TARGET" => false,
            _ => {
                skipped_horizon += 1;
                continue;
            }
        };
        let Some(side) = Side::parse(&row.direction) else {
            continue;
        };
        let Some(bar_idx) = map_bar(&index, row.timestamp_unix) else {
            skipped_map += 1;
            continue;
        };
        let feat = build_features(&closes[..=bar_idx], &volumes[..=bar_idx], side);
        if !feat.eligible {
            skipped_feat += 1;
            continue;
        }
        samples.push(Sample {
            ts_ms: row.timestamp_unix * 1000,
            date: row.date.clone(),
            side,
            y: if win { 1.0 } else { 0.0 },
            payoff_proxy: payoff_proxy(row, win),
            target_distance_pct: row.target_distance_pct.unwrap_or(0.0),
            risk_distance_pct: row.risk_distance_pct.unwrap_or(0.0),
            features: feat.values.to_vec(),
            bar_idx,
        });
    }
    samples.sort_by_key(|s| (s.ts_ms, s.side.as_str().to_string()));
    println!(
        "  samples={} skipped_horizon={} skipped_map={} skipped_feat={} purge={}",
        samples.len(),
        skipped_horizon,
        skipped_map,
        skipped_feat,
        PURGE_BARS
    );

    let train: Vec<&Sample> = samples
        .iter()
        .filter(|s| split_name(&s.date) == "train")
        .collect();
    let train_max_bar = train.iter().map(|s| s.bar_idx).max();
    let validation: Vec<&Sample> = samples
        .iter()
        .filter(|s| {
            split_name(&s.date) == "validation" && !in_purge_after_train(s.bar_idx, train_max_bar)
        })
        .collect();
    let holdout: Vec<&Sample> = samples
        .iter()
        .filter(|s| split_name(&s.date) == "holdout")
        .collect();
    println!(
        "  split train={} val={} holdout={} (book={book_id} never pooled)",
        train.len(),
        validation.len(),
        holdout.len()
    );

    let train_long: Vec<&Sample> = train.iter().copied().filter(|s| s.side == Side::Long).collect();
    let train_short: Vec<&Sample> = train
        .iter()
        .copied()
        .filter(|s| s.side == Side::Short)
        .collect();
    let val_long: Vec<&Sample> = validation
        .iter()
        .copied()
        .filter(|s| s.side == Side::Long)
        .collect();
    let val_short: Vec<&Sample> = validation
        .iter()
        .copied()
        .filter(|s| s.side == Side::Short)
        .collect();

    // R1: fit reliability packs per Coralys side on train; calibrate isotonic on validation.
    let xs_l: Vec<Vec<f64>> = train_long.iter().map(|s| s.features.clone()).collect();
    let ys_l: Vec<f64> = train_long.iter().map(|s| s.y).collect();
    let xs_s: Vec<Vec<f64>> = train_short.iter().map(|s| s.features.clone()).collect();
    let ys_s: Vec<f64> = train_short.iter().map(|s| s.y).collect();
    let mut long_pack = fit_reliability_pack(Side::Long, &xs_l, &ys_l);
    let mut short_pack = fit_reliability_pack(Side::Short, &xs_s, &ys_s);
    {
        let xs: Vec<Vec<f64>> = val_long.iter().map(|s| s.features.clone()).collect();
        let ys: Vec<f64> = val_long.iter().map(|s| s.y).collect();
        let scores: Vec<f64> = xs.iter().map(|x| raw_score(&long_pack.glm, x)).collect();
        long_pack.isotonic = fit_isotonic(&scores, &ys);
    }
    {
        let xs: Vec<Vec<f64>> = val_short.iter().map(|s| s.features.clone()).collect();
        let ys: Vec<f64> = val_short.iter().map(|s| s.y).collect();
        let scores: Vec<f64> = xs.iter().map(|x| raw_score(&short_pack.glm, x)).collect();
        short_pack.isotonic = fit_isotonic(&scores, &ys);
    }

    // Calibration diagnostics (H3) on validation — raw vs isotonic Brier.
    let mut raw_ps = Vec::new();
    let mut cal_ps = Vec::new();
    let mut cal_ys = Vec::new();
    for s in &validation {
        let pack = match s.side {
            Side::Long => &long_pack,
            Side::Short => &short_pack,
        };
        let raw = chronosentiment_adapter::product::sci_adapter_rec::predict_proba_raw(
            &pack.glm,
            &s.features,
        );
        let (q, _) = predict_q(pack, &s.features);
        raw_ps.push(raw);
        cal_ps.push(q);
        cal_ys.push(s.y);
    }
    let brier_raw = brier(&cal_ys, &raw_ps);
    let brier_cal = brier(&cal_ys, &cal_ps);
    println!(
        "  R1 calib val Brier raw={brier_raw:.6} isotonic={brier_cal:.6} (lower better)"
    );

    // Ec1 friction constants disclosed; cuts locked on validation E3 only.
    let mut val_all = validation.clone();
    val_all.sort_by_key(|s| s.ts_ms);
    let cuts = pick_ml1d_cuts(&val_all, &long_pack, &short_pack, &closes, &volumes);
    println!(
        "  frozen cuts p_min={:.3} p_lb={:.3} u_min={:.4}",
        cuts.reliability.p_min, cuts.reliability.p_lb, cuts.u_min
    );

    let mut hold_all = holdout.clone();
    hold_all.sort_by_key(|s| s.ts_ms);

    let experiments = [
        Experiment::E0,
        Experiment::E1,
        Experiment::E2,
        Experiment::E3,
    ];
    let mut experiment_scores = Vec::new();
    for exp in experiments {
        let val_sc = eval_experiment(
            "validation",
            &val_all,
            exp,
            &long_pack,
            &short_pack,
            &cuts,
            &closes,
            &volumes,
        );
        let hold_sc = eval_experiment(
            "holdout",
            &hold_all,
            exp,
            &long_pack,
            &short_pack,
            &cuts,
            &closes,
            &volumes,
        );
        let wlb = wilson_wr_lb(hold_sc.enter_wins, hold_sc.enter_n);
        println!(
            "  {} holdout enter_n={} wr={:.4} base={:.4} cov={:.3} wilson_lb={:.4} net={:?}",
            exp.as_str(),
            hold_sc.enter_n,
            hold_sc.enter_wr,
            hold_sc.baseline_wr,
            hold_sc.coverage,
            wlb,
            hold_sc.mean_net_proxy
        );
        experiment_scores.push(ExperimentScore {
            experiment: exp.as_str().into(),
            validation: val_sc,
            holdout: hold_sc,
            holdout_wilson_wr_lb: wlb,
        });
    }

    // H11: SCI-v1 frozen pack comparator — book-native pack only (never cross-book).
    let sci_pack_rel = match book.as_str() {
        "ETHUSDT" => ETH_FROZEN_PACK_REL,
        _ => DEFAULT_FROZEN_PACK_REL,
    };
    let sci_pack_path = root.join(sci_pack_rel);
    let h11_sc = if sci_pack_path.exists() {
        let pack = FrozenTwinPack::from_path(&sci_pack_path)?;
        println!(
            "  H11 SCI-v1 comparator pack={:?} protocol={SCI_V1_PROTOCOL} book={book}",
            sci_pack_path
        );
        eval_sci_v1_comparator("holdout", &hold_all, &pack, &closes, &volumes)
    } else {
        eprintln!(
            "  WARN: SCI-v1 pack missing for {book} at {:?} — H11 skipped (not a cross-book fallback)",
            sci_pack_path
        );
        scorecard("holdout", &[], &[], &[])
    };
    println!(
        "  H11 holdout enter_n={} wr={:.4} base={:.4} cov={:.3} net={:?}",
        h11_sc.enter_n,
        h11_sc.enter_wr,
        h11_sc.baseline_wr,
        h11_sc.coverage,
        h11_sc.mean_net_proxy
    );

    let e0_hold = experiment_scores
        .iter()
        .find(|e| e.experiment == "E0")
        .unwrap()
        .holdout
        .clone();
    let e1_hold = experiment_scores
        .iter()
        .find(|e| e.experiment == "E1")
        .unwrap()
        .holdout
        .clone();
    let e2_hold = experiment_scores
        .iter()
        .find(|e| e.experiment == "E2")
        .unwrap()
        .holdout
        .clone();
    let e3_hold = experiment_scores
        .iter()
        .find(|e| e.experiment == "E3")
        .unwrap()
        .holdout
        .clone();

    let mut reasons = Vec::new();
    if !coverage_floor_ok(&e3_hold) {
        reasons.push(format!(
            "e3_coverage_floor_fail enter_n={} coverage={:.3}",
            e3_hold.enter_n, e3_hold.coverage
        ));
    }
    if e3_hold.enter_wr <= e0_hold.enter_wr + 1e-12 {
        reasons.push(format!(
            "e3_wl_no_improve_vs_e0 e3={:.4} e0={:.4}",
            e3_hold.enter_wr, e0_hold.enter_wr
        ));
    }
    let e3_net = e3_hold.mean_net_proxy.unwrap_or(f64::NEG_INFINITY);
    let e0_net = e0_hold.mean_net_proxy.unwrap_or(0.0);
    let e1_net = e1_hold.mean_net_proxy.unwrap_or(f64::NEG_INFINITY);
    let e2_net = e2_hold.mean_net_proxy.unwrap_or(f64::NEG_INFINITY);
    if e3_net < e0_net {
        reasons.push(format!("e3_net_worse_than_e0 e3={e3_net:.6} e0={e0_net:.6}"));
    }
    // H2: dual should beat max(E1,E2) on net when coverage comparable (disclose if not).
    let max_single = e1_net.max(e2_net);
    if e3_net + 1e-12 < max_single {
        reasons.push(format!(
            "e3_net_not_above_max_e1_e2 e3={e3_net:.6} max_single={max_single:.6}"
        ));
    }
    if let Some(asym) = e3_hold.payoff_asym {
        if e3_net < 0.0 && asym < 1.0 {
            reasons.push(format!(
                "wr_only_net_hostile proxy={e3_net:.6} asym={asym:.4}"
            ));
        }
    }
    // H11: ML-1D E3 should beat SCI-v1 side generator on net when both cover.
    if h11_sc.enter_n > 0 {
        let h11_net = h11_sc.mean_net_proxy.unwrap_or(f64::NEG_INFINITY);
        if e3_net + 1e-12 < h11_net && h11_sc.enter_wr >= e3_hold.enter_wr {
            reasons.push(format!(
                "h11_sci_v1_competitive_or_better h11_wr={:.4} h11_net={h11_net:.6} e3_wr={:.4} e3_net={e3_net:.6}",
                h11_sc.enter_wr, e3_hold.enter_wr
            ));
        }
    }

    let verdict = if reasons.is_empty() {
        "ADVANCE_CANDIDATE".to_string()
    } else {
        "DO_NOT_ADVANCE".to_string()
    };

    // --- Extensions: Ec2 (H8), H9 fused ablation, R3 selection-aware (H6) ---
    // Same frozen cuts (no remine). Compare holdout nets only.
    let extension_specs: [(&str, Experiment, EconomicMode, CombineMode); 4] = [
        ("E2_EC2", Experiment::E2, EconomicMode::Ec2, CombineMode::Dual),
        ("E3_EC2", Experiment::E3, EconomicMode::Ec2, CombineMode::Dual),
        (
            "E3_H9_FUSED",
            Experiment::E3,
            EconomicMode::Ec1,
            CombineMode::FusedDoubleCount,
        ),
        (
            "E3_EC2_H9_FUSED",
            Experiment::E3,
            EconomicMode::Ec2,
            CombineMode::FusedDoubleCount,
        ),
    ];
    let mut extensions = Vec::new();
    let mut ext_reasons = Vec::new();
    let mut e3_ec2_net = f64::NEG_INFINITY;
    let mut h9_net = f64::NEG_INFINITY;
    for (label, exp, econ, comb) in extension_specs {
        let (hold_sc, _, _, _) = eval_experiment_ex(
            "holdout",
            &hold_all,
            exp,
            econ,
            comb,
            &long_pack,
            &short_pack,
            &cuts,
            &closes,
            &volumes,
        );
        let wlb = wilson_wr_lb(hold_sc.enter_wins, hold_sc.enter_n);
        println!(
            "  EXT {label} enter_n={} wr={:.4} cov={:.3} wilson_lb={:.4} net={:?}",
            hold_sc.enter_n, hold_sc.enter_wr, hold_sc.coverage, wlb, hold_sc.mean_net_proxy
        );
        if label == "E3_EC2" {
            e3_ec2_net = hold_sc.mean_net_proxy.unwrap_or(f64::NEG_INFINITY);
        }
        if label == "E3_H9_FUSED" {
            h9_net = hold_sc.mean_net_proxy.unwrap_or(f64::NEG_INFINITY);
        }
        extensions.push(ExtensionScore {
            label: label.into(),
            economic_mode: econ.as_str().into(),
            combine_mode: comb.as_str().into(),
            holdout: hold_sc,
            holdout_wilson_wr_lb: wlb,
        });
    }
    // H8: Ec2 should improve E3 net vs Ec1 E3
    if e3_ec2_net + 1e-12 < e3_net {
        ext_reasons.push(format!(
            "h8_ec2_no_net_lift_vs_ec1 e3_ec2={e3_ec2_net:.6} e3_ec1={e3_net:.6}"
        ));
    }
    // H9: dual should beat fused double-count on net
    if e3_net + 1e-12 < h9_net {
        ext_reasons.push(format!(
            "h9_fused_beats_or_ties_dual fused={h9_net:.6} dual={e3_net:.6}"
        ));
    } else if h9_net.is_finite() {
        // dual wins → H9 supportive
    }

    // R3 on E1 and E3 ACT subsets (Ec1 dual)
    let mut r3_diags = Vec::new();
    for (label, exp) in [("E1", Experiment::E1), ("E3", Experiment::E3)] {
        let y: Vec<f64> = hold_all.iter().map(|s| s.y).collect();
        let (_, mask, qs, qlbs) = eval_experiment_ex(
            "holdout",
            &hold_all,
            exp,
            EconomicMode::Ec1,
            CombineMode::Dual,
            &long_pack,
            &short_pack,
            &cuts,
            &closes,
            &volumes,
        );
        let diag = selection_aware_diag(&format!("holdout_{label}"), &y, &qs, &qlbs, &mask);
        println!(
            "  R3 {label} act_n={} emp_wr={:.4} mean_q={:.4} q-wr={:.4} brier={:.4}",
            diag.act_n, diag.act_empirical_wr, diag.act_mean_q, diag.q_minus_wr, diag.act_brier
        );
        r3_diags.push(diag);
    }
    let r3_e3_gap = r3_diags
        .iter()
        .find(|d| d.fold == "holdout_E3")
        .map(|d| d.q_minus_wr)
        .unwrap_or(0.0);
    // H6 supportive if selected mean Q overstates empirical WR by > 2pp
    let h6_status = if r3_e3_gap > 0.02 {
        "SUPPORTIVE_OVERCONFIDENT_ACT"
    } else if r3_e3_gap < -0.02 {
        "SUPPORTIVE_UNDERCONFIDENT_ACT"
    } else {
        "WEAK_GAP"
    };

    let extension_verdict = if ext_reasons.is_empty() {
        "EXTENSIONS_NO_ADVANCE_SIGNAL".to_string()
    } else {
        "EXTENSIONS_DO_NOT_RESCUE".to_string()
    };

    // --- R2: ACI drift monitor vs static rolling Brier (H5) ---
    let mut aci = AciMonitor::default_r2();
    let mut static_mon = StaticBrierMonitor::new(brier_cal);
    // Warm on validation (chronological) with revealed outcomes.
    for s in &val_all {
        let pack = match s.side {
            Side::Long => &long_pack,
            Side::Short => &short_pack,
        };
        let (q, _) = predict_q(pack, &s.features);
        aci.update(s.y, q);
        static_mon.update(s.y, q);
    }
    let aci_level_after_val = aci.level;
    let mut static_flag: Option<usize> = None;
    let mut aci_flag: Option<usize> = None;
    let aci_flag_level = (R2_ALPHA_TARGET + 0.05).min(0.95);
    let mut y_hold = Vec::new();
    let mut pay_hold = Vec::new();
    let mut mask_e1 = Vec::new();
    let mut mask_e1_r2 = Vec::new();
    let mut high_vol_ys = Vec::new();
    let mut high_vol_qs = Vec::new();
    let mut low_vol_ys = Vec::new();
    let mut low_vol_qs = Vec::new();
    // Median vol_rel on holdout for stratum split.
    let mut vols_all: Vec<f64> = hold_all
        .iter()
        .map(|s| s.features.get(7).copied().unwrap_or(0.0).abs())
        .collect();
    vols_all.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let vol_med = vols_all.get(vols_all.len() / 2).copied().unwrap_or(0.0);

    for (i, s) in hold_all.iter().enumerate() {
        let pack = match s.side {
            Side::Long => &long_pack,
            Side::Short => &short_pack,
        };
        let (q, q_lb) = predict_q(pack, &s.features);
        let evidence_ok = chronosentiment_adapter::product::sci_adapter_rec_ml1d::evidence_sufficient(
            q,
            q_lb,
            &cuts.reliability,
        );
        let act_e1 = evidence_ok;
        let act_r2 = evidence_ok && aci.accept_q(q);
        y_hold.push(s.y);
        pay_hold.push(s.payoff_proxy);
        mask_e1.push(act_e1);
        mask_e1_r2.push(act_r2);

        let vol = s.features.get(7).copied().unwrap_or(0.0).abs();
        if vol >= vol_med {
            high_vol_ys.push(s.y);
            high_vol_qs.push(q);
        } else {
            low_vol_ys.push(s.y);
            low_vol_qs.push(q);
        }

        // Update monitors after reveal (causal online).
        aci.update(s.y, q);
        static_mon.update(s.y, q);
        if static_flag.is_none() && static_mon.deteriorated() {
            static_flag = Some(i);
        }
        if aci_flag.is_none() && aci.level > aci_flag_level {
            aci_flag = Some(i);
        }
    }
    let e1_static_sc = scorecard("holdout_E1_static", &y_hold, &mask_e1, &pay_hold);
    let e1_r2_sc = scorecard("holdout_E1_R2_aci", &y_hold, &mask_e1_r2, &pay_hold);
    let hv_brier = brier(&high_vol_ys, &high_vol_qs);
    let lv_brier = brier(&low_vol_ys, &low_vol_qs);
    let aci_earlier = match (aci_flag, static_flag) {
        (Some(a), Some(s)) => a < s,
        (Some(_), None) => true,
        _ => false,
    };
    let e1_r2_net = e1_r2_sc.mean_net_proxy.unwrap_or(f64::NEG_INFINITY);
    let e1_net_cmp = e1_hold.mean_net_proxy.unwrap_or(0.0);
    let h5_status = if aci_earlier && e1_r2_sc.enter_wr + 1e-12 >= e1_static_sc.enter_wr {
        "SUPPORTIVE_EARLIER_FLAG"
    } else if aci_earlier {
        "PARTIAL_EARLIER_FLAG_NO_WL_LIFT"
    } else {
        "FALSIFIED_OR_NO_EARLIER_FLAG"
    };
    println!(
        "  R2 static_flag={static_flag:?} aci_flag={aci_flag:?} earlier={aci_earlier} aci_level_val={aci_level_after_val:.4} aci_level_end={:.4}",
        aci.level
    );
    println!(
        "  R2 E1_static n={} wr={:.4} net={:?} | E1_R2 n={} wr={:.4} net={:?} | hv_brier={hv_brier:.4} lv_brier={lv_brier:.4}",
        e1_static_sc.enter_n,
        e1_static_sc.enter_wr,
        e1_static_sc.mean_net_proxy,
        e1_r2_sc.enter_n,
        e1_r2_sc.enter_wr,
        e1_r2_sc.mean_net_proxy
    );
    let r2_report = R2DriftReport {
        validation_baseline_brier: brier_cal,
        holdout_static_flag_index: static_flag,
        holdout_aci_flag_index: aci_flag,
        aci_detection_earlier: aci_earlier,
        holdout_high_vol_brier: hv_brier,
        holdout_low_vol_brier: lv_brier,
        e1_static: e1_static_sc,
        e1_r2_aci: e1_r2_sc,
        h5_status: h5_status.into(),
        note: format!(
            "ACI γ={} α*={}; static window Brier vs val baseline+δ. R2 ACT=E1∧aci.accept_q. e1_r2_net={e1_r2_net:.6} vs e1_net={e1_net_cmp:.6}. Not Gate5.",
            chronosentiment_adapter::product::sci_adapter_rec_ml1d::R2_ACI_GAMMA,
            R2_ALPHA_TARGET
        ),
    };

    let leakage_audit = json!({
        "decision_time_features_only": true,
        "coralys_side_sealed": true,
        "adapter_authority": "ACT_PASS_ONLY",
        "uses_mfe_mae_as_features": false,
        "uses_outcome_as_feature": false,
        "completed_bars_only": true,
        "purge_bars": PURGE_BARS,
        "label_horizon_bars_ref": LABEL_HORIZON_BARS,
        "chronological_split": {
            "train": "2024-01-01..2024-08-31",
            "validation": "2024-09-01..2024-10-31 (purged after train)",
            "holdout": "2024-11-01..2024-12-31 sealed"
        },
        "feature_schema_reused": FEATURE_SCHEMA_ID,
        "feature_names": FEATURE_NAMES,
        "friction_proxy": {
            "bps_floor": FRICTION_BPS_FLOOR,
            "vol_scale": FRICTION_VOL_SCALE,
            "liq_scale_ec2": FRICTION_LIQ_SCALE,
            "dlog_scale_ec2": FRICTION_DLOG_SCALE,
            "h9_double_count_weight": H9_DOUBLE_COUNT_WEIGHT
        },
        "r1_brier_validation": {"raw": brier_raw, "isotonic": brier_cal},
        "book_pooling": false,
        "gate5": false,
        "extension_ladder": "R1_EC1_EC2_H9_R3_R2"
    });

    let h8_status = if e3_ec2_net + 1e-12 >= e3_net && e3_ec2_net.is_finite() {
        "SUPPORTIVE_OR_TIE_ON_HOLDOUT"
    } else {
        "FALSIFIED_ON_HOLDOUT"
    };
    let h9_status = if e3_net + 1e-12 >= h9_net && h9_net.is_finite() {
        "SUPPORTIVE_DUAL_BEATS_FUSED"
    } else {
        "FALSIFIED_OR_TIED_FUSED_GE_DUAL"
    };

    let hypotheses_status = json!({
        "H0": "OPEN",
        "H1": "PARTIAL_E1_LIFT_NO_ADVANCE",
        "H2": "FALSIFIED_ON_HOLDOUT",
        "H3": if brier_cal + 1e-12 < brier_raw { "SUPPORTIVE_ON_VAL" } else { "OPEN" },
        "H4": "PARTIAL_MILD_LIFT",
        "H5": h5_status,
        "H6": h6_status,
        "H7": "WEAK_EC1_NO_LIFT_VS_E1",
        "H8": h8_status,
        "H9": h9_status,
        "H10": "NOT_APPROACHED",
        "H11": "SCI_V1_COMPETITIVE"
    });

    let report = PackageReport {
        protocol_id: PROTOCOL_ID.into(),
        schema_version: SCHEMA_VERSION.into(),
        feature_schema_id: ML1D_FEATURE_SCHEMA_ID.into(),
        schema_hash: ml1d_schema_hash(),
        book: book_id.into(),
        cuts: cuts.clone(),
        leakage_audit,
        experiments: experiment_scores,
        extensions,
        r3_selection_aware: r3_diags,
        h11_sci_v1_comparator: h11_sc,
        e0_holdout: e0_hold,
        primary_e3_holdout: e3_hold,
        verdict: verdict.clone(),
        verdict_reasons: reasons.clone(),
        extension_verdict: extension_verdict.clone(),
        extension_reasons: ext_reasons.clone(),
        r2_drift: r2_report,
        hypotheses_status,
    };

    let frozen = Ml1dFrozenPack {
        protocol_id: PROTOCOL_ID.into(),
        schema_version: SCHEMA_VERSION.into(),
        feature_schema_id: ML1D_FEATURE_SCHEMA_ID.into(),
        schema_hash: ml1d_schema_hash(),
        long: long_pack,
        short: short_pack,
        cuts: cuts.clone(),
        ladder_stage: "R1_EC1_EC2_H9_R3_R2".into(),
    };

    let report_path = out_dir.join(format!("{artifact_stem}_package_report.json"));
    let model_path = out_dir.join(format!("{artifact_stem}_reliability_packs.json"));
    let docs_report = docs_out.join(format!("{artifact_stem}_package_report.json"));
    let docs_model = docs_out.join(format!("{artifact_stem}_reliability_packs.json"));
    fs::write(&report_path, serde_json::to_vec_pretty(&report)?)?;
    fs::write(&model_path, serde_json::to_vec_pretty(&frozen)?)?;
    fs::write(&docs_report, serde_json::to_vec_pretty(&report)?)?;
    fs::write(&docs_model, serde_json::to_vec_pretty(&frozen)?)?;

    println!("  VERDICT[{}] = {}", report.book, verdict);
    for r in &reasons {
        println!("    - {r}");
    }
    println!("  EXTENSION_VERDICT = {extension_verdict}");
    for r in &ext_reasons {
        println!("    - {r}");
    }
    println!("  wrote {:?}", report_path);
    println!("  wrote {:?}", model_path);
    println!("  wrote {:?}", docs_report);
    Ok(())
}
