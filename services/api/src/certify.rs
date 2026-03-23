use chronosentiment_core::*;
use crate::{ApiError, CertificationResponse};
use serde_json;

pub fn handle_certify(sim: &SimulationResult) -> Result<CertificationResponse, ApiError> {
    // 1. Run SAME simulation twice internally (or compare against current)
    // The requirement says: "Run SAME simulation twice internally"
    // Since we have 'sim', we'll run it again and compare.
    
    let replay_sim = run_simulation(ExecutionMode::Real); 

    // 2. Compare event sequences and find divergence point
    let mut divergence_point = None;
    let max_len = std::cmp::max(sim.events.len(), replay_sim.events.len());
    
    for i in 0..max_len {
        let e1 = sim.events.get(i);
        let e2 = replay_sim.events.get(i);
        
        if e1 != e2 {
            divergence_point = Some(e1.map(|e| e.sequence_id()).or(e2.map(|e| e.sequence_id())).unwrap_or(i as u64));
            break;
        }
    }

    // 3. Compare State Hashes (using JSON serialization for deterministic hashing)
    let hash_1 = hash_simulation_events(&sim.events);
    let hash_2 = hash_simulation_events(&replay_sim.events);

    let passes = hash_1 == hash_2 && divergence_point.is_none();

    let fingerprint = Some(crate::DeterminismFingerprint {
        engine_version: "v1.0-deterministic-core".to_string(),
        event_count: sim.events.len(),
        final_hash: hash_1.clone(),
        config_hash: "default-config-hash".to_string(), // In a real system, this would be a hash of the simulation config
    });

    Ok(CertificationResponse {
        status: if passes { "PASS".to_string() } else { "FAIL".to_string() },
        hash_1,
        hash_2,
        divergence_point,
        fingerprint,
    })
}

fn hash_simulation_events(events: &[SimEvent]) -> String {
    // In a real system, we'd use SHA256. For this demo, we use a stringified representation
    // of the event stream, which is sufficient for determinism checks.
    let serialized = serde_json::to_string(events).unwrap_or_default();
    
    // Simple "hash" for demonstration: total length + first 16 chars + last 16 chars
    if serialized.len() < 32 {
        serialized
    } else {
        format!("{}:{}...{}", 
            serialized.len(),
            &serialized[0..16], 
            &serialized[serialized.len()-16..]
        )
    }
}
