use cvrp::CvrpInstance;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

fn get_node_demand(instance: &CvrpInstance, id: usize) -> i32 {
    if id == 1 {
        0
    } else {
        instance
            .customers
            .iter()
            .find(|c| c.id == id)
            .unwrap()
            .demand
    }
}

fn get_node_by_id<'a>(instance: &'a CvrpInstance, id: usize) -> &'a cvrp::Node {
    if id == 1 {
        &instance.depot
    } else {
        instance.customers.iter().find(|c| c.id == id).unwrap()
    }
}

fn calc_route_distance(instance: &CvrpInstance, route: &Vec<usize>) -> f64 {
    if route.is_empty() {
        return 0.0;
    }
    let mut dist = 0.0;
    let depot = &instance.depot;
    let first = get_node_by_id(instance, route[0]);
    dist += instance.distance(depot, first);
    for i in 0..(route.len() - 1) {
        let a = get_node_by_id(instance, route[i]);
        let b = get_node_by_id(instance, route[i + 1]);
        dist += instance.distance(a, b);
    }
    let last = get_node_by_id(instance, *route.last().unwrap());
    dist += instance.distance(last, depot);
    dist
}

fn get_canonical_edges(routes: &Vec<Vec<usize>>) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    let depot_id = 1;
    for r in routes {
        if r.is_empty() {
            continue;
        }
        let mut prev = depot_id;
        for &node in r {
            edges.push((prev.min(node), prev.max(node)));
            prev = node;
        }
        edges.push((prev.min(depot_id), prev.max(depot_id)));
    }
    edges.sort();
    edges
}

#[derive(Clone, Debug)]
struct State {
    routes: Vec<Vec<usize>>,
    dist: f64,
    peak_dist: f64,
    overlap_target: usize,
    overlap_source: usize,
    path: Vec<(String, f64, usize, usize, f64)>, // (Move, Dist, Target_Ov, Source_Ov, Peak)
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.peak_dist == other.peak_dist && self.overlap_target == other.overlap_target
    }
}
impl Eq for State {}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // Minimax priority: minimize peak_dist.
        // If tied, maximize overlap_target.
        // We use Reverse for min-heap.
        match other
            .peak_dist
            .partial_cmp(&self.peak_dist)
            .unwrap_or(Ordering::Equal)
        {
            Ordering::Equal => self.overlap_target.cmp(&other.overlap_target),
            other_cmp => other_cmp,
        }
    }
}

fn generate_neighbors(
    state: &State,
    instance: &CvrpInstance,
    diff_edges: &HashSet<(usize, usize)>,
    target_edges: &HashSet<(usize, usize)>,
    source_edges: &HashSet<(usize, usize)>,
) -> Vec<State> {
    let mut neighbors = Vec::new();
    let routes = &state.routes;

    // We will generate Relocate, Exchange, 2-opt, 2-opt*
    // To filter difference space: we find edges ADDED or REMOVED.
    // If ANY added/removed edge is in diff_edges, we keep it.

    let old_canon = get_canonical_edges(routes);
    let old_set: HashSet<_> = old_canon.into_iter().collect();

    let mut add_neighbor = |new_routes: Vec<Vec<usize>>, move_name: String| {
        // Check capacity
        for r in &new_routes {
            let mut load = 0;
            for &n in r {
                load += get_node_demand(instance, n);
            }
            if load > instance.capacity {
                return;
            }
        }

        let new_canon = get_canonical_edges(&new_routes);
        let new_set: HashSet<_> = new_canon.iter().cloned().collect();

        let mut involves_diff = false;
        for e in old_set.symmetric_difference(&new_set) {
            if diff_edges.contains(e) {
                involves_diff = true;
                break;
            }
        }

        if !involves_diff {
            return;
        }

        let overlap_tgt = new_set.intersection(target_edges).count();
        let dist: f64 = new_routes
            .iter()
            .map(|r| calc_route_distance(instance, r))
            .sum();
        let overlap_src = new_set.intersection(source_edges).count();
        let new_peak = state.peak_dist.max(dist);

        let mut new_path = state.path.clone();
        new_path.push((move_name, dist, overlap_tgt, overlap_src, new_peak));

        neighbors.push(State {
            routes: new_routes,
            dist,
            peak_dist: new_peak,
            overlap_target: overlap_tgt,
            overlap_source: overlap_src,
            path: new_path,
        });
    };

    let num_routes = routes.len();

    // 1. Relocate
    for r1 in 0..num_routes {
        for i in 0..routes[r1].len() {
            let node = routes[r1][i];
            for r2 in 0..num_routes {
                let limit = if r1 == r2 {
                    routes[r2].len()
                } else {
                    routes[r2].len() + 1
                };
                for j in 0..limit {
                    if r1 == r2 && (i == j || i + 1 == j) {
                        continue;
                    }
                    let mut new_routes = routes.clone();
                    new_routes[r1].remove(i);
                    if r1 == r2 && j > i {
                        new_routes[r2].insert(j - 1, node);
                    } else {
                        new_routes[r2].insert(j, node);
                    }
                    new_routes.retain(|r| !r.is_empty());
                    add_neighbor(new_routes, format!("Relocate({} to R{})", node, r2));
                }
            }
        }
    }

    // 2. Exchange
    for r1 in 0..num_routes {
        for i in 0..routes[r1].len() {
            for r2 in r1..num_routes {
                let start_j = if r1 == r2 { i + 1 } else { 0 };
                for j in start_j..routes[r2].len() {
                    let mut new_routes = routes.clone();
                    let temp = new_routes[r1][i];
                    new_routes[r1][i] = new_routes[r2][j];
                    new_routes[r2][j] = temp;
                    add_neighbor(
                        new_routes,
                        format!("Exchange({}, {})", routes[r1][i], routes[r2][j]),
                    );
                }
            }
        }
    }

    // 3. Intra-route 2-opt
    for r1 in 0..num_routes {
        let len = routes[r1].len();
        if len >= 2 {
            for i in 0..(len - 1) {
                for j in (i + 1)..len {
                    let mut new_routes = routes.clone();
                    new_routes[r1][i..=j].reverse();
                    add_neighbor(new_routes, format!("2-opt(R{}, {}, {})", r1, i, j));
                }
            }
        }
    }

    // 4. Inter-route 2-opt* (Cross-exchange)
    for r1 in 0..num_routes {
        for r2 in (r1 + 1)..num_routes {
            for i in 0..=routes[r1].len() {
                for j in 0..=routes[r2].len() {
                    let mut new_routes = routes.clone();
                    let tail1: Vec<usize> = new_routes[r1].drain(i..).collect();
                    let tail2: Vec<usize> = new_routes[r2].drain(j..).collect();
                    new_routes[r1].extend(tail2);
                    new_routes[r2].extend(tail1);
                    new_routes.retain(|r| !r.is_empty());
                    add_neighbor(new_routes, format!("2-opt*(R{}, R{})", r1, r2));
                }
            }
        }
    }

    neighbors
}

