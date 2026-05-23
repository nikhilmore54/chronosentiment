use serde::{Serialize, Deserialize};
use crate::topology::TopologyField;
use crate::cognition::{CognitionGeometry, MemoryState};

/// The canonical replay observability artifact.
/// This strictly captures raw, mechanical deformation, avoiding any explanatory interpretations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OccupancyTrace {
    pub tick_index: u64,
    pub price: f64,
    pub occupancy: f64,
    pub overlap: f64,
    pub acceptance_ratio: f64,
    pub strictness_ratio: f64,
}

/// Generates a replay-equivalent occupancy trace for a specific Topology and Cognition pair.
pub fn generate_occupancy_traces(
    prices: &[f64],
    topology: TopologyField,
    geometry: CognitionGeometry,
) -> Vec<OccupancyTrace> {
    let mut baseline = MemoryState::new(geometry);
    let mut fragmented = MemoryState::new(geometry);
    
    let total_ticks = prices.len() as u64;
    let mut traces = Vec::with_capacity(prices.len());
    
    for (i, &price) in prices.iter().enumerate() {
        let tick_index = i as u64;
        
        // 1. Topology Deformation
        let deformation = topology.apply(tick_index, total_ticks);
        
        // 2. State Ingestion
        baseline.ingest(price); // Perfect continuity
        
        // Deterministic pseudo-random acceptance evaluation for fragmented observation
        let hash_int = tick_index.wrapping_mul(1103515245).wrapping_add(12345);
        let normalized = (hash_int % 1000) as f64 / 1000.0;
        let is_accepted = normalized <= deformation.acceptance_ratio;
        
        if is_accepted {
            fragmented.ingest(price);
        }
        
        // 3. Morphological Evaluation
        let overlap = fragmented.overlap_ratio(&baseline);
        let occupancy = 1.0 - overlap;
        
        // 4. Trace Emission
        traces.push(OccupancyTrace {
            tick_index,
            price,
            occupancy,
            overlap,
            acceptance_ratio: deformation.acceptance_ratio,
            strictness_ratio: deformation.strict_ratio,
        });
    }
    
    traces
}
