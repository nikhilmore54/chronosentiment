// examples/demo.rs

use chronosentiment_mvp_demo::*;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let real_sim = run_simulation(ExecutionMode::Real);
    let ideal_sim = run_simulation(ExecutionMode::Ideal);

    // Check for replay arguments
    let mut from_ts: Option<u64> = None;
    let mut to_ts: Option<u64> = None;
    let mut step: Option<usize> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--from" => {
                if i + 1 < args.len() {
                    from_ts = args[i + 1].parse().ok();
                    i += 1;
                }
            }
            "--to" => {
                if i + 1 < args.len() {
                    to_ts = args[i + 1].parse().ok();
                    i += 1;
                }
            }
            "--step" => {
                if i + 1 < args.len() {
                    step = args[i + 1].parse().ok();
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    if let Some(s) = step {
        replay_step(&real_sim, s);
        return;
    }

    if let (Some(f), Some(t)) = (from_ts, to_ts) {
        replay_range(&real_sim, f, t);
        return;
    }

    // Default MVP Output
    println!("--- ChronoSentiment MVP Demo ---");

    println!("\nOrders:");
    let order_ids = ["O1", "O2", "O3"];
    for id in &order_ids {
        if let Some(outcome) = real_sim.order_outcomes.get(*id) {
            println!("{} | Filled: {} | Remaining: {}", outcome.order_id, outcome.filled_quantity, outcome.remaining_quantity);
        }
    }

    println!("\n--- Strategy Performance ---");
    println!("Ideal Execution:");
    println!("PnL: {}", ideal_sim.pnl);
    println!("Trades: {}", ideal_sim.trades);

    println!("\nReal Execution:");
    println!("PnL: {}", real_sim.pnl);
    println!("Trades: {}", real_sim.trades);

    let ideal_ga = run_ga(ExecutionMode::Ideal);
    let real_ga = run_ga(ExecutionMode::Real);

    println!("\n--- GA Optimization ---");
    println!("Best Strategy (Ideal):");
    println!("{}", ideal_ga.best_config);

    println!("\nBest Strategy (Real):");
    println!("{}", real_ga.best_config);

    println!("\n--- Insight ---");
    println!("Execution changed both:");
    println!("- Performance");
    println!("- Optimal strategy");

    // Mandatory Timeline
    println!();
    print_timeline(&real_sim);

    // Mandatory Trade Inspector
    println!();
    let inspection = inspect_trade("O1", &real_sim);
    print_inspection(&inspection);
}

fn print_inspection(inspection: &TradeInspection) {
    println!("--- Trade Inspector ---");
    println!("\nOrder: {}", inspection.decision.order_id);
    
    println!("\n[Decision]");
    let side_str = match inspection.decision.side {
        Side::Buy => "BUY",
        Side::Sell => "SELL",
    };
    println!("{} {} @{}", side_str, inspection.decision.quantity, inspection.decision.price);
    println!("Created at: {}", inspection.decision.timestamp);

    println!("\n[Execution]");
    println!("Arrived at: {}", inspection.execution.arrival_time);
    println!("Initial Queue Ahead: {}", inspection.execution.queue_ahead_initial);
    
    print!("Queue Progression: [");
    for (i, q) in inspection.execution.queue_progression.iter().enumerate() {
        if i > 0 { print!(" → "); }
        print!("{}", q);
    }
    println!("]");

    println!("\n--- Causal Chain ---");
    for (i, event) in inspection.execution.causal_chain.iter().enumerate() {
        let prefix = if i == 0 { "" } else { "→ " };
        match event {
            SimEvent::OrderIntent { ts, .. } => println!("{}OrderIntent (t={})", prefix, ts),
            SimEvent::OrderEnteredQueue { ts, .. } => println!("{}OrderEnteredQueue (t={})", prefix, ts),
            SimEvent::QueueProgression { ts, .. } => println!("{}QueueProgression (t={})", prefix, ts),
            SimEvent::PartialFill { ts, .. } => println!("{}PartialFill (t={})", prefix, ts),
            _ => {}
        }
    }

    println!("\nFills:");
    for fill in &inspection.execution.fills {
        println!("- t={} → qty={} @{}", fill.ts, fill.qty, fill.price);
    }

    println!("\n[Outcome]");
    println!("Filled: {}", inspection.outcome.filled_quantity);
    println!("Remaining: {}", inspection.outcome.remaining_quantity);
}
