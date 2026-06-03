#[derive(Debug, Clone, PartialEq)]
pub struct EvolutionConfig {
    pub population_size: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub elite_count: usize,
    pub generation_limit: usize,
    pub seed: Option<u64>,
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
        }
    }
}
