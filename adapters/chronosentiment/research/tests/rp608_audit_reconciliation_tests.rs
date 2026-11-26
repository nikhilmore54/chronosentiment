use std::path::PathBuf;
use std::process::Command;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn test_rp608_ledger_reconciliation() {
    let bin_path = env!("CARGO_BIN_EXE_rp608_research_audit");
    let output = Command::new(bin_path)
        .arg("audit-ledger")
        .current_dir(crate_root().join("../.."))
        .output()
        .expect("Failed to execute rp608_research_audit audit-ledger");

    assert!(output.status.success(), "audit-ledger mode must exit with code 0");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Total Trades:        2029"), "Must audit exactly 2,029 trades");
    assert!(stdout.contains("Long Trades:         860 | Short Trades: 1169"), "Direction partition mismatch");
    assert!(stdout.contains("Reported Gross PnL:  Rs +9993.52 (Recalculated: Rs +9993.52, Mismatch: 0.0000 Paisa)"),
            "Gross PnL mismatch detected against reference ledger");
    assert!(stdout.contains("Reported Friction:   Rs 181029.04 (Recalculated: Rs 181029.04, Mismatch: 0.0000 Paisa)"),
            "Friction mismatch detected against reference ledger");
    assert!(stdout.contains("Reported Net PnL:    Rs -171035.52 (Recalculated: Rs -171035.52, Mismatch: 0.0000 Paisa)"),
            "Net PnL mismatch detected against reference ledger");
    assert!(stdout.contains("Max Row Errors:      Gross: 0.0000 Paisa | Friction: 0.0000 Paisa | Net: 0.0000 Paisa"),
            "Trade-by-trade individual row recalculation mismatch detected");
    assert!(stdout.contains("Mean Gross Return:   +0.52 bps | Mean Net Expectancy: -8.48 bps"),
            "Mean return bps mismatch");
    assert!(stdout.contains("Win Rates:           Gross 33.0% | Net 20.4%"),
            "Win rate parity mismatch");
}

#[test]
fn test_rp608_bar_integrity_audit() {
    let bin_path = env!("CARGO_BIN_EXE_rp608_research_audit");
    let output = Command::new(bin_path)
        .arg("audit-bars")
        .current_dir(crate_root().join("../.."))
        .output()
        .expect("Failed to execute rp608_research_audit audit-bars");

    assert!(output.status.success(), "audit-bars mode must exit with code 0");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Symbols Audited:     99/99"), "All 99 symbols must be complete");
    assert!(stdout.contains("Timestamp Anomalies: 0"), "Timestamp integrity must show 0 anomalies");
    assert!(stdout.contains("Trades Valid Hist:   2029/2029"), "100% of trades must have complete prior bar history");
    assert!(stdout.contains("Trades Missing Hist: 0"), "0 trades may have missing history");
}

#[test]
fn test_rp608_inverse_replay_reconciliation() {
    let bin_path = env!("CARGO_BIN_EXE_rp608_research_audit");
    let output = Command::new(bin_path)
        .arg("replay-inverse")
        .current_dir(crate_root().join("../.."))
        .output()
        .expect("Failed to execute rp608_research_audit replay-inverse");

    assert!(output.status.success(), "replay-inverse mode must exit with code 0");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Variant B: Exact Algebraic Reversal
    assert!(stdout.contains("Variant B (Exact Reversal):"), "Variant B section missing");
    assert!(stdout.contains("Gross PnL:         Rs -9993.52 (-0.52 bps)"), "Variant B gross return mismatch");
    assert!(stdout.contains("Net PnL:           Rs -191022.56 (-9.52 bps)"), "Variant B net return mismatch");
    assert!(stdout.contains("Win Rate:          Gross 67.0% | Net 32.7%"), "Variant B win rate mismatch");

    // Variant C: Forward Path Replay with Mirrored Stops
    assert!(stdout.contains("Variant C (Mirrored Stops Path Replay - Conservative Fill):"), "Variant C section missing");
    assert!(stdout.contains("Stopped Out:       398/2029 (19.6%)"), "Variant C stop-out count mismatch");
    assert!(stdout.contains("Ambiguous Bars:    4 bars touching both stop and exit"), "Ambiguous bars count mismatch");
    assert!(stdout.contains("Gross PnL:         Rs +12205.67 (+0.59 bps)"), "Variant C gross PnL mismatch");
    assert!(stdout.contains("Net PnL:           Rs -168823.37 (-8.41 bps)"), "Variant C net PnL mismatch");
}

