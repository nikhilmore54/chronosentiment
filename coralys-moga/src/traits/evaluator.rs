use rand::rngs::StdRng;

pub trait Genome: Clone + Send + Sync {}

pub trait GenomeFactory<G: Genome> {
    fn create(&self, rng: &mut StdRng) -> G;
}

pub trait Evaluated: Clone {
    type Genome: Genome;
    fn fitness(&self) -> f64;
    fn is_valid(&self) -> bool;
    fn genome(&self) -> &Self::Genome;
}

pub trait FitnessEvaluator<G: Genome> {
    type Evaluation: Evaluated<Genome = G>;

    fn evaluate(
        &self,
        genome: &G,
        metrics: &crate::runtime::optimization::metric::MetricReport,
    ) -> Self::Evaluation;
}

pub trait MutationOperator<G: Genome> {
    fn mutate(&self, genome: &mut G, rng: &mut StdRng);
}

pub trait CrossoverOperator<G: Genome> {
    fn crossover(&self, parent_a: &G, parent_b: &G, rng: &mut StdRng) -> (G, G);
}

pub trait SelectionStrategy<E: Evaluated> {
    /// Selects `count` number of individuals from the given population based on their evaluation.
    fn select<'a>(&self, evaluations: &'a [E], count: usize) -> Vec<&'a E>;
}
