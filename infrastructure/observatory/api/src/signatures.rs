//! Canonical signature authority for ChronoSentiment.
//!
//! All signatures use BLAKE3 (256-bit, cryptographically secure).
//! This module is the sole source of signature computation — no other
//! module may compute signatures independently.
//!
//! Constitutional authority: Constitutional Law One — every certified
//! value must be traceable to a backend-computed signature.
//!
//! ## Attestation vs metadata (Phase D)
//! `compute_replay_signature` is a **METADATA_SIGNATURE** — not event attestation.
//! Event-grounded digests use `compute_event_stream_hash` per
//! `docs/contracts/REPLAY_ATTESTATION_CONTRACT_v1.md`.

use std::collections::BTreeMap;

use chronosentiment_core::SimEvent;

use crate::dto::{CanonicalEvent, NarrativeBlock, SourceLayer};

/// Compute the kernel_signature for a single event.
///
/// Input: `sequence_id || timestamp_ns || event_type || source_layer || payload_json`
/// Output: BLAKE3 hex digest (64 hex chars)
pub fn compute_event_signature(
    sequence_id: u64,
    timestamp_ns: u64,
    event_type: &str,
    source_layer: &SourceLayer,
    payload: &serde_json::Value,
) -> String {
    let source_layer_str = format!("{:?}", source_layer).to_uppercase();
    let payload_str = serde_json::to_string(payload).unwrap_or_default();
    let input = format!(
        "{}:{}:{}:{}:{}",
        sequence_id, timestamp_ns, event_type, source_layer_str, payload_str
    );
    blake3::hash(input.as_bytes()).to_hex().to_string()
}

/// Compute the replay_signature for a replay response.
///
/// **METADATA_SIGNATURE** — NOT event attestation. Input is session metadata only.
/// For event-grounded certification use `compute_event_stream_hash`.
///
/// Input: `session_id || strategy_id || requested_sequence_id || certification_state || event_count`
/// Output: BLAKE3 hex digest (64 hex chars)
pub fn compute_replay_signature(
    session_id: &uuid::Uuid,
    strategy_id: &str,
    requested_sequence_id: u64,
    certification_state: &str,
    event_count: usize,
) -> String {
    let input = format!(
        "{}:{}:{}:{}:{}",
        session_id, strategy_id, requested_sequence_id, certification_state, event_count
    );
    blake3::hash(input.as_bytes()).to_hex().to_string()
}

