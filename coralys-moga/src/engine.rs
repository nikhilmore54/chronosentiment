use crate::config::EvolutionConfig;
use crate::traits::{
    CrossoverOperator, Evaluated, FitnessEvaluator, Genome, GenomeFactory, MutationOperator,
    ImprovementOperator,
};
use crate::observatory::{PipelineObserver, ProcessingEvent};
use crate::metrics::evolution::{EvolutionMetrics, ProcessorMetrics};
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct GaResult<E: Evaluated> {
    pub global_best: E,
    pub generation_history: Vec<E>,
    pub average_fitness_history: Vec<f64>,
    pub final_fitnesses: Vec<f64>,
    pub run_id: String,
    pub timestamp: i64,
    pub top_10: Vec<E>,
}

pub struct EvolutionEngine<
    G: Genome,
    F: FitnessEvaluator<G>,
    M: MutationOperator<G>,
    C: CrossoverOperator<G>,
    Factory: GenomeFactory<G>,
> {
    pub evaluator: F,
    pub mutator: M,
    pub crossover: C,
    pub factory: Factory,
    pub observer: Option<std::sync::Arc<dyn PipelineObserver<G>>>,
    metrics: Option<std::sync::Mutex<EvolutionMetrics>>,
    processors: Vec<Box<dyn ImprovementOperator<G>>>,
    _marker: std::marker::PhantomData<G>,
}

impl<
    G: Genome,
    F: FitnessEvaluator<G>,
    M: MutationOperator<G>,
    C: CrossoverOperator<G>,
    Factory: GenomeFactory<G>,
