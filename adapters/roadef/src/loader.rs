use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use anyhow::{Context, Result};
use crate::models::{Network, TrafficMatrix, Scenario, Solution};

pub fn load_network<P: AsRef<Path>>(path: P) -> Result<Network> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let network: Network = serde_json::from_reader(reader).context("Failed to parse network JSON")?;
    Ok(network)
}

pub fn load_traffic_matrix<P: AsRef<Path>>(path: P) -> Result<TrafficMatrix> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let tm: TrafficMatrix = serde_json::from_reader(reader).context("Failed to parse traffic matrix JSON")?;
    Ok(tm)
}

pub fn load_scenario<P: AsRef<Path>>(path: P) -> Result<Scenario> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let scenario: Scenario = serde_json::from_reader(reader).context("Failed to parse scenario JSON")?;
    Ok(scenario)
}

pub fn load_solution<P: AsRef<Path>>(path: P) -> Result<Solution> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let solution: Solution = serde_json::from_reader(reader).context("Failed to parse solution JSON")?;
    Ok(solution)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_set_a_01() {
        let net = load_network("repo/challenge-roadef-2026-main/setA/setA-01-net.json").unwrap();
        assert_eq!(net.directed, true);
        assert!(!net.nodes.is_empty());
        assert!(!net.links.is_empty());

        let tm = load_traffic_matrix("repo/challenge-roadef-2026-main/setA/setA-01-tm.json").unwrap();
        assert!(tm.num_time_slots > 0);
        assert!(!tm.demands.is_empty());

        let scenario = load_scenario("repo/challenge-roadef-2026-main/setA/setA-01-scenario.json").unwrap();
        assert!(scenario.max_segments >= 0);
        assert!(!scenario.budget.is_empty());
    }
}
