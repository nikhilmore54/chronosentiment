use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitStrategy {
    Greedy,
    PrinsDP,
    DPFallbackToGreedy,
}

pub fn get_split_strategy() -> SplitStrategy {
    static STRATEGY: OnceLock<SplitStrategy> = OnceLock::new();
    *STRATEGY.get_or_init(|| {
        match std::env::var("CVRP_SPLIT_STRATEGY").as_deref() {
            Ok("GREEDY") => SplitStrategy::Greedy,
            Ok("PRINS_DP") => SplitStrategy::PrinsDP,
            _ => SplitStrategy::DPFallbackToGreedy,
        }
    })
}

use coralys_moga::traits::{FitnessEvaluator, MutationOperator, CrossoverOperator, Evaluated, ImprovementOperator};
use coralys_moga::runtime::optimization::metric::MetricReport;
use coralys_core::operators::ConstraintModel;

use coralys_core::Outcome;
use rand::Rng;
use crate::{CvrpCandidate, CvrpEvaluation, CvrpInstance};

pub struct CvrpEvaluator {
    pub instance: CvrpInstance,
}

impl FitnessEvaluator<CvrpCandidate> for CvrpEvaluator {
    type Evaluation = CvrpOutcomeWrapper;

    fn evaluate(&self, candidate: &CvrpCandidate, _metrics: &coralys_moga::runtime::optimization::metric::MetricReport) -> Self::Evaluation {
        let n = candidate.permutation.len();
        if n == 0 {
            let eval = CvrpEvaluation {
                candidate: candidate.clone(),
                total_distance: 0.0,
                num_vehicles: 0,
                routes: Vec::new(),
                total_distance_integer: 0.0,
                total_distance_float: 0.0,
            };
            return CvrpOutcomeWrapper {
                fitness_array: vec![100000.0],
                eval,
            };
        }

        let k_limit = self.instance.max_vehicles.unwrap_or(n);

let strategy = get_split_strategy();

        let mut routes = Vec::new();
        let mut total_distance = 0.0;
        let mut dp_failed = false;

        if strategy == SplitStrategy::PrinsDP || strategy == SplitStrategy::DPFallbackToGreedy {
            // 2D DP Split
            let mut v = vec![vec![f64::INFINITY; n + 1]; k_limit + 1];
            let mut parent = vec![vec![0; n + 1]; k_limit + 1];
            v[0][0] = 0.0;

            for r in 1..=k_limit {
                for i in 0..n {
                    if v[r - 1][i] == f64::INFINITY { continue; }
                    let mut load = 0;
                    let mut route_dist = 0.0;
                    let mut last_node = &self.instance.depot;

                    for j in (i + 1)..=n {
                        let cust_idx = candidate.permutation[j - 1];
                        let customer = &self.instance.customers[cust_idx];
                        load += customer.demand;
                        if load > self.instance.capacity {
                            break;
                        }

                        route_dist += self.instance.distance(last_node, customer);
                        let total_route_dist = route_dist + self.instance.distance(customer, &self.instance.depot);
                        let cost = v[r - 1][i] + total_route_dist;
                        if cost < v[r][j] {
                            v[r][j] = cost;
                            parent[r][j] = i;
                        }
                        last_node = customer;
                    }
                }
            }

            let mut best_r = 0;
            let mut best_cost = f64::INFINITY;
            for r in 1..=k_limit {
                if v[r][n] < best_cost {
                    best_cost = v[r][n];
                    best_r = r;
                }
            }

            if best_cost < f64::INFINITY {
                let mut curr = n;
                let mut curr_r = best_r;
                while curr > 0 && curr_r > 0 {
                    let prev = parent[curr_r][curr];
                    let mut route = Vec::with_capacity(curr - prev);
                    for k in prev..curr {
                        let cust_idx = candidate.permutation[k];
                        route.push(self.instance.customers[cust_idx].id);
                    }
                    routes.push(route);
                    curr = prev;
                    curr_r -= 1;
                }
                routes.reverse();
                total_distance = best_cost;
            } else {
                dp_failed = true;
                if strategy == SplitStrategy::PrinsDP {
                    routes = Vec::new();
                    total_distance = 1000000.0;
                }
            }
        }

        if strategy == SplitStrategy::Greedy || (strategy == SplitStrategy::DPFallbackToGreedy && dp_failed) {
            routes.clear();
            let mut current_route = Vec::new();
            let mut current_load = 0;
            for &cust_idx in &candidate.permutation {
                let customer = &self.instance.customers[cust_idx];
                if current_load + customer.demand > self.instance.capacity {
                    routes.push(current_route.clone());
                    current_route = Vec::new();
                    current_load = 0;
                }
                current_route.push(self.instance.customers[cust_idx].id);
                current_load += customer.demand;
            }
            if !current_route.is_empty() {
                routes.push(current_route);
            }
            total_distance = self.instance.evaluate_routes_distance(&routes, crate::DistanceMetric::EuclideanFloat);
        }

        let num_vehicles = routes.len();
        let total_distance_integer = self.instance.evaluate_routes_distance(&routes, crate::DistanceMetric::TspLibEuc2D);
        let total_distance_float = self.instance.evaluate_routes_distance(&routes, crate::DistanceMetric::EuclideanFloat);

        let eval = CvrpEvaluation {
            candidate: candidate.clone(),
            total_distance,
            num_vehicles,
            routes,
            total_distance_integer,
            total_distance_float,
        };

        CvrpOutcomeWrapper {
            fitness_array: vec![100000.0 - total_distance],
            eval,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CvrpOutcomeWrapper {
    pub fitness_array: Vec<f64>,
    pub eval: CvrpEvaluation,
}

impl Outcome for CvrpOutcomeWrapper {
    type Sol = CvrpCandidate;
    fn objectives(&self) -> &[f64] {
        &self.fitness_array
    }
    fn is_valid(&self) -> bool {
        true
    }
    fn solution(&self) -> &Self::Sol {
        &self.eval.candidate
    }
}

impl Evaluated for CvrpOutcomeWrapper {
    type Genome = CvrpCandidate;
    fn fitness(&self) -> f64 {
        self.fitness_array[0]
    }
    fn is_valid(&self) -> bool {
        true
    }
    fn genome(&self) -> &Self::Genome {
        &self.eval.candidate
    }
}

pub struct CvrpMutator {
    pub entropy_scale: f64,
    pub instance: CvrpInstance,
    pub radius_policy: crate::RadiusPolicy,
    pub nearest_neighbors: Vec<Vec<usize>>,
}

impl CvrpMutator {
    pub fn new(instance: CvrpInstance, radius_policy: crate::RadiusPolicy) -> Self {
        let n = instance.customers.len();
        let mut nearest_neighbors = vec![Vec::new(); n];
        for i in 0..n {
            let mut dists: Vec<(usize, f64)> = (0..n)
                .filter(|&j| i != j)
                .map(|j| (j, instance.distance(&instance.customers[i], &instance.customers[j])))
                .collect();
            dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            nearest_neighbors[i] = dists.into_iter().map(|x| x.0).collect();
        }
        Self {
            entropy_scale: 1.0,
            instance,
            radius_policy,
            nearest_neighbors,
        }
    }
}

impl MutationOperator<CvrpCandidate> for CvrpMutator {
    fn mutate(&self, candidate: &mut CvrpCandidate, rng: &mut rand::rngs::StdRng) {
        let size = candidate.permutation.len();
        if size < 2 { return; }

        // Unconditional mutation when called (MOGA engine regulates rate)
        {
            // Pick one mutation strategy randomly (Swap, Insert, Inversion, Block, or Relocate)
            let strategy = rng.gen_range(0..5);
            let mut i = rng.gen_range(0..size);
            
            let cust_i = candidate.permutation[i];
            let neighbors = &self.nearest_neighbors[cust_i];
            
            let mut j_cust = match self.radius_policy {
                crate::RadiusPolicy::Control => candidate.permutation[rng.gen_range(0..size)],
                crate::RadiusPolicy::LocalBiased => {
                    let p = rng.gen_range(0..100);
                    if p < 70 && neighbors.len() >= 5 {
                        neighbors[rng.gen_range(0..5)]
                    } else if p < 90 && neighbors.len() >= 10 {
                        neighbors[rng.gen_range(5..10)]
                    } else {
                        candidate.permutation[rng.gen_range(0..size)]
                    }
                },
                crate::RadiusPolicy::ExtremeLocal => {
                    let p = rng.gen_range(0..100);
                    if p < 95 && neighbors.len() >= 3 {
                        neighbors[rng.gen_range(0..3)]
                    } else {
                        candidate.permutation[rng.gen_range(0..size)]
                    }
                }
            };
            
            let mut j = candidate.permutation.iter().position(|&x| x == j_cust).unwrap_or(0);
            if i == j { j = (j + 1) % size; }

            let old_perm = candidate.permutation.clone();

            match strategy {
                0 => { // Swap
                    candidate.permutation.swap(i, j);
                    candidate.last_mutation_op = Some("Swap".to_string());
                },
                1 => { // Insert
                    let val = candidate.permutation.remove(i);
                    let insert_pos = rng.gen_range(0..candidate.permutation.len());
                    candidate.permutation.insert(insert_pos, val);
                    candidate.last_mutation_op = Some("Insert".to_string());
                },
                2 => { // Inversion (2-opt approx)
                    if i > j { std::mem::swap(&mut i, &mut j); }
                    candidate.permutation[i..=j].reverse();
                    candidate.last_mutation_op = Some("Inversion".to_string());
                },
                3 => { // Block exchange
                    if i > j { std::mem::swap(&mut i, &mut j); }
                    let block_size = rng.gen_range(1..=(j-i).max(1).min(size/4).max(1));
                    if j + block_size <= size {
                        for k in 0..block_size {
                            candidate.permutation.swap(i+k, j+k);
                        }
                    } else {
                        candidate.permutation.swap(i, j);
                    }
                    candidate.last_mutation_op = Some("Block".to_string());
                },
                _ => { // Route-aware Relocate
                    let mut routes = Vec::new();
                    let mut current_route = Vec::new();
                    let mut current_load = 0;
                    
                    for &cust_idx in &candidate.permutation {
                        let customer = &self.instance.customers[cust_idx];
                        if current_load + customer.demand > self.instance.capacity {
                            routes.push(current_route.clone());
                            current_route = Vec::new();
                            current_load = 0;
                        }
                        current_route.push(cust_idx);
                        current_load += customer.demand;
                    }
                    if !current_route.is_empty() {
                        routes.push(current_route);
                    }
                    
                    if routes.len() > 1 {
                        let r1 = rng.gen_range(0..routes.len());
                        let mut r2 = rng.gen_range(0..routes.len());
                        if r1 == r2 { r2 = (r2 + 1) % routes.len(); }
                        
                        if !routes[r1].is_empty() {
                            let item_idx = rng.gen_range(0..routes[r1].len());
                            let item = routes[r1].remove(item_idx);
                            
                            let insert_pos = if routes[r2].is_empty() { 0 } else { rng.gen_range(0..=routes[r2].len()) };
                            routes[r2].insert(insert_pos, item);
                            
                            let mut new_perm = Vec::with_capacity(size);
                            for route in routes {
                                new_perm.extend(route);
                            }
                            candidate.permutation = new_perm;
                        }
                    }
                    candidate.last_mutation_op = Some("Relocate".to_string());
                }
            }

            let mut first_diff = None;
            let mut last_diff = None;
            for k in 0..size {
                if old_perm[k] != candidate.permutation[k] {
                    if first_diff.is_none() { first_diff = Some(k); }
                    last_diff = Some(k);
                }
            }
            candidate.last_mutation_radius = match (first_diff, last_diff) {
                (Some(f), Some(l)) => Some(l - f),
                _ => Some(0),
            };
        }
    }
}

pub struct CvrpCrossover;

impl CrossoverOperator<CvrpCandidate> for CvrpCrossover {
    fn crossover(&self, parent1: &CvrpCandidate, parent2: &CvrpCandidate, rng: &mut rand::rngs::StdRng) -> (CvrpCandidate, CvrpCandidate) {
        let size = parent1.permutation.len();
        if size == 0 {
            return (parent1.clone(), parent2.clone());
        }
        let mut i = rng.gen_range(0..size);
        let mut j = rng.gen_range(0..size);
        if i > j { std::mem::swap(&mut i, &mut j); }

        let mut child1_perm = vec![None; size];
        let mut child2_perm = vec![None; size];
        for k in i..=j {
            child1_perm[k] = Some(parent1.permutation[k]);
            child2_perm[k] = Some(parent2.permutation[k]);
        }

        let mut current_idx1 = (j + 1) % size;
        let mut current_idx2 = (j + 1) % size;
        for k in 0..size {
            let p2_idx = (j + 1 + k) % size;
            let val2 = parent2.permutation[p2_idx];
            if !child1_perm.contains(&Some(val2)) {
                child1_perm[current_idx1] = Some(val2);
                current_idx1 = (current_idx1 + 1) % size;
            }

            let p1_idx = (j + 1 + k) % size;
            let val1 = parent1.permutation[p1_idx];
            if !child2_perm.contains(&Some(val1)) {
                child2_perm[current_idx2] = Some(val1);
                current_idx2 = (current_idx2 + 1) % size;
            }
        }

        (
            CvrpCandidate { permutation: child1_perm.into_iter().map(|x| x.unwrap()).collect(), last_mutation_op: None, last_mutation_radius: None, route_boundary_changes: None },
            CvrpCandidate { permutation: child2_perm.into_iter().map(|x| x.unwrap()).collect(), last_mutation_op: None, last_mutation_radius: None, route_boundary_changes: None }
        )
    }
}

pub struct CvrpCrossoverRoutePreserving {
    pub instance: crate::CvrpInstance,
}

impl CrossoverOperator<CvrpCandidate> for CvrpCrossoverRoutePreserving {
    fn crossover(&self, parent1: &CvrpCandidate, parent2: &CvrpCandidate, rng: &mut rand::rngs::StdRng) -> (CvrpCandidate, CvrpCandidate) {
        use rand::Rng;
        let size = parent1.permutation.len();
        if size == 0 { return (parent1.clone(), parent2.clone()); }

        let mut child1_perm = Vec::with_capacity(size);
        let mut child2_perm = Vec::with_capacity(size);

        let p1_routes = get_routes(parent1, &self.instance);
        let r1 = if p1_routes.is_empty() { Vec::new() } else { p1_routes[rng.gen_range(0..p1_routes.len())].clone() };
        child1_perm.extend(r1.iter().cloned());
        for &cust in &parent2.permutation {
            if !r1.contains(&cust) { child1_perm.push(cust); }
        }

        let p2_routes = get_routes(parent2, &self.instance);
        let r2 = if p2_routes.is_empty() { Vec::new() } else { p2_routes[rng.gen_range(0..p2_routes.len())].clone() };
        child2_perm.extend(r2.iter().cloned());
        for &cust in &parent1.permutation {
            if !r2.contains(&cust) { child2_perm.push(cust); }
        }

        (
            CvrpCandidate { permutation: child1_perm, last_mutation_op: None, last_mutation_radius: None, route_boundary_changes: None },
            CvrpCandidate { permutation: child2_perm, last_mutation_op: None, last_mutation_radius: None, route_boundary_changes: None }
        )
    }
}

fn get_routes(candidate: &CvrpCandidate, instance: &crate::CvrpInstance) -> Vec<Vec<usize>> {
    let evaluator = CvrpEvaluator { instance: instance.clone() };
    let outcome = evaluator.evaluate(candidate, &MetricReport::default());
    let mut routes = Vec::new();
    
    let strategy = get_split_strategy();
    let is_greedy = strategy == SplitStrategy::Greedy || (strategy == SplitStrategy::DPFallbackToGreedy && outcome.eval.routes.is_empty());

    if is_greedy {
        let mut current_route = Vec::new();
        let mut current_load = 0;
        for &cust_idx in &candidate.permutation {
            let customer = &instance.customers[cust_idx];
            if current_load + customer.demand > instance.capacity {
                routes.push(current_route.clone());
                current_route = Vec::new();
                current_load = 0;
            }
            current_route.push(cust_idx);
            current_load += customer.demand;
        }
        if !current_route.is_empty() {
            routes.push(current_route);
        }
    } else {
        for r in outcome.eval.routes {
            let mut route_indices = Vec::with_capacity(r.len());
            for node_id in r {
                if let Some(idx) = instance.customers.iter().position(|c| c.id == node_id) {
                    route_indices.push(idx);
                }
            }
            routes.push(route_indices);
        }
    }
    routes
}

pub enum CvrpCrossoverVariant {
    OX1(CvrpCrossover),
    RoutePreserving(CvrpCrossoverRoutePreserving),
}

impl CrossoverOperator<CvrpCandidate> for CvrpCrossoverVariant {
    fn crossover(&self, parent1: &CvrpCandidate, parent2: &CvrpCandidate, rng: &mut rand::rngs::StdRng) -> (CvrpCandidate, CvrpCandidate) {
        match self {
            Self::OX1(c) => c.crossover(parent1, parent2, rng),
            Self::RoutePreserving(c) => c.crossover(parent1, parent2, rng),
        }
    }
}

pub struct CvrpLocalSearch {
    pub instance: crate::CvrpInstance,
}

impl coralys_core::operators::ImprovementOperator<CvrpCandidate, CvrpConstraintModel> for CvrpLocalSearch {
    type Error = CvrpOperatorError;
fn improve(&self, candidate: &mut CvrpCandidate, _model: &CvrpConstraintModel, _budget: &coralys_core::operators::OperatorBudget) -> Result<bool, Self::Error> {
        let evaluator = CvrpEvaluator { instance: self.instance.clone() };
        let outcome = evaluator.evaluate(candidate, &MetricReport::default());
        let mut routes = Vec::new();
        
        let strategy = get_split_strategy();
        let is_greedy = strategy == SplitStrategy::Greedy || (strategy == SplitStrategy::DPFallbackToGreedy && outcome.eval.routes.is_empty());

        if is_greedy {
            let mut current_route = Vec::new();
            let mut current_load = 0;
            for &cust_idx in &candidate.permutation {
                let customer = &self.instance.customers[cust_idx];
                if current_load + customer.demand > self.instance.capacity {
                    routes.push(current_route);
                    current_route = Vec::new();
                    current_load = 0;
                }
                current_route.push(cust_idx);
                current_load += customer.demand;
            }
            if !current_route.is_empty() {
                routes.push(current_route);
            }
        } else {
            for r in outcome.eval.routes {
                let mut route_indices = Vec::with_capacity(r.len());
                for node_id in r {
                    if let Some(idx) = self.instance.customers.iter().position(|c| c.id == node_id) {
                        route_indices.push(idx);
                    }
                }
                routes.push(route_indices);
            }
        }

        // Helper: get node reference by index in route (-1 or >= len = depot)
        let get_node = |route: &[usize], pos: isize| -> &crate::Node {
            if pos < 0 || pos as usize >= route.len() {
                &self.instance.depot
            } else {
                &self.instance.customers[route[pos as usize]]
            }
        };

        let get_route_distance = |route: &[usize]| -> f64 {
            if route.is_empty() { return 0.0; }
            let mut r_dist = 0.0;
            let mut last_node = &self.instance.depot;
            for &cust_idx in route {
                let customer = &self.instance.customers[cust_idx];
                r_dist += self.instance.distance(last_node, customer);
                last_node = customer;
            }
            r_dist += self.instance.distance(last_node, &self.instance.depot);
            r_dist
        };

        let mut route_distances: Vec<f64> = routes.iter().map(|r| get_route_distance(r)).collect();

        let n = self.instance.customers.len();
        let mut min_dists = vec![f64::MAX; n];
        for i in 0..n {
            let cust_i = &self.instance.customers[i];
            for j in 0..n {
                if i != j {
                    let d = self.instance.distance(cust_i, &self.instance.customers[j]);
                    if d < min_dists[i] {
                        min_dists[i] = d;
                    }
                }
            }
        }

        let mut current_best = route_distances.iter().sum::<f64>();
        let mut improving = true;
        while improving {
            improving = false;

            'outer: {
                // 1. Intra-route improvements using O(1) delta evaluation
                for r in 0..routes.len() {
                    let len = routes[r].len();
                    if len < 2 { continue; }

                    // 2-opt: reverse segment [i..=j]
                    // Old edges: d(i-1,i) + d(j,j+1); New edges: d(i-1,j) + d(i,j+1)
                    for i in 0..len {
                        for j in (i+1)..len {
                            let d_old = self.instance.distance(get_node(&routes[r], i as isize - 1), get_node(&routes[r], i as isize))
                                      + self.instance.distance(get_node(&routes[r], j as isize), get_node(&routes[r], j as isize + 1));
                            let d_new = self.instance.distance(get_node(&routes[r], i as isize - 1), get_node(&routes[r], j as isize))
                                      + self.instance.distance(get_node(&routes[r], i as isize), get_node(&routes[r], j as isize + 1));
                            let delta = d_new - d_old;
                            if delta < -1e-9 {
                                routes[r][i..=j].reverse();
                                route_distances[r] += delta;
                                current_best += delta;
                                improving = true;
                                break 'outer;
                            }
                        }
                    }

                    // Intra-swap: swap positions i and j using O(1) delta evaluation
                    for i in 0..len {
                        for j in (i+1)..len {
                            let delta = if j == i + 1 {
                                // Adjacent: only outer edges change (middle edge d(i,j) is symmetric)
                                let d_old = self.instance.distance(get_node(&routes[r], i as isize - 1), get_node(&routes[r], i as isize))
                                          + self.instance.distance(get_node(&routes[r], j as isize), get_node(&routes[r], j as isize + 1));
                                let d_new = self.instance.distance(get_node(&routes[r], i as isize - 1), get_node(&routes[r], j as isize))
                                          + self.instance.distance(get_node(&routes[r], i as isize), get_node(&routes[r], j as isize + 1));
                                d_new - d_old
                            } else {
                                // Non-adjacent: 4 edges change
                                let d_old = self.instance.distance(get_node(&routes[r], i as isize - 1), get_node(&routes[r], i as isize))
                                          + self.instance.distance(get_node(&routes[r], i as isize), get_node(&routes[r], i as isize + 1))
                                          + self.instance.distance(get_node(&routes[r], j as isize - 1), get_node(&routes[r], j as isize))
                                          + self.instance.distance(get_node(&routes[r], j as isize), get_node(&routes[r], j as isize + 1));
                                // After swap: node at i becomes routes[r][j], node at j becomes routes[r][i]
                                let node_i = get_node(&routes[r], i as isize);
                                let node_j = get_node(&routes[r], j as isize);
                                let d_new = self.instance.distance(get_node(&routes[r], i as isize - 1), node_j)
                                          + self.instance.distance(node_j, get_node(&routes[r], i as isize + 1))
                                          + self.instance.distance(get_node(&routes[r], j as isize - 1), node_i)
                                          + self.instance.distance(node_i, get_node(&routes[r], j as isize + 1));
                                d_new - d_old
                            };
                            if delta < -1e-9 {
                                routes[r].swap(i, j);
                                route_distances[r] += delta;
                                current_best += delta;
                                improving = true;
                                break 'outer;
                            }
                        }
                    }

                    // Intra-relocate: remove node at i, insert at position j (clone-based, safe)
                    for i in 0..len {
                        for j in 0..len {
                            if i == j { continue; }
                            let mut test_route = routes[r].clone();
                            let val = test_route.remove(i);
                            let insert_pos = if j > i { j - 1 } else { j };
                            test_route.insert(insert_pos, val);
                            let new_r_dist = get_route_distance(&test_route);
                            let delta = new_r_dist - route_distances[r];
                            if delta < -1e-9 {
                                routes[r] = test_route;
                                route_distances[r] = new_r_dist;
                                current_best += delta;
                                improving = true;
                                break 'outer;
                            }
                        }
                    }
                
                }

                // 2. Inter-route Relocate (move a customer from route A to route B)
                for r1 in 0..routes.len() {
                    for r2 in 0..routes.len() {
                        if r1 == r2 { continue; }
                        let len1 = routes[r1].len();
                        for i in 0..len1 {
                            let cust_idx = routes[r1][i];

                            // Spatial pruning
                            if !routes[r2].is_empty() {
                                let limit_dist = min_dists[cust_idx] * 4.0;
                                let mut is_close = false;
                                for &other_idx in &routes[r2] {
                                    let d = self.instance.distance(&self.instance.customers[cust_idx], &self.instance.customers[other_idx]);
                                    if d <= limit_dist {
                                        is_close = true;
                                        break;
                                    }
                                }
                                if !is_close { continue; }
                            }

                            let demand = self.instance.customers[cust_idx].demand;

                            // Check capacity constraint on target route
                            let load2: i32 = routes[r2].iter().map(|&idx| self.instance.customers[idx].demand).sum();
                            if load2 + demand > self.instance.capacity {
                                continue;
                            }

                            // O(1) delta for removing cust_idx from r1 at position i
                            let node_prev1 = get_node(&routes[r1], i as isize - 1);
                            let node_ci = &self.instance.customers[cust_idx];
                            let node_next1 = get_node(&routes[r1], i as isize + 1);
                            let removal_delta_r1 = self.instance.distance(node_prev1, node_next1)
                                                 - self.instance.distance(node_prev1, node_ci)
                                                 - self.instance.distance(node_ci, node_next1);

                            let len2 = routes[r2].len();
                            for j in 0..=len2 {
                                // O(1) delta for inserting cust_idx at position j in r2
                                let node_prev2 = if j == 0 { &self.instance.depot } else { &self.instance.customers[routes[r2][j - 1]] };
                                let node_next2 = if j == len2 { &self.instance.depot } else { &self.instance.customers[routes[r2][j]] };
                                let insertion_delta_r2 = self.instance.distance(node_prev2, node_ci)
                                                       + self.instance.distance(node_ci, node_next2)
                                                       - self.instance.distance(node_prev2, node_next2);
                                let delta = removal_delta_r1 + insertion_delta_r2;
                                if delta < -1e-9 {
                                    let val = routes[r1].remove(i);
                                    routes[r2].insert(j, val);
                                    route_distances[r1] += removal_delta_r1;
                                    route_distances[r2] += insertion_delta_r2;
                                    current_best += delta;
                                    improving = true;
                                    break 'outer;
                                }
                            }
                        }
                    }
                }

                // 3. Inter-route Swap (swap a customer between route A and route B)
                for r1 in 0..routes.len() {
                    for r2 in (r1+1)..routes.len() {
                        let len1 = routes[r1].len();
                        let len2 = routes[r2].len();
                        let load1: i32 = routes[r1].iter().map(|&idx| self.instance.customers[idx].demand).sum();
                        let load2: i32 = routes[r2].iter().map(|&idx| self.instance.customers[idx].demand).sum();

                        for i in 0..len1 {
                            let c1 = routes[r1][i];

                            // Spatial pruning for swap partner
                            let limit_dist1 = min_dists[c1] * 4.0;
                            let mut is_close = false;
                            for &other_idx in &routes[r2] {
                                let d = self.instance.distance(&self.instance.customers[c1], &self.instance.customers[other_idx]);
                                if d <= limit_dist1 {
                                    is_close = true;
                                    break;
                                }
                            }
                            if !is_close { continue; }

                            for j in 0..len2 {
                                let c2 = routes[r2][j];
                                let d1 = self.instance.customers[c1].demand;
                                let d2 = self.instance.customers[c2].demand;

                                // Check capacities if swapped
                                if load1 - d1 + d2 > self.instance.capacity || load2 - d2 + d1 > self.instance.capacity {
                                    continue;
                                }

                                // O(1) delta for swapping c1 (r1[i]) with c2 (r2[j])
                                let node_c1 = &self.instance.customers[c1];
                                let node_c2 = &self.instance.customers[c2];
                                let old_r1 = self.instance.distance(get_node(&routes[r1], i as isize - 1), node_c1)
                                           + self.instance.distance(node_c1, get_node(&routes[r1], i as isize + 1));
                                let old_r2 = self.instance.distance(get_node(&routes[r2], j as isize - 1), node_c2)
                                           + self.instance.distance(node_c2, get_node(&routes[r2], j as isize + 1));
                                let new_r1 = self.instance.distance(get_node(&routes[r1], i as isize - 1), node_c2)
                                           + self.instance.distance(node_c2, get_node(&routes[r1], i as isize + 1));
                                let new_r2 = self.instance.distance(get_node(&routes[r2], j as isize - 1), node_c1)
                                           + self.instance.distance(node_c1, get_node(&routes[r2], j as isize + 1));
                                let delta = (new_r1 + new_r2) - (old_r1 + old_r2);
                                if delta < -1e-9 {
                                    routes[r1][i] = c2;
                                    routes[r2][j] = c1;
                                    route_distances[r1] += new_r1 - old_r1;
                                    route_distances[r2] += new_r2 - old_r2;
                                    current_best += delta;
                                    improving = true;
                                    break 'outer;
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut new_perm = Vec::with_capacity(candidate.permutation.len());
        for route in routes {
            new_perm.extend(route);
        }
        candidate.permutation = new_perm;
        Ok(true)
    }
}

pub struct CvrpRouteAwareMutator {
    pub instance: CvrpInstance,
}

impl MutationOperator<CvrpCandidate> for CvrpRouteAwareMutator {
    fn mutate(&self, candidate: &mut CvrpCandidate, rng: &mut rand::rngs::StdRng) {
        let size = candidate.permutation.len();
        if size < 2 { return; }

        // Decode routes using exact evaluator logic (storing values, not indices)
        let mut routes: Vec<Vec<usize>> = Vec::new();
        let mut current_route: Vec<usize> = Vec::new();
        let mut current_load = 0;

        for &cust_idx in &candidate.permutation {
            let customer = &self.instance.customers[cust_idx];
            if current_load + customer.demand > self.instance.capacity {
                routes.push(current_route);
                current_route = Vec::new();
                current_load = 0;
            }
            current_route.push(cust_idx);
            current_load += customer.demand;
        }
        if !current_route.is_empty() {
            routes.push(current_route);
        }

        let mut valid_routes: Vec<usize> = routes.iter().enumerate().filter(|(_, r)| r.len() >= 2).map(|(idx, _)| idx).collect();
        if valid_routes.is_empty() { return; }

        let route_idx = valid_routes[rng.gen_range(0..valid_routes.len())];
        let route = &mut routes[route_idx];

        let r_len = route.len();
        let mut i = rng.gen_range(0..r_len);
        let mut j = rng.gen_range(0..r_len);
        if i == j { j = (j + 1) % r_len; }

        let strategy = rng.gen_range(0..3);
        let op_name;
        match strategy {
            0 => {
                route.swap(i, j);
                op_name = "RouteSwap";
            },
            1 => {
                let val = route.remove(i);
                let insert_pos = if j > i { j - 1 } else { j };
                route.insert(insert_pos, val);
                op_name = "RouteRelocate";
            },
            _ => { // 2-opt
                if i > j { std::mem::swap(&mut i, &mut j); }
                route[i..=j].reverse();
                op_name = "Route2Opt";
            }
        }

        // Flatten back to permutation
        let mut new_perm = Vec::with_capacity(size);
        for r in routes {
            new_perm.extend(r);
        }
        candidate.permutation = new_perm;
        candidate.last_mutation_op = Some(op_name.to_string());
        candidate.last_mutation_radius = Some(if i > j { i - j } else { j - i });
        candidate.route_boundary_changes = Some(0); // Explicitly 0 by construction
    }
}

#[derive(Debug, Clone)]
pub enum CvrpViolation {
    VehicleLimitExceeded { actual: usize, limit: usize },
}

pub struct CvrpConstraintModel {
    pub instance: crate::CvrpInstance,
}

impl coralys_core::operators::ConstraintModel<CvrpCandidate> for CvrpConstraintModel {
    type Violation = CvrpViolation;

    fn evaluate_violations(&self, candidate: &CvrpCandidate) -> Vec<Self::Violation> {
        let limit = self.instance.max_vehicles.unwrap_or(999);
        
        let mut actual = 0;
        let mut current_load = 0;
        for &cust_idx in &candidate.permutation {
            let customer = &self.instance.customers[cust_idx];
            if current_load + customer.demand > self.instance.capacity {
                actual += 1;
                current_load = 0;
            }
            current_load += customer.demand;
        }
        if current_load > 0 {
            actual += 1;
        }

        let mut violations = Vec::new();
        if actual > limit {
            violations.push(CvrpViolation::VehicleLimitExceeded { actual, limit });
        }
        violations
    }
}

#[derive(Debug)]
pub struct CvrpOperatorError(pub String);
impl std::fmt::Display for CvrpOperatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
}
impl std::error::Error for CvrpOperatorError {}

pub struct VehicleLimitRepairHeuristic {
    pub instance: crate::CvrpInstance,
}

impl coralys_core::operators::RepairOperator<CvrpCandidate, CvrpConstraintModel> for VehicleLimitRepairHeuristic {
    type Error = CvrpOperatorError;

    fn repair(&self, candidate: &mut CvrpCandidate, model: &CvrpConstraintModel, _budget: &coralys_core::operators::OperatorBudget) -> Result<bool, Self::Error> {
        let violations = model.evaluate_violations(candidate);
        if violations.is_empty() { return Ok(true); }

        if let Some(CvrpViolation::VehicleLimitExceeded { limit, .. }) = violations.first() {
                let mut routes = Vec::new();
                let mut current_route = Vec::new();
                let mut current_load = 0;
                for &cust_idx in &candidate.permutation {
                    let customer = &self.instance.customers[cust_idx];
                    if current_load + customer.demand > self.instance.capacity {
                        routes.push((current_route.clone(), current_load));
                        current_route = Vec::new();
                        current_load = 0;
                    }
                    current_route.push(cust_idx);
                    current_load += customer.demand;
                }
                if !current_route.is_empty() {
                    routes.push((current_route, current_load));
                }

                if routes.len() <= *limit {
                    return Ok(false);
                }

                // Find the route with the smallest load
                let mut min_idx = 0;
                let mut min_load = 999999;
                for (idx, (_, load)) in routes.iter().enumerate() {
                    if *load < min_load {
                        min_load = *load;
                        min_idx = idx;
                    }
                }

                let (small_route, _) = routes.remove(min_idx);
                let mut success = true;
                for cust in small_route {
                    let demand = self.instance.customers[cust].demand;
                    let mut found = false;
                    for (r_idx, (r, load)) in routes.iter_mut().enumerate() {
                        if *load + demand <= self.instance.capacity {
                            r.push(cust);
                            *load += demand;
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        success = false;
                        break;
                    }
                }

                if success {
                    let mut new_perm = Vec::with_capacity(candidate.permutation.len());
                    for (r, _) in routes {
                        new_perm.extend(r);
                    }
                    candidate.permutation = new_perm;
                    Ok(true)
                } else {
                    Ok(false)
                }
        } else {
            Ok(false)
        }
    }
}

pub struct BinPackingRepairHeuristic {
    pub instance: crate::CvrpInstance,
}

impl coralys_core::operators::RepairOperator<CvrpCandidate, CvrpConstraintModel> for BinPackingRepairHeuristic {
    type Error = CvrpOperatorError;

    fn repair(&self, candidate: &mut CvrpCandidate, model: &CvrpConstraintModel, _budget: &coralys_core::operators::OperatorBudget) -> Result<bool, Self::Error> {
        let violations = model.evaluate_violations(candidate);
        if violations.is_empty() { return Ok(true); }

        if let Some(CvrpViolation::VehicleLimitExceeded { limit, .. }) = violations.first() {
                let mut customers_with_demands: Vec<(usize, i32)> = candidate.permutation.iter().map(|&idx| {
                    (idx, self.instance.customers[idx].demand)
                }).collect();

                // Sort customers in descending order of demand for Best-Fit Decreasing
                customers_with_demands.sort_by(|a, b| b.1.cmp(&a.1));

                let mut bins: Vec<(Vec<usize>, i32)> = Vec::new();
                let capacity = self.instance.capacity;

                for (idx, demand) in customers_with_demands {
                    let mut best_bin_idx: Option<usize> = None;
                    let mut min_remaining = capacity + 1;

                    for (b_idx, (_, load)) in bins.iter().enumerate() {
                        let remaining = capacity - load;
                        if remaining >= demand && remaining < min_remaining {
                            min_remaining = remaining;
                            best_bin_idx = Some(b_idx);
                        }
                    }

                    if let Some(b_idx) = best_bin_idx {
                        bins[b_idx].0.push(idx);
                        bins[b_idx].1 += demand;
                    } else {
                        bins.push((vec![idx], demand));
                    }
                }

                if bins.len() <= *limit {
                    let mut new_perm = Vec::with_capacity(candidate.permutation.len());
                    for (b_custs, _) in bins {
                        new_perm.extend(b_custs);
                    }
                    candidate.permutation = new_perm;
                    Ok(true)
                } else {
                    Ok(false)
                }
        } else {
            Ok(false)
        }
    }
}

pub struct SpatialBinPackingRepairHeuristic {
    pub instance: crate::CvrpInstance,
}

impl coralys_core::operators::RepairOperator<CvrpCandidate, CvrpConstraintModel> for SpatialBinPackingRepairHeuristic {
    type Error = CvrpOperatorError;

    fn repair(&self, candidate: &mut CvrpCandidate, model: &CvrpConstraintModel, _budget: &coralys_core::operators::OperatorBudget) -> Result<bool, Self::Error> {
        let violations = model.evaluate_violations(candidate);
        if violations.is_empty() { return Ok(true); }

        if let Some(CvrpViolation::VehicleLimitExceeded { limit, .. }) = violations.first() {
                let mut customers_with_demands: Vec<(usize, i32)> = candidate.permutation.iter().map(|&idx| {
                    (idx, self.instance.customers[idx].demand)
                }).collect();

                // Sort descending by demand
                customers_with_demands.sort_by(|a, b| b.1.cmp(&a.1));

                let mut bins: Vec<(Vec<usize>, i32)> = Vec::new();
                let capacity = self.instance.capacity;

                for (idx, demand) in customers_with_demands {
                    let customer = &self.instance.customers[idx];
                    let mut best_bin_idx: Option<usize> = None;
                    let mut min_score = f64::INFINITY;

                    for (b_idx, (b_custs, load)) in bins.iter().enumerate() {
                        let remaining = capacity - load;
                        if remaining >= demand {
                            // Compute spatial distance to nearest customer in this bin
                            let mut min_dist = f64::INFINITY;
                            for &other_idx in b_custs {
                                let other = &self.instance.customers[other_idx];
                                let d = self.instance.distance(customer, other);
                                if d < min_dist { min_dist = d; }
                            }
                            if min_dist == f64::INFINITY {
                                min_dist = self.instance.distance(&self.instance.depot, customer);
                            }

                            // Combined score: Distance + 50.0 * Capacity_Slack
                            let slack = remaining as f64 / capacity as f64;
                            let score = min_dist + 50.0 * slack; 

                            if score < min_score {
                                min_score = score;
                                best_bin_idx = Some(b_idx);
                            }
                        }
                    }

                    if let Some(b_idx) = best_bin_idx {
                        bins[b_idx].0.push(idx);
                        bins[b_idx].1 += demand;
                    } else {
                        bins.push((vec![idx], demand));
                    }
                }

                if bins.len() <= *limit {
                    let mut new_perm = Vec::with_capacity(candidate.permutation.len());
                    for (b_custs, _) in bins {
                        new_perm.extend(b_custs);
                    }
                    candidate.permutation = new_perm;
                    Ok(true)
                } else {
                    Ok(false)
                }
        } else {
            Ok(false)
        }
    }
}


#[cfg(test)]
mod evaluator_tests {
    use super::*;
    use crate::{CvrpInstance, Node, DistanceMetric, CvrpCandidate};
    use coralys_moga::traits::FitnessEvaluator;

    #[test]
    fn test_vehicle_limit_enforcement() {
        let depot = Node { id: 1, x: 0.0, y: 0.0, demand: 0 };
        let customers = vec![
            Node { id: 2, x: 10.0, y: 0.0, demand: 60 },
            Node { id: 3, x: 20.0, y: 0.0, demand: 60 },
            Node { id: 4, x: 30.0, y: 0.0, demand: 60 },
        ];
        
        let mut instance = CvrpInstance {
            capacity: 100,
            depot,
            customers,
            distance_metric: DistanceMetric::TspLibEuc2D,
            max_vehicles: Some(2),
            explicit_matrix: vec![],
        };

        let evaluator = CvrpEvaluator { instance: instance.clone() };
        let candidate = CvrpCandidate {
            permutation: vec![0, 1, 2],
            last_mutation_op: None,
            last_mutation_radius: None,
            route_boundary_changes: None,
        };

        let res = evaluator.evaluate(&candidate, &coralys_moga::runtime::optimization::metric::MetricReport::default());
        // Exceeds max_vehicles (2), so it is infeasible and penalized (returns 0 routes)
        assert!(res.eval.total_distance > 10000.0, "Should apply penalty to infeasible solution");
        assert_eq!(res.eval.num_vehicles, 0);

        instance.max_vehicles = Some(3);
        let evaluator2 = CvrpEvaluator { instance };
        let res2 = evaluator2.evaluate(&candidate, &coralys_moga::runtime::optimization::metric::MetricReport::default());
        // Feasible within max_vehicles (3), so no penalty
        assert!(res2.eval.total_distance < 1000.0, "Should be feasible with 3 vehicles");
        assert_eq!(res2.eval.num_vehicles, 3);
    }

    #[test]
    fn test_repair_and_improve_contracts() {
        use coralys_core::operators::{ConstraintModel, RepairOperator, ImprovementOperator, OperatorBudget};
        
        let depot = Node { id: 1, x: 0.0, y: 0.0, demand: 0 };
        let customers = vec![
            Node { id: 2, x: 10.0, y: 0.0, demand: 60 },
            Node { id: 3, x: 20.0, y: 0.0, demand: 60 },
            Node { id: 4, x: 30.0, y: 0.0, demand: 60 },
            Node { id: 5, x: 40.0, y: 0.0, demand: 10 },
        ];
        
        let instance = CvrpInstance {
            capacity: 100,
            depot,
            customers,
            distance_metric: DistanceMetric::TspLibEuc2D,
            max_vehicles: Some(2),
            explicit_matrix: vec![],
        };

        let model = CvrpConstraintModel { instance: instance.clone() };
        let repair_op = BinPackingRepairHeuristic { instance: instance.clone() };
        let improve_op = CvrpLocalSearch { instance: instance.clone() };
        let budget = OperatorBudget { max_iterations: 10, max_time_ms: 1000 };

        // 1. Create an INFEASIBLE candidate
        let mut candidate = CvrpCandidate {
            permutation: vec![0, 1, 2, 3],
            last_mutation_op: None,
            last_mutation_radius: None,
            route_boundary_changes: None,
        };
        
        assert!(!model.is_feasible(&candidate), "Initial candidate should be infeasible");

        // 2. Repair operator contract: if it returns Ok(true), it MUST be feasible.
        let repair_result = repair_op.repair(&mut candidate, &model, &budget).unwrap();
        if repair_result {
            assert!(model.is_feasible(&candidate), "RepairOperator returned true, candidate MUST be feasible");
        }
        
        // 3. Force feasible
        let instance_feas = CvrpInstance {
            max_vehicles: Some(3),
            ..instance.clone()
        };
        let model_feas = CvrpConstraintModel { instance: instance_feas.clone() };
        let repair_feas = BinPackingRepairHeuristic { instance: instance_feas.clone() };
        let improve_feas = CvrpLocalSearch { instance: instance_feas.clone() };
        
        let mut candidate_feas = CvrpCandidate {
            permutation: vec![0, 1, 2, 3],
            last_mutation_op: None,
            last_mutation_radius: None,
            route_boundary_changes: None,
        };
        let rep = repair_feas.repair(&mut candidate_feas, &model_feas, &budget).unwrap();
        assert!(rep, "Should successfully repair with 3 vehicles");
        assert!(model_feas.is_feasible(&candidate_feas), "Candidate is feasible now");

        // 4. Improvement operator contract: feasible in -> feasible out
        improve_feas.improve(&mut candidate_feas, &model_feas, &budget).unwrap();
        assert!(model_feas.is_feasible(&candidate_feas), "ImprovementOperator MUST preserve feasibility");
    }
}
