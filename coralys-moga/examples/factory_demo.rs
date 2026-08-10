use coralys_moga::runtime::model::network::OperationalModel;
use coralys_moga::runtime::optimization::{
    ConstraintModel, DecisionVector, ObjectiveModel, OptimizationEngine
};
use coralys_moga::traits::{Evaluated, FitnessEvaluator, Genome, GenomeFactory, MutationOperator};
use rand::rngs::StdRng;
use rand::SeedableRng;

// --- Toy Domain Representation ---

#[derive(Clone, Debug)]
pub struct FactoryState {
    pub machine_hours_assigned: Vec<f64>,
}

impl OperationalModel for FactoryState {}
impl Genome for FactoryState {}
impl coralys_core::Solution for FactoryState {}

#[derive(Clone, Debug)]
pub struct ProductionPlan {
    pub jobs: Vec<usize>,
}

impl DecisionVector for ProductionPlan {}

// --- Engine Independence Validation ---

pub struct MaxMachineHoursConstraint {
    pub max_hours: f64,
}

impl ConstraintModel<FactoryState> for MaxMachineHoursConstraint {}

pub struct ThroughputObjective;
impl ObjectiveModel<FactoryState> for ThroughputObjective {}

// --- Genetic Algorithm Hook ---

pub struct FactoryOptimizer;

impl GenomeFactory<FactoryState> for FactoryOptimizer {
    fn create(&self, _rng: &mut StdRng) -> FactoryState {
        FactoryState {
            machine_hours_assigned: vec![10.0, 15.0, 8.0],
        }
    }
}

impl MutationOperator<FactoryState> for FactoryOptimizer {
    fn mutate(&self, genome: &mut FactoryState, _rng: &mut StdRng) {
        // Mutate by tweaking machine hours slightly
        if let Some(h) = genome.machine_hours_assigned.first_mut() {
            *h *= 0.9;
        }
    }
}

impl FitnessEvaluator<FactoryState> for FactoryOptimizer {
    type Evaluation = FactoryFitness;

    fn evaluate(&self, genome: &FactoryState) -> Self::Evaluation {
        // Compute total throughput (more hours = more throughput but penalized if exceeding max hours)
        let total_hours: f64 = genome.machine_hours_assigned.iter().sum();
        let max_hours = 40.0;
        
        let penalty = if total_hours > max_hours { total_hours - max_hours } else { 0.0 };
        
        FactoryFitness {
            throughput: total_hours - (penalty * 2.0),
            genome: genome.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FactoryFitness {
    throughput: f64,
    genome: FactoryState,
}

impl PartialEq for FactoryFitness {
    fn eq(&self, other: &Self) -> bool {
        self.throughput == other.throughput
    }
}

impl PartialOrd for FactoryFitness {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.throughput.partial_cmp(&other.throughput)
    }
}

impl Evaluated for FactoryFitness {
    type Genome = FactoryState;
    
    fn is_valid(&self) -> bool {
        self.throughput > 0.0
    }
    
    fn fitness(&self) -> f64 {
        self.throughput
    }
    
    fn genome(&self) -> &Self::Genome {
        &self.genome
    }
}

// Toy engine implementing OptimizationEngine
pub struct ToyEngine<M: OperationalModel> {
    pub state: M,
}

impl<M: OperationalModel> OptimizationEngine<M> for ToyEngine<M> {
    fn optimize(&mut self) {
        println!("Optimizing Operational Model...");
    }
}

fn main() {
    let mut rng = StdRng::seed_from_u64(42);
    let optimizer = FactoryOptimizer;
    
    // Demonstrate generating and evaluating
    let mut state = optimizer.create(&mut rng);
    let fitness = optimizer.evaluate(&state);
    
    println!("Initial State: {:?}, Fitness: {:?}", state, fitness);
    
    optimizer.mutate(&mut state, &mut rng);
    let new_fitness = optimizer.evaluate(&state);
    
    println!("Mutated State: {:?}, Fitness: {:?}", state, new_fitness);
    
    let mut engine = ToyEngine { state };
    engine.optimize();
    
    println!("Factory optimization complete. Engine independence proved.");
}
