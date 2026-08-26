use super::models::{InrcHistory, InrcScenario, InrcWeekData};
use std::fs;
use std::path::Path;

pub fn parse_scenario<P: AsRef<Path>>(path: P) -> Result<InrcScenario, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(path)?;
    let scenario: InrcScenario = serde_json::from_str(&data)?;
    Ok(scenario)
}

pub fn parse_history<P: AsRef<Path>>(path: P) -> Result<InrcHistory, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(path)?;
    let history: InrcHistory = serde_json::from_str(&data)?;
    Ok(history)
}

pub fn parse_week_data<P: AsRef<Path>>(
    path: P,
) -> Result<InrcWeekData, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(path)?;
    let week_data: InrcWeekData = serde_json::from_str(&data)?;
    Ok(week_data)
}
