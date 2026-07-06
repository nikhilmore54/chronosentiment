pub struct Context {
    pub tags: Vec<String>,
}

pub struct SolverState<'a> {
    pub demand_idx: usize,
    pub path_nodes: &'a [u64],
    pub has_interventions: bool,
    pub volume: f64,
}

pub trait EcologyAdapter {
    /// Extracts structural facts from the solver state.
    /// This adapter MUST NOT contain logic for computing pressure, confidence,
    /// trend, or branch ranking. It is strictly a translation layer.
    fn extract_context(&self, state: &SolverState) -> Context;
}
