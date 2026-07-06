use std::collections::HashMap;
use coralys_core::analysis::{InstanceFeatures, DifficultyAssessment, ConfigurationPolicy, InstanceAnalyzer};
use crate::{CvrpInstance, Node, DistanceMetric};

#[derive(Debug, Clone)]
pub struct CvrpInstanceFeatures {
    pub customer_count: usize,
    pub vehicle_limit: usize,
    pub capacity: i32,
    
    // Demands
    pub total_demand: i32,
    pub avg_demand: f64,
    pub min_demand: i32,
    pub max_demand: i32,
    pub demand_variance: f64,
    
    // Capacity
    pub packing_density: f64,
    pub theoretical_min_vehicles: usize,
    pub vehicle_slack: i32,
    pub capacity_slack: f64,
    
    // Spatial
    pub bbox_width: f64,
    pub bbox_height: f64,
    pub bbox_area: f64,
    pub avg_nn_distance: f64,
    pub customer_density: f64,
    pub depot_centrality: f64,
    pub clustering_estimate: f64,
    
    // Constraints
    pub constraint_tightness: f64,
    pub expected_feasibility_difficulty: &'static str,
}

impl InstanceFeatures for CvrpInstanceFeatures {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CvrpSizeDifficulty {
    Small,
    Medium,
    Large,
    Huge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CvrpPackingDifficulty {
    Loose,
    Tight,
    Extreme,
}

#[derive(Debug, Clone)]
pub struct CvrpDifficultyAssessment {
    pub size_difficulty: CvrpSizeDifficulty,
    pub packing_difficulty: CvrpPackingDifficulty,
    pub spatial_complexity: &'static str,
    pub demand_distribution: &'static str,
    pub label: &'static str,
}

impl DifficultyAssessment for CvrpDifficultyAssessment {
    fn difficulty_label(&self) -> &'static str {
        self.label
    }
}

#[derive(Debug, Clone)]
pub struct CvrpConfigurationPolicy {
    pub population_size: usize,
    pub generation_limit: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub local_search_intensity: usize,
    pub repair_intensity: usize,
    pub diversity_preservation: bool,
    pub route_preserving_crossover: bool,
    pub rationale: Vec<String>,
}

impl ConfigurationPolicy for CvrpConfigurationPolicy {}

pub struct CvrpInstanceAnalyzer;

impl InstanceAnalyzer<CvrpInstance> for CvrpInstanceAnalyzer {
    type Features = CvrpInstanceFeatures;
    type Difficulty = CvrpDifficultyAssessment;
    type Policy = CvrpConfigurationPolicy;

