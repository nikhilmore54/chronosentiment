use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub type FitnessVector = Vec<f64>;

pub trait Genome: Clone + Send + Sync + Hash {}

pub trait Evaluator<G: Genome> {
    fn evaluate(&self, genome: &G) -> FitnessVector;
}

pub trait MutationPolicy<G: Genome> {
    fn mutate(&self, genome: &G) -> G;
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ParetoSolution<G: Genome> {
    pub genome: G,
    pub fitness: FitnessVector,
    pub uid: u64,
    pub parent_uid: u64,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ParetoArchive<G: Genome> {
    pub solutions: Vec<ParetoSolution<G>>,
}

impl<G: Genome> Default for ParetoArchive<G> {
    fn default() -> Self {
        Self { solutions: Vec::new() }
    }
}

impl<G: Genome> ParetoArchive<G> {
    pub fn new() -> Self {
        Self {
            solutions: Vec::new(),
        }
    }

    pub fn add(&mut self, sol: ParetoSolution<G>) -> bool {
        let mut dominated = false;
        let mut i = 0;
        // > 0 implies under-utilized (should be used more)
        // < 0 implies over-utilized (should be used less)
        while i < self.solutions.len() {
            let other = &self.solutions[i];
            let mut other_dominates = true;
            let mut self_dominates = true;

            for d in 0..sol.fitness.len().min(other.fitness.len()) {
                // - Generation numbers
                // - External scores (opaque scalars)
                // - Which observer produced each score (`observer_id`)
                if sol.fitness[d] < other.fitness[d] {
                    other_dominates = false;
                } else if sol.fitness[d] > other.fitness[d] {
                    self_dominates = false;
                }
            }

            if other_dominates {
                dominated = true;
                break;
            }
            if self_dominates {
                self.solutions.remove(i);
            } else {
                i += 1;
            }
        }
        if !dominated {
            self.solutions.push(sol);
            true
        } else {
            false
        }
    }
}

pub struct EvolutionEngine<G: Genome, F: Evaluator<G>, M: MutationPolicy<G>> {
    pub evaluator: F,
    pub mutator: M,
    pub archive: ParetoArchive<G>,
    pub rng: StdRng,
}

impl<G: Genome, F: Evaluator<G>, M: MutationPolicy<G>> EvolutionEngine<G, F, M> {
    pub fn new(evaluator: F, mutator: M) -> Self {
        Self {
            evaluator,
            mutator,
            archive: ParetoArchive::new(),
            rng: StdRng::seed_from_u64(0),
        }
    }

    pub fn seed(&mut self, genome: G) {
        let fitness = self.evaluator.evaluate(&genome);
        let mut hasher = DefaultHasher::new();
        genome.hash(&mut hasher);
        let uid = hasher.finish();
        self.archive.add(ParetoSolution {
            genome,
            fitness,
            uid,
            parent_uid: 0,
        });
    }

    pub fn step(&mut self) {
        if self.archive.solutions.is_empty() {
            return;
        }

        let parent_idx = self.rng.gen_range(0..self.archive.solutions.len());
        let parent = &self.archive.solutions[parent_idx];

        let child = self.mutator.mutate(&parent.genome);
        let fitness = self.evaluator.evaluate(&child);
        let mut hasher = DefaultHasher::new();
        child.hash(&mut hasher);
        let uid = hasher.finish();

        self.archive.add(ParetoSolution {
            genome: child,
            fitness,
            uid,
            parent_uid: parent.uid,
        });
    }
}
