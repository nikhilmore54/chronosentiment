use super::memory::ResourceId;
use super::signal::EcologySignal;

/// Trait for a target that can apply EcologySignal guidance.
pub trait EcologyGuidanceTarget<R: ResourceId> {
    /// Apply the given EcologySignal to the target.
    fn apply_signal(&mut self, resource: R, signal: EcologySignal);
}
