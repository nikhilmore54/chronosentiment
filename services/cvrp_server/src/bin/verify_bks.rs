use cvrp::CvrpInstance;

fn get_node_by_id(instance: &CvrpInstance, id: usize) -> &cvrp::Node {
    if id == 1 {
        &instance.depot
    } else {
        instance.customers.iter().find(|c| c.id == id).unwrap()
    }
}

fn calc_route_distance_int(instance: &CvrpInstance, route: &Vec<usize>) -> i32 {
    if route.is_empty() {
        return 0;
    }
    let mut dist = 0;
    let depot = &instance.depot;

    let first = get_node_by_id(instance, route[0]);
    dist += instance.distance(depot, first).round() as i32;

    for i in 0..(route.len() - 1) {
        let a = get_node_by_id(instance, route[i]);
        let b = get_node_by_id(instance, route[i + 1]);
        dist += instance.distance(a, b).round() as i32;
    }

    let last = get_node_by_id(instance, *route.last().unwrap());
    dist += instance.distance(last, depot).round() as i32;
    dist
}

fn main() {
    let instance = CvrpInstance::a_n32_k5();

    let r_bks = vec![
        vec![15, 29, 12, 5, 24, 4, 3, 7],           // BKS1
        vec![27, 8, 14, 18, 20, 32, 22],            // BKS2
        vec![30, 19, 9, 10, 23, 16, 11, 26, 6, 21], // BKS3
        vec![13, 2, 17, 31],                        // BKS4
        vec![28, 25],                               // BKS5
    ];

    let dist_int: i32 = r_bks
        .iter()
        .map(|r| calc_route_distance_int(&instance, r))
        .sum();
    println!("Rounded Integer Distance (EUC_2D): {}", dist_int);
}
