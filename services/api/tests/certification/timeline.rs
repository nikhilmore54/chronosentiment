use chronosentiment_core::*;

pub fn test_strict_ordering(sim: &SimulationResult) -> Result<(), String> {
    // 1. (timestamp, sequence_id) strictly increasing
    // The event log MUST be strictly monotonic on (ts, seq_id)
    
    let mut last_ts = 0;
    let mut last_seq = 0;
    
    for (idx, event) in sim.events.iter().enumerate() {
        let ts = event.timestamp();
        let seq = event.sequence_id();
        
        if idx > 0 {
            if ts < last_ts {
                return Err(format!("Timestamp violation at index {}: {} < {}", idx, ts, last_ts));
            }
            if ts == last_ts && seq <= last_seq {
                return Err(format!("Sequence ID violation at index {} for same timestamp {}: {} <= {}", idx, ts, seq, last_seq));
            }
        }
        
        last_ts = ts;
        last_seq = seq;
    }
    
    Ok(())
}