    fn analyze(&self, instance: &CvrpInstance) -> Result<(Self::Features, Self::Difficulty, Self::Policy), String> {
        let customer_count = instance.customers.len();
        if customer_count == 0 {
            return Err("Instance has no customers".to_string());
        }
        
        let vehicle_limit = instance.max_vehicles.unwrap_or(customer_count);
        let capacity = instance.capacity;
        
        // Demand stats
        let mut total_demand = 0;
        let mut min_demand = i32::MAX;
        let mut max_demand = i32::MIN;
        for cust in &instance.customers {
            total_demand += cust.demand;
            if cust.demand < min_demand { min_demand = cust.demand; }
            if cust.demand > max_demand { max_demand = cust.demand; }
        }
        
        let avg_demand = total_demand as f64 / customer_count as f64;
        
        let mut variance_sum = 0.0;
        for cust in &instance.customers {
            variance_sum += (cust.demand as f64 - avg_demand).powi(2);
        }
        let demand_variance = variance_sum / customer_count as f64;
        
        // Capacity metrics
        let theoretical_min_vehicles = (total_demand as f64 / capacity as f64).ceil() as usize;
        let packing_density = total_demand as f64 / (vehicle_limit as f64 * capacity as f64);
        let vehicle_slack = vehicle_limit as i32 - theoretical_min_vehicles as i32;
        let capacity_slack = 1.0 - (total_demand as f64 / (vehicle_limit as f64 * capacity as f64));
        
        // Spatial Metrics
        let mut min_x = instance.depot.x;
        let mut max_x = instance.depot.x;
        let mut min_y = instance.depot.y;
        let mut max_y = instance.depot.y;
        for cust in &instance.customers {
            if cust.x < min_x { min_x = cust.x; }
            if cust.x > max_x { max_x = cust.x; }
            if cust.y < min_y { min_y = cust.y; }
            if cust.y > max_y { max_y = cust.y; }
        }
        
        let bbox_width = max_x - min_x;
        let bbox_height = max_y - min_y;
        let bbox_area = bbox_width * bbox_height;
        
        // Avg Nearest Neighbor Distance
        let mut nn_sum = 0.0;
        for (i, c1) in instance.customers.iter().enumerate() {
            let mut min_dist = f64::INFINITY;
            for (j, c2) in instance.customers.iter().enumerate() {
                if i == j { continue; }
                let d = instance.distance(c1, c2);
                if d < min_dist {
                    min_dist = d;
                }
            }
            if min_dist < f64::INFINITY {
                nn_sum += min_dist;
            }
        }
        let avg_nn_distance = nn_sum / customer_count as f64;
        
        // Depot Centrality
        let mut depot_dist_sum = 0.0;
        for cust in &instance.customers {
            depot_dist_sum += instance.distance(&instance.depot, cust);
        }
        let avg_depot_dist = depot_dist_sum / customer_count as f64;
        let bbox_diagonal = (bbox_width.powi(2) + bbox_height.powi(2)).sqrt();
        let depot_centrality = if bbox_diagonal > 0.0 {
            1.0 - (avg_depot_dist / bbox_diagonal)
        } else {
            1.0
        };
        
        let customer_density = customer_count as f64 / (if bbox_area > 0.0 { bbox_area } else { 1.0 });
        
        // Clustering Estimate (Ratio of nearest neighbor to average distance from centroid)
        let mut center_x = 0.0;
        let mut center_y = 0.0;
        for cust in &instance.customers {
            center_x += cust.x;
            center_y += cust.y;
        }
        center_x /= customer_count as f64;
        center_y /= customer_count as f64;
        let centroid = Node { id: 9999, x: center_x, y: center_y, demand: 0 };
        let mut centroid_dist_sum = 0.0;
        for cust in &instance.customers {
            centroid_dist_sum += instance.distance(&centroid, cust);
        }
        let avg_centroid_dist = centroid_dist_sum / customer_count as f64;
        let clustering_estimate = if avg_centroid_dist > 0.0 {
            avg_nn_distance / avg_centroid_dist
        } else {
            1.0
        };
        
        // Constraint tightness
        let constraint_tightness = packing_density;
        let expected_feasibility_difficulty = if constraint_tightness > 0.98 {
            "Extreme"
        } else if constraint_tightness > 0.92 {
            "High"
        } else {
            "Moderate"
        };
        
        let features = CvrpInstanceFeatures {
            customer_count,
            vehicle_limit,
            capacity,
            total_demand,
            avg_demand,
            min_demand,
            max_demand,
            demand_variance,
            packing_density,
            theoretical_min_vehicles,
            vehicle_slack,
            capacity_slack,
            bbox_width,
            bbox_height,
            bbox_area,
            avg_nn_distance,
            customer_density,
            depot_centrality,
            clustering_estimate,
            constraint_tightness,
            expected_feasibility_difficulty,
        };
        
        // Difficulty Assessment
        let size_difficulty = if customer_count > 500 {
            CvrpSizeDifficulty::Huge
        } else if customer_count > 150 {
            CvrpSizeDifficulty::Large
        } else if customer_count > 50 {
            CvrpSizeDifficulty::Medium
        } else {
            CvrpSizeDifficulty::Small
        };
        
        let packing_difficulty = if packing_density > 0.98 {
            CvrpPackingDifficulty::Extreme
        } else if packing_density > 0.90 {
            CvrpPackingDifficulty::Tight
        } else {
            CvrpPackingDifficulty::Loose
        };
        
        let spatial_complexity = if clustering_estimate < 0.25 {
            "Highly Clustered"
        } else if clustering_estimate < 0.5 {
            "Semi-Clustered"
        } else {
            "Uniformly Distributed"
        };
        
        let demand_distribution = if demand_variance > 1000.0 {
            "High Variance"
        } else {
            "Homogeneous"
        };
        
        let label = match (size_difficulty, packing_difficulty) {
            (CvrpSizeDifficulty::Huge, _) => "Scale Dominant (Huge)",
            (CvrpSizeDifficulty::Large, CvrpPackingDifficulty::Extreme) => "Constraint & Scale Tight (Large/Extreme)",
            (CvrpSizeDifficulty::Large, _) => "Scale Dominant (Large)",
            (_, CvrpPackingDifficulty::Extreme) => "Constraint Dominant (Extreme)",
            (_, CvrpPackingDifficulty::Tight) => "Constraint Tight",
            _ => "Standard",
        };
        
        let difficulty = CvrpDifficultyAssessment {
            size_difficulty,
            packing_difficulty,
            spatial_complexity,
            demand_distribution,
            label,
        };
        
        // Configuration Policy
        let mut population_size = 100;
        let mut generation_limit = 30;
        let mut mutation_rate = 0.2;
        let mut crossover_rate = 0.8;
        let mut local_search_intensity = 5;
        let mut repair_intensity = 10;
        let mut diversity_preservation = true;
        let mut route_preserving_crossover = true;
        let mut rationale = Vec::new();
        
        // Rules
        match size_difficulty {
            CvrpSizeDifficulty::Huge => {
                population_size = 300;
                generation_limit = 100;
                rationale.push("Large instance detected. Increase exploration budget.".to_string());
            }
            CvrpSizeDifficulty::Large => {
                population_size = 200;
                generation_limit = 50;
                rationale.push("Medium-large scale. Boost population size and generations.".to_string());
            }
            _ => {
                rationale.push("Small/Medium scale. Keep standard evolutionary budget.".to_string());
            }
        }
        
        match packing_difficulty {
            CvrpPackingDifficulty::Extreme => {
                repair_intensity = 30;
                local_search_intensity = 15;
                mutation_rate = 0.3;
                rationale.push("Packing density exceeds 98%. Aggressive repair is recommended.".to_string());
            }
            CvrpPackingDifficulty::Tight => {
                repair_intensity = 20;
                local_search_intensity = 10;
                rationale.push("Tight packing capacity. Increase local search and repair intensity.".to_string());
            }
            _ => {}
        }
        
        if clustering_estimate < 0.4 {
            route_preserving_crossover = true;
            rationale.push("Spatial clustering detected. Route-preserving crossover is recommended.".to_string());
        } else {
            rationale.push("Uniform spatial distribution. Rely on standard crossover.".to_string());
        }
        
        let policy = CvrpConfigurationPolicy {
            population_size,
            generation_limit,
            mutation_rate,
            crossover_rate,
            local_search_intensity,
            repair_intensity,
            diversity_preservation,
            route_preserving_crossover,
            rationale,
        };
        
        Ok((features, difficulty, policy))
    }
}

pub fn generate_analysis_report(
    features: &CvrpInstanceFeatures,
    difficulty: &CvrpDifficultyAssessment,
    policy: &CvrpConfigurationPolicy,
) -> String {
    let mut out = Vec::new();
    out.push("Instance Analysis".to_string());
    out.push("".to_string());
    out.push(format!("  Customers: {}", features.customer_count));
    out.push(format!("  Vehicles: {}", features.vehicle_limit));
    out.push(format!("  Packing Density: {:.2}%", features.packing_density * 100.0));
    out.push(format!("  Demand Variance: {:.2}", features.demand_variance));
    out.push(format!("  Constraint Tightness: {:.2}", features.constraint_tightness));
    out.push(format!("  Difficulty Classification: {}", difficulty.label));
    out.push("".to_string());
    out.push("Recommended Configuration".to_string());
    out.push("".to_string());
    out.push(format!("  Population: {}", policy.population_size));
    out.push(format!("  Generations: {}", policy.generation_limit));
    out.push(format!("  Mutation: {:.2}", policy.mutation_rate));
    out.push(format!("  Crossover: {}", if policy.route_preserving_crossover { "Route-Preserving" } else { "Standard" }));
    out.push(format!("  Repair Intensity: {}", policy.repair_intensity));
    out.push(format!("  Local Search: {}", policy.local_search_intensity));
    out.push(format!("  Diversity Strategy: {}", if policy.diversity_preservation { "Active" } else { "Standard" }));
    out.push("".to_string());
    out.push("Engineering Rationale".to_string());
    out.push("".to_string());
    for r in &policy.rationale {
        out.push(format!("  - {}", r));
    }
    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cvrp_analysis_deterministic() {
        let instance = CvrpInstance::a_n32_k5();
        let analyzer = CvrpInstanceAnalyzer;
        
        let (f1, d1, p1) = analyzer.analyze(&instance).unwrap();
        let (f2, d2, p2) = analyzer.analyze(&instance).unwrap();
        
        assert_eq!(f1.customer_count, f2.customer_count);
        assert_eq!(f1.total_demand, f2.total_demand);
        assert_eq!(f1.packing_density, f2.packing_density);
        assert_eq!(d1.label, d2.label);
        assert_eq!(p1.population_size, p2.population_size);
        assert_eq!(p1.generation_limit, p2.generation_limit);
        assert_eq!(p1.rationale, p2.rationale);
    }
}
