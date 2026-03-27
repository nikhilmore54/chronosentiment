use clap::Parser;
use std::fs::File;
use std::io::{self, Write};
use prettytable::{Table, Row, Cell};
use chronosentiment_core::edge_decay::run_edge_decay;
use chronosentiment_core::pipeline::{run_pipeline_with_config};
use chronosentiment_core::ga::GaConfig;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the Binance JSONL data file
    #[clap(long)]
    input: String,

    /// Path to save the JSON report
    #[clap(long, default_value = "results/edge_decay.json")]
    output: String,

    /// Path to save the CSV report
    #[clap(long, default_value = "results/edge_decay.csv")]
    csv: String,

    /// Fixed horizon in ticks for PnL calculation
    #[clap(long, default_value = "20")]
    horizon: usize,

    /// Number of top strategies to evaluate
    #[clap(long, default_value = "5")]
    top_k: usize,

    /// Slippage in basis points (for SpreadSlippage model)
    #[clap(long, default_value = "5.0")]
    slippage_bps: f64,

    /// Asset name to process (e.g., BTCUSDT)
    #[clap(long, default_value = "BTCUSDT")]
    asset_name: String,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    println!("🚀 Starting Edge Decay Analysis for {}...", args.asset_name);
    println!("📍 Input: {}", args.input);

    // 1. Run the GA pipeline to get an initial SignalsSnapshot for the strategy profiles
    println!("🧬 Running GA pipeline to generate strategy profiles...");
    let ga_config = GaConfig::default();
    let initial_signal_snapshot = run_pipeline_with_config(&args.input, ga_config);

    if initial_signal_snapshot.signals.is_empty() {
        return Err(io::Error::new(io::ErrorKind::Other, "GA pipeline produced no signals. Check your input data."));
    }

    // 2. Run the edge decay comparative analysis
    println!("📊 Running comparative execution models...");
    let results = run_edge_decay(
        &args.input,
        args.asset_name.clone(),
        args.horizon,
        args.top_k,
        args.slippage_bps,
        &initial_signal_snapshot,
    );

    // Ensure directory exists
    if let Some(parent) = std::path::Path::new(&args.output).parent() {
        std::fs::create_dir_all(parent)?;
    }
    if let Some(parent) = std::path::Path::new(&args.csv).parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 3. Output to JSON
    let json = serde_json::to_string_pretty(&results).unwrap();
    let mut file = File::create(&args.output)?;
    file.write_all(json.as_bytes())?;
    println!("✅ JSON report saved to {}", args.output);

    // 4. Output to CSV
    let mut csv_file = File::create(&args.csv)?;
    writeln!(csv_file, "model,avg_pnl,total_pnl,win_rate,edge_retention,eff_rate,band,decay_pct")?;
    for res in &results {
        writeln!(
            csv_file,
            "{:?},{:.6},{:.6},{:.2},{:.4},{:.4},{:?},{:.4}",
            res.model, res.avg_pnl, res.total_pnl, res.win_rate, res.edge_retention,
            res.effective_signal_rate, res.tradability_band, res.edge_decay_pct
        )?;
    }
    println!("✅ CSV report saved to {}", args.csv);

    // 5. Output to Console Table
    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("MODEL"),
        Cell::new("AVG_PNL"),
        Cell::new("TOTAL_PNL"),
        Cell::new("WIN_RATE"),
        Cell::new("EDGE_RET"),
        Cell::new("EFF_RATE"),
        Cell::new("BAND"),
        Cell::new("DECAY %"),
    ]));

    for res in &results {
        table.add_row(Row::new(vec![
            Cell::new(&format!("{:?}", res.model)),
            Cell::new(&format!("{:.6}", res.avg_pnl)),
            Cell::new(&format!("{:.6}", res.total_pnl)),
            Cell::new(&format!("{:.2}%", res.win_rate * 100.0)),
            Cell::new(&format!("{:.4}", res.edge_retention)),
            Cell::new(&format!("{:.4}", res.effective_signal_rate)),
            Cell::new(&format!("{:?}", res.tradability_band)),
            Cell::new(&format!("{:.2}%", res.edge_decay_pct * 100.0)),
        ]));
    }

    println!("\n🏆 Edge Decay Comparative Summary:");
    table.printstd();

    Ok(())
}
