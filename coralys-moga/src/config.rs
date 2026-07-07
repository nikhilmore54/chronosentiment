#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct EvolutionConfig {
    pub population_size: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub elite_count: usize,
    pub generation_limit: usize,
    pub seed: Option<u64>,
    pub tournament_size: Option<usize>,
    pub termination_policy: Option<crate::termination::TerminationPolicy>,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            population_size: 100,
            mutation_rate: 0.1,
            crossover_rate: 0.8,
            elite_count: 10,
            generation_limit: 100,
            seed: None,
            tournament_size: None,
            termination_policy: None,
        }
    }
}
impl EvolutionConfig {
    /// Demo configuration with deterministic seed and recommended parameters
    pub fn demo() -> Self {
        Self {
            population_size: 100,
            mutation_rate: 1.0,
            crossover_rate: 0.8,
            elite_count: 2,
            generation_limit: 100,
            seed: Some(42),
            tournament_size: Some(3),
            termination_policy: None,
        }
    }
}


