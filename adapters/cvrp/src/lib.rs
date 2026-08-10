use coralys_core::{Solution, Outcome};
use serde::{Deserialize, Serialize};

/// Qualification subsystem — GOV-008 / GOV-009
pub mod qualification;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasinTransition {
    pub parent_routes: Vec<Vec<usize>>,
    pub mutated_routes: Vec<Vec<usize>>,
    pub optimized_routes: Vec<Vec<usize>>,
    pub damage_edge_count: usize,
    pub repair_edge_count: usize,
    pub optimized_distance: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GuidanceMode {
    Control,
    Shadow,
    Soft,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CalibrationBucket {
    pub predicted_count: usize,
    pub elite_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CvrpGenerationState {
    pub generation: usize,
    pub best_distance: f64,
    pub p10_distance: f64,
    pub p25_distance: f64,
    pub median_distance: f64,
    pub p75_distance: f64,
    pub worst_distance: f64,
    pub average_distance: f64,
    pub feasible_population_ratio: f64,
    pub diversity_score: f64,
    pub elite_diversity_score: f64,
    pub best_routes: Vec<Vec<usize>>,
    pub deepest_basin_transition: Option<BasinTransition>,
    pub operator_counts: std::collections::HashMap<String, usize>,
    
    // Memetic Telemetry
    pub mutation_damage_avg: f64,
    pub basin_depth_avg: f64,
    pub new_basin_discovery_rate: f64,
    pub parent_reversion_rate: f64,
    pub damage_edge_count_avg: f64,
    pub repair_edge_count_avg: f64,
    
    // Basin Geometry
    pub unique_basins_seen: usize,
    pub global_basin_revisit_rate: f64,
    pub recent_basin_revisit_rate: f64,
    pub elite_basin_discovery_rate: f64,

    // Telemetry for Governance Diagnosis
    pub generations_since_improvement: usize,
    pub local_improving_generated: usize,
    pub global_improving_generated: usize,
    pub improving_accepted: usize,
    pub improving_rejected: usize,
    pub crossover_structural_damage_ratio: f64,
    pub route_diversity_score: f64,
    pub parent_similarity: f64,
    pub offspring_novelty: f64,
    pub elite_similarity: f64,
    pub top_10_parent_ratio: f64,
    pub top_20_parent_ratio: f64,
    pub bottom_50_parent_ratio: f64,
    pub elite_offspring_survival_rate: f64,
    pub top10_offspring_rate: f64,

    // Shadow Mode Telemetry
    pub guidance_mode: GuidanceMode,
    pub shadow_rejected_offspring: usize,
    pub shadow_local_search_work_saved: usize,
    pub shadow_rejected_elites: usize,
    pub mean_p_elite: f64,
    pub calibration_error: f64,
    pub calibration_buckets: std::collections::HashMap<String, CalibrationBucket>,
    
    pub innovation_telemetry: Option<coralys_core::memory::InnovationTelemetry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Node {
    pub id: usize,
    pub x: f64,
    pub y: f64,
    pub demand: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistanceMetric {
    TspLibEuc2D,
    EuclideanFloat,
    /// TSPLIB EXPLICIT matrix — distances stored in CvrpInstance.explicit_matrix
    ExplicitMatrix,
}

fn default_distance_metric() -> DistanceMetric {
    DistanceMetric::EuclideanFloat
}

pub trait DistanceCalculator: Send + Sync {
    fn distance(&self, a: &Node, b: &Node) -> f64;
}

#[derive(Debug, Clone, Copy)]
pub struct TspLibEuc2DCalculator;
impl DistanceCalculator for TspLibEuc2DCalculator {
    fn distance(&self, a: &Node, b: &Node) -> f64 {
        ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt().round()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EuclideanFloatCalculator;
impl DistanceCalculator for EuclideanFloatCalculator {
    fn distance(&self, a: &Node, b: &Node) -> f64 {
        ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CvrpInstance {
    pub capacity: i32,
    pub depot: Node,
    pub customers: Vec<Node>,
    #[serde(default = "default_distance_metric")]
    pub distance_metric: DistanceMetric,
    pub max_vehicles: Option<usize>,
    /// Explicit distance matrix for TSPLIB EXPLICIT instances.
    /// Indexed by node id (1-based): matrix[i][j] = distance from node i to node j.
    /// Only populated when distance_metric == ExplicitMatrix.
    #[serde(default)]
    pub explicit_matrix: Vec<Vec<f64>>,
}

impl CvrpInstance {
    pub fn a_n32_k5() -> Self {
        let coords = vec![
            (1, 82.0, 76.0, 0),    // Depot
            (2, 96.0, 44.0, 19),
            (3, 50.0, 5.0, 21),
            (4, 49.0, 8.0, 6),
            (5, 13.0, 7.0, 19),
            (6, 29.0, 89.0, 7),
            (7, 58.0, 30.0, 12),
            (8, 84.0, 39.0, 16),
            (9, 14.0, 24.0, 6),
            (10, 2.0, 39.0, 16),
            (11, 3.0, 82.0, 8),
            (12, 5.0, 10.0, 14),
            (13, 98.0, 52.0, 21),
            (14, 84.0, 25.0, 16),
            (15, 61.0, 59.0, 3),
            (16, 1.0, 65.0, 22),
            (17, 88.0, 51.0, 18),
            (18, 91.0, 2.0, 19),
            (19, 19.0, 32.0, 1),
            (20, 93.0, 3.0, 24),
            (21, 50.0, 93.0, 8),
            (22, 98.0, 14.0, 12),
            (23, 5.0, 42.0, 4),
            (24, 42.0, 9.0, 8),
            (25, 61.0, 62.0, 24),
            (26, 9.0, 97.0, 24),
            (27, 80.0, 55.0, 2),
            (28, 57.0, 69.0, 20),
            (29, 23.0, 15.0, 15),
            (30, 20.0, 70.0, 2),
            (31, 85.0, 60.0, 14),
            (32, 98.0, 5.0, 9),
        ];

        let mut depot = Node { id: 0, x: 0.0, y: 0.0, demand: 0 };
        let mut customers = Vec::new();

        for (id, x, y, demand) in coords {
            let node = Node { id, x, y, demand };
            if id == 1 {
                depot = node;
            } else {
                customers.push(node);
            }
        }

        Self {
            capacity: 100,
            depot,
            customers,
            distance_metric: DistanceMetric::EuclideanFloat,
            max_vehicles: Some(5),
            explicit_matrix: vec![],
        }
    }

    

    /// CVRPLIB A-n46-k7: 45 customers, 7 vehicles, capacity=100, BKS=914 (TspLibEuc2D)
    /// Source: https://galgos.inf.puc-rio.br/cvrplib/index.php/en/download/instance/18
    /// Verified: total demand=603, max capacity=7×100=700, feasible.
    pub fn a_n46_k7() -> Self {
        // (id, x, y, demand) — node 1 is depot (demand=0)
        let coords: Vec<(usize, f64, f64, i32)> = vec![
            (1,  75.0, 55.0,  0),  // Depot
            (2,   7.0, 75.0, 12),
            (3,  77.0,  1.0, 26),
            (4,  51.0, 25.0,  1),
            (5,  81.0, 25.0, 20),
            (6,  59.0, 37.0,  2),
            (7,  93.0, 45.0, 13),
            (8,  43.0, 21.0, 20),
            (9,  35.0, 53.0,  7),
            (10, 77.0, 63.0, 10),
            (11, 37.0, 13.0, 15),
            (12, 37.0, 51.0,  7),
            (13, 27.0, 31.0, 24),
            (14, 95.0, 31.0, 10),
            (15, 87.0, 43.0, 12),
            (16, 23.0, 65.0, 23),
            (17,  9.0, 51.0, 13),
            (18, 73.0, 81.0, 19),
            (19,  3.0,  1.0,  9),
            (20, 41.0, 61.0, 12),
            (21, 29.0, 81.0,  6),
            (22, 51.0, 95.0,  9),
            (23, 49.0, 25.0, 22),
            (24, 81.0, 53.0, 18),
            (25,  7.0, 51.0, 19),
            (26, 21.0,  5.0, 20),
            (27, 91.0, 35.0, 24),
            (28, 17.0, 81.0, 10),
            (29, 61.0, 69.0,  4),
            (30, 27.0, 97.0, 20),
            (31, 83.0, 23.0, 15),
            (32, 21.0, 93.0, 13),
            (33, 59.0, 31.0, 12),
            (34, 27.0, 53.0,  3),
            (35,  9.0, 91.0,  7),
            (36, 11.0, 27.0, 18),
            (37, 59.0, 41.0,  3),
            (38, 67.0,  1.0, 23),
            (39, 77.0, 39.0,  1),
            (40, 47.0, 29.0, 17),
            (41,  3.0, 89.0, 13),
            (42, 33.0, 87.0,  6),
            (43, 17.0, 45.0, 22),
            (44, 91.0, 41.0, 20),
            (45, 23.0,  3.0, 21),
            (46, 97.0, 61.0,  2),
        ];

        let mut depot = Node { id: 0, x: 0.0, y: 0.0, demand: 0 };
        let mut customers = Vec::new();

        for (id, x, y, demand) in coords {
            let node = Node { id, x, y, demand };
            if id == 1 {
                depot = node;
            } else {
                customers.push(node);
            }
        }

        Self {
            capacity: 100,
            depot,
            customers,
            distance_metric: DistanceMetric::EuclideanFloat,
            max_vehicles: Some(7),
            explicit_matrix: vec![],
        }
    }

    pub fn distance(&self, a: &Node, b: &Node) -> f64 {
        match self.distance_metric {
            DistanceMetric::TspLibEuc2D => TspLibEuc2DCalculator.distance(a, b),
            DistanceMetric::EuclideanFloat => EuclideanFloatCalculator.distance(a, b),
            DistanceMetric::ExplicitMatrix => {
                let i = a.id;
                let j = b.id;
                if i < self.explicit_matrix.len() && j < self.explicit_matrix[i].len() {
                    self.explicit_matrix[i][j]
                } else if j < self.explicit_matrix.len() && i < self.explicit_matrix[j].len() {
                    // Symmetric fallback
                    self.explicit_matrix[j][i]
                } else {
                    0.0
                }
            }
        }
    }

    pub fn evaluate_routes_distance(&self, routes: &Vec<Vec<usize>>, metric: DistanceMetric) -> f64 {
        let mut total = 0.0;
        for route in routes {
            if route.is_empty() { continue; }
            let mut last_node = &self.depot;
            for &node_id in route {
                let customer = if node_id == self.depot.id {
                    &self.depot
                } else {
                    self.customers.iter().find(|c| c.id == node_id).unwrap()
                };
                total += match metric {
                    DistanceMetric::TspLibEuc2D => TspLibEuc2DCalculator.distance(last_node, customer),
                    DistanceMetric::EuclideanFloat => EuclideanFloatCalculator.distance(last_node, customer),
                    DistanceMetric::ExplicitMatrix => self.distance(last_node, customer),
                };
                last_node = customer;
            }
            total += match metric {
                DistanceMetric::TspLibEuc2D => TspLibEuc2DCalculator.distance(last_node, &self.depot),
                DistanceMetric::EuclideanFloat => EuclideanFloatCalculator.distance(last_node, &self.depot),
                DistanceMetric::ExplicitMatrix => self.distance(last_node, &self.depot),
            };
        }
        total
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RadiusPolicy {
    Control,
    LocalBiased,
    ExtremeLocal,
}

/// Genome represents a permutation of customer indices (0 to customers.len()-1)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CvrpCandidate {
    pub permutation: Vec<usize>,
    pub last_mutation_op: Option<String>,
    pub last_mutation_radius: Option<usize>,
    pub route_boundary_changes: Option<usize>,
}

use coralys_moga::traits::{Genome, GenomeFactory};
use rand::seq::SliceRandom;

impl Solution for CvrpCandidate {}
impl Genome for CvrpCandidate {}

pub struct CvrpGenomeFactory {
    pub num_customers: usize,
}

impl GenomeFactory<CvrpCandidate> for CvrpGenomeFactory {
    fn create(&self, rng: &mut rand::rngs::StdRng) -> CvrpCandidate {
        let mut perm: Vec<usize> = (0..self.num_customers).collect();
        perm.shuffle(rng);
        CvrpCandidate { permutation: perm, last_mutation_op: None, last_mutation_radius: None, route_boundary_changes: None }
    }
}

pub struct CvrpClusteredGenomeFactory {
    pub instance: CvrpInstance,
}

impl GenomeFactory<CvrpCandidate> for CvrpClusteredGenomeFactory {
    fn create(&self, rng: &mut rand::rngs::StdRng) -> CvrpCandidate {
        let depot = &self.instance.depot;
        let mut customers_with_angles: Vec<(usize, f64)> = self.instance.customers.iter().enumerate().map(|(idx, cust)| {
            let dx = cust.x - depot.x;
            let dy = cust.y - depot.y;
            let angle = dy.atan2(dx);
            (idx, angle)
        }).collect();

        // Sort by polar angle (sweep order)
        customers_with_angles.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let mut perm: Vec<usize> = customers_with_angles.into_iter().map(|(idx, _)| idx).collect();

        // Add some perturbation for diversity
        if rand::Rng::r#gen::<f64>(rng) < 0.9 {
            let num_swaps = rand::Rng::gen_range(rng, 1..=(self.instance.customers.len() / 5).max(2));
            for _ in 0..num_swaps {
                let i = rand::Rng::gen_range(rng, 0..perm.len());
                let j = rand::Rng::gen_range(rng, 0..perm.len());
                perm.swap(i, j);
            }
        } else {
            perm.shuffle(rng);
        }

        let mut candidate = CvrpCandidate {
            permutation: perm,
            last_mutation_op: None,
            last_mutation_radius: None,
            route_boundary_changes: None,
        };

        // Run feasibility repair on initialization to start GA search from feasible bounds
        let mut repair_framework = coralys_moga::FeasibilityRepairFramework::new(15);
        repair_framework.add_checker(Box::new(moga_impl::CvrpConstraintChecker { instance: self.instance.clone() }));
        repair_framework.add_heuristic(Box::new(moga_impl::VehicleLimitRepairHeuristic { instance: self.instance.clone() }));
        repair_framework.add_heuristic(Box::new(moga_impl::BinPackingRepairHeuristic { instance: self.instance.clone() }));
        repair_framework.add_heuristic(Box::new(moga_impl::SpatialBinPackingRepairHeuristic { instance: self.instance.clone() }));

        use coralys_moga::traits::ImprovementOperator;
        repair_framework.improve(&mut candidate);

        candidate
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct CvrpEvaluation {
    pub candidate: CvrpCandidate,
    pub total_distance: f64,
    pub num_vehicles: usize,
    pub routes: Vec<Vec<usize>>, // For visualization
    pub total_distance_integer: f64,
    pub total_distance_float: f64,
}

impl Outcome for CvrpEvaluation {
    type Sol = CvrpCandidate;
    fn objectives(&self) -> &[f64] {
        // MOGA minimizes, so we negate total_distance.
        // Or if we configure MOGA to maximize, we use -total_distance.
        // MOGA usually maximizes fitness, so fitness = -total_distance.
        // Wait, MOGA `FitnessEvaluator` expects an Outcome, we need to store fitness somewhere.
        // Let's store -total_distance as a 1D vector and return a reference.
        // But the trait signature needs `&[f64]`. We'll manage this in the evaluator wrapper.
        unimplemented!() // Outcome isn't directly compatible this way without a backing array. We'll fix below.
    }
    fn is_valid(&self) -> bool {
        true
    }
    fn solution(&self) -> &Self::Sol {
        &self.candidate
    }
}

// We will implement coralys-moga traits in a separate module.
pub mod moga_impl;
pub mod analysis;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CvrpState {
    pub gen_state: CvrpGenerationState,
    pub best_candidate: Option<CvrpCandidate>,
    pub reference: StateReference,
}

pub struct CvrpDecisionPlugin {
    pub instance: CvrpInstance,
    pub current_state: Mutex<CvrpState>,
}

impl CvrpDecisionPlugin {
    pub fn new(instance: CvrpInstance) -> Self {
        let gen_state = CvrpGenerationState {
            generation: 0,
            best_distance: f64::MAX,
            p10_distance: 0.0,
            p25_distance: 0.0,
            median_distance: 0.0,
            p75_distance: 0.0,
            worst_distance: 0.0,
            average_distance: 0.0,
            feasible_population_ratio: 1.0,
            diversity_score: 0.0,
            elite_diversity_score: 0.0,
            best_routes: Vec::new(),
            deepest_basin_transition: None,
            operator_counts: std::collections::HashMap::new(),
            mutation_damage_avg: 0.0,
            basin_depth_avg: 0.0,
            new_basin_discovery_rate: 0.0,
            parent_reversion_rate: 0.0,
            damage_edge_count_avg: 0.0,
            repair_edge_count_avg: 0.0,
            unique_basins_seen: 0,
            global_basin_revisit_rate: 0.0,
            recent_basin_revisit_rate: 0.0,
            elite_basin_discovery_rate: 0.0,
            generations_since_improvement: 0,
            local_improving_generated: 0,
            global_improving_generated: 0,
            improving_accepted: 0,
            improving_rejected: 0,
            crossover_structural_damage_ratio: 0.0,
            route_diversity_score: 0.0,
            parent_similarity: 0.0,
            offspring_novelty: 0.0,
            elite_similarity: 0.0,
            top_10_parent_ratio: 0.0,
            top_20_parent_ratio: 0.0,
            bottom_50_parent_ratio: 0.0,
            elite_offspring_survival_rate: 0.0,
            top10_offspring_rate: 0.0,
            guidance_mode: GuidanceMode::Control,
            shadow_rejected_offspring: 0,
            shadow_local_search_work_saved: 0,
            shadow_rejected_elites: 0,
            mean_p_elite: 0.0,
            calibration_error: 0.0,
            calibration_buckets: std::collections::HashMap::new(),
            innovation_telemetry: None,
        };

        let reference = StateReference {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            plugin: "cvrp".to_string(),
            metadata: std::collections::HashMap::new(),
        };

        Self {
            instance,
            current_state: Mutex::new(CvrpState {
                gen_state,
                best_candidate: None,
                reference,
            }),
        }
    }
}

use coralys_core::{DecisionPlugin, DecisionProposal, EvaluationResult, Violation, StateReference, SimulationResult};
use uuid::Uuid;
use chrono::Utc;
use std::sync::Mutex;
use coralys_moga::traits::FitnessEvaluator;


impl DecisionPlugin for CvrpDecisionPlugin {
    type State = CvrpState;
    type Evaluation = EvaluationResult;

    fn current_state(&self) -> Self::State {
        self.current_state.lock().unwrap().clone()
    }

    fn evaluate(&self, state: &Self::State) -> Self::Evaluation {
        let candidate = state.best_candidate.clone().unwrap_or_else(|| {
            CvrpCandidate {
                permutation: (0..self.instance.customers.len()).collect(),
                last_mutation_op: None,
                last_mutation_radius: None,
                route_boundary_changes: None,
            }
        });

        let evaluator = moga_impl::CvrpEvaluator { instance: self.instance.clone() };
        let outcome = evaluator.evaluate(&candidate, &coralys_moga::runtime::optimization::metric::MetricReport::default());
        
        translate_evaluation(&candidate, &outcome.eval, &self.instance)
    }

    fn simulate(
        &self,
        state: &Self::State,
        proposal: &DecisionProposal,
    ) -> SimulationResult<Self::State> {
        let candidate: CvrpCandidate = serde_json::from_value(proposal.payload.clone())
            .map_err(|e| format!("Failed to deserialize CVRP candidate: {}", e))?;

        let evaluator = moga_impl::CvrpEvaluator { instance: self.instance.clone() };
        let outcome = evaluator.evaluate(&candidate, &coralys_moga::runtime::optimization::metric::MetricReport::default());

        let mut new_state = state.clone();
        new_state.gen_state.best_distance = outcome.eval.total_distance;
        new_state.gen_state.best_routes = outcome.eval.routes.clone();
        new_state.best_candidate = Some(candidate);

        new_state.reference = StateReference {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            plugin: "cvrp".to_string(),
            metadata: std::collections::HashMap::new(),
        };

        Ok(new_state)
    }

    fn execute(&mut self, proposal: &DecisionProposal) {
        let state = self.current_state();
        if let Ok(new_state) = self.simulate(&state, proposal) {
            *self.current_state.lock().unwrap() = new_state;
        }
    }
}

fn translate_evaluation(
    _candidate: &CvrpCandidate,
    evaluation: &CvrpEvaluation,
    instance: &CvrpInstance,
) -> EvaluationResult {
    let objectives = vec![100000.0 - evaluation.total_distance];

    let mut hard_constraint_violations = Vec::new();
    for (r_idx, route) in evaluation.routes.iter().enumerate() {
        let mut load = 0;
        for &cust_idx in route {
            // customers in CvrpInstance is 0-indexed, but coords starts at customer index 2 (depot is 1)
            // Wait, Node.id is customer.id. Let's find customer node in instance
            if let Some(cust) = instance.customers.iter().find(|c| c.id == cust_idx) {
                load += cust.demand;
            }
        }
        if load > instance.capacity {
            hard_constraint_violations.push(Violation {
                constraint_id: format!("route_{}_capacity", r_idx),
                severity: "Hard".to_string(),
                value: Some(load as f64),
                expected: format!("<= {}", instance.capacity),
                actual: load.to_string(),
                description: format!("Route {} exceeds vehicle capacity", r_idx),
                penalty: (load - instance.capacity) * 100,
            });
        }
    }

    let mut metrics = std::collections::HashMap::new();
    metrics.insert("total_distance".to_string(), evaluation.total_distance);
    metrics.insert("num_vehicles".to_string(), evaluation.num_vehicles as f64);

    EvaluationResult {
        objectives,
        hard_constraint_violations,
        soft_constraint_violations: Vec::new(),
        metrics,
    }
}

pub fn solve_auto_config(
    instance: &CvrpInstance,
    auto_config: bool,
) -> Result<(CvrpEvaluation, String), String> {
    use coralys_core::analysis::InstanceAnalyzer;
    use coralys_moga::{EvolutionConfig, EvolutionEngineBuilder};

    let (config, report) = if auto_config {
        let analyzer = analysis::CvrpInstanceAnalyzer;
        let (features, difficulty, policy) = analyzer.analyze(instance)?;
        let report = analysis::generate_analysis_report(&features, &difficulty, &policy);
        
        let config = EvolutionConfig {
            population_size: policy.population_size,
            elite_count: policy.population_size / 10,
            generation_limit: policy.generation_limit,
            mutation_rate: policy.mutation_rate,
            crossover_rate: policy.crossover_rate,
            seed: Some(42),
            tournament_size: Some(5),
            ..Default::default()
        };
        (config, report)
    } else {
        let config = EvolutionConfig {
            population_size: 100,
            elite_count: 10,
            generation_limit: 30,
            mutation_rate: 0.2,
            crossover_rate: 0.8,
            seed: Some(42),
            tournament_size: Some(5),
            ..Default::default()
        };
        (config, "Manual baseline configuration used.".to_string())
    };

    let evaluator = moga_impl::CvrpEvaluator { instance: instance.clone() };
    let mutator = moga_impl::CvrpMutator::new(instance.clone(), RadiusPolicy::Control);
    let crossover = moga_impl::CvrpCrossoverRoutePreserving { instance: instance.clone() };
    let factory = CvrpClusteredGenomeFactory { instance: instance.clone() };
    let local_search = moga_impl::CvrpLocalSearch { instance: instance.clone() };

    let mut repair_framework = coralys_moga::FeasibilityRepairFramework::new(10);
    repair_framework.add_checker(Box::new(moga_impl::CvrpConstraintChecker { instance: instance.clone() }));
    repair_framework.add_heuristic(Box::new(moga_impl::VehicleLimitRepairHeuristic { instance: instance.clone() }));
    repair_framework.add_heuristic(Box::new(moga_impl::BinPackingRepairHeuristic { instance: instance.clone() }));
    repair_framework.add_heuristic(Box::new(moga_impl::SpatialBinPackingRepairHeuristic { instance: instance.clone() }));

    let engine = EvolutionEngineBuilder::new()
        .with_evaluator(evaluator)
        .with_mutator(mutator)
        .with_crossover(crossover)
        .with_factory(factory)
        .with_improvement(repair_framework)
        .add_processor(local_search)
        .build()
        .map_err(|e| format!("Engine build error: {}", e))?;

    let res = engine.run_ga_evolution(config).map_err(|e| format!("Evolution error: {}", e))?;
    Ok((res.global_best.eval, report))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cvrp_plugin_lifecycle() {
        let mut plugin = CvrpDecisionPlugin::new(CvrpInstance::a_n32_k5());
        
        let initial_state = plugin.current_state();
        assert_eq!(initial_state.reference.plugin, "cvrp");
        
        let initial_eval = plugin.evaluate(&initial_state);
        assert!(initial_eval.objectives[0] > 0.0);
        
        let candidate = CvrpCandidate {
            permutation: (0..31).collect(),
            last_mutation_op: None,
            last_mutation_radius: None,
            route_boundary_changes: None,
        };
        
        let proposal = DecisionProposal {
            priority: 1.0,
            estimated_gain: 100.0,
            affected_resources: vec![],
            violations_resolved: vec![],
            confidence: 1.0,
            payload: serde_json::to_value(&candidate).unwrap(),
        };
        
        let simulated_state = plugin.simulate(&initial_state, &proposal).unwrap();
        assert_ne!(simulated_state.reference.id, initial_state.reference.id);
        assert!(simulated_state.best_candidate.is_some());
        
        plugin.execute(&proposal);
        let current_state = plugin.current_state();
        assert_ne!(current_state.reference.id, initial_state.reference.id);
        
        let final_eval = plugin.evaluate(&current_state);
        assert!(final_eval.objectives[0] > 0.0);
    }

    #[test]
    fn test_cvrp_regression_a_n32_k5() {
        use crate::moga_impl::{CvrpEvaluator, CvrpMutator, CvrpCrossover, CvrpLocalSearch};
        use coralys_moga::{EvolutionConfig, EvolutionEngineBuilder};

        let mut instance = CvrpInstance::a_n32_k5();
        instance.distance_metric = DistanceMetric::TspLibEuc2D;
        
        let evaluator = CvrpEvaluator { instance: instance.clone() };
        let mutator = CvrpMutator::new(instance.clone(), RadiusPolicy::Control);
        let crossover = CvrpCrossover;
        let factory = CvrpGenomeFactory { num_customers: instance.customers.len() };
        let local_search = CvrpLocalSearch { instance: instance.clone() };

        let config = EvolutionConfig {
            population_size: 200,
            elite_count: 20,
            generation_limit: 50,
            mutation_rate: 0.2,
            crossover_rate: 0.8,
            seed: Some(42),
            tournament_size: Some(5),
            ..Default::default()
        };

        let engine = EvolutionEngineBuilder::new()
            .with_evaluator(evaluator)
            .with_mutator(mutator)
            .with_crossover(crossover)
            .with_factory(factory)
            .with_improvement(local_search)
            .build()
            .unwrap();

        let result = engine.run_ga_evolution(config).unwrap();
        let best_dist = result.global_best.eval.total_distance_integer;
        assert_eq!(best_dist, 784.0, "Regression: CVRP A-n32-k5 Best distance is {}, expected 784.0", best_dist);
     }

     #[test]
     fn test_solve_auto_config() {
         let mut instance = CvrpInstance::a_n32_k5();
         instance.distance_metric = DistanceMetric::TspLibEuc2D;
         
         let (eval, report) = solve_auto_config(&instance, true).unwrap();
         assert!(eval.total_distance > 0.0);
         assert!(report.contains("Instance Analysis"));
         assert!(report.contains("Recommended Configuration"));
         assert!(report.contains("Engineering Rationale"));
     }
}

