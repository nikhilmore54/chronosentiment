use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use serde_json::to_writer_pretty;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InputRecord {
    pub id: String,
    pub name: String,
    // add other columns as needed
}

#[derive(Debug, Serialize, Clone)]
pub struct Assignment {
    pub nurse_id: String,
    pub shift: String,
    pub day: u32,
}

/// Import CSV into a JSON file for later steps.
pub fn import_data(csv_path: &PathBuf, out_json: &PathBuf) -> Result<(), Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().from_path(csv_path)?;
    let mut records: Vec<InputRecord> = Vec::new();
    for result in rdr.deserialize() {
        let rec: InputRecord = result?;
        records.push(rec);
    }
    let file = File::create(out_json)?;
    to_writer_pretty(file, &records)?;
    Ok(())
}

/// Placeholder optimizer – returns a single dummy assignment.
pub fn run_optimizer(_data_path: &PathBuf, out_json: &PathBuf) -> Result<(), Box<dyn Error>> {
    // In production replace with real optimizer call.
    let assignments = vec![Assignment {
        nurse_id: "1".to_string(),
        shift: "day".to_string(),
        day: 1,
    }];
    let file = File::create(out_json)?;
    to_writer_pretty(file, &assignments)?;
    Ok(())
}

/// Generate explanations for assignments.
pub fn generate_explanations(assignments_path: &PathBuf) -> Result<Vec<String>, Box<dyn Error>> {
    let file = File::open(assignments_path)?;
    let assignments: Vec<Assignment> = serde_json::from_reader(file)?;
    let explanations = assignments
        .iter()
        .map(|a| format!("Nurse {} assigned to {} on day {}", a.nurse_id, a.shift, a.day))
        .collect();
    Ok(explanations)
}

/// Validate constraints – always true for now.
pub fn validate_constraints(_assignments_path: &PathBuf) -> Result<bool, Box<dyn Error>> {
    Ok(true)
}

/// Placeholder edit – currently a no‑op (could modify JSON if needed).
pub fn edit_schedule(_assignments_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    // No changes for now.
    Ok(())
}

/// Placeholder approve – always succeeds.
pub fn approve_schedule(_assignments_path: &PathBuf) -> Result<bool, Box<dyn Error>> {
    Ok(true)
}

/// Export assignments to CSV.
pub fn export_schedule(assignments_path: &PathBuf, out_csv: &PathBuf) -> Result<(), Box<dyn Error>> {
    let file = File::open(assignments_path)?;
    let assignments: Vec<Assignment> = serde_json::from_reader(file)?;
    let mut wtr = WriterBuilder::new().has_headers(true).from_path(out_csv)?;
    wtr.write_record(&["nurse_id", "shift", "day"])?;
    for a in assignments {
        wtr.write_record(&[&a.nurse_id, &a.shift, &a.day.to_string()])?;
    }
    wtr.flush()?;
    Ok(())
}

/// Save a deterministic replay log (JSON).
pub fn save_replay(assignments_path: &PathBuf, out_json: &PathBuf) -> Result<(), Box<dyn Error>> {
    std::fs::copy(assignments_path, out_json)?;
    Ok(())
}

/// Runs the full GA optimization pipeline using the ScheduleOptimizer.
/// Returns a `GaResult` containing the best `ScheduleEvaluation`.
pub fn run_optimization(context: &std::sync::Arc<crate::optimization::ScheduleContext>, config: coralys_moga::config::EvolutionConfig) -> coralys_moga::engine::GaResult<crate::optimization::ScheduleEvaluation> {
    use crate::optimization::ScheduleOptimizer;
    use coralys_moga::engine::EvolutionEngine;
    // Create the optimizer which implements required traits.
    let optimizer = ScheduleOptimizer::new(context.clone());
    // Build the evolution engine with the optimizer as evaluator, mutator, crossover, and factory.
    let engine = EvolutionEngine::new(
        optimizer.clone(), // evaluator
        optimizer.clone(), // mutator
        optimizer.clone(), // crossover
        optimizer.clone(), // factory
    );
    // Run the GA evolution and return the result.
    engine.run_ga_evolution(config)

