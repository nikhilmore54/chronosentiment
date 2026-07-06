use std::collections::HashMap;
use std::hash::Hash;

/// Trait defining a resource that can be tracked in an EcologyMemory.
pub trait ResourceId: Eq + Hash + Clone + Copy {}
impl<T: Eq + Hash + Clone + Copy> ResourceId for T {}

/// Represents an accumulated measure of resource activity (e.g., assignments, distance).
pub type MeasureValue = f64;

/// A generic state accumulator mapping resources to multiple tracked measures.
#[derive(Clone, Debug)]
pub struct EcologyMemory<R: ResourceId> {
    /// Maps a string metric name to a map of ResourceId -> Value
    measures: HashMap<String, HashMap<R, MeasureValue>>,
}

impl<R: ResourceId> EcologyMemory<R> {
    pub fn new() -> Self {
        Self {
            measures: HashMap::new(),
        }
    }

    /// Add a value to the cumulative total of a specific measure for a given resource.
    pub fn accumulate(&mut self, resource: R, measure_name: &str, value: MeasureValue) {
        let entry = self
            .measures
            .entry(measure_name.to_string())
            .or_default()
            .entry(resource)
            .or_insert(0.0);
        *entry += value;
    }

    /// Get the cumulative total of a specific measure for a given resource.
    pub fn get_measure(&self, resource: R, measure_name: &str) -> MeasureValue {
        self.measures
            .get(measure_name)
            .and_then(|m| m.get(&resource))
            .copied()
            .unwrap_or(0.0)
    }

    /// Get all resources that have been tracked for a specific measure.
    pub fn get_tracked_resources(&self, measure_name: &str) -> Vec<R> {
        self.measures
            .get(measure_name)
            .map(|m| m.keys().copied().collect())
            .unwrap_or_default()
    }
}

impl<R: ResourceId> Default for EcologyMemory<R> {
    fn default() -> Self {
        Self::new()
    }
}
