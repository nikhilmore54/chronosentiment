use chronosentiment_core::*;
use serde_json::Value;

pub fn test_event_schema_lock() -> Result<(), String> {
    // 1. Generate events
    let sim = run_simulation(ExecutionMode::Real);
    
    // 2. Extract schema information (types + field names) for each event variant
    // We use JSON serialization to inspect the field structure generically
    let mut schema_fingerprint = Vec::new();
    
    for event in &sim.events {
        let serialized = serde_json::to_value(event).map_err(|e| e.to_string())?;
        if let Value::Object(map) = serialized {
            // Variant name is often part of the object if externally tagged, 
            // or we can extract it if we know the structure.
            // For SimEvent, it's externally tagged by default with serde.
            let variant_name = map.keys().next().ok_or("Invalid event structure")?.clone();
            let fields = if let Some(Value::Object(inner_map)) = map.get(&variant_name) {
                let mut f: Vec<String> = inner_map.keys().cloned().collect();
                f.sort();
                f
            } else {
                Vec::new()
            };
            
            let entry = format!("{}: [{}]", variant_name, fields.join(", "));
            if !schema_fingerprint.contains(&entry) {
                schema_fingerprint.push(entry);
            }
        }
    }
    
    schema_fingerprint.sort();
    let current_schema = schema_fingerprint.join("\n");

    // 3. Compare against COMMITTED SNAPSHOT
    // This snapshot represents the "Event Contract" that must not break
    let committed_schema = r#"MarketEvent: [parent_sequence_id, price, quantity, sequence_id, subtype, ts]
OrderEnteredQueue: [order_id, parent_sequence_id, price, quantity_ahead, sequence_id, ts]
OrderIntent: [order_id, parent_sequence_id, price, quantity, sequence_id, side, ts]
PartialFill: [filled_qty, order_id, parent_sequence_id, price, sequence_id, ts]
QueueProgression: [new_quantity_ahead, order_id, parent_sequence_id, sequence_id, ts]"#;

    if current_schema != committed_schema {
        return Err(format!(
            "EVENT SCHEMA DRIFT DETECTED!\n\nExpected:\n{}\n\nFound:\n{}",
            committed_schema, current_schema
        ));
    }

    Ok(())
}
