use crate::traits::{Genome, GenomeFactory, Evaluated, FitnessEvaluator, MutationOperator, CrossoverOperator};
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

pub struct GaConfig {
    pub population_size: usize,
    pub generations: usize,
    pub seed: u64,
}

pub struct EvolutionEngine<
    G: Genome, 
    F: FitnessEvaluator<G>, 
    M: MutationOperator<G>, 
    C: CrossoverOperator<G>,
    Factory: GenomeFactory<G>
> {
    evaluator: F,
    mutator: M,
    crossover: C,
    factory: Factory,
    _marker: std::marker::PhantomData<G>,
}

impl<
    G: Genome, 
    F: FitnessEvaluator<G>, 
    M: MutationOperator<G>, 
    C: CrossoverOperator<G>,
    Factory: GenomeFactory<G>
> EvolutionEngine<G, F, M, C, Factory> {

    pub fn initialize_population(&self, config: &GaConfig, rng: &mut StdRng) -> Vec<G> {
        (0..config.population_size)
            .map(|_| self.factory.create(rng))
            .collect()
    }

    pub fn tournament_selection<'a>(
        &self,
        evaluations: &'a [F::Evaluation],
        k: usize,
        rng: &mut StdRng,
    ) -> &'a F::Evaluation {
        let mut best: Option<&'a F::Evaluation> = None;
        for _ in 0..k {
            let idx = rng.gen_range(0..evaluations.len());
            let eval = &evaluations[idx];
            if best.is_none() || eval.fitness() > best.unwrap().fitness() {
                best = Some(eval);
            }
        }
        best.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct BitGenome { bits: Vec<bool> }
    impl Genome for BitGenome {}

    struct BitGenomeFactory;
    impl GenomeFactory<BitGenome> for BitGenomeFactory {
        fn create(&self, _rng: &mut StdRng) -> BitGenome {
            BitGenome { bits: vec![] }
        }
    }

    #[derive(Clone)]
    struct BitEvaluation { fitness: f64, valid: bool, genome: BitGenome }
    impl Evaluated for BitEvaluation {
        type Genome = BitGenome;
        fn fitness(&self) -> f64 { self.fitness }
        fn is_valid(&self) -> bool { self.valid }
        fn genome(&self) -> &Self::Genome { &self.genome }
    }

    struct DummyEvaluator;
    impl FitnessEvaluator<BitGenome> for DummyEvaluator {
        type Evaluation = BitEvaluation;
        fn evaluate(&self, _candidate: &BitGenome) -> Self::Evaluation { 
            BitEvaluation { fitness: 1.0, valid: true, genome: _candidate.clone() } 
        }
    }

    struct DummyMutator;
    impl MutationOperator<BitGenome> for DummyMutator {
        fn mutate(&self, _candidate: &mut BitGenome, _rng: &mut rand::rngs::StdRng) {}
    }

    struct DummyCrossover;
    impl CrossoverOperator<BitGenome> for DummyCrossover {
        fn crossover(&self, _parent1: &BitGenome, _parent2: &BitGenome, _rng: &mut rand::rngs::StdRng) -> (BitGenome, BitGenome) {
            (BitGenome { bits: vec![] }, BitGenome { bits: vec![] })
        }
    }

    #[test]
    fn test_genericity_proof_refactored() {
        let engine = EvolutionEngine {
            evaluator: DummyEvaluator,
            mutator: DummyMutator,
            crossover: DummyCrossover,
            factory: BitGenomeFactory,
            _marker: std::marker::PhantomData,
        };

        let config = GaConfig { population_size: 10, generations: 2, seed: 42 };
        let pop = engine.initialize_population(&config);
        assert_eq!(pop.len(), 10);

        let mut rng = StdRng::seed_from_u64(config.seed);
        let evals = vec![
            BitEvaluation { fitness: 0.5, valid: true, genome: BitGenome { bits: vec![] } },
            BitEvaluation { fitness: 0.9, valid: true, genome: BitGenome { bits: vec![] } },
            BitEvaluation { fitness: 0.1, valid: true, genome: BitGenome { bits: vec![] } },
        ];

        let best = engine.tournament_selection(&evals, 2, &mut rng);
        assert!(best.fitness() > 0.0);
    }
}
