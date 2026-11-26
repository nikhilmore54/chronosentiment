//! UltraCrew Story 1 prototype – end‑to‑end workflow implemented in Rust.
//
// This binary demonstrates the full Alpha flow: import → optimize → explain →
// validate → approve → export.
// It uses the existing Coralys/MOGA optimizer (placeholder) and the csv &
// serde crates for I/O. The implementation is intentionally simple – the goal
// is to have a runnable Rust binary that can evolve directly into the product.

use std::error::Error;
use std::fs::File;
use std::path::PathBuf;

use csv::ReaderBuilder;
use csv::WriterBuilder;
use serde::{Deserialize, Serialize};
use serde_json::to_writer_pretty;

/// Record representing a row from the input CSV. Extend as needed.
#[derive(Debug, Deserialize, Serialize, Clone)]
struct InputRecord {
    id: String,
    name: String,
    // add other columns here
}

/// Simple schedule assignment – placeholder for real optimizer output.
#[derive(Debug, Serialize, Clone)]
struct Assignment {
    nurse_id: String,
    shift: String,
    day: u32,
}

/// Placeholder optimizer – in production replace with a call into the
/// Coralys‑MOGA library (e.g., via a CLI or a Rust API).
fn run_optimizer(_data: &[InputRecord]) -> Vec<Assignment> {
    // TODO: integrate actual optimizer
    vec![Assignment {
        nurse_id: "1".to_string(),
        shift: "day".to_string(),
        day: 1,
    }]
}

/// Generate a minimal explanation for each assignment.
fn generate_explanation(assignments: &[Assignment]) -> Vec<String> {
    assignments
        .iter()
        .map(|a| {
            format!(
                "Nurse {} assigned to {} on day {}",
                a.nurse_id, a.shift, a.day
            )
        })
        .collect()
}

/// Validate constraints – always true for now.
fn validate_constraints(_assignments: &[Assignment]) -> bool {
    // TODO: implement real business rule checks
    true
}

/// Export the schedule as CSV.
fn export_schedule(assignments: &[Assignment], out_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let mut wtr = WriterBuilder::new().has_headers(true).from_path(out_path)?;
    wtr.write_record(&["nurse_id", "shift", "day"])?;
    for a in assignments {
        wtr.write_record(&[&a.nurse_id, &a.shift, &a.day.to_string()])?;
    }
    wtr.flush()?;
    Ok(())
}

/// Save a deterministic replay log (JSON).
fn save_replay(assignments: &[Assignment], out_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let file = File::create(out_path)?;
    to_writer_pretty(file, assignments)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Paths – in a real binary they would be CLI arguments.
    let data_path = PathBuf::from("data/sample_dataset.csv");
    let out_schedule = PathBuf::from("results/story1_schedule.csv");
    let out_replay = PathBuf::from("results/story1_replay.json");

    // 1. Import
    let mut rdr = ReaderBuilder::new().from_path(&data_path)?;
    let mut records: Vec<InputRecord> = Vec::new();
    for result in rdr.deserialize() {
        let rec: InputRecord = result?;
        records.push(rec);
    }

    // 2. Optimize (placeholder)
    let assignments = run_optimizer(&records);

    // 3. Explain
    let explanations = generate_explanation(&assignments);
    // In a full product we would persist explanations; here we just print.
    for e in &explanations {
        println!("Explanation: {}", e);
    }

    // 4. Validate
    if !validate_constraints(&assignments) {
        return Err("Constraint validation failed".into());
    }

    // 5. Export schedule
    export_schedule(&assignments, &out_schedule)?;

    // 6. Save replay
    save_replay(&assignments, &out_replay)?;

    println!("Story 1 completed successfully");
    Ok(())
}
