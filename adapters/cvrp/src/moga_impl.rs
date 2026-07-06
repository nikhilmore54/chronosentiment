use coralys_moga::traits::{FitnessEvaluator, MutationOperator, CrossoverOperator, Evaluated, ImprovementOperator};

use coralys_core::Outcome;
use rand::Rng;
use crate::{CvrpCandidate, CvrpEvaluation, CvrpInstance};

pub struct CvrpEvaluator {
    pub instance: CvrpInstance,
}

impl FitnessEvaluator<CvrpCandidate> for CvrpEvaluator {
    type Evaluation = CvrpOutcomeWrapper;

    fn evaluate(&self, candidate: &CvrpCandidate) -> Self::Evaluation {
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

        let mut routes = Vec::new();
        let total_distance;

        if best_cost < f64::INFINITY {
            // Feasible under k_limit
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
            // Infeasible under k_limit! Return empty routes and apply a massive penalty
            routes = Vec::new();
            total_distance = 1000000.0;
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
            // Pick one mutation strategy randomly (Swap, Insert, Relocate only)
            let strategy = match rng.gen_range(0..3) {
                0 => 0, // Swap
                1 => 1, // Insert
                _ => 4, // Relocate
            };
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
    let mut routes = Vec::new();
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
    if !current_route.is_empty() { routes.push(current_route); }
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

impl ImprovementOperator<CvrpCandidate> for CvrpLocalSearch {
    fn improve(&self, candidate: &mut CvrpCandidate) {
        let evaluator = CvrpEvaluator { instance: self.instance.clone() };
        let outcome = evaluator.evaluate(candidate);
        let mut routes = Vec::new();
        
        if outcome.eval.routes.is_empty() {
            // Infeasible under k_limit! Fallback to greedy packing split to at least optimize something
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
            // Map node IDs back to indices in self.instance.customers
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

        let get_total_distance = |routes: &Vec<Vec<usize>>| -> f64 {
            let mut total = 0.0;
            for route in routes {
                if route.is_empty() { continue; }
                let mut last_node = &self.instance.depot;
                for &cust_idx in route {
                    let customer = &self.instance.customers[cust_idx];
                    total += self.instance.distance(last_node, customer);
                    last_node = customer;
                }
                total += self.instance.distance(last_node, &self.instance.depot);
            }
            total
        };

        let mut current_best = get_total_distance(&routes);
        let mut improving = true;
        while improving {
            improving = false;

            'outer: {
                // 1. Intra-route improvements
                for r in 0..routes.len() {
                    let len = routes[r].len();
                    if len < 2 { continue; }

                    // Exhaustive 2-opt within the route
                    for i in 0..len {
                        for j in (i+1)..len {
                            let mut test_routes = routes.clone();
                            test_routes[r][i..=j].reverse();
                            let dist = get_total_distance(&test_routes);
                            if dist < current_best {
                                current_best = dist;
                                routes = test_routes;
                                improving = true;
                                break 'outer;
                            }
                        }
                    }

                    // Exhaustive Swap within the route
                    for i in 0..len {
                        for j in (i+1)..len {
                            let mut test_routes = routes.clone();
                            test_routes[r].swap(i, j);
                            let dist = get_total_distance(&test_routes);
                            if dist < current_best {
                                current_best = dist;
                                routes = test_routes;
                                improving = true;
                                break 'outer;
                            }
                        }
                    }

                    // Exhaustive Relocate within the route
                    for i in 0..len {
                        for j in 0..len {
                            if i == j { continue; }
                            let mut test_routes = routes.clone();
                            let val = test_routes[r].remove(i);
                            let insert_pos = if j > i { j - 1 } else { j };
                            test_routes[r].insert(insert_pos, val);
                            let dist = get_total_distance(&test_routes);
                            if dist < current_best {
                                current_best = dist;
                                routes = test_routes;
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
                            let demand = self.instance.customers[cust_idx].demand;

                            // Check capacity constraint on target route
                            let load2: i32 = routes[r2].iter().map(|&idx| self.instance.customers[idx].demand).sum();
                            if load2 + demand > self.instance.capacity {
                                continue;
                            }

                            let len2 = routes[r2].len();
                            for j in 0..=len2 {
                                let mut test_routes = routes.clone();
                                let val = test_routes[r1].remove(i);
                                test_routes[r2].insert(j, val);
                                let dist = get_total_distance(&test_routes);
                                if dist < current_best {
                                    current_best = dist;
                                    routes = test_routes;
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
                            for j in 0..len2 {
                                let c1 = routes[r1][i];
                                let c2 = routes[r2][j];
                                let d1 = self.instance.customers[c1].demand;
                                let d2 = self.instance.customers[c2].demand;

                                // Check capacities if swapped
                                if load1 - d1 + d2 > self.instance.capacity || load2 - d2 + d1 > self.instance.capacity {
                                    continue;
                                }

                                let mut test_routes = routes.clone();
                                test_routes[r1][i] = c2;
                                test_routes[r2][j] = c1;
                                let dist = get_total_distance(&test_routes);
                                if dist < current_best {
                                    current_best = dist;
                                    routes = test_routes;
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
        };

        let evaluator = CvrpEvaluator { instance: instance.clone() };
        let candidate = CvrpCandidate {
            permutation: vec![0, 1, 2],
            last_mutation_op: None,
            last_mutation_radius: None,
            route_boundary_changes: None,
        };

        let res = evaluator.evaluate(&candidate);
        // Exceeds max_vehicles (2), so it is infeasible and penalized (returns 0 routes)
        assert!(res.eval.total_distance > 10000.0, "Should apply penalty to infeasible solution");
        assert_eq!(res.eval.num_vehicles, 0);

        instance.max_vehicles = Some(3);
        let evaluator2 = CvrpEvaluator { instance };
        let res2 = evaluator2.evaluate(&candidate);
        // Feasible within max_vehicles (3), so no penalty
        assert!(res2.eval.total_distance < 1000.0, "Should be feasible with 3 vehicles");
        assert_eq!(res2.eval.num_vehicles, 3);
    }
}
