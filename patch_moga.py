import re

with open("adapters/cvrp/src/moga_impl.rs", "r") as f:
    content = f.read()

# Insert the strategy enum and get_split_strategy function at the top of the file
import_statement = "use std::sync::OnceLock;\n\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum SplitStrategy {\n    Greedy,\n    PrinsDP,\n    DPFallbackToGreedy,\n}\n\npub fn get_split_strategy() -> SplitStrategy {\n    static STRATEGY: OnceLock<SplitStrategy> = OnceLock::new();\n    *STRATEGY.get_or_init(|| {\n        match std::env::var(\"CVRP_SPLIT_STRATEGY\").as_deref() {\n            Ok(\"GREEDY\") => SplitStrategy::Greedy,\n            Ok(\"PRINS_DP\") => SplitStrategy::PrinsDP,\n            _ => SplitStrategy::DPFallbackToGreedy,\n        }\n    })\n}\n\n"

if "SplitStrategy" not in content:
    content = import_statement + content

# In CvrpEvaluator::evaluate, find the DP split logic and wrap it
# Find:
#         // 2D DP Split
#         let mut v = vec![vec![f64::INFINITY; n + 1]; k_limit + 1];
# ...
#         let mut routes = Vec::new();
#         let total_distance;
# 
#         if best_cost < f64::INFINITY {
#             // Feasible under k_limit
# ...
#         } else {
#             // Infeasible under k_limit! Return empty routes and apply a massive penalty
#             routes = Vec::new();
#             total_distance = 1000000.0;
#         }

evaluate_logic = """
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
"""

# Now replace the old logic in CvrpEvaluator::evaluate
pattern = r"        // 2D DP Split\n        let mut v = vec!\[vec!\[f64::INFINITY; n \+ 1\]; k_limit \+ 1\];.*?        total_distance = 1000000\.0;\n        \}"
content = re.sub(pattern, evaluate_logic.strip(), content, flags=re.DOTALL)


# Now in get_routes
get_routes_logic = """
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
"""

pattern_get_routes = r"fn get_routes\(candidate: &CvrpCandidate, instance: &crate::CvrpInstance\) -> Vec<Vec<usize>> \{.*?    routes\n\}"
content = re.sub(pattern_get_routes, get_routes_logic.strip(), content, flags=re.DOTALL)


# Now in CvrpLocalSearch::improve
local_search_logic = """
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
"""

pattern_local_search = r"    fn improve\(&self, candidate: &mut CvrpCandidate, _model: &CvrpConstraintModel, _budget: &coralys_core::operators::OperatorBudget\) -> Result<bool, Self::Error> \{.*?        \} else \{.*?            \}\n        \}"
content = re.sub(pattern_local_search, local_search_logic.strip(), content, flags=re.DOTALL)


with open("adapters/cvrp/src/moga_impl.rs", "w") as f:
    f.write(content)