#[test]
fn test_rp608_pre_entry_exhaustion_feature_attribution() {
    let bin_path = env!("CARGO_BIN_EXE_rp608_research_audit");
    let output = Command::new(bin_path)
        .arg("extract-features")
        .current_dir(crate_root().join("../.."))
        .output()
        .expect("Failed to execute rp608_research_audit extract-features");

    assert!(output.status.success(), "extract-features mode must exit with code 0");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Cohort Size:         186 trades (9.2% of population)"),
            "Candidate overextension cohort size mismatch");
    assert!(stdout.contains("Baseline Perf:       Gross -5.42 bps | Net -14.42 bps"),
            "Baseline overextended cohort performance mismatch");
    assert!(stdout.contains("Inverse Perf:        Gross +5.42 bps | Net -3.58 bps"),
            "Inverse overextended cohort performance mismatch");
    assert!(stdout.contains("Hurdle (+14.0 bps): FAILED (Net expectancy is -3.58 bps)"),
            "Falsification hurdle assertion mismatch");
    assert!(stdout.contains("Candidate closed as DESCRIPTIVE FAILURE MECHANISM only."),
            "Disposition conclusion mismatch");
}

#[test]
fn test_rp608_prospective_policy_comparison() {
    let bin_path = env!("CARGO_BIN_EXE_rp608_research_audit");
    let output = Command::new(bin_path)
        .arg("compare-policies")
        .current_dir(crate_root().join("../.."))
        .output()
        .expect("Failed to execute rp608_research_audit compare-policies");

    assert!(output.status.success(), "compare-policies mode must exit with code 0");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Total Eligible Signals:   2029"), "Must evaluate exactly 2029 eligible signal IDs");
    assert!(stdout.contains("Overextended (Flagged):   186 (9.2%)"), "Must flag exactly 186 trades");
    assert!(stdout.contains("Policy 1 Participation:   90.8%"), "Policy 1 participation rate mismatch");

    // Policy comparisons on identical opportunity IDs
    assert!(stdout.contains("Policy 0 (Baseline):     Net: Rs -171035.52 | Per Signal: Rs -84.30 | Return: -8.48 bps"),
            "Policy 0 baseline performance mismatch");
    assert!(stdout.contains("Policy 1 (No-Trade Gate): Net: Rs -144308.32 | Per Signal: Rs -71.12 | Return: -7.16 bps"),
            "Policy 1 performance mismatch");
    assert!(stdout.contains("Policy 2 (Reversal):     Net: Rs -150747.42 | Per Signal: Rs -74.30 | Return: -7.48 bps"),
            "Policy 2 performance mismatch");

    // Primary Scale-Invariant Session-Clustered Inference (bps per opportunity, df = 3)
    assert!(stdout.contains("Positive Sessions:       4/4 (100.0%) [Required >= ceil(0.75*S) = 3 | Passed: true]"),
            "All 4 historical sessions must show positive delta");
    assert!(stdout.contains("Mean Session Delta:      +1.33 bps per opportunity [Gate 1 Hurdle >= +1.50 bps | Passed: false]"),
            "Mean session delta bps mismatch");
    assert!(stdout.contains("Pooled Descriptive Delta:+1.32 bps per opportunity across all 2029 signals"),
            "Pooled descriptive delta mismatch");
    assert!(stdout.contains("Session t-statistic:     4.0022"), "Session-clustered t-statistic mismatch");
    assert!(stdout.contains("One-sided p-value:       0.0140"), "One-sided Student-t p-value mismatch");
    assert!(stdout.contains("Two-sided p-value:       0.0280"), "Two-sided Student-t p-value mismatch");

    // Empirical Data-Integrity Check (P_ref = C_-1 vs O_entry)
    assert!(stdout.contains("Audited Candle Gaps:     Mean 1.8207 bps | Max 29.9065 bps (Formula strictly uses authoritative P_ref)"),
            "Audited candle gaps check mismatch");

    // Secondary Rupee Session Aggregates (df = 3)
    assert!(stdout.contains("Mean Session Delta:      Rs +6681.80"), "Mean rupee session delta mismatch");
    assert!(stdout.contains("Session t-statistic:     4.1691"), "Rupee session-clustered t-statistic mismatch");

    // Secondary Trade-Level Paired Comparison (df = 2028)
    assert!(stdout.contains("Delta Net PnL (Saved):   Rs +26727.20 (Rs +13.17 per signal)"),
            "Delta net PnL mismatch");
    assert!(stdout.contains("Paired t-statistic:      5.3417"), "Paired t-statistic mismatch");

    // Fee Model Reconciliation
    assert!(stdout.contains("Statutory & Exchange:  1.82 bps (0.345 bps turnover + 0.010 bps SEBI + 0.150 bps stamp + 1.250 bps STT + 0.062 bps GST)"),
            "Statutory fee component rate mismatch");
    assert!(stdout.contains("Total Turnover Rate:   5.00 bps (Historical Rate: 0.00045 per side)"), "Fee model rate mismatch");
    assert!(stdout.contains("Total Round-Trip:      10.00 bps"), "Entry notional fee mismatch");
    assert!(stdout.contains("Total Realized Friction: Rs 181029.04"), "Realized friction mismatch");

    // Prospective Itemized Fee & Adverse Slippage Stress Evaluation
    assert!(stdout.contains("Policy 1 Base Itemized Net: Rs -206344.56 (-10.24 bps per opportunity)"),
            "Prospective itemized net mismatch");
    assert!(stdout.contains("Policy 1 Stressed Net:      Rs -224617.72 (-11.15 bps per opportunity with +1.00 bps slippage stress on entry notional)"),
            "Prospective stressed net mismatch");
    assert!(stdout.contains("Gate 3 Stressed Edge Preserved: false (Requires E[Net]_stressed > 0.00 bps)"),
            "Gate 3 stressed edge assertion mismatch");

    assert!(stdout.contains("Policy 1 (skipping overextended entries) is strictly superior to Policy 2 (reversing)."),
            "Superiority conclusion mismatch");
    assert!(stdout.contains("Multi-Dimensional Gate 3: Positive net expectancy is necessary but NOT sufficient."),
            "Gate 3 criteria mismatch");
}

