//! live_observatory — deterministic telemetry emitter for cs-ingest replay-step.
//!
//! Protocol:
//!   stdin:  one JSON array of SymbolicCandle per line
//!   stdout: one [TELEMETRY] line per candle, in the same order
//!
//! [TELEMETRY] format (must match telemetry.rs regex):
//!   [TELEMETRY] ts=<i64> sym=<str> margin=<f64> conv=<f64> eq=<f64>
//!               eff=<f64> den=<f64> res=<f64> comp=<f64> range=<f64> bias=<f64>

use std::io::{self, BufRead, Write};

use serde::Deserialize;

#[derive(Deserialize)]
struct SymbolicCandle {
    symbol: String,
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

/// Compute telemetry metrics from a single OHLCV candle.
/// All values are bounded to avoid NaN/Inf in downstream PCA.
fn emit_telemetry(c: &SymbolicCandle) -> String {
    let eps = 1e-10_f64;
    let range = (c.high - c.low).max(eps);
    let mid = (c.high + c.low) / 2.0;

    // margin: close/open ratio — >1.0 for bullish bars
    let margin = (c.close / (c.open + eps)).clamp(0.5, 2.0);

    // conv: convergence — how tight the bar range is relative to price
    let conv = (1.0 - range / (c.close * 0.05 + eps)).clamp(0.0, 1.0);

    // eq: equilibrium — midpoint vs close
    let eq = (mid / (c.close + eps)).clamp(0.5, 1.5);

    // eff: directional efficiency — fraction of range used directionally
    let eff = ((c.close - c.open) / range).clamp(-1.0, 1.0);

    // den: continuation density — inverse of normalized range (tight = high density)
    let den = (1.0 - range / (c.close * 0.05 + eps)).clamp(-1.0, 1.0);

    // res: resilience — how much price recovered from the low
    let res = ((c.close - c.low) / range).clamp(0.0, 1.0);

    // comp: compression ratio — range as multiple of 1% of price
    let comp = (range / (c.close * 0.01 + eps)).clamp(0.01, 10.0);

    // range: pre-range — range as fraction of close (small for crypto: 0.005–0.05)
    let pre_range = (range / (c.close + eps)).clamp(0.0, 1.0);

    // bias: pre-bias — directional bias as fraction of close
    let bias = ((c.close - c.open) / (c.close + eps)).clamp(-1.0, 1.0);

    // volume is parsed but not emitted — kept for future extension
    let _ = c.volume;

    format!(
        "[TELEMETRY] ts={} sym={} margin={:.6} conv={:.6} eq={:.6} eff={:.6} den={:.6} res={:.6} comp={:.6} range={:.6} bias={:.6}",
        c.timestamp, c.symbol,
        margin, conv, eq, eff, den, res, comp, pre_range, bias
    )
}

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let candles: Vec<SymbolicCandle> = match serde_json::from_str(line) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[live_observatory] parse error: {e}");
                continue;
            }
        };

        for candle in &candles {
            let telemetry = emit_telemetry(candle);
            if writeln!(out, "{telemetry}").is_err() {
                return;
            }
        }
        if out.flush().is_err() {
            return;
        }
    }
}
