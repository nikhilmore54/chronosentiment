use crate::graph::Digraph;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;

#[derive(Copy, Clone)]
struct State {
    cost: f64,
    position: u64,
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}
impl Eq for State {}

// BinaryHeap is a max-heap, we want a min-heap
impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // Notice that the we flip the ordering on costs.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other.cost.partial_cmp(&self.cost).unwrap_or(Ordering::Equal)
            .then_with(|| self.position.cmp(&other.position))
    }
}
impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct DijkstraResult {
    pub dist: HashMap<u64, f64>,
    pub preds: HashMap<u64, Vec<usize>>, // Array of arc indices
}

pub fn backward_dijkstra(graph: &Digraph, target: u64, disabled_arcs: &HashSet<u64>) -> DijkstraResult {
    let mut dist: HashMap<u64, f64> = HashMap::new();
    let mut preds: HashMap<u64, Vec<usize>> = HashMap::new();
    let mut heap = BinaryHeap::new();

    for &node in &graph.nodes {
        dist.insert(node, f64::INFINITY);
    }

    dist.insert(target, 0.0);
    heap.push(State { cost: 0.0, position: target });

    while let Some(State { cost, position }) = heap.pop() {
        if cost > *dist.get(&position).unwrap_or(&f64::INFINITY) {
            continue;
        }

        // Backward search: look at incoming arcs
        if let Some(in_edges) = graph.in_arcs.get(&position) {
            for &arc_idx in in_edges {
                let arc = &graph.arcs[arc_idx];
                if disabled_arcs.contains(&arc.id) {
                    continue;
                }
                
                let next = State { cost: cost + arc.metric, position: arc.from };
                let current_dist = *dist.get(&next.position).unwrap_or(&f64::INFINITY);

                if next.cost < current_dist {
                    heap.push(next);
                    dist.insert(next.position, next.cost);
                    preds.insert(next.position, vec![arc_idx]);
                } else if next.cost == current_dist {
                    preds.entry(next.position).or_default().push(arc_idx);
                }
            }
        }
    }

    DijkstraResult { dist, preds }
}

pub fn route_ecmp(
    graph: &Digraph,
    dijkstra_result: &DijkstraResult,
    source: u64,
    target: u64,
    flow: f64,
    arc_flow: &mut HashMap<u64, f64>
) -> bool {
    // If target not reached
    if *dijkstra_result.dist.get(&source).unwrap_or(&f64::INFINITY) == f64::INFINITY {
        return false;
    }

    let mut node_flow: HashMap<u64, f64> = HashMap::new();
    node_flow.insert(source, flow);

    // To process nodes in topological order of the shortest-path DAG,
    // we can process them in order of increasing distance from the target.
    // However, we are pushing flow FORWARD, so we process nodes in decreasing distance from the target.
    let mut nodes: Vec<u64> = dijkstra_result.dist.keys().cloned()
        .filter(|&k| *dijkstra_result.dist.get(&k).unwrap() != f64::INFINITY)
        .collect();
    
    // Sort descending by distance from target
    nodes.sort_by(|a, b| dijkstra_result.dist.get(b).unwrap().partial_cmp(dijkstra_result.dist.get(a).unwrap()).unwrap());

    for v in nodes {
        let f = *node_flow.get(&v).unwrap_or(&0.0);
        if f > 0.0 && v != target {
            if let Some(preds) = dijkstra_result.preds.get(&v) {
                let f_split = f / (preds.len() as f64);
                for &arc_idx in preds {
                    let arc = &graph.arcs[arc_idx];
                    *arc_flow.entry(arc.id).or_insert(0.0) += f_split;
                    *node_flow.entry(arc.to).or_insert(0.0) += f_split;
                }
            }
        }
    }

    true
}

