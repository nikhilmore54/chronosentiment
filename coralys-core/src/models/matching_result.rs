use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchingResult<TAssignment> {
    pub cardinality: usize,
    pub unmatched_supply: usize,
    pub unmatched_demand: usize,
    pub assignments: Vec<TAssignment>,
}
