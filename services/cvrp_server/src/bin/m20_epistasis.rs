use cvrp::CvrpInstance;
use std::collections::HashSet;

fn get_node_by_id(instance: &CvrpInstance, id: usize) -> &cvrp::Node {
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

fn get_edges(routes: &Vec<Vec<usize>>) -> HashSet<(usize, usize)> {
    let mut edges = HashSet::new();
    let depot_id = 1;
    for r in routes {
        let mut prev = depot_id;
        for &node in r {
            edges.insert((prev.min(node), prev.max(node)));
            prev = node;
        }
        edges.insert((prev.min(depot_id), prev.max(depot_id)));
    }
    edges
}

fn evaluate_config(
    name: &str,
    routes: &Vec<Vec<usize>>,
    instance: &CvrpInstance,
    bks_edges: &HashSet<(usize, usize)>,
) {
    let mut total_distance = 0.0;
    let mut capacity_violations = 0;

    let mut seen_nodes = HashSet::new();
    let mut duplicate_nodes = HashSet::new();

    for r in routes {
        total_distance += calc_route_distance(instance, r);
        let mut load = 0;
        for &n in r {
            load += get_node_by_id(instance, n).demand;
            if !seen_nodes.insert(n) {
                duplicate_nodes.insert(n);
            }
        }
        if load > instance.capacity {
            capacity_violations += 1;
        }
    }

    let mut missing_nodes = Vec::new();
    for i in 2..=32 {
        if !seen_nodes.contains(&i) {
            missing_nodes.push(i);
        }
    }

    let edges = get_edges(routes);
    let overlap = edges.intersection(bks_edges).count();

    println!(
        "| {:8} | {:.4} | {:7} | {:9} | {:8} | {:11} |",
        name,
        total_distance,
        missing_nodes.len(),
        duplicate_nodes.len(),
        capacity_violations,
        overlap
    );
}

fn main() {
    let instance = CvrpInstance::a_n32_k5();

    // Level 1: Anchor Distances
    let r_797 = vec![
        vec![15, 19, 9, 12, 5, 29, 24, 3, 4], // R1
        vec![7, 18, 20, 32, 22, 14, 27],      // R2
        vec![21, 6, 26, 11, 16, 10, 23, 30],  // R3
        vec![13, 2, 8, 17, 31],               // R4
        vec![28, 25],                         // R5
    ];

    let r_bks = vec![
        vec![15, 29, 12, 5, 24, 4, 3, 7],           // BKS1
        vec![27, 8, 14, 18, 20, 32, 22],            // BKS2
        vec![30, 19, 9, 10, 23, 16, 11, 26, 6, 21], // BKS3
        vec![13, 2, 17, 31],                        // BKS4
        vec![28, 25],                               // BKS5
    ];

    let dist_797: f64 = r_797
        .iter()
        .map(|r| calc_route_distance(&instance, r))
        .sum();
    let dist_bks: f64 = r_bks
        .iter()
        .map(|r| calc_route_distance(&instance, r))
        .sum();

    let edges_797 = get_edges(&r_797);
    let edges_bks = get_edges(&r_bks);

    let shared = edges_797.intersection(&edges_bks).count();
    let diff = edges_797.symmetric_difference(&edges_bks).count();

    println!("### Level 1: Verification");
    println!("797 Distance: {:.4}", dist_797);
    println!("BKS Distance: {:.4}", dist_bks);
    println!("Shared Edges: {}", shared);
    println!("Diff Edges: {}", diff);

    // Level 2: Segment Audit
    println!("\n### Level 2: Segment Audit");
    let c1_797 = vec![15, 19, 9, 12, 5, 29, 24, 3, 4];
    let c1_bks = vec![15, 29, 12, 5, 24, 4, 3, 7];
    println!(
        "Cluster 1 - 797 Segment Intrinsic Cost: {:.4}",
        calc_route_distance(&instance, &c1_797)
    );
    println!(
        "Cluster 1 - BKS Segment Intrinsic Cost: {:.4}",
        calc_route_distance(&instance, &c1_bks)
    );

    let c2_797 = vec![7, 18, 20, 32, 22, 14, 27];
    let c2_bks = vec![27, 8, 14, 18, 20, 32, 22];
    println!(
        "Cluster 2 - 797 Segment Intrinsic Cost: {:.4}",
        calc_route_distance(&instance, &c2_797)
    );
    println!(
        "Cluster 2 - BKS Segment Intrinsic Cost: {:.4}",
        calc_route_distance(&instance, &c2_bks)
    );

    let c3_797 = vec![21, 6, 26, 11, 16, 10, 23, 30];
    let c3_bks = vec![30, 19, 9, 10, 23, 16, 11, 26, 6, 21];
    println!(
        "Cluster 3 - 797 Segment Intrinsic Cost: {:.4}",
        calc_route_distance(&instance, &c3_797)
    );
    println!(
        "Cluster 3 - BKS Segment Intrinsic Cost: {:.4}",
        calc_route_distance(&instance, &c3_bks)
    );

    // Level 3: Full Epistasis
    println!("\n### Level 3: Full Epistasis Matrix");
    println!("| Config   | Distance | Missing | Duplicate | Capacity | BKS Overlap |");
    println!("|----------|----------|---------|-----------|----------|-------------|");

    let configs = vec![
        ("Base", vec![0, 0, 0]),
        ("C1", vec![1, 0, 0]),
        ("C2", vec![0, 1, 0]),
        ("C3", vec![0, 0, 1]),
        ("C1+C2", vec![1, 1, 0]),
        ("C1+C3", vec![1, 0, 1]),
        ("C2+C3", vec![0, 1, 1]),
        ("All", vec![1, 1, 1]),
    ];

    for (name, swaps) in configs {
        let mut routes = Vec::new();
        routes.push(if swaps[0] == 1 {
            r_bks[0].clone()
        } else {
            r_797[0].clone()
        });
        routes.push(if swaps[1] == 1 {
            r_bks[1].clone()
        } else {
            r_797[1].clone()
        });
        routes.push(if swaps[2] == 1 {
            r_bks[2].clone()
        } else {
            r_797[2].clone()
        });
        routes.push(if swaps[1] == 1 {
            r_bks[3].clone()
        } else {
            r_797[3].clone()
        }); // C2 also affects R4
        routes.push(r_797[4].clone()); // R5 is invariant

        evaluate_config(name, &routes, &instance, &edges_bks);
    }
}