fn search_path(
    instance: &CvrpInstance,
    start_routes: Vec<Vec<usize>>,
    target_routes: Vec<Vec<usize>>,
    direction_name: &str,
) {
    println!(
        "\n=== Starting Bidirectional Search: {} ===",
        direction_name
    );

    let start_edges: HashSet<_> = get_canonical_edges(&start_routes).into_iter().collect();
    let target_edges: HashSet<_> = get_canonical_edges(&target_routes).into_iter().collect();

    let diff_edges: HashSet<_> = start_edges
        .symmetric_difference(&target_edges)
        .cloned()
        .collect();

    let start_dist: f64 = start_routes
        .iter()
        .map(|r| calc_route_distance(instance, r))
        .sum();
    let target_dist: f64 = target_routes
        .iter()
        .map(|r| calc_route_distance(instance, r))
        .sum();

    println!(
        "Start Dist: {:.4}, Target Dist: {:.4}",
        start_dist, target_dist
    );
    println!(
        "Start Overlap: {}",
        start_edges.intersection(&target_edges).count()
    );

    let initial_state = State {
        routes: start_routes,
        dist: start_dist,
        peak_dist: start_dist,
        overlap_target: start_edges.intersection(&target_edges).count(),
        overlap_source: start_edges.len(),
        path: vec![(
            "Start".to_string(),
            start_dist,
            start_edges.intersection(&target_edges).count(),
            start_edges.len(),
            start_dist,
        )],
    };

    let mut pq = BinaryHeap::new();
    pq.push(initial_state);

    let mut visited: HashMap<Vec<(usize, usize)>, f64> = HashMap::new(); // State -> Best Peak
    let mut expanded = 0;
    const MAX_EXPANSIONS: usize = 30000;

    while let Some(state) = pq.pop() {
        if state.overlap_target == 36 {
            println!("Target Reached!");
            println!("Minimum Peak Distance: {:.4}", state.peak_dist);
            println!("Path Length: {} steps", state.path.len() - 1);
            for (step, (m, d, tgt, src, p)) in state.path.iter().enumerate() {
                println!(
                    "{:3} | {:20} | Dist: {:.4} | Peak: {:.4} | TgtOv: {} | SrcOv: {}",
                    step, m, d, p, tgt, src
                );
            }
            return;
        }

        let canon = get_canonical_edges(&state.routes);
        if visited.contains_key(&canon) {
            continue;
        }
        visited.insert(canon, state.peak_dist);

        expanded += 1;
        if expanded % 1000 == 0 {
            println!(
                "Expanded {}, PQ size: {}, Current state overlap: {}, Current peak: {:.4}",
                expanded,
                pq.len(),
                state.overlap_target,
                state.peak_dist
            );
        }

        if expanded >= 5000 {
            println!(
                "Hit expansion limit. Target not reached. Best Overlap: {}, Best Peak: {:.4}",
                state.overlap_target, state.peak_dist
            );
            break;
        }

        let neighbors =
            generate_neighbors(&state, instance, &diff_edges, &target_edges, &start_edges);
        for n in neighbors {
            pq.push(n);
        }
    }
}

fn main() {
    let instance = CvrpInstance::a_n32_k5();

    let r_797 = vec![
        vec![15, 19, 9, 12, 5, 29, 24, 3, 4],
        vec![7, 18, 20, 32, 22, 14, 27],
        vec![21, 6, 26, 11, 16, 10, 23, 30],
        vec![13, 2, 8, 17, 31],
        vec![28, 25],
    ];

    let r_bks = vec![
        vec![15, 29, 12, 5, 24, 4, 3, 7],
        vec![27, 8, 14, 18, 20, 32, 22],
        vec![30, 19, 9, 10, 23, 16, 11, 26, 6, 21],
        vec![13, 2, 17, 31],
        vec![28, 25],
    ];

    search_path(&instance, r_bks.clone(), r_797.clone(), "BKS -> 797");
}
