use crate::traits::{Genome, GenomeFactory, Evaluated, FitnessEvaluator, MutationOperator, CrossoverOperator};
use crate::config::EvolutionConfig;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct GaResult<E: Evaluated> {
    pub global_best: E,
    pub generation_history: Vec<E>,
    pub run_id: String,
    pub timestamp: i64,
    pub top_10: Vec<E>,
}

pub struct EvolutionEngine<
    G: Genome, 
    F: FitnessEvaluator<G>, 
    M: MutationOperator<G>, 
    C: CrossoverOperator<G>,
    Factory: GenomeFactory<G>
> {
    pub evaluator: F,
    pub mutator: M,
    pub crossover: C,
    pub factory: Factory,
    _marker: std::marker::PhantomData<G>,
}

impl<
    G: Genome, 
    F: FitnessEvaluator<G>, 
    M: MutationOperator<G>, 
    C: CrossoverOperator<G>,
    Factory: GenomeFactory<G>
> EvolutionEngine<G, F, M, C, Factory> {
    pub fn new(evaluator: F, mutator: M, crossover: C, factory: Factory) -> Self {
        Self {
            evaluator,
            mutator,
            crossover,
            factory,
            _marker: std::marker::PhantomData,
        }
    }

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

    pub fn run_ga_evolution(&self, config: EvolutionConfig) -> GaResult<F::Evaluation> {
        let mut rng = StdRng::seed_from_u64(config.seed.unwrap_or(0));

        let mut population = self.initialize_population(&config, &mut rng);
        let mut global_best: Option<F::Evaluation> = None;
        let mut history = Vec::new();

        for _gen in 0..config.generation_limit {
            let mut evals: Vec<F::Evaluation> = population
                .iter()
                .map(|c| self.evaluator.evaluate(c))
                .filter(|e| e.is_valid())
                .collect();

            if evals.is_empty() {
                population = self.initialize_population(&config, &mut rng);
                continue;
            }

            evals.sort_by(|a, b| b.fitness().partial_cmp(&a.fitness()).unwrap_or(Ordering::Equal));

            let gen_best = evals[0].clone();
            if global_best.is_none() || gen_best.fitness() > global_best.as_ref().unwrap().fitness() {
                global_best = Some(gen_best.clone());
            }
            history.push(gen_best.clone());

            let mut next_gen = Vec::with_capacity(config.population_size);
            next_gen.extend(evals.iter().take(2).map(|e| e.genome().clone()));

            while next_gen.len() < config.population_size {
                let parent1 = self.tournament_selection(&evals, 3, &mut rng);
                let parent2 = self.tournament_selection(&evals, 3, &mut rng);

                let (mut child, _child2) = self.crossover.crossover(parent1.genome(), parent2.genome(), &mut rng);
                self.mutator.mutate(&mut child, &mut rng);
                next_gen.push(child);
            }

            population = next_gen;
        }

        let best = global_best.unwrap_or_else(|| {
            let dummy = self.initialize_population(&config, &mut rng).into_iter().next().unwrap();
            self.evaluator.evaluate(&dummy)
        });

        GaResult {
            global_best: best,
            generation_history: history,
            run_id: "generic-run".to_string(),
            timestamp: 0,
            top_10: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct BitGenome { bits: Vec<bool> }
    impl Genome for BitGenome {}

    struct BitGenomeFactory;
    impl GenomeFactory<BitGenome> for BitGenomeFactory {
        fn create(&self, _rng: &mut StdRng) -> BitGenome {
            BitGenome { bits: vec![true, false] }
        }
    }

    #[derive(Clone, Debug, PartialEq)]
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
        fn mutate(&self, _candidate: &mut BitGenome, _rng: &mut StdRng) {}
    }

    struct DummyCrossover;
    impl CrossoverOperator<BitGenome> for DummyCrossover {
        fn crossover(&self, _parent1: &BitGenome, _parent2: &BitGenome, _rng: &mut StdRng) -> (BitGenome, BitGenome) {
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

        let config = EvolutionConfig { population_size: 10, generation_limit: 2, seed: Some(42), ..Default::default() };
        let result = engine.run_ga_evolution(config);

        assert_eq!(result.generation_history.len(), 2);
        assert!(result.global_best.fitness() > 0.0);
    }
}
