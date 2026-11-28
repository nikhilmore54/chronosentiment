//! RP-608-SCI-ADAPTER-REC-ML1D-P1B — Frozen ML-1D packs → live-plan transfer.
//!
//! Observational only. Applies sealed 2024 ML-1D reliability packs + cuts to
//! 2026 crypto trade-plan outcomes with real net_pnl. Not an ADVANCE packet.
//! Coralys no-touch · books never pooled · authority 0%.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use chronosentiment_adapter::product::sci_adapter_rec::{
    coverage_floor_ok, scorecard, wl_improves, FoldScorecard, Side,
};
use chronosentiment_adapter::product::sci_adapter_rec_ml1d::{
    decide_ml1d_ex, wilson_wr_lb, CombineMode, EconomicMode, Experiment, Ml1dFrozenPack,
    PROTOCOL_ID, SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
struct Candle {
    close: f64,
    volume: f64,
    close_time_ms: i64,
    #[serde(default)]
    open_time_ms: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct LivePlan {
    direction: String,
    entry_time_ms: i64,
    #[serde(default)]
    exit_reason: Option<String>,
    #[serde(default)]
    net_pnl_usdt: Option<f64>,
    #[serde(default)]
    profit_target_price: Option<f64>,
    #[serde(default)]
    stop_loss_price: Option<f64>,
    #[serde(default)]
    entry_price: Option<f64>,
    #[serde(default)]
    #[allow(dead_code)]
    friction_usdt: Option<f64>,
    #[serde(default)]
    #[allow(dead_code)]
    protocol_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct TransferReport {
    protocol_id: String,
    schema_version: String,
    package: String,
    book: String,
    label_source: String,
    candle_source: String,
    frozen_pack: String,
    n_plans: usize,
    n_joined: usize,
    n_skipped_map: usize,
    n_skipped_feat: usize,
    n_horizon_censored: usize,
    span_entry_ms: [i64; 2],
    experiments: Vec<serde_json::Value>,
    verdict: String,
    verdict_reasons: Vec<String>,
    note: String,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn load_candles(path: &Path) -> Result<Vec<Candle>, Box<dyn std::error::Error>> {
    Ok(serde_json::from_reader(BufReader::new(File::open(path)?))?)
}

fn load_plans(path: &Path) -> Result<Vec<LivePlan>, Box<dyn std::error::Error>> {
    let mut out = Vec::new();
    for line in BufReader::new(File::open(path)?).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        out.push(serde_json::from_str(&line)?);
    }
    Ok(out)
}

fn build_index(candles: &[Candle]) -> BTreeMap<i64, usize> {
    let mut m = BTreeMap::new();
    for (i, c) in candles.iter().enumerate() {
        m.insert(c.close_time_ms, i);
        if let Some(ot) = c.open_time_ms {
            m.insert(ot, i);
        }
    }
    m
}

fn map_bar(index: &BTreeMap<i64, usize>, entry_ms: i64) -> Option<usize> {
    // Last completed bar at or before entry.
    index.range(..=entry_ms).next_back().map(|(_, &i)| i)
}

fn distances(plan: &LivePlan) -> (f64, f64) {
    let entry = plan.entry_price.unwrap_or(0.0).abs().max(1e-12);
    let tgt = plan
        .profit_target_price
        .map(|p| ((p - entry).abs() / entry) * 100.0)
        .unwrap_or(0.0);
    let risk = plan
        .stop_loss_price
        .map(|p| ((p - entry).abs() / entry) * 100.0)
        .unwrap_or(0.0);
    (tgt, risk)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = repo_root();
    let candles_path = root.join("datasets/crypto_15m/binance_btcusdt_2026q4_15m.json");
    let plans_path = root.join(
        "artifacts/rp608_econ_edge_diag/source_journals_2026-11-05/crypto/trade_plan_outcomes.jsonl",
    );
    let pack_path = root.join("docs/research/rp608_sci_adapter_rec_ml1d/btc_ml1d_reliability_packs.json");
    let out_dir = root.join("docs/research/rp608_sci_adapter_rec_ml1d");
    fs::create_dir_all(&out_dir)?;

    println!("[{PROTOCOL_ID}] P1B transfer — frozen packs → live plans");
    println!("  candles={:?}", candles_path);
    println!("  plans={:?}", plans_path);
    println!("  pack={:?}", pack_path);

    let candles = load_candles(&candles_path)?;
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let volumes: Vec<f64> = candles.iter().map(|c| c.volume).collect();
    let index = build_index(&candles);
    let plans = load_plans(&plans_path)?;
    let raw = fs::read_to_string(&pack_path)?;
    let pack: Ml1dFrozenPack = serde_json::from_str(&raw)?;

    let mut y = Vec::new();
    let mut pay = Vec::new();
    let mut mask_e0 = Vec::new();
    let mut mask_e1 = Vec::new();
    let mut mask_e3 = Vec::new();
    let mut skipped_map = 0usize;
    let mut skipped_feat = 0usize;
    let mut censored = 0usize;
    let mut joined = 0usize;
    let mut tmin = i64::MAX;
    let mut tmax = i64::MIN;

    for plan in &plans {
        let Some(side) = Side::parse(&plan.direction) else {
            continue;
        };
        // Censor ambiguous / horizon expiry for W/L (TP/SL only) — disclose.
        let win = match plan.exit_reason.as_deref() {
            Some("PROFIT_TARGET_HIT") => true,
            Some("STOP_LOSS_HIT") => false,
            Some("MAX_HORIZON_60M_EXPIRED") | Some("AMBIGUOUS_STOP_TARGET") => {
                censored += 1;
                continue;
            }
            _ => {
                // Fallback: net_pnl sign if reason missing
                match plan.net_pnl_usdt {
                    Some(n) if n > 0.0 => true,
                    Some(n) if n < 0.0 => false,
                    _ => {
                        censored += 1;
                        continue;
                    }
                }
            }
        };
        let Some(bar_idx) = map_bar(&index, plan.entry_time_ms) else {
            skipped_map += 1;
            continue;
        };
        tmin = tmin.min(plan.entry_time_ms);
        tmax = tmax.max(plan.entry_time_ms);
        let (tgt, risk) = distances(plan);
        let stamp_e0 = decide_ml1d_ex(
            Experiment::E0,
            EconomicMode::Ec1,
            CombineMode::Dual,
            side,
            &closes[..=bar_idx],
            &volumes[..=bar_idx],
            &pack.long,
            &pack.short,
            &pack.cuts,
            tgt,
            risk,
        );
        if stamp_e0.band
            == chronosentiment_adapter::product::sci_adapter_rec::Band::Insufficient
        {
            skipped_feat += 1;
            continue;
        }
        let stamp_e1 = decide_ml1d_ex(
            Experiment::E1,
            EconomicMode::Ec1,
            CombineMode::Dual,
            side,
            &closes[..=bar_idx],
            &volumes[..=bar_idx],
            &pack.long,
            &pack.short,
            &pack.cuts,
            tgt,
            risk,
        );
        let stamp_e3 = decide_ml1d_ex(
            Experiment::E3,
            EconomicMode::Ec1,
            CombineMode::Dual,
            side,
            &closes[..=bar_idx],
            &volumes[..=bar_idx],
            &pack.long,
            &pack.short,
            &pack.cuts,
            tgt,
            risk,
        );
        joined += 1;
        y.push(if win { 1.0 } else { 0.0 });
        // Live friction net as payoff proxy (USDT) — not geometry %.
        pay.push(plan.net_pnl_usdt.unwrap_or(0.0));
        mask_e0.push(stamp_e0.act);
        mask_e1.push(stamp_e1.act);
        mask_e3.push(stamp_e3.act);
    }

    let sc_e0 = scorecard("p1b_E0", &y, &mask_e0, &pay);
    let sc_e1 = scorecard("p1b_E1", &y, &mask_e1, &pay);
    let sc_e3 = scorecard("p1b_E3", &y, &mask_e3, &pay);

    let mut reasons = Vec::new();
    // Transfer package is observational: ADVANCE only if E1/E3 clearly beat E0
    // on W/L+net with coverage floor — still not a twin authorization.
    if !coverage_floor_ok(&sc_e1) && !coverage_floor_ok(&sc_e3) {
        reasons.push("coverage_floor_fail_e1_and_e3".into());
    }
    let best = if sc_e3.mean_net_proxy.unwrap_or(f64::NEG_INFINITY)
        >= sc_e1.mean_net_proxy.unwrap_or(f64::NEG_INFINITY)
    {
        &sc_e3
    } else {
        &sc_e1
    };
    if !wl_improves(best) {
        reasons.push(format!(
            "no_wl_lift_vs_e0 best_wr={:.4} e0_wr={:.4}",
            best.enter_wr, sc_e0.enter_wr
        ));
    }
    if best.mean_net_proxy.unwrap_or(0.0) <= sc_e0.mean_net_proxy.unwrap_or(0.0) {
        reasons.push(format!(
            "no_net_lift_vs_e0 best={:?} e0={:?}",
            best.mean_net_proxy, sc_e0.mean_net_proxy
        ));
    }
    // Thin n / short span → never ADVANCE_CANDIDATE for twin.
    reasons.push(format!(
        "span_too_short_for_sealed_firewall joined={joined} (~11d live plans)"
    ));
    let verdict = "DO_NOT_ADVANCE_TRANSFER_OBSERVATIONAL".to_string();

    fn row(sc: &FoldScorecard) -> serde_json::Value {
        json!({
            "fold": sc.fold,
            "enter_n": sc.enter_n,
            "enter_wr": sc.enter_wr,
            "baseline_wr": sc.baseline_wr,
            "coverage": sc.coverage,
            "wilson_wr_lb": wilson_wr_lb(sc.enter_wins, sc.enter_n),
            "payoff_asym": sc.payoff_asym,
            "mean_net_usdt": sc.mean_net_proxy,
        })
    }

    let report = TransferReport {
        protocol_id: PROTOCOL_ID.into(),
        schema_version: SCHEMA_VERSION.into(),
        package: "ML1D_P1B_LIVE_PLAN_TRANSFER".into(),
        book: "BTCUSDT_LIVE_PLAN_OVERLAY".into(),
        label_source: plans_path.display().to_string(),
        candle_source: candles_path.display().to_string(),
        frozen_pack: pack_path.display().to_string(),
        n_plans: plans.len(),
        n_joined: joined,
        n_skipped_map: skipped_map,
        n_skipped_feat: skipped_feat,
        n_horizon_censored: censored,
        span_entry_ms: [tmin, tmax],
        experiments: vec![row(&sc_e0), row(&sc_e1), row(&sc_e3)],
        verdict: verdict.clone(),
        verdict_reasons: reasons.clone(),
        note: "Frozen 2024 ML-1D packs applied to 2026 live-plan TP/SL outcomes with real net_pnl_usdt. Not Gate5. Not twin. Horizon/ambiguous censored.".into(),
    };

    let out_path = out_dir.join("btc_ml1d_p1b_transfer_report.json");
    fs::write(&out_path, serde_json::to_vec_pretty(&report)?)?;
    println!(
        "  joined={joined} skipped_map={skipped_map} skipped_feat={skipped_feat} censored={censored}"
    );
    println!(
        "  E0 n={} wr={:.4} net={:?}",
        sc_e0.enter_n, sc_e0.enter_wr, sc_e0.mean_net_proxy
    );
    println!(
        "  E1 n={} wr={:.4} net={:?}",
        sc_e1.enter_n, sc_e1.enter_wr, sc_e1.mean_net_proxy
    );
    println!(
        "  E3 n={} wr={:.4} net={:?}",
        sc_e3.enter_n, sc_e3.enter_wr, sc_e3.mean_net_proxy
    );
    println!("  VERDICT = {verdict}");
    for r in &reasons {
        println!("    - {r}");
    }
    println!("  wrote {:?}", out_path);
    Ok(())
}
