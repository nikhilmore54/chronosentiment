use crate::traits::Genome;

#[derive(Debug, Clone)]
pub struct Organism<G: Genome> {
    pub genome: G,
}

#[derive(Debug, Clone)]
pub struct Population<G: Genome> {
    pub organisms: Vec<Organism<G>>,
}

#[derive(Debug, Clone)]
pub struct EliteArchive<G: Genome> {
    pub elites: Vec<Organism<G>>,
}

#[derive(Debug, Clone, Default)]
pub struct EvolutionState {
    pub generation: usize,
    pub evaluations: usize,
}

#[derive(Debug, Clone)]
pub struct GenerationResult {
    pub generation: usize,
    pub population_size: usize,
}