#[test]
fn test_rp608_itemized_cost_model_deterministic_fixtures() {
    // Test deterministic trade friction fixture on Rs 2,00,000 notional:
    // Q = 200, Entry = 1000.0, Exit = 1000.0 (Turnover = 400,000, Notional = 200,000)
    let entry_p = 1000.0;
    let exit_p = 1000.0;
    let qty = 200.0;
    let entry_notional = entry_p * qty; // 200,000
    let total_turnover = (entry_p + exit_p) * qty; // 400,000

    // Component math:
    // 1. Exchange Turnover Fee: 0.00345% on Total Turnover (Rs 4,00,000) = Rs 13.80 (0.345 bps of turnover)
    let exchange_turnover: f64 = total_turnover * 0.0000345;
    assert!((exchange_turnover - 13.80_f64).abs() < 1e-6);

    // 2. SEBI Regulatory Fee: 0.00010% on Total Turnover (Rs 4,00,000) = Rs 0.40 (0.010 bps of turnover)
    let sebi_fee: f64 = total_turnover * 0.0000010;
    assert!((sebi_fee - 0.40_f64).abs() < 1e-6);

    // 3. Stamp Duty: 0.00300% on Buy Leg Only (Rs 2,00,000) = Rs 6.00 (0.150 bps of turnover)
    let stamp_duty: f64 = entry_notional * 0.0000300;
    assert!((stamp_duty - 6.00_f64).abs() < 1e-6);

    // 4. STT: 0.02500% on Sell Leg Only (Rs 2,00,000) = Rs 50.00 (1.250 bps of turnover)
    let stt: f64 = exit_notional_stt(entry_p, qty);
    assert!((stt - 50.00_f64).abs() < 1e-6);

    // 5. GST: 18% on Exchange Turnover Fee = Rs 2.484 (0.0621 bps of turnover)
    let gst_exchange: f64 = exchange_turnover * 0.18;
    assert!((gst_exchange - 2.484_f64).abs() < 1e-6);

    // Statutory Subtotal = Rs 13.80 + Rs 0.40 + Rs 6.00 + Rs 50.00 + Rs 2.484 = Rs 72.684
    let statutory_subtotal: f64 = exchange_turnover + sebi_fee + stamp_duty + stt + gst_exchange;
    assert!((statutory_subtotal - 72.684_f64).abs() < 1e-6);

    // Verify dual-basis conversion parity:
    // (A) Turnover basis: 1.8171 bps on Rs 4,00,000 turnover = Rs 72.684
    let stat_turnover_bps: f64 = (statutory_subtotal / total_turnover) * 10000.0;
    assert!((stat_turnover_bps - 1.8171_f64).abs() < 1e-4);
    assert!((total_turnover * (stat_turnover_bps / 10000.0) - 72.684_f64).abs() < 1e-4);

    // (B) Entry notional basis: 3.6342 bps on Rs 2,00,000 entry notional = Rs 72.684
    let stat_notional_bps: f64 = (statutory_subtotal / entry_notional) * 10000.0;
    assert!((stat_notional_bps - 3.6342_f64).abs() < 1e-4);
    assert!((entry_notional * (stat_notional_bps / 10000.0) - 72.684_f64).abs() < 1e-4);

    let brokerage_gst: f64 = 47.20; // Flat Rs 40 + 18% GST on 2 legs
    let brok_turnover_bps: f64 = (brokerage_gst / total_turnover) * 10000.0;
    assert!((brok_turnover_bps - 1.18_f64).abs() < 0.01); // 1.18 bps turnover

    let base_slippage: f64 = (entry_notional * 0.0002) + (entry_notional * 0.0002); // 80.00 (2.0 bps entry + 2.0 bps exit)
    let slip_turnover_bps: f64 = (base_slippage / total_turnover) * 10000.0;
    assert!((slip_turnover_bps - 2.00_f64).abs() < 0.01); // 2.00 bps turnover

    let total_base_friction: f64 = statutory_subtotal + brokerage_gst + base_slippage;
    assert!((total_base_friction - 199.884_f64).abs() < 1e-4);
    let total_notional_bps: f64 = (total_base_friction / entry_notional) * 10000.0;
    assert!((total_notional_bps - 9.9942_f64).abs() < 0.01); // ~10.00 bps round-trip on entry notional

    // Adverse Slippage Stress: +1.00 bps of one-leg entry notional added ONCE
    let stress_slippage: f64 = entry_notional * 0.0001; // 20.00
    let total_stressed_friction: f64 = total_base_friction + stress_slippage;
    assert!((total_stressed_friction - 219.884_f64).abs() < 1e-4);
    let stressed_notional_bps: f64 = (total_stressed_friction / entry_notional) * 10000.0;
    assert!((stressed_notional_bps - 10.9942_f64).abs() < 0.01); // ~11.00 bps round-trip on entry notional
}

fn exit_notional_stt(price: f64, qty: f64) -> f64 {
    price * qty * 0.0002500
}

#[test]
fn test_rp608_session_consistency_ceil_arithmetic() {
    // 75% consistency requirement: minimum required positive sessions = ceil(0.75 * S)
    let ceil_pos = |s: usize| -> usize { (0.75 * s as f64).ceil() as usize };

    // S = 4 historical sessions: ceil(3.0) = 3 sessions
    assert_eq!(ceil_pos(4), 3);

    // S = 14 sessions: ceil(10.5) = 11 sessions
    assert_eq!(ceil_pos(14), 11);

    // S = 15 sessions (pre-registered prospective sample size): ceil(11.25) = 12 sessions
    assert_eq!(ceil_pos(15), 12);

    // S = 16 sessions: ceil(12.0) = 12 sessions
    assert_eq!(ceil_pos(16), 12);

    // S = 20 sessions: ceil(15.0) = 15 sessions
    assert_eq!(ceil_pos(20), 15);
}