pub fn expand_sr_path(
    graph: &Digraph,
    source: u64,
    target: u64,
    waypoints: &[u64],
    disabled_arcs: &HashSet<u64>,
    flow: f64,
    arc_flow: &mut HashMap<u64, f64>
) -> bool {
    // If no waypoints, just route source to target
    if waypoints.is_empty() {
        let res = backward_dijkstra(graph, target, disabled_arcs);
        return route_ecmp(graph, &res, source, target, flow, arc_flow);
    }

    // Otherwise, route through waypoints
    let mut path = vec![source];
    path.extend_from_slice(waypoints);
    path.push(target);

    for i in 0..path.len() - 1 {
        let u = path[i];
        let v = path[i+1];
        if u != v {
            let res = backward_dijkstra(graph, v, disabled_arcs);
            let ok = route_ecmp(graph, &res, u, v, flow, arc_flow);
            if !ok {
                return false;
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Network, NetworkNode, NetworkLink};

    fn make_test_graph() -> Digraph {
        let network = Network {
            directed: true,
            multigraph: false,
            nodes: vec![
                NetworkNode { id: 0, name: None },
                NetworkNode { id: 1, name: None },
                NetworkNode { id: 2, name: None },
                NetworkNode { id: 3, name: None },
            ],
            links: vec![
                NetworkLink { id: 10, from: 0, to: 1, metric: 10.0, capacity: 100.0 },
                NetworkLink { id: 11, from: 0, to: 2, metric: 10.0, capacity: 100.0 },
                NetworkLink { id: 12, from: 1, to: 3, metric: 10.0, capacity: 100.0 },
                NetworkLink { id: 13, from: 2, to: 3, metric: 10.0, capacity: 100.0 },
                NetworkLink { id: 14, from: 0, to: 3, metric: 30.0, capacity: 100.0 },
            ],
        };
        Digraph::new(&network)
    }

    #[test]
    fn test_ecmp_diamond() {
        let graph = make_test_graph();
        let disabled = HashSet::new();
        
        let mut arc_flow = HashMap::new();
        expand_sr_path(&graph, 0, 3, &[], &disabled, 100.0, &mut arc_flow);

        // Shortest paths to 3 from 0 are 0->1->3 (cost 20) and 0->2->3 (cost 20).
        // 0->3 has cost 30, so it shouldn't get flow.
        // Flow of 100 should be split evenly: 50 on 0->1, 50 on 0->2, 50 on 1->3, 50 on 2->3.
        assert_eq!(*arc_flow.get(&10).unwrap_or(&0.0), 50.0);
        assert_eq!(*arc_flow.get(&11).unwrap_or(&0.0), 50.0);
        assert_eq!(*arc_flow.get(&12).unwrap_or(&0.0), 50.0);
        assert_eq!(*arc_flow.get(&13).unwrap_or(&0.0), 50.0);
        assert_eq!(*arc_flow.get(&14).unwrap_or(&0.0), 0.0);
    }

    #[test]
    fn test_sr_path_waypoints() {
        let graph = make_test_graph();
        let disabled = HashSet::new();
        
        let mut arc_flow = HashMap::new();
        // Force flow to go 0 -> 2 -> 1 -> 3
        // Wait, 2 -> 1 doesn't exist.
        // Let's force 0 -> 1 -> 3, by using waypoint 1.
        expand_sr_path(&graph, 0, 3, &[1], &disabled, 100.0, &mut arc_flow);

        // Path: 0 -> 1 -> 3. So arc 10 and 12 get 100.
        assert_eq!(*arc_flow.get(&10).unwrap_or(&0.0), 100.0);
        assert_eq!(*arc_flow.get(&11).unwrap_or(&0.0), 0.0);
        assert_eq!(*arc_flow.get(&12).unwrap_or(&0.0), 100.0);
        assert_eq!(*arc_flow.get(&13).unwrap_or(&0.0), 0.0);
    }

    #[test]
    fn test_intervention_disconnect() {
        let graph = make_test_graph();
        let mut disabled = HashSet::new();
        disabled.insert(12); // disable 1->3
        disabled.insert(13); // disable 2->3

        let mut arc_flow = HashMap::new();
        // Now the only path to 3 is 0->3 (arc 14, cost 30).
        let ok = expand_sr_path(&graph, 0, 3, &[], &disabled, 100.0, &mut arc_flow);
        assert!(ok);
        assert_eq!(*arc_flow.get(&14).unwrap_or(&0.0), 100.0);

        // If we also disable 14, disconnected
        disabled.insert(14);
        let mut arc_flow2 = HashMap::new();
        let ok = expand_sr_path(&graph, 0, 3, &[], &disabled, 100.0, &mut arc_flow2);
        assert!(!ok); // Should fail to route
    }
}
