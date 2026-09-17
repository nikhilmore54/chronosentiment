//! Fill displacement vs frozen T0 geometry over a universe-audit JSON.
//!
//! Does not re-ingest tape. Does not change TARGET / STOP / HORIZON.
//!
//! ```text
//! cargo run -p chronosentiment_adapter --bin deferred_live_fill_displacement -- \
//!   --from datasets/deferred_live_universe_loss_audit.json
//! ```

use std::path::PathBuf;

use chronosentiment_adapter::product::{
    summarize_fill_displacement, DeferredLiveAuditRow, FillDisplacementRow, UniverseAuditSummary,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct AuditFile {
    rows: Vec<DeferredLiveAuditRow>,
    #[allow(dead_code)]
    summary: Option<UniverseAuditSummary>,
}

#[derive(Serialize)]
struct Report {
    summary: chronosentiment_adapter::product::FillDisplacementSummary,
    rows: Vec<FillDisplacementRow>,
}

struct Args {
    from: PathBuf,
    out: PathBuf,
}

fn parse_args() -> Result<Args, String> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut from = PathBuf::from("datasets/deferred_live_universe_loss_audit.json");
    let mut out = PathBuf::from("datasets/deferred_live_fill_displacement.json");
    let mut i = 0;
    while i < raw.len() {
        match raw[i].as_str() {
            "--from" => {
                i += 1;
                from = PathBuf::from(raw.get(i).cloned().ok_or("--from needs a value")?);
            }
            "--out" => {
                i += 1;
                out = PathBuf::from(raw.get(i).cloned().ok_or("--out needs a value")?);
            }
            other => return Err(format!("unknown arg {other}")),
        }
        i += 1;
    }
    Ok(Args { from, out })
}

fn fmt_pct(v: Option<f64>) -> String {
    v.map(|x| format!("{:+.2}%", x * 100.0))
        .unwrap_or_else(|| "n/a".into())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args().map_err(|e| e)?;
    let audit: AuditFile = serde_json::from_str(&std::fs::read_to_string(&args.from)?)?;
    let mut rows: Vec<FillDisplacementRow> = audit
        .rows
        .iter()
        .filter_map(FillDisplacementRow::from_audit_row)
        .collect();
    rows.sort_by(|a, b| {
        a.geometry
            .fill_vs_decision
            .partial_cmp(&b.geometry.fill_vs_decision)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.symbol.cmp(&b.symbol))
    });
    let summary = summarize_fill_displacement(&rows);

    println!(
        "[fill-displacement] n={} STOP={} from {} (no re-ingest, driver unchanged)",
        summary.n,
        summary.n_stop,
        args.from.display()
    );
    println!();
    println!("FILL VS DECISION");
    println!("────────────────");
    for b in &summary.fill_buckets {
        println!(
            "  {:<28} n={:<2} STOP={} mean_fill={} mean_consumed={} mean_rem_stop={} mean_out={}",
            b.bucket,
            b.n,
            b.n_stop,
            fmt_pct(b.mean_fill_vs_decision),
            fmt_pct(b.mean_stop_span_consumed),
            fmt_pct(b.mean_stop_room_from_fill),
            fmt_pct(b.mean_outcome),
        );
    }
    println!();
    println!("T0 STOP SPAN CONSUMED BY FILL");
    println!("─────────────────────────────");
    for b in &summary.stop_buckets {
        println!(
            "  {:<28} n={:<2} STOP={} mean_consumed={} mean_rem_stop={} mean_out={}",
            b.bucket,
            b.n,
            b.n_stop,
            fmt_pct(b.mean_stop_span_consumed),
            fmt_pct(b.mean_stop_room_from_fill),
            fmt_pct(b.mean_outcome),
        );
    }
    println!();
    println!(
        "STOPs in severe adverse fill: {} / {}",
        summary.n_stop_in_severe_adverse, summary.n_stop
    );
    println!(
        "STOPs in majority/exhausted stop span: {} / {}",
        summary.n_stop_in_majority_or_exhausted, summary.n_stop
    );
    println!();
    println!(
        "{:<12} {:<12} {:<6} {:<6} {:>9} {:>9} {:>9} {:>9} {:>8} {}",
        "SYMBOL", "DATE", "DIR", "EXIT", "FILL_VS", "T0_STOP", "REM_STOP", "CONSUMED", "OUT", "BUCKET"
    );
    for row in &rows {
        println!(
            "{:<12} {:<12} {:<6} {:<6} {:>8.2}% {:>8.2}% {:>8.2}% {:>8.1}% {:>7.2}% {} | {}",
            row.symbol,
            row.date,
            row.direction,
            row.exit_reason,
            row.geometry.fill_vs_decision * 100.0,
            row.geometry.t0_stop_room * 100.0,
            row.geometry.stop_room_from_fill * 100.0,
            row.geometry.stop_span_consumed * 100.0,
            row.outcome * 100.0,
            row.fill_bucket.as_str(),
            row.stop_bucket.as_str(),
        );
    }

    let report = Report { summary, rows };
    if let Some(parent) = args.out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&args.out, serde_json::to_string_pretty(&report)?)?;
    println!();
    println!("[fill-displacement] wrote {}", args.out.display());
    println!("[fill-displacement] {}", report.summary.note);
    Ok(())
}
