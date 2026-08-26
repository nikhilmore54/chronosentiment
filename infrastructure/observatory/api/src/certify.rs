use crate::{ApiError, CertificationResponse};
use chronosentiment_core::{harness::run_simulation_harness, *};
use serde_json;

pub fn handle_certify(sim: &SimulationResult) -> Result<CertificationResponse, ApiError> {
    // 1. Run SAME simulation twice internally (or compare against current)
    // The requirement says: "Run SAME simulation twice internally"
    // Since we have 'sim', we'll run it again and compare.

    // Reconstruct CreateOrder from the simulation events or a default for replay
    let replay_fill_probability = 0.5;
    let create_orders_for_replay = sim
        .events
        .iter()
        .filter_map(|event| {
            if let SimEvent::OrderIntent {
                order_id,
                side,
                price,
                quantity,
                timestamp,
                ..
            } = event
            {
                Some(CreateOrder {
                    order_id: order_id.clone(),
                    side: *side,
                    price: *price,
                    quantity: *quantity,
                    timestamp: *timestamp,
                    fill_probability: replay_fill_probability,
                })
            } else {
                None
            }
        })
        .collect::<Vec<CreateOrder>>();

    let market_events_for_replay = sim
        .events
        .iter()
        .filter_map(|event| {
            if let SimEvent::MarketEvent {
                subtype,
                price,
                quantity,
                side,
                timestamp,
                ..
            } = event
            {
                Some(MarketEvent {
                    subtype: *subtype,
                    price: *price,
                    quantity: *quantity,
                    side: *side,
                    exchange_ts: *timestamp,
                })
            } else {
                None
            }
        })
        .collect::<Vec<MarketEvent>>();

    let replay_mode = ExecutionMode::Real;
    let (_, replay_sim, _) = run_simulation_harness(
        replay_mode,
        market_events_for_replay,
        create_orders_for_replay.clone(),
    );

    // 2. Compare event sequences and find divergence point
    let mut divergence_point = None;
    let max_len = std::cmp::max(sim.events.len(), replay_sim.events.len());

    for i in 0..max_len {
        let e1 = sim.events.get(i);
        let e2 = replay_sim.events.get(i);

        if e1 != e2 {
            divergence_point = Some(
                e1.map(|e| e.sequence_id())
                    .or(e2.map(|e| e.sequence_id()))
                    .unwrap_or(i as u64),
            );
            break;
        }
    }

    // 3. Compare State Hashes (using JSON serialization for deterministic hashing)
    let hash_1 = hash_simulation_events(&sim.events);
    let hash_2 = hash_simulation_events(&replay_sim.events);

    let passes = hash_1 == hash_2 && divergence_point.is_none();

    let config_hash = certification_config_hash(
        replay_mode,
        replay_fill_probability,
        &create_orders_for_replay,
    );

    let fingerprint = Some(crate::DeterminismFingerprint {
        engine_version: "v1.0-deterministic-core".to_string(),
        event_count: sim.events.len(),
        final_hash: hash_1.clone(),
        config_hash,
    });

    Ok(CertificationResponse {
        status: if passes {
            "PASS".to_string()
        } else {
            "FAIL".to_string()
        },
        hash_1,
        hash_2,
        divergence_point,
        fingerprint,
    })
}

fn certification_config_hash(
    mode: ExecutionMode,
    fill_probability: f64,
    orders: &[CreateOrder],
) -> String {
    #[derive(serde::Serialize)]
    struct CertificationConfig<'a> {
        execution_mode: &'static str,
        fill_probability: f64,
        orders: &'a [CreateOrder],
    }

    let execution_mode = match mode {
        ExecutionMode::Real => "real",
        ExecutionMode::Ideal => "ideal",
    };

    let payload = CertificationConfig {
        execution_mode,
        fill_probability,
        orders,
    };
    let serialized = serde_json::to_string(&payload).unwrap_or_default();
    blake3::hash(serialized.as_bytes()).to_hex().to_string()
}

fn hash_simulation_events(events: &[SimEvent]) -> String {
    // In a real system, we'd use SHA256. For this demo, we use a stringified representation
    // of the event stream, which is sufficient for determinism checks.
    let serialized = serde_json::to_string(events).unwrap_or_default();

    // Simple "hash" for demonstration: total length + first 16 chars + last 16 chars
    if serialized.len() < 32 {
        serialized
    } else {
        format!(
            "{}:{}...{}",
            serialized.len(),
            &serialized[0..16],
            &serialized[serialized.len() - 16..]
        )
    }
}
