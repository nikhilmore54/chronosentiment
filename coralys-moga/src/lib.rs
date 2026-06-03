pub mod config;
pub mod state;
pub mod traits;
pub mod engine_proof;
pub mod engine;

pub use config::EvolutionConfig;
pub use state::{EliteArchive, EvolutionState, GenerationResult, Organism, Population};
pub use traits::{
    CrossoverOperator, FitnessEvaluator, Genome, MutationOperator, SelectionStrategy, Evaluated, GenomeFactory
};

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct TestGenome {
        pub value: f64,
    }

    impl Genome for TestGenome {}

    #[derive(Clone, Debug, PartialEq)]
    struct TestEvaluation { fitness: f64, valid: bool, genome: TestGenome }
    impl Evaluated for TestEvaluation {
        type Genome = TestGenome;
        fn fitness(&self) -> f64 { self.fitness }
        fn is_valid(&self) -> bool { self.valid }
        fn genome(&self) -> &Self::Genome { &self.genome }
    }

    struct DummyEvaluator;
    impl FitnessEvaluator<TestGenome> for DummyEvaluator {
        type Evaluation = TestEvaluation;
        fn evaluate(&self, genome: &TestGenome) -> Self::Evaluation {
            TestEvaluation { fitness: genome.value, valid: true, genome: genome.clone() }
        }
    }

    struct DummyMutation;
    impl MutationOperator<TestGenome> for DummyMutation {
        fn mutate(&self, genome: &mut TestGenome, _rng: &mut rand::rngs::StdRng) {
            genome.value += 1.0;
        }
    }

    struct DummyCrossover;
    impl CrossoverOperator<TestGenome> for DummyCrossover {
        fn crossover(&self, parent_a: &TestGenome, parent_b: &TestGenome, _rng: &mut rand::rngs::StdRng) -> (TestGenome, TestGenome) {
            let avg = (parent_a.value + parent_b.value) / 2.0;
            (TestGenome { value: avg }, TestGenome { value: avg })
        }
    }

    struct DummySelection;
    impl SelectionStrategy<TestEvaluation> for DummySelection {
        fn select<'a>(&self, evaluations: &'a [TestEvaluation], count: usize) -> Vec<&'a TestEvaluation> {
            evaluations.iter().take(count).collect()
        }
    }

    #[test]
    fn test_trait_compilation() {
        let config = EvolutionConfig::default();
        assert_eq!(config.population_size, 100);

        let mut genome = TestGenome { value: 0.0 };
        let evaluator = DummyEvaluator;
        let selection = DummySelection;

        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);
        let mutation = DummyMutation;
        mutation.mutate(&mut genome, &mut rng);
        assert_eq!(genome.value, 1.0);

        let crossover = DummyCrossover;
        let parent_b = TestGenome { value: 1.0 };
        let (child1, child2) = crossover.crossover(&genome, &parent_b, &mut rng);
        assert_eq!(child1.value, 1.0);
        assert_eq!(child2.value, 1.0);

        let pop = vec![evaluator.evaluate(&genome)];
        let selected = selection.select(&pop, 1);
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].fitness(), 1.0);
    }

    #[derive(Clone, Debug, PartialEq)]
    struct BitGenome {
        bits: Vec<bool>,
    }

    impl Genome for BitGenome {}

    #[derive(Clone, Debug, PartialEq)]
    struct BitEvaluation { fitness: f64, valid: bool, genome: BitGenome }
    impl Evaluated for BitEvaluation {
        type Genome = BitGenome;
        fn fitness(&self) -> f64 { self.fitness }
        fn is_valid(&self) -> bool { self.valid }
        fn genome(&self) -> &Self::Genome { &self.genome }
    }

    struct BitEvaluator;
    impl FitnessEvaluator<BitGenome> for BitEvaluator {
        type Evaluation = BitEvaluation;
        fn evaluate(&self, genome: &BitGenome) -> Self::Evaluation {
            let count = genome.bits.iter().filter(|&&b| b).count() as f64;
            BitEvaluation { fitness: count, valid: true, genome: genome.clone() }
        }
    }

    #[test]
    fn test_state_compilation_with_bitgenome() {
        let genome = BitGenome { bits: vec![true, false, true] };
        let org = Organism { genome: genome.clone() };
        
        let pop = Population { organisms: vec![org.clone()] };
        let archive = EliteArchive { elites: vec![org.clone()] };
        
        let mut state = EvolutionState::default();
        state.generation += 1;
        state.evaluations += 1;

        assert_eq!(state.generation, 1);
        assert_eq!(pop.organisms.len(), 1);
        assert_eq!(archive.elites.len(), 1);
    }
}