> EvolutionEngine<G, F, M, C, Factory>
{
    pub fn new(evaluator: F, mutator: M, crossover: C, factory: Factory) -> Self {
        Self {
            evaluator,
            mutator,
            crossover,
            factory,
            observer: None,
            metrics: None,
            processors: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn metrics_snapshot(&self) -> Option<EvolutionMetrics> {
        self.metrics.as_ref().map(|m| m.lock().unwrap().clone())
    }

    pub fn add_processor(&mut self, processor: Box<dyn ImprovementOperator<G>>) {
        self.processors.push(processor);
    }

    pub fn clear_processors(&mut self) {
        self.processors.clear();
    }

    pub fn processor_count(&self) -> usize {
        self.processors.len()
    }
}

pub struct EvolutionEngineBuilder<G, F, M, C, Factory>
where
    G: Genome,
    F: FitnessEvaluator<G>,
    M: MutationOperator<G>,
    C: CrossoverOperator<G>,
    Factory: GenomeFactory<G>,
{
    evaluator: Option<F>,
    mutator: Option<M>,
    crossover: Option<C>,
    factory: Option<Factory>,
    processors: Vec<Box<dyn ImprovementOperator<G>>>,
    observer: Option<std::sync::Arc<dyn PipelineObserver<G>>>,
    metrics_enabled: bool,
    _marker: std::marker::PhantomData<G>,
}

impl<G, F, M, C, Factory> EvolutionEngineBuilder<G, F, M, C, Factory>
where
    G: Genome,
    F: FitnessEvaluator<G>,
    M: MutationOperator<G>,
    C: CrossoverOperator<G>,
    Factory: GenomeFactory<G>,
{
    pub fn new() -> Self {
        Self {
            evaluator: None,
            mutator: None,
            crossover: None,
            factory: None,
            processors: Vec::new(),
            observer: None,
            metrics_enabled: false,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn with_evaluator(mut self, evaluator: F) -> Self {
        self.evaluator = Some(evaluator);
        self
    }

    pub fn with_mutator(mut self, mutator: M) -> Self {
        self.mutator = Some(mutator);
        self
    }

    pub fn with_crossover(mut self, crossover: C) -> Self {
        self.crossover = Some(crossover);
        self
    }

    pub fn with_factory(mut self, factory: Factory) -> Self {
        self.factory = Some(factory);
        self
    }

    pub fn with_observer(mut self, observer: std::sync::Arc<dyn PipelineObserver<G>>) -> Self {
        self.observer = Some(observer);
        self
    }

    pub fn enable_metrics(mut self) -> Self {
        self.metrics_enabled = true;
        self
    }

    pub fn with_improvement<I>(mut self, improvement: I) -> Self
    where
        I: ImprovementOperator<G> + 'static,
    {
        self.processors.clear();
        self.processors.push(Box::new(improvement));
        self
    }

    pub fn add_processor<I>(mut self, processor: I) -> Self
    where
        I: ImprovementOperator<G> + 'static,
    {
        self.processors.push(Box::new(processor));
        self
    }

    pub fn add_processors(mut self, mut processors: Vec<Box<dyn ImprovementOperator<G>>>) -> Self {
        self.processors.append(&mut processors);
        self
    }

    pub fn build(self) -> Result<EvolutionEngine<G, F, M, C, Factory>, String> {
        let evaluator = self.evaluator.ok_or_else(|| "evaluator is required".to_string())?;
        let mutator = self.mutator.ok_or_else(|| "mutator is required".to_string())?;
        let crossover = self.crossover.ok_or_else(|| "crossover is required".to_string())?;
        let factory = self.factory.ok_or_else(|| "factory is required".to_string())?;
        let metrics = if self.metrics_enabled {
            Some(std::sync::Mutex::new(EvolutionMetrics::default()))
        } else {
            None
        };
        Ok(EvolutionEngine {
            evaluator,
            mutator,
            crossover,
            factory,
            observer: self.observer,
            metrics,
            processors: self.processors,
            _marker: std::marker::PhantomData,
        })
    }
}

impl<
    G: Genome,
    F: FitnessEvaluator<G>,
    M: MutationOperator<G>,
    C: CrossoverOperator<G>,
    Factory: GenomeFactory<G>,
> EvolutionEngine<G, F, M, C, Factory>
{

    pub fn initialize_population(&self, config: &EvolutionConfig, rng: &mut StdRng) -> Vec<G> {
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

    pub fn run_ga_evolution(
        &self,
        config: EvolutionConfig,
    ) -> Result<GaResult<F::Evaluation>, String> {
        // Initialise RNG: deterministic when a seed is supplied, otherwise random.
        let mut rng = match config.seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_entropy(),
        };

        let mut population = self.initialize_population(&config, &mut rng);
        // Demo header and timer start
        let start = std::time::Instant::now();
        // Diagnostic header removed for production demo
        // Counters for each generation
        let mut elite_preserved: usize;
        let mut crossover_ops: usize;
        let mut mutation_ops: usize;
        let mut global_best: Option<F::Evaluation> = None;
        let mut history = Vec::new();
        let mut average_history = Vec::new();
        let mut final_fitnesses = Vec::new();
        let mut stagnation_counter = 0;
        let mut total_evaluations = 0;

        // Configuration validation removed for now.
        let mut _gen = 0;
        let mut stddev = 0.0;
        let policy = config.termination_policy.clone().unwrap_or(crate::termination::TerminationPolicy::FixedGenerations(config.generation_limit));

        loop {
            // Check termination policy
            let elapsed = start.elapsed();
            let avg_fitness = if history.is_empty() { 0.0 } else { average_history.last().copied().unwrap_or(0.0) };
            let term_state = crate::termination::TerminationState {
                generation: _gen,
                elapsed_time: elapsed,
                best_fitness: global_best.as_ref().map(|g| g.fitness()).unwrap_or(0.0),
                average_fitness: avg_fitness,
                fitness_stddev: stddev,
                stagnation_generations: stagnation_counter,
            };
            if policy.should_terminate(&term_state) {
                break;
            }

            // Instrument only for generation 98
            let instrument = _gen == 98;
            // Capture previous global best for logging later
            let prev_global_best = global_best.clone();
            let mut evals: Vec<F::Evaluation> = population
                .iter()
                .map(|c| self.evaluator.evaluate(c))
                .filter(|e| e.is_valid())
                .collect::<Vec<_>>();
            if instrument {
                // After evaluation, before sorting
                // Evaluation fitness diagnostics removed
            }

            if evals.is_empty() {
                population = self.initialize_population(&config, &mut rng);
                _gen += 1;
                continue;
            }

            evals.sort_by(|a, b| {
                b.fitness()
                    .partial_cmp(&a.fitness())
                    .unwrap_or(Ordering::Equal)
            });
            if instrument {
                // After sorting — show distance (100000 - fitness) for readability
                let fit_to_dist = |f: f64| 100000.0 - f;
                let first = evals.first().map(|e| fit_to_dist(e.fitness())).unwrap_or(f64::NAN);
                let second = evals.get(1).map(|e| fit_to_dist(e.fitness())).unwrap_or(f64::NAN);
                let last = evals.last().map(|e| fit_to_dist(e.fitness())).unwrap_or(f64::NAN);
                eprintln!("[INSTR] Sorted distances: best={:.1}, second={:.1}, worst={:.1}", first, second, last);
            }
            // Top fitness after sort log removed
            // Bottom fitness after sort log removed

            // Elite fitness log removed

            // Global best diagnostics removed; update logic retained
            let gen_best = evals[0].clone();
            let is_improvement = if let Some(ref prev) = global_best {
                gen_best.fitness() > prev.fitness()
            } else {
                true
            };

            if is_improvement {
                stagnation_counter = 0;
            } else {
                stagnation_counter += 1;
            }

            if instrument {
                let fit_to_dist = |f: f64| 100000.0 - f;
                let new_global = if global_best.is_none() || gen_best.fitness() > global_best.as_ref().unwrap().fitness() {
                    Some(gen_best.clone())
                } else { global_best.clone() };
                let prev_dist = prev_global_best.as_ref().map(|g| fit_to_dist(g.fitness())).unwrap_or(f64::NAN);
                let pop_best_dist = fit_to_dist(evals[0].fitness());
                let new_dist = new_global.as_ref().map(|g| fit_to_dist(g.fitness())).unwrap_or(f64::NAN);
                eprintln!("[INSTR] Prev Global Best dist: {:.1}", prev_dist);
                eprintln!("[INSTR] Current Population Best dist: {:.1}", pop_best_dist);
                eprintln!("[INSTR] New Global Best dist: {:.1}", new_dist);
                global_best = new_global;
            } else {
                if global_best.is_none() || gen_best.fitness() > global_best.as_ref().unwrap().fitness() {
                    global_best = Some(gen_best.clone());
                }
            }
            let avg_fitness = if evals.is_empty() {
                0.0
            } else {
                evals.iter().map(|e| e.fitness()).sum::<f64>() / evals.len() as f64
            };

            let variance = if evals.is_empty() {
                0.0
            } else {
                evals.iter()
                    .map(|e| {
                        let diff = e.fitness() - avg_fitness;
                        diff * diff
                    })
                    .sum::<f64>() / evals.len() as f64
            };
            stddev = variance.sqrt();

            total_evaluations += evals.len();
            let initial_best = history.first().map(|e: &F::Evaluation| e.fitness()).unwrap_or(gen_best.fitness());
            let convergence_rate = gen_best.fitness() - initial_best;
            let elapsed = start.elapsed();

            if let Some(ref m_lock) = self.metrics {
                let mut m = m_lock.lock().unwrap();
                m.generation = _gen;
                m.best_fitness = gen_best.fitness();
                m.average_fitness = avg_fitness;
                m.worst_fitness = evals.last().map(|e| e.fitness()).unwrap_or(0.0);
                m.fitness_stddev = stddev;
                m.convergence_rate = convergence_rate;
                m.stagnation_generations = stagnation_counter;
                m.evaluation_count = total_evaluations;
                m.elapsed_time = elapsed;
                
                m.best_history.push(gen_best.fitness());
                m.average_history.push(avg_fitness);
            }

            average_history.push(avg_fitness);
            final_fitnesses = evals.iter().map(|e| e.fitness()).collect();
            history.push(gen_best.clone());

            let mut next_gen = Vec::with_capacity(config.population_size);
            // Preserve the top elite individuals as defined by config.elite_count.
            elite_preserved = config.elite_count;
            if instrument {
                let fit_to_dist = |f: f64| 100000.0 - f;
                let elite_slice = &evals[0..config.elite_count];
                let elite_best_dist = elite_slice.first().map(|e| fit_to_dist(e.fitness())).unwrap_or(f64::NAN);
                let elite_worst_dist = elite_slice.last().map(|e| fit_to_dist(e.fitness())).unwrap_or(f64::NAN);
                let pop_best_dist = fit_to_dist(evals[0].fitness());
                let pop_worst_dist = fit_to_dist(evals.last().map(|e| e.fitness()).unwrap_or(f64::NAN));
                // Count unique distances in elite to detect homogeneity
                let unique_elite = elite_slice.iter()
                    .map(|e| (e.fitness() * 10000.0).round() as i64)
                    .collect::<std::collections::HashSet<_>>().len();
                eprintln!("[INSTR] Elite count: {} | unique: {} | best dist: {:.1} | worst dist: {:.1}",
                    config.elite_count, unique_elite, elite_best_dist, elite_worst_dist);
                eprintln!("[INSTR] Population best dist: {:.1} | worst dist: {:.1}",
                    pop_best_dist, pop_worst_dist);
            }
            let mut unique_elites = Vec::with_capacity(config.elite_count);
            let mut seen_fitness = std::collections::HashSet::new();
            for e in &evals {
                let key = (e.fitness() * 10000.0).round() as i64;
                if !seen_fitness.contains(&key) {
                    seen_fitness.insert(key);
                    unique_elites.push(e.genome().clone());
                    if unique_elites.len() >= config.elite_count {
                        break;
                    }
                }
            }
            if unique_elites.len() < config.elite_count {
                for e in &evals {
                    unique_elites.push(e.genome().clone());
                    if unique_elites.len() >= config.elite_count {
                        break;
                    }
                }
            }
            next_gen.extend(unique_elites);

            let t_size = config.tournament_size.unwrap_or(3);
            crossover_ops = 0;
            mutation_ops = 0;
            while next_gen.len() < config.population_size {
                let parent1 = self.tournament_selection(&evals, t_size, &mut rng);
                let mut parent2 = self.tournament_selection(&evals, t_size, &mut rng);

                let key1 = (parent1.fitness() * 10000.0).round() as i64;
                let mut attempts = 0;
                while (parent2.fitness() * 10000.0).round() as i64 == key1 && attempts < 5 {
                    parent2 = self.tournament_selection(&evals, t_size, &mut rng);
                    attempts += 1;
                }

                // Apply crossover with probability config.crossover_rate.
                let mut crossover_applied = false;
                let (mut child, _child2) = if rng.r#gen::<f64>() < config.crossover_rate {
                    crossover_applied = true;
                    crossover_ops += 1;
                    self.crossover
                        .crossover(parent1.genome(), parent2.genome(), &mut rng)
                } else {
                    // No crossover: clone one parent.
                    (parent1.genome().clone(), parent2.genome().clone())
                };
                // Apply mutation with probability config.mutation_rate.
                let mut mutation_applied = false;
                if rng.r#gen::<f64>() < config.mutation_rate {
                    mutation_applied = true;
                    mutation_ops += 1;
                    self.mutator.mutate(&mut child, &mut rng);
                }
                
                // If neither crossover nor mutation was applied, the child is an exact clone of the parent.
                // Force a mutation to maintain population diversity.
                if !crossover_applied && !mutation_applied {
                    self.mutator.mutate(&mut child, &mut rng);
                }
                for (idx, processor) in self.processors.iter().enumerate() {
                    let start_time = std::time::Instant::now();
                    processor.improve(&mut child);
                    let duration = start_time.elapsed();

                    if let Some(ref m_lock) = self.metrics {
                        let mut m = m_lock.lock().unwrap();
                        let proc_m = m.processors.entry(idx).or_insert_with(|| ProcessorMetrics {
                            processor_name: format!("Processor {}", idx),
                            invocation_count: 0,
                            total_runtime: std::time::Duration::ZERO,
                            average_runtime: std::time::Duration::ZERO,
                            maximum_runtime: std::time::Duration::ZERO,
                            minimum_runtime: std::time::Duration::MAX,
                            candidates_processed: 0,
                        });
                        proc_m.invocation_count += 1;
                        proc_m.candidates_processed += 1;
                        proc_m.total_runtime += duration;
                        proc_m.average_runtime = proc_m.total_runtime / proc_m.invocation_count as u32;
                        if duration > proc_m.maximum_runtime {
                            proc_m.maximum_runtime = duration;
                        }
                        if duration < proc_m.minimum_runtime {
                            proc_m.minimum_runtime = duration;
                        }
                    }

                    if let Some(ref observer) = self.observer {
                        let event = ProcessingEvent::new(idx, duration, _gen);
                        observer.on_event(&event);
                    }
                }
                next_gen.push(child);
            }

            population = next_gen;
            if instrument {
                // Next population diagnostics removed
            }
            // Log generation statistics
            // Generation summary log removed
            _gen += 1;
        }

        let best = global_best.unwrap_or_else(|| {
            let dummy = self
                .initialize_population(&config, &mut rng)
                .into_iter()
                .next()
                .unwrap();
            self.evaluator.evaluate(&dummy)
        });

        Ok(GaResult {
            global_best: best,
            generation_history: history,
            average_fitness_history: average_history,
            final_fitnesses,
            run_id: "generic-run".to_string(),
            timestamp: 0,
            top_10: Vec::new(),
        })
    }
}

pub struct MogaOutcomeWrapper<G: Genome> {
    pub result: coralys_core::EvaluationResult,
    pub genome: G,
}

impl<G: Genome> Clone for MogaOutcomeWrapper<G> {
    fn clone(&self) -> Self {
        Self {
            result: self.result.clone(),
            genome: self.genome.clone(),
        }
    }
}

impl<G: Genome> Evaluated for MogaOutcomeWrapper<G> {
    type Genome = G;
    fn fitness(&self) -> f64 {
        self.result.objectives.first().copied().unwrap_or(0.0)
    }
    fn is_valid(&self) -> bool {
        self.result.hard_constraint_violations.is_empty()
    }
    fn genome(&self) -> &Self::Genome {
        &self.genome
    }
}

pub struct PluginFitnessEvaluator<'a, P: coralys_core::DecisionPlugin, G: Genome> {
    pub plugin: &'a P,
    pub state: &'a P::State,
    pub _marker: std::marker::PhantomData<G>,
}

impl<'a, P, G> FitnessEvaluator<G> for PluginFitnessEvaluator<'a, P, G>
where
    G: Genome + serde::Serialize,
    P: coralys_core::DecisionPlugin<Evaluation = coralys_core::EvaluationResult>,
{
    type Evaluation = MogaOutcomeWrapper<G>;

    fn evaluate(&self, genome: &G) -> Self::Evaluation {
        let payload = serde_json::to_value(genome).unwrap();
        let proposal = coralys_core::DecisionProposal {
            priority: 1.0,
            estimated_gain: 0.0,
            affected_resources: vec![],
            violations_resolved: vec![],
            confidence: 1.0,
            payload,
        };

        if let Ok(sim_state) = self.plugin.simulate(self.state, &proposal) {
            let result = self.plugin.evaluate(&sim_state);
            MogaOutcomeWrapper {
                result,
                genome: genome.clone(),
            }
        } else {
            MogaOutcomeWrapper {
                result: coralys_core::EvaluationResult {
                    objectives: vec![0.0],
                    hard_constraint_violations: vec![coralys_core::Violation {
                        constraint_id: "simulation_failed".to_string(),
                        severity: "Hard".to_string(),
                        value: None,
                        expected: "Ok".to_string(),
                        actual: "Error".to_string(),
                        description: "Simulation failed".to_string(),
                        penalty: 1000,
                    }],
                    soft_constraint_violations: vec![],
                    metrics: std::collections::HashMap::new(),
                },
                genome: genome.clone(),
            }
        }
    }
}

pub struct MogaReasoningEngine<G, M, C, Factory, P>
where
    G: Genome,
    M: MutationOperator<G> + Clone,
    C: CrossoverOperator<G> + Clone,
    Factory: GenomeFactory<G> + Clone,
    P: coralys_core::DecisionPlugin,
{
    pub mutator: M,
    pub crossover: C,
    pub factory: Factory,
    _marker: std::marker::PhantomData<(G, P)>,
}

impl<G, M, C, Factory, P> MogaReasoningEngine<G, M, C, Factory, P>
where
    G: Genome,
    M: MutationOperator<G> + Clone,
    C: CrossoverOperator<G> + Clone,
    Factory: GenomeFactory<G> + Clone,
    P: coralys_core::DecisionPlugin,
{
    pub fn new(mutator: M, crossover: C, factory: Factory) -> Self {
        Self {
            mutator,
            crossover,
            factory,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<G, M, C, Factory, P> coralys_core::ReasoningEngine for MogaReasoningEngine<G, M, C, Factory, P>
where
    G: Genome + serde::Serialize + for<'de> serde::Deserialize<'de>,
    M: MutationOperator<G> + Clone,
    C: CrossoverOperator<G> + Clone,
    Factory: GenomeFactory<G> + Clone,
    P: coralys_core::DecisionPlugin<Evaluation = coralys_core::EvaluationResult>,
{
    type Plugin = P;
    type Config = EvolutionConfig;

    fn solve(
        &self,
        plugin: &mut Self::Plugin,
        config: &Self::Config,
    ) -> Result<Vec<coralys_core::DecisionProposal>, String> {
        let state = plugin.current_state();
        let evaluator = PluginFitnessEvaluator {
            plugin,
            state: &state,
            _marker: std::marker::PhantomData,
        };

        let engine = EvolutionEngine::new(
            evaluator,
            self.mutator.clone(),
            self.crossover.clone(),
            self.factory.clone(),
        );

        let ga_result = engine.run_ga_evolution(config.clone())?;

        let best_genome = ga_result.global_best.genome();
        let payload = serde_json::to_value(best_genome)
            .map_err(|e| format!("Failed to serialize best genome: {}", e))?;

        Ok(vec![coralys_core::DecisionProposal {
            priority: 1.0,
            estimated_gain: ga_result.global_best.fitness(),
            affected_resources: vec![],
            violations_resolved: vec![],
            confidence: 1.0,
            payload,
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct BitGenome {
        bits: Vec<bool>,
    }
    impl Genome for BitGenome {}

    struct BitGenomeFactory;
    impl GenomeFactory<BitGenome> for BitGenomeFactory {
        fn create(&self, _rng: &mut StdRng) -> BitGenome {
            BitGenome {
                bits: vec![true, false],
            }
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    struct BitEvaluation {
        fitness: f64,
        valid: bool,
        genome: BitGenome,
    }
    impl Evaluated for BitEvaluation {
        type Genome = BitGenome;
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
    impl FitnessEvaluator<BitGenome> for DummyEvaluator {
        type Evaluation = BitEvaluation;
        fn evaluate(&self, _candidate: &BitGenome) -> Self::Evaluation {
            BitEvaluation {
                fitness: 1.0,
                valid: true,
                genome: _candidate.clone(),
            }
        }
    }

    struct DummyMutator;
    impl MutationOperator<BitGenome> for DummyMutator {
        fn mutate(&self, _candidate: &mut BitGenome, _rng: &mut StdRng) {}
    }

    struct DummyCrossover;
    impl CrossoverOperator<BitGenome> for DummyCrossover {
        fn crossover(
            &self,
            _parent1: &BitGenome,
            _parent2: &BitGenome,
            _rng: &mut StdRng,
        ) -> (BitGenome, BitGenome) {
            (BitGenome { bits: vec![] }, BitGenome { bits: vec![] })
        }
    }

    #[test]
    fn test_evolution_engine_end_to_end() {
        let engine = EvolutionEngine::new(
            DummyEvaluator,
            DummyMutator,
            DummyCrossover,
            BitGenomeFactory,
        );

        let config = EvolutionConfig {
            population_size: 10,
            generation_limit: 2,
            seed: Some(42),
            ..Default::default()
        };
        let ga_result = engine.run_ga_evolution(config).expect("evolution failed");
        assert_eq!(ga_result.generation_history.len(), 2);
        assert!(ga_result.global_best.fitness() > 0.0);
    }
}

