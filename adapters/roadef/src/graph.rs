use crate::models::Network;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Arc {
    pub id: u64,
    pub from: u64,
    pub to: u64,
    pub metric: f64,
    pub capacity: f64,
}

pub struct Digraph {
    pub nodes: Vec<u64>,
    pub arcs: Vec<Arc>,
    pub out_arcs: HashMap<u64, Vec<usize>>,
    pub in_arcs: HashMap<u64, Vec<usize>>,
    pub arc_by_id: HashMap<u64, usize>,
}

impl Digraph {
    pub fn new(network: &Network) -> Self {
        let mut nodes = Vec::new();
        for node in &network.nodes {
            nodes.push(node.id);
        }

        let mut arcs = Vec::new();
        let mut out_arcs: HashMap<u64, Vec<usize>> = HashMap::new();
        let mut in_arcs: HashMap<u64, Vec<usize>> = HashMap::new();
        let mut arc_by_id = HashMap::new();

        for (i, link) in network.links.iter().enumerate() {
            let arc = Arc {
                id: link.id,
                from: link.from,
                to: link.to,
                metric: link.metric,
                capacity: link.capacity,
            };
            out_arcs.entry(link.from).or_default().push(i);
            in_arcs.entry(link.to).or_default().push(i);
            arc_by_id.insert(link.id, i);
            arcs.push(arc);
        }

        Self {
            nodes,
            arcs,
            out_arcs,
            in_arcs,
            arc_by_id,
        }
    }
}
