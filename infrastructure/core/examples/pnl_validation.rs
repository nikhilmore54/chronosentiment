use chronosentiment_core::{
    ExecutionEngine,
    MarketEvent,
    MarketEventType,
    Side,
};
use chronosentiment_core::ga::OrderIntent;

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

fn generate_market(n: usize) -> Vec<MarketEvent> {
    let mut rng = StdRng::seed_from_u64(42);

    let mut price: u64 = 100_000;
    let mut events = Vec::with_capacity(n);

    for i in 0..n {
        // random walk price
        let drift: i64 = rng.gen_range(-5..5);
        price = (price as i64 + drift).max(1) as u64;

        let qty = rng.gen_range(1..200);

        let subtype = match rng.gen_range(0..3) {
            0 => MarketEventType::Trade,
            1 => MarketEventType::NewOrder,
            _ => MarketEventType::Cancel,
        };

        events.push(MarketEvent {
            subtype,
            price,
            quantity: qty,
            side: None,
            exchange_ts: i as u64,
        });
    }

    events
}

fn main() {
    let mut engine = ExecutionEngine::default();
    let market = generate_market(100_000);

    let mut rng = StdRng::seed_from_u64(7);

    let mut total_theoretical_pnl = 0.0;
    let mut total_realized_pnl = 0.0;
    let mut theoretical_pnl_vec = Vec::new();
    let mut realized_pnl_vec = Vec::new();
    let mut efficiency_vec = Vec::new();
    let mut trades = 0;
    let mut valid_efficiency_samples = 0;

    for _i in 0..50_000 {
        let idx = rng.gen_range(11..(market.len() - 100));

        // 1. Refined Trend Detection (User instructed: sum of last 10 deltas)
        let trend = chronosentiment_core::compute_trend_deltas(&market, idx, 10);

        // 2. Controlled Edge Injection
        let edge = 2;
        let adjusted_price = if trend > 0 {
            market[idx].price.saturating_sub(edge)
        } else {
            market[idx].price.saturating_add(edge)
        };

        // 3. Deterministic Side based on Trend
        let side = if trend > 0 { Side::Buy } else { Side::Sell };

        let intent = OrderIntent {
            symbol: "VALIDATION".to_string(),
            side,
            price: adjusted_price,
            quantity: rng.gen_range(10..100) as u32,
            tp_target: 100,
            sl_target: 100,
            holding_period: 60,
        };

        let exec = engine.execute(intent.clone(), &market, idx);

        if exec.filled_quantity == 0 {
            continue;
        }

        // Simulate exit (simple round trip)
        let exit_idx = (exec.exit_index + 20).min(market.len() - 1);
        let exit_price = market[exit_idx].price;

        // 4. THEORETICAL PnL (NO execution friction)
        let theoretical_entry = adjusted_price;
        let theoretical_exit = market[exit_idx].price;

        let theoretical_pnl = match intent.side {
            Side::Buy => (theoretical_exit as f64 - theoretical_entry as f64)
                / theoretical_entry.max(1) as f64,
            Side::Sell => (theoretical_entry as f64 - theoretical_exit as f64)
                / theoretical_entry.max(1) as f64,
        };

        // 5. REALIZED PnL (POST ESE execution)
        let realized_pnl = match intent.side {
            Side::Buy => (exit_price as f64 - exec.exit_price as f64) / exec.exit_price.max(1) as f64,
            Side::Sell => (exec.exit_price as f64 - exit_price as f64) / exec.exit_price.max(1) as f64,
        };

        theoretical_pnl_vec.push(theoretical_pnl);
        realized_pnl_vec.push(realized_pnl);
        
        total_theoretical_pnl += theoretical_pnl;
        total_realized_pnl += realized_pnl;
        trades += 1;

        // Guard against divide-by-noise efficiency
        if theoretical_pnl.abs() > 1e-6 {
            efficiency_vec.push(realized_pnl / theoretical_pnl);
            valid_efficiency_samples += 1;
        }
    }

    println!("\n================ EDGE VALIDATION ================");
    println!("\nTrades: {}", trades);
    println!("Efficiency Samples: {}", valid_efficiency_samples);

    if trades > 0 {
        // Stats helper
        let get_stats = |mut v: Vec<f64>| -> (f64, f64, f64, f64) {
            if v.is_empty() { return (0.0, 0.0, 0.0, 0.0); }
            let avg = v.iter().sum::<f64>() / v.len() as f64;
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let p5 = v[(v.len() as f64 * 0.05) as usize];
            let p50 = v[(v.len() as f64 * 0.50) as usize];
            let p95 = v[(v.len() as f64 * 0.95) as usize];
            (avg, p5, p50, p95)
        };

        let (t_avg, t_p5, t_p50, t_p95) = get_stats(theoretical_pnl_vec);
        let (r_avg, r_p5, r_p50, r_p95) = get_stats(realized_pnl_vec);
        let (e_avg, e_p5, e_p50, e_p95) = get_stats(efficiency_vec);

        println!("\n--- THEORETICAL ---");
        println!("Avg PnL: {:.6}", t_avg);
        println!("P5:      {:.6}", t_p5);
        println!("P50:     {:.6}", t_p50);
        println!("P95:     {:.6}", t_p95);

        println!("\n--- REALIZED ---");
        println!("Avg PnL: {:.6}", r_avg);
        println!("P5:      {:.6}", r_p5);
        println!("P50:     {:.6}", r_p50);
        println!("P95:     {:.6}", r_p95);

        println!("\n--- EXECUTION EFFICIENCY ---");
        let total_efficiency = total_realized_pnl / total_theoretical_pnl.max(1e-9);
        println!("Total Efficiency: {:.4}", total_efficiency);
        println!("E-P5:  {:.4}", e_p5);
        println!("E-P50: {:.4}", e_p50);
        println!("E-P95: {:.4}", e_p95);

        println!("\n--- EDGE RETENTION ---");
        println!("Avg Theoretical Edge: {:.6}", t_avg);
        println!("Avg Realized Edge:    {:.6}", r_avg);
        println!("Edge Retention %:      {:.2}%", (r_avg / t_avg.max(1e-9)) * 100.0);
    } else {
        println!("No trades executed.");
    }

    println!("\n=================================================");
}