use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkNode {
    pub id: u64,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkLink {
    pub id: u64,
    pub from: u64,
    pub to: u64,
    pub metric: f64,
    pub capacity: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Network {
    pub directed: bool,
    pub multigraph: bool,
    pub nodes: Vec<NetworkNode>,
    pub links: Vec<NetworkLink>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Demand {
    pub s: u64,
    pub t: u64,
    pub v: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrafficMatrix {
    pub num_time_slots: usize,
    pub demands: Vec<Demand>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BudgetConstraint {
    pub t: usize,
    pub value: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Intervention {
    pub t: usize,
    pub links: Vec<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Scenario {
    pub max_segments: i32,
    pub budget: Vec<BudgetConstraint>,
    pub interventions: Vec<Intervention>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SrPath {
    pub d: usize,
    pub t: usize,
    pub w: Vec<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Solution {
    pub srpaths: Vec<SrPath>,
}
