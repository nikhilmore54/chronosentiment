pub mod benchmark;
pub mod benchmark_framework;
pub mod config;
pub mod ecology;
pub mod engine;
pub mod engine_proof;
pub mod metrics;
pub mod runtime;
pub mod state;
pub mod termination;
pub mod traits;

pub mod observatory;

pub use metrics::evolution::{EvolutionMetrics, ProcessorMetrics};

pub use benchmark_framework::{
    ConvergenceMetrics, EngineMetrics, ExecutionMetrics, MogaBenchmarkReport, SolutionQuality,
};

pub use config::EvolutionConfig;
pub use engine::{
    EvolutionEngineBuilder, MogaOutcomeWrapper, MogaReasoningEngine, PluginFitnessEvaluator,
};
pub use observatory::{
    GenerationObserver, PipelineObserver, ProcessingEvent, ProcessingMetricsCollector,
};
pub use state::{EliteArchive, EvolutionState, GenerationResult, Organism, Population};
pub use termination::{TerminationPolicy, TerminationState};
pub use traits::{
    AssignmentSolver, CrossoverOperator, Evaluated, FitnessEvaluator, Genome, GenomeFactory,
    ImprovementOperator, LocalSearchOperator, MutationOperator, NoOpImprovement,
    ObservedTransitionMetric, RegionIdentifier, SelectionStrategy,
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
    struct TestEvaluation {
        fitness: f64,
        valid: bool,
        genome: TestGenome,
    }
    impl Evaluated for TestEvaluation {
        type Genome = TestGenome;
        fn fitness(&self) -> f64 {
            self.fitness
        }
        fn is_valid(&self) -> bool {
            self.valid
        }
        fn genome(&self) -> &Self::Genome {
            &self.genome
        }
    }

    struct DummyEvaluator;
    impl FitnessEvaluator<TestGenome> for DummyEvaluator {
        type Evaluation = TestEvaluation;
        fn evaluate(
            &self,
            genome: &TestGenome,
            _metrics: &crate::runtime::optimization::metric::MetricReport,
        ) -> Self::Evaluation {
            TestEvaluation {
                fitness: genome.value,
                valid: true,
                genome: genome.clone(),
            }
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
        fn crossover(
            &self,
            parent_a: &TestGenome,
            parent_b: &TestGenome,
            _rng: &mut rand::rngs::StdRng,
        ) -> (TestGenome, TestGenome) {
            let avg = (parent_a.value + parent_b.value) / 2.0;
            (TestGenome { value: avg }, TestGenome { value: avg })
        }
    }

    struct DummySelection;
    impl SelectionStrategy<TestEvaluation> for DummySelection {
        fn select<'a>(
            &self,
            evaluations: &'a [TestEvaluation],
            count: usize,
        ) -> Vec<&'a TestEvaluation> {
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

        let pop = vec![evaluator.evaluate(
            &genome,
            &crate::runtime::optimization::metric::MetricReport::default(),
        )];
        let selected = selection.select(&pop, 1);
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].fitness(), 1.0);
    }

    #[derive(Clone, Debug, PartialEq)]
    struct BitGenome {
        bits: Vec<bool>,
    }

    impl Genome for BitGenome {}

    #[test]
    fn test_state_compilation_with_bitgenome() {
        let genome = BitGenome {
            bits: vec![true, false, true],
        };
        let org = Organism {
            genome: genome.clone(),
        };

        let pop = Population {
            organisms: vec![org.clone()],
        };
        let archive = EliteArchive {
            elites: vec![org.clone()],
        };

        let mut state = EvolutionState::default();
        state.generation += 1;
        state.evaluations += 1;

        assert_eq!(state.generation, 1);
        assert_eq!(pop.organisms.len(), 1);
        assert_eq!(archive.elites.len(), 1);
    }

    struct DummyGenomeFactory;
    impl GenomeFactory<TestGenome> for DummyGenomeFactory {
        fn create(&self, _rng: &mut rand::rngs::StdRng) -> TestGenome {
            TestGenome { value: 0.0 }
        }
    }

    struct IncrementImprovement;
    impl ImprovementOperator<TestGenome> for IncrementImprovement {
        fn improve(&self, genome: &mut TestGenome) {
            genome.value += 10.0;
        }
    }

    #[test]
    fn test_engine_builder_default_pipeline() {
        let builder = EvolutionEngineBuilder::new()
            .with_evaluator(DummyEvaluator)
            .with_mutator(DummyMutation)
            .with_crossover(DummyCrossover)
            .with_factory(DummyGenomeFactory);

        let engine = builder.build().unwrap();
        let config = EvolutionConfig {
            population_size: 10,
            elite_count: 2,
            generation_limit: 2,
            ..Default::default()
        };
        let result = engine.run_ga_evolution(config).unwrap();
        // Since NoOpImprovement is default, values shouldn't increment by 10.0
        assert!(result.global_best.genome.value < 10.0);
    }

    #[test]
    fn test_engine_builder_with_improvement() {
        let builder = EvolutionEngineBuilder::new()
            .with_evaluator(DummyEvaluator)
            .with_mutator(DummyMutation)
            .with_crossover(DummyCrossover)
            .with_factory(DummyGenomeFactory)
            .with_improvement(IncrementImprovement);

        let engine = builder.build().unwrap();
        let config = EvolutionConfig {
            population_size: 10,
            elite_count: 2,
            generation_limit: 2,
            ..Default::default()
        };
        let result = engine.run_ga_evolution(config).unwrap();
        // With IncrementImprovement configured, offspring should be improved (value += 10.0 per generation/offspring)
        assert!(result.global_best.genome.value >= 10.0);
    }

    struct DoubleImprovement;
    impl ImprovementOperator<TestGenome> for DoubleImprovement {
        fn improve(&self, genome: &mut TestGenome) {
            genome.value *= 2.0;
        }
    }

    #[test]
    fn test_engine_builder_multiple_processors() {
        let builder = EvolutionEngineBuilder::new()
            .with_evaluator(DummyEvaluator)
            .with_mutator(DummyMutation)
            .with_crossover(DummyCrossover)
            .with_factory(DummyGenomeFactory)
            .add_processor(IncrementImprovement) // adds 10.0
            .add_processor(DoubleImprovement); // multiplies by 2.0 (order matters!)

        let mut engine = builder.build().unwrap();
        assert_eq!(engine.processor_count(), 2);

        // Verify clear works
        engine.clear_processors();
        assert_eq!(engine.processor_count(), 0);

        // Re-add to run and verify
        engine.add_processor(Box::new(IncrementImprovement));
        engine.add_processor(Box::new(DoubleImprovement));
        assert_eq!(engine.processor_count(), 2);

        let config = EvolutionConfig {
            population_size: 10,
            elite_count: 2,
            generation_limit: 2,
            ..Default::default()
        };
        let result = engine.run_ga_evolution(config).unwrap();
        // Since (value + 10.0) * 2.0 is run for offspring, the value should be >= 20.0
        assert!(result.global_best.genome.value >= 20.0);
    }

    #[test]
    fn test_engine_builder_observability() {
        let metrics = std::sync::Arc::new(ProcessingMetricsCollector::new());
        let builder = EvolutionEngineBuilder::new()
            .with_evaluator(DummyEvaluator)
            .with_mutator(DummyMutation)
            .with_crossover(DummyCrossover)
            .with_factory(DummyGenomeFactory)
            .add_processor(IncrementImprovement)
            .add_processor(DoubleImprovement)
            .with_observer(metrics.clone());

        let engine = builder.build().unwrap();
        let config = EvolutionConfig {
            population_size: 10,
            elite_count: 2,
            generation_limit: 2,
            ..Default::default()
        };
        let _result = engine.run_ga_evolution(config).unwrap();

        // Ensure metrics are collected
        let count = *metrics.processed_count.lock().unwrap();
        assert!(count > 0, "processed count should be greater than zero");

        {
            let counts = metrics.execution_counts.lock().unwrap();
            assert_eq!(*counts.get(&0).unwrap_or(&0), count / 2);
            assert_eq!(*counts.get(&1).unwrap_or(&0), count / 2);
        }

        // average_time check (should execute and return a valid duration)
        let avg_time_0 = metrics.average_time(0);
        let avg_time_1 = metrics.average_time(1);
        assert!(avg_time_0 >= std::time::Duration::ZERO);
        assert!(avg_time_1 >= std::time::Duration::ZERO);
    }

    #[test]
    fn test_engine_builder_evolution_metrics() {
        let builder = EvolutionEngineBuilder::new()
            .with_evaluator(DummyEvaluator)
            .with_mutator(DummyMutation)
            .with_crossover(DummyCrossover)
            .with_factory(DummyGenomeFactory)
            .add_processor(IncrementImprovement)
            .add_processor(DoubleImprovement)
            .enable_metrics(true);

        let engine = builder.build().unwrap();
        let config = EvolutionConfig {
            population_size: 10,
            elite_count: 2,
            generation_limit: 5,
            ..Default::default()
        };
        let _result = engine.run_ga_evolution(config).unwrap();

        // Assert metrics accumulation on the snapshot
        let m = engine
            .metrics_snapshot()
            .expect("metrics should be enabled");
        assert_eq!(m.generation, 4); // generation limit is 5, so 0 to 4
        assert!(m.best_fitness >= 20.0);
        assert!(m.worst_fitness > 0.0);
        assert!(m.average_fitness > 0.0);
        assert!(m.fitness_stddev >= 0.0);
        assert!(m.evaluation_count > 0);
        assert!(m.elapsed_time > std::time::Duration::ZERO);

        assert_eq!(m.best_history.len(), 5);
        assert_eq!(m.average_history.len(), 5);

        // Assert processor metrics
        assert!(m.processors.contains_key(&0));
        assert!(m.processors.contains_key(&1));
        let p0 = m.processors.get(&0).unwrap();
        let p1 = m.processors.get(&1).unwrap();
        assert!(p0.invocation_count > 0);
        assert!(p1.invocation_count > 0);
        assert!(p0.candidates_processed > 0);
        assert!(p0.total_runtime > std::time::Duration::ZERO);
        assert!(p0.average_runtime > std::time::Duration::ZERO);
        assert!(p0.maximum_runtime > std::time::Duration::ZERO);
        assert!(p0.minimum_runtime <= p0.maximum_runtime);

        // Verify disabled metrics returns None
        let disabled_builder = EvolutionEngineBuilder::new()
            .with_evaluator(DummyEvaluator)
            .with_mutator(DummyMutation)
            .with_crossover(DummyCrossover)
            .with_factory(DummyGenomeFactory);
        let disabled_engine = disabled_builder.build().unwrap();
        assert!(disabled_engine.metrics_snapshot().is_none());
    }
}
