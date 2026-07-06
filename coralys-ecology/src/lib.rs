pub mod traits;
pub mod models;
pub mod diagnostics;
pub mod progress;
pub mod state;


pub use traits::{MemoryModel, TopologyModel};
pub use models::{CognitionGeometry, MemoryState, TopologyField, DeformationState};
pub use diagnostics::{DiagnosticResult, Metric};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_implementations_compile() {
        // Test MemoryState
        let mut memory = MemoryState::new(CognitionGeometry::RollingBounded { window: 3 });
        memory.observe(1.0);
        memory.observe(2.0);
        memory.observe(3.0);
        memory.observe(4.0);

        let final_state = memory.state();
        assert_eq!(final_state.buffer, vec![2.0, 3.0, 4.0]);

        // Test TopologyField
        let topology = TopologyField::UniformDelay { delay_ticks: 5 };
        let deformation = topology.transform((10, 100));
        assert_eq!(deformation.acceptance_ratio, 1.0);
        assert_eq!(deformation.strict_ratio, 0.0);
    }
}
