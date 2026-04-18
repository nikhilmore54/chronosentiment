use chronosentiment_core::{
    ese::{ExecutionEngine, ExecutionResult},
    ga::{compute_consensus_alpha, GaConfig, ScenarioPair, Strategy, OrderIntent},
    csv_source::CsvCandleSource,
    data_source::CandleSource,
    market_adapter::Candle,
    MarketEvent, MarketEventType, Side,
};



// ==============================
// CONFIG
// ==============================

const ALPHA_THRESHOLD: f64 = 0.0001; // Extremely low for the demo

const WINDOW_SIZE: usize = 60;

// ==============================
// LIVE ENGINE
// ==============================

struct LiveEngine {
    strategies: Vec<Strategy>,
    ese: ExecutionEngine,
    total_pnl: f64,
    trades_count: usize,
}

impl LiveEngine {
    fn new(strategies: Vec<Strategy>) -> Self {
        Self {
            strategies,
            ese: ExecutionEngine::default(),
            total_pnl: 0.0,
            trades_count: 0,
        }
    }

    fn run(&mut self) {
        println!("🚀 Starting Live Engine (ESE-Aligned)...");

        let source = CsvCandleSource {
            path: "../data/nse/5m/AXISBANK.NS.csv".to_string(),
        };

        let all_candles = source.get_candles_sync();
        if all_candles.len() < WINDOW_SIZE + 100 {
            println!("❌ Insufficient data in CSV");
            return;
        }

        // Simulate a sliding window moving through time
        for i in 0..(all_candles.len() - WINDOW_SIZE) {
            let window = &all_candles[i..i + WINDOW_SIZE];
            
            // ==============================
            // 1. CONVERT TO EVENTS (REALITY CONSTRUCTION)
            // ==============================
            let mut market_events = Vec::new();
            for candle in window {
                market_events.extend(self.candle_to_market_events(candle));
            }

            // ==============================
            // 2. CONSTRUCT SCENARIO (CANONICAL)
            // ==============================
            let scenario = ScenarioPair {
                name: "live_stream",
                signal_symbol: "NSE:NIFTY",
                execution_symbol: "NSE:NIFTY",
                signal: &market_events,
                execution: &market_events,
            };

            let config = GaConfig::default();

            // ==============================
            // 3. CONSENSUS ALPHA (GROUND TRUTH)
            // ==============================
            let report = compute_consensus_alpha(&self.strategies, &scenario, &config);

            // ==============================
            // 4. EXTRACT SIGNAL
            // ==============================
            if let Some(top_signal) = report.top_signals.first() {
                println!("🔍 Best Signal Alpha: {:.4} (Report Size: {})", top_signal.alpha_score, report.top_signals.len());
                if top_signal.alpha_score >= ALPHA_THRESHOLD {

                    let strategy = &self.strategies[0]; // Simplified for demo
                    let tp_offset = strategy.take_profit as u64;
                    let sl_offset = strategy.stop_loss as u64;

                    let side = if top_signal.alpha_score > 0.0 { Side::Buy } else { Side::Sell };
                    let last_price = market_events.last().map(|e| e.price).unwrap_or(0);

                    let tp_target = if side == Side::Buy { last_price + tp_offset } else { last_price.saturating_sub(tp_offset) };
                    let sl_target = if side == Side::Buy { last_price.saturating_sub(sl_offset) } else { last_price + sl_offset };

                    let intent = OrderIntent {
                        symbol: "NSE:NIFTY".to_string(),
                        side,
                        quantity: 1,
                        price: last_price,
                        tp_target,
                        sl_target,
                        holding_period: strategy.holding_period as u32,
                    };

                    // ==============================
                    // 5. EXECUTION (STRICT ESE ROUTING)
                    // ==============================
                    let execution = self.ese.execute(intent, &market_events, i % WINDOW_SIZE);

                    // ==============================
                    // 6. UPDATE STATE (FROM OUTCOME)
                    // ==============================
                    self.apply_execution_result(&execution);

                    println!(
                        "📊 Status: {:?} | Reason: {:?} | PnL: {:.4} | Queue: {:.0} | Liq: {:.0}",
                        execution.status,
                        execution.exit_reason,
                        execution.realized_pnl,
                        execution.queue_pressure,
                        execution.arrival_liquidity
                    );
                }
            }
        }

        println!("🏁 Demo Complete. Final Trades: {} | Final PnL: {:.4}", self.trades_count, self.total_pnl);
    }

    fn candle_to_market_events(&self, candle: &Candle) -> Vec<MarketEvent> {
        vec![
            MarketEvent {
                subtype: MarketEventType::Trade,
                price: candle.open,
                quantity: candle.volume / 4,
                side: None,
                exchange_ts: candle.timestamp,
            },
            MarketEvent {
                subtype: MarketEventType::Trade,
                price: candle.high,
                quantity: candle.volume / 4,
                side: None,
                exchange_ts: candle.timestamp + 1,
            },
            MarketEvent {
                subtype: MarketEventType::Trade,
                price: candle.low,
                quantity: candle.volume / 4,
                side: None,
                exchange_ts: candle.timestamp + 2,
            },
            MarketEvent {
                subtype: MarketEventType::Trade,
                price: candle.close,
                quantity: candle.volume / 4,
                side: None,
                exchange_ts: candle.timestamp + 3,
            },
        ]
    }

    fn apply_execution_result(&mut self, result: &ExecutionResult) {
        if result.filled_quantity > 0 {
            self.total_pnl += result.realized_pnl;
            self.trades_count += 1;
        }
    }
}

fn main() {
    let strategies = load_elite_strategies();
    let mut engine = LiveEngine::new(strategies);
    engine.run();
}

fn load_elite_strategies() -> Vec<Strategy> {
    println!("🧪 Loading real elite strategies from core/elite/latest.json");
    let path = "core/elite/latest.json";
    let content = std::fs::read_to_string(path).unwrap_or_else(|_| {
        println!("⚠️  Could not find latest.json, falling back to dummy");
        String::from("{\"strategies\": []}")
    });
    
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    let mut strategies = Vec::new();
    
    if let Some(strats) = json.get("strategies").and_then(|s| s.as_array()) {
        for s_obj in strats {
            if let Some(s) = s_obj.get("strategy") {
                let strategy: Strategy = serde_json::from_value(s.clone()).unwrap();
                strategies.push(strategy);
            }
        }
    }

    if strategies.is_empty() {
        strategies.push(Strategy {
            take_profit: 150,
            stop_loss: 75,
            selectivity: 1, 
            w_conviction: 10,
            w_volatility: 5,
            holding_period: 50,
            ..Strategy::default()
        });
    }

    println!("✅ Loaded {} strategies", strategies.len());
    strategies
}