/// Canonical event stream hash for domain attestation (Phase D).
///
/// See `REPLAY_ATTESTATION_CONTRACT_v1.md` §6.1.
pub fn compute_event_stream_hash(
    events: &[SimEvent],
    substrate_reference: &str,
    engine_mode: &str,
) -> String {
    let mut ordered: Vec<&SimEvent> = events.iter().collect();
    ordered.sort_by_key(|event| event.sequence_id());

    let header = format!(
        "event_stream_v1:{}:{}:{}\n",
        events.len(),
        substrate_reference,
        engine_mode
    );
    let body = ordered
        .iter()
        .map(|event| serde_json::to_string(event).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");

    blake3::hash(format!("{header}{body}").as_bytes())
        .to_hex()
        .to_string()
}

/// Canonical scenario result hash for domain attestation (Phase D).
///
/// See `REPLAY_ATTESTATION_CONTRACT_v1.md` §6.2.
pub fn compute_scenario_result_hash(
    scenario_id: &str,
    domain_class: &str,
    engine_mode: &str,
    fitness: f64,
    execution_fitness: f64,
    avg_pnl: f64,
    std_dev: f64,
    max_drawdown: f64,
    trade_count: usize,
) -> String {
    let payload = BTreeMap::from([
        ("avg_pnl", serde_json::json!(avg_pnl)),
        ("domain_class", serde_json::json!(domain_class)),
        ("engine_mode", serde_json::json!(engine_mode)),
        ("execution_fitness", serde_json::json!(execution_fitness)),
        ("fitness", serde_json::json!(fitness)),
        ("max_drawdown", serde_json::json!(max_drawdown)),
        ("scenario_id", serde_json::json!(scenario_id)),
        ("std_dev", serde_json::json!(std_dev)),
        ("trade_count", serde_json::json!(trade_count)),
    ]);
    let canonical = serde_json::to_string(&payload).unwrap_or_default();
    blake3::hash(format!("result_v1:{canonical}").as_bytes())
        .to_hex()
        .to_string()
}

/// Canonical aggregation hash (Phase D Level 4).
///
/// `domain_results` MUST be sorted by `scenario_id` ascending.
/// See `REPLAY_ATTESTATION_CONTRACT_v1.md` §6.3.
pub fn compute_aggregate_hash(reducer_id: &str, domain_results: &[(&str, &str)]) -> String {
    let pairs = domain_results
        .iter()
        .map(|(scenario_id, result_hash)| format!("{scenario_id}:{result_hash}"))
        .collect::<Vec<_>>()
        .join(",");
    let input = format!("aggregate_v1:{reducer_id}:{pairs}");
    blake3::hash(input.as_bytes()).to_hex().to_string()
}

/// Compute the trace_signature for a decision trace.
///
/// Input: `trace_id || anchor_sequence_id || strategy_id || decision_action || decision_verdict`
/// Output: BLAKE3 hex digest (64 hex chars)
pub fn compute_trace_signature(
    trace_id: &uuid::Uuid,
    anchor_sequence_id: u64,
    strategy_id: &str,
    decision_action: &str,
    decision_verdict: &str,
) -> String {
    let input = format!(
        "{}:{}:{}:{}:{}",
        trace_id, anchor_sequence_id, strategy_id, decision_action, decision_verdict
    );
    blake3::hash(input.as_bytes()).to_hex().to_string()
}

/// Compute the telemetry_signature for a governor telemetry record.
///
/// Input: `telemetry_id || anchor_sequence_id || event_class || governor_id`
/// Output: BLAKE3 hex digest (64 hex chars)
pub fn compute_telemetry_signature(
    telemetry_id: &uuid::Uuid,
    anchor_sequence_id: u64,
    event_class: &str,
    governor_id: &str,
) -> String {
    let input = format!(
        "{}:{}:{}:{}",
        telemetry_id, anchor_sequence_id, event_class, governor_id
    );
    blake3::hash(input.as_bytes()).to_hex().to_string()
}

/// Compute the observatory_signature for an observatory state snapshot.
///
/// Input: `snapshot_id || snapshot_sequence_id || system_phase || throttle_state || sync_ratio`
/// Output: BLAKE3 hex digest (64 hex chars)
pub fn compute_observatory_signature(
    snapshot_id: &uuid::Uuid,
    snapshot_sequence_id: u64,
    system_phase: &str,
    throttle_state: &str,
    sync_ratio: f64,
) -> String {
    let input = format!(
        "{}:{}:{}:{}:{:.6}",
        snapshot_id, snapshot_sequence_id, system_phase, throttle_state, sync_ratio
    );
    blake3::hash(input.as_bytes()).to_hex().to_string()
}

/// Sign a batch of canonical events, mutating their `kernel_signature` fields.
/// This is the authoritative signing pass — called once per event stream.
pub fn sign_event_batch(events: &mut Vec<CanonicalEvent>) {
    for event in events.iter_mut() {
        event.kernel_signature = compute_event_signature(
            event.sequence_id,
            event.timestamp_ns,
            &event.event_type,
            &event.source_layer,
            &event.payload,
        );
    }
}

/// Build a narrative block's content hash for audit purposes.
/// Not a signature field in the schema — used for internal integrity checks.
pub fn hash_narrative_block(block: &NarrativeBlock) -> String {
    let input = format!(
        "{}:{}:{:?}:{}",
        block.block_id, block.sequence_id, block.group, block.narrative
    );
    blake3::hash(input.as_bytes()).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_event_signature_is_deterministic() {
        let payload = serde_json::json!({ "order_id": "O1", "quantity": 100 });
        let sig1 = compute_event_signature(
            1,
            1000000,
            "ORDER_QUEUED",
            &SourceLayer::Sequencer,
            &payload,
        );
        let sig2 = compute_event_signature(
            1,
            1000000,
            "ORDER_QUEUED",
            &SourceLayer::Sequencer,
            &payload,
        );
        assert_eq!(sig1, sig2);
        assert_eq!(sig1.len(), 64); // BLAKE3 hex = 32 bytes = 64 hex chars
    }

    #[test]
    fn test_event_signature_changes_with_input() {
        let payload = serde_json::json!({});
        let sig1 = compute_event_signature(
            1,
            1000000,
            "ORDER_QUEUED",
            &SourceLayer::Sequencer,
            &payload,
        );
        let sig2 = compute_event_signature(
            2,
            1000000,
            "ORDER_QUEUED",
            &SourceLayer::Sequencer,
            &payload,
        );
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_replay_signature_is_deterministic() {
        let id = Uuid::new_v4();
        let sig1 = compute_replay_signature(&id, "strat_100_200_50_30", 42, "CERTIFIED", 150);
        let sig2 = compute_replay_signature(&id, "strat_100_200_50_30", 42, "CERTIFIED", 150);
        assert_eq!(sig1, sig2);
        assert_eq!(sig1.len(), 64);
    }

    #[test]
    fn test_trace_signature_is_deterministic() {
        let id = Uuid::new_v4();
        let sig1 = compute_trace_signature(&id, 42, "strat_100_200_50_30", "BUY", "STRONG_BUY");
        let sig2 = compute_trace_signature(&id, 42, "strat_100_200_50_30", "BUY", "STRONG_BUY");
        assert_eq!(sig1, sig2);
        assert_eq!(sig1.len(), 64);
    }

    #[test]
    fn test_event_stream_hash_is_deterministic() {
        use chronosentiment_core::{MarketEventType, Side, SimEvent};
        let events = vec![SimEvent::MarketEvent {
            sequence_id: 1,
            parent_sequence_id: None,
            subtype: MarketEventType::Trade,
            price: 100,
            quantity: 10,
            side: Some(Side::Buy),
            timestamp: 1,
        }];
        let h1 = compute_event_stream_hash(&events, "deterministic_demo_v1", "REAL");
        let h2 = compute_event_stream_hash(&events, "deterministic_demo_v1", "REAL");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn test_replay_signature_differs_from_event_stream_hash() {
        use chronosentiment_core::{MarketEventType, Side, SimEvent};
        let id = Uuid::new_v4();
        let events = vec![SimEvent::MarketEvent {
            sequence_id: 1,
            parent_sequence_id: None,
            subtype: MarketEventType::Trade,
            price: 100,
            quantity: 10,
            side: Some(Side::Buy),
            timestamp: 1,
        }];
        let metadata_sig = compute_replay_signature(&id, "strat_1", 42, "CERTIFIED", 1);
        let event_hash = compute_event_stream_hash(&events, "deterministic_demo_v1", "REAL");
        assert_ne!(metadata_sig, event_hash);
    }

    #[test]
    fn test_aggregate_hash_is_deterministic() {
        let pairs = [
            ("deterministic_demo", "abc123"),
            ("deterministic_demo_execution", "def456"),
        ];
        let h1 = compute_aggregate_hash("robust_min_execution_fitness", &pairs);
        let h2 = compute_aggregate_hash("robust_min_execution_fitness", &pairs);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn test_aggregate_hash_changes_when_result_hash_changes() {
        let base = [
            ("deterministic_demo", "abc123"),
            ("deterministic_demo_execution", "def456"),
        ];
        let mutated = [
            ("deterministic_demo", "abc123"),
            ("deterministic_demo_execution", "000000"),
        ];
        assert_ne!(
            compute_aggregate_hash("robust_min_execution_fitness", &base),
            compute_aggregate_hash("robust_min_execution_fitness", &mutated)
        );
    }
}
