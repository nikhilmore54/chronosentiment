//! RP-608-SCI-ADAPTER-REC-001 — P1 offline sealed package (BTC 15m first).
//!
//! Chronological train / purge / validation / holdout. Adaptive GLM A→B ladder.
//! Joint p̂ + Wilson CI cuts locked on validation before holdout score.
//! Coralys crates untouched. Authority 0%.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use chronosentiment_adapter::product::sci_adapter_rec::{
    build_features, coverage_floor_ok, decide_generator, fit_isotonic,
    fit_logistic_elastic, fit_logistic_l2, raw_score, schema_hash, scorecard, wl_improves, Band,
    FoldScorecard, JointCuts, Side, SideModelPack, ELASTIC_L1, ELASTIC_L2, FEATURE_NAMES,
    FEATURE_SCHEMA_ID, LABEL_HORIZON_BARS, L2_LAMBDA, PROTOCOL_ID, PURGE_BARS, SCHEMA_VERSION,
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
    features: Vec<f64>,
    bar_idx: usize,
}

#[derive(Debug, Serialize)]
struct PackageReport {
    protocol_id: String,
    schema_version: String,
    feature_schema_id: String,
    schema_hash: String,
    book: String,
    ladder_stage: String,
    cuts: JointCuts,
    leakage_audit: serde_json::Value,
    validation: FoldScorecard,
    holdout: FoldScorecard,
    baseline_holdout_wr: f64,
    verdict: String,
    verdict_reasons: Vec<String>,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn load_candles(path: &Path) -> Result<Vec<Candle>, Box<dyn std::error::Error>> {
    let f = File::open(path)?;
    let candles: Vec<Candle> = serde_json::from_reader(BufReader::new(f))?;
    Ok(candles)
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
        // index by close_time seconds and ms
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
    // nearest close_time_ms <= ms
    index.range(..=ms).next_back().map(|(_, &i)| i)
}

fn payoff_proxy(row: &RecRow, win: bool) -> f64 {
    // Decision-time geometry distances as signed proxy (not live friction).
    let tgt = row.target_distance_pct.unwrap_or(0.0);
    let risk = row.risk_distance_pct.unwrap_or(0.0);
    if win {
        tgt
    } else {
        -risk
    }
}

fn split_name(date: &str) -> &'static str {
    // date like 2024-01-01T08:44:59Z
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

fn fit_side_pack(samples: &[&Sample], stage_b: bool) -> SideModelPack {
    let xs: Vec<Vec<f64>> = samples.iter().map(|s| s.features.clone()).collect();
    let ys: Vec<f64> = samples.iter().map(|s| s.y).collect();
    let glm = if stage_b {
        fit_logistic_elastic(&xs, &ys, ELASTIC_L1, ELASTIC_L2)
    } else {
        fit_logistic_l2(&xs, &ys, L2_LAMBDA)
    };
    let scores: Vec<f64> = xs.iter().map(|x| raw_score(&glm, x)).collect();
    // Isotonic is fit on validation in caller; placeholder here unused.
    let iso = fit_isotonic(&scores, &ys);
    SideModelPack {
        side: samples[0].side.as_str().into(),
        glm,
        isotonic: iso,
    }
}

fn eval_fold(
    fold: &str,
    samples: &[&Sample],
    long_pack: &SideModelPack,
    short_pack: &SideModelPack,
    cuts: &JointCuts,
    closes: &[f64],
    volumes: &[f64],
) -> (FoldScorecard, Vec<bool>) {
    let mut y = Vec::with_capacity(samples.len());
    let mut mask = Vec::with_capacity(samples.len());
    let mut pay = Vec::with_capacity(samples.len());
    for s in samples {
        y.push(s.y);
        pay.push(s.payoff_proxy);
        // Generator decision uses full history up to bar; compare chosen side to row side.
        let c = &closes[..=s.bar_idx];
        let v = &volumes[..=s.bar_idx];
        let stamp = decide_generator(long_pack, short_pack, c, v, cuts);
        let enter = stamp.band == Band::Enter && stamp.side == s.side.as_str();
        mask.push(enter);
    }
    (scorecard(fold, &y, &mask, &pay), mask)
}

fn pick_cuts_on_validation(
    val_long: &[&Sample],
    val_short: &[&Sample],
    long_glm: &SideModelPack,
    short_glm: &SideModelPack,
    closes: &[f64],
    volumes: &[f64],
) -> (JointCuts, FoldScorecard, String) {
    // Fit isotonic on validation scores for each side (calibration fold).
    let mut long_pack = long_glm.clone();
    let mut short_pack = short_glm.clone();
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

    let mut val_all: Vec<&Sample> = val_long.iter().chain(val_short.iter()).copied().collect();
    val_all.sort_by_key(|s| s.ts_ms);

    // Predeclared grid relative to ~30% REC-001-H base rate (not 50%+ soft-score cuts).
    let grid_p: [f64; 8] = [0.30, 0.32, 0.35, 0.38, 0.42, 0.45, 0.50, 0.55];
    let grid_lb: [f64; 7] = [0.18, 0.22, 0.25, 0.28, 0.30, 0.32, 0.35];
    let mut best: Option<(JointCuts, FoldScorecard, f64)> = None;
    for &p_min in &grid_p {
        for &p_lb in &grid_lb {
            if p_lb > p_min {
                continue;
            }
            let cuts = JointCuts { p_min, p_lb };
            let (sc, _) = eval_fold(
                "validation",
                &val_all,
                &long_pack,
                &short_pack,
                &cuts,
                closes,
                volumes,
            );
            if !coverage_floor_ok(&sc) {
                continue;
            }
            // Require W/L improvement on validation to be eligible; then max lift.
            if !wl_improves(&sc) {
                continue;
            }
            let lift = sc.enter_wr - sc.baseline_wr;
            let key = lift * 1000.0 + sc.enter_wr * 10.0 + sc.coverage;
            let replace = match &best {
                None => true,
                Some((_, _, k)) => key > *k,
            };
            if replace {
                best = Some((cuts, sc, key));
            }
        }
    }
    match best {
        Some((cuts, sc, _)) => (cuts, sc, long_pack.glm.ladder_stage.clone()),
        None => {
            // Fail-closed conservative cuts if nothing meets coverage.
            let cuts = JointCuts {
                p_min: 0.55,
                p_lb: 0.45,
            };
            let (sc, _) = eval_fold(
                "validation",
                &val_all,
                &long_pack,
                &short_pack,
                &cuts,
                closes,
                volumes,
            );
            (cuts, sc, long_pack.glm.ladder_stage.clone())
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = repo_root();
    let candles_path = root.join("datasets/crypto_15m/binance_btcusdt_2024_15m.json");
    let rec_path = root.join("artifacts/rp608_sci_adapter_rec_p1/BTCUSDT.jsonl");
    let out_dir = root.join("artifacts/rp608_sci_adapter_rec_p1");
    fs::create_dir_all(&out_dir)?;

    if !rec_path.exists() {
        // gunzip helper path
        let gz = root.join("datasets/recommendation/historical_crypto/BTCUSDT.jsonl.gz");
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

    println!("[{PROTOCOL_ID}] P1 offline package — BTC 15m price+volume");
    println!("  schema={} hash={}", FEATURE_SCHEMA_ID, schema_hash());
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
            "HORIZON" => {
                skipped_horizon += 1;
                continue;
            }
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
        // Leakage: features use closes/volumes[..=bar_idx] only.
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
            features: feat.values.to_vec(),
            bar_idx,
        });
    }

    samples.sort_by_key(|s| (s.ts_ms, s.side.as_str().to_string()));
    println!(
        "  samples={} skipped_horizon={} skipped_map={} skipped_feat={} label_horizon>={} purge={}",
        samples.len(),
        skipped_horizon,
        skipped_map,
        skipped_feat,
        LABEL_HORIZON_BARS,
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
        "  split train={} val={} holdout={} (book=BTC never pooled)",
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

    // Stage A
    let mut long_pack = fit_side_pack(&train_long, false);
    let mut short_pack = fit_side_pack(&train_short, false);
    let (cuts_a, val_sc_a, _) = pick_cuts_on_validation(
        &val_long,
        &val_short,
        &long_pack,
        &short_pack,
        &closes,
        &volumes,
    );

    let mut ladder = "A".to_string();
    let mut cuts = cuts_a;
    let mut val_sc = val_sc_a;
    let stage_a_ok = coverage_floor_ok(&val_sc) && wl_improves(&val_sc);
    if !stage_a_ok {
        println!("  ladder: Stage A failed val gates → escalate Stage B (elastic)");
        long_pack = fit_side_pack(&train_long, true);
        short_pack = fit_side_pack(&train_short, true);
        let (cuts_b, val_sc_b, _) = pick_cuts_on_validation(
            &val_long,
            &val_short,
            &long_pack,
            &short_pack,
            &closes,
            &volumes,
        );
        cuts = cuts_b;
        val_sc = val_sc_b;
        ladder = "B".to_string();
        long_pack.glm.ladder_stage = "B".into();
        short_pack.glm.ladder_stage = "B".into();
    } else {
        println!("  ladder: Stage A retained");
    }

    // Refit isotonic on validation with frozen packs (already done inside pick_cuts;
    // re-run to attach final iso maps to packs used for holdout).
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

    // Freeze cuts BEFORE holdout (already chosen on validation only).
    println!(
        "  frozen cuts p_min={:.3} p_lb={:.3} ladder={}",
        cuts.p_min, cuts.p_lb, ladder
    );

    let mut hold_all = holdout.clone();
    hold_all.sort_by_key(|s| s.ts_ms);
    let (hold_sc, _) = eval_fold(
        "holdout",
        &hold_all,
        &long_pack,
        &short_pack,
        &cuts,
        &closes,
        &volumes,
    );

    let mut reasons = Vec::new();
    let cov_ok = coverage_floor_ok(&hold_sc);
    let wl_ok = wl_improves(&hold_sc);
    if !cov_ok {
        reasons.push(format!(
            "coverage_floor_fail enter_n={} coverage={:.3}",
            hold_sc.enter_n, hold_sc.coverage
        ));
    }
    if !wl_ok {
        reasons.push(format!(
            "wl_no_improve enter_wr={:.4} baseline_wr={:.4}",
            hold_sc.enter_wr, hold_sc.baseline_wr
        ));
    }
    // Gate 2 v2: net/payoff are required disclosure. Hard-fail only clear WR-only
    // net destruction (asym < 1 and net_proxy < 0). Mild negative proxy with
    // asym ≥ 1 is disclosed, not an automatic veto (frozen exit geometry).
    if wl_ok {
        if let Some(net) = hold_sc.mean_net_proxy {
            let asym = hold_sc.payoff_asym.unwrap_or(0.0);
            if net < 0.0 && asym < 1.0 {
                reasons.push(format!(
                    "wr_only_net_hostile proxy={net:.6} asym={asym:.4}"
                ));
            }
        }
    }
    let verdict = if reasons.is_empty() {
        "ADVANCE_CANDIDATE".to_string()
    } else {
        "DO_NOT_ADVANCE".to_string()
    };

    let leakage_audit = json!({
        "decision_time_features_only": true,
        "uses_mfe_mae_as_features": false,
        "uses_outcome_as_feature": false,
        "completed_bars_only": true,
        "purge_bars": PURGE_BARS,
        "chronological_split": {
            "train": "2024-01-01..2024-08-31",
            "validation": "2024-09-01..2024-10-31 (purged after train)",
            "holdout": "2024-11-01..2024-12-31 sealed"
        },
        "feature_names": FEATURE_NAMES,
        "label": "TARGET_BEFORE_RISK=win; RISK_BEFORE_TARGET=loss; HORIZON censored",
        "book_pooling": false
    });

    let report = PackageReport {
        protocol_id: PROTOCOL_ID.into(),
        schema_version: SCHEMA_VERSION.into(),
        feature_schema_id: FEATURE_SCHEMA_ID.into(),
        schema_hash: schema_hash(),
        book: "BTCUSDT_15M".into(),
        ladder_stage: ladder.clone(),
        cuts: cuts.clone(),
        leakage_audit,
        validation: val_sc,
        holdout: hold_sc.clone(),
        baseline_holdout_wr: hold_sc.baseline_wr,
        verdict: verdict.clone(),
        verdict_reasons: reasons.clone(),
    };

    let report_path = out_dir.join("btc_p1_package_report.json");
    fs::write(&report_path, serde_json::to_vec_pretty(&report)?)?;
    let model_path = out_dir.join("btc_p1_side_models.json");
    fs::write(
        &model_path,
        serde_json::to_vec_pretty(&json!({
            "long": long_pack,
            "short": short_pack,
            "cuts": cuts,
            "ladder_stage": ladder,
        }))?,
    )?;

    println!("  validation: enter_n={} wr={:.4} base_wr={:.4} cov={:.3}",
        report.validation.enter_n, report.validation.enter_wr, report.validation.baseline_wr, report.validation.coverage);
    println!("  holdout:    enter_n={} wr={:.4} base_wr={:.4} cov={:.3} asym={:?} net_proxy={:?}",
        report.holdout.enter_n, report.holdout.enter_wr, report.holdout.baseline_wr, report.holdout.coverage,
        report.holdout.payoff_asym, report.holdout.mean_net_proxy);
    println!("  VERDICT[{}] = {}", report.book, verdict);
    for r in &reasons {
        println!("    - {r}");
    }
    println!("  wrote {:?}", report_path);
    println!("  wrote {:?}", model_path);
    Ok(())
}
