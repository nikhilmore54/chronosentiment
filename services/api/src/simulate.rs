use chronosentiment_core::{*, ga::GaConfig, harness::run_simulation_harness};
use crate::ApiError;
use rand::{Rng, SeedableRng};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SimulateInput {
    pub mode: String,
    pub dataset: Option<String>,
    pub seed: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SimulateOutput {
    pub pnl: i64,
    pub trade_count: u64,
    pub events: Vec<SimEvent>,
    pub state_hash: String,
    #[serde(skip)]
    pub original_result: SimulationResult, // Internal use
}

pub fn handle_simulate(input: SimulateInput) -> Result<SimulateOutput, ApiError> {
    let mode = match input.mode.as_str() {
        "real" => ExecutionMode::Real,
        "ideal" => ExecutionMode::Ideal,
        _ => return Err(ApiError::InvalidInput("mode must be 'real' or 'ideal'".to_string())),
    };

    let mut rng = rand::rngs::StdRng::seed_from_u64(input.seed);

    let source = chronosentiment_core::FolderCandleSource {
        folder_path: "/Users/nikhil/ChronoSentiment_MEGA_FINAL/test_assets".to_string(),
    };
    let assets_with_candles = source.load_all();
    let mut all_scenarios = std::collections::HashMap::new();
    for (asset, candles) in assets_with_candles {
        let asset_scenarios = chronosentiment_core::pipeline::scenarios_from_candles(&asset, &candles);
        all_scenarios.extend(asset_scenarios);
    }
    
    let default_scenario = all_scenarios.values().next().ok_or_else(|| ApiError::InternalError("No real market scenarios found in test_assets".to_string()))?;
    let market_events = default_scenario.clone();

    let first_event_price = market_events.first().map(|e| e.price).unwrap_or(100);
    let first_event_timestamp = market_events.first().map(|e| e.exchange_ts).unwrap_or(0);

    let config = GaConfig {
        population_size: 1,
        generations: 1,
        mutation_rate: 0.0,
        seed: input.seed,
        order_id_prefix: "SIMULATE".to_string(),
        order_price: first_event_price,
        order_quantity_for_strategy: 100,
        order_timestamp: first_event_timestamp,
        lambda: 0.5,
        initial_queue_threshold: 200,
        ..GaConfig::default()
    };

    let create_orders = vec![CreateOrder {
        order_id: "sim_order_1".to_string(),
        side: Side::Buy,
        price: config.order_price,
        quantity: config.order_quantity_for_strategy,
        timestamp: config.order_timestamp,
        fill_probability: rng.gen_range(0.0..1.0),
    }];

    // Baseline Validation 1: Determinism Check
    let (_, res1, _) = run_simulation_harness(mode, market_events.clone(), create_orders.clone());
    let (_, res2, _) = run_simulation_harness(mode, market_events.clone(), create_orders.clone());

    if res1.pnl != res2.pnl || res1.trades != res2.trades || res1.events.len() != res2.events.len() {
        return Err(ApiError::InternalError("Determinism violation detected".to_string()));
    }

    // Baseline Validation 2: Event Identity
    for i in 0..res1.events.len() {
        if res1.events[i] != res2.events[i] {
            return Err(ApiError::InternalError(format!("Event mismatch at sequence {}", i)));
        }
    }

    let state_hash = blake3::hash(serde_json::to_string(&res1).unwrap_or_default().as_bytes()).to_hex().to_string();

    Ok(SimulateOutput {
        pnl: res1.pnl,
        trade_count: res1.trades,
        events: res1.events.clone(),
        state_hash,
        original_result: res1,
    })
}
