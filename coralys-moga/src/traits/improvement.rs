use crate::traits::Genome;

pub trait ImprovementOperator<G: Genome> {
    fn improve(&self, genome: &mut G);
}

pub struct NoOpImprovement;

impl<G: Genome> ImprovementOperator<G> for NoOpImprovement {
    fn improve(&self, _genome: &mut G) {}
}

pub trait LocalSearchOperator<G: Genome> {
    fn search(&self, genome: &mut G);
}

impl<G: Genome, T: ImprovementOperator<G> + ?Sized> LocalSearchOperator<G> for T {
    fn search(&self, genome: &mut G) {
        self.improve(genome);
    }
}

pub trait ObservedTransitionMetric<G: Genome> {
    fn magnitude(&self, source: &G, result_after_repair: &G) -> f64;
}

pub trait RegionIdentifier<G: Genome> {
    type RegionId: std::hash::Hash + std::cmp::Eq + Clone;
    fn region_of(&self, state: &G) -> Self::RegionId;
}
