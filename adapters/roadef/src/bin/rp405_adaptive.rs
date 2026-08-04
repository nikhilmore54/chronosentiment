/// rp405_adaptive -- RP-405: Adaptive Operator Selection (Hyper-Heuristic)
/// Classification: Research binary (RP-405).

use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Reverse;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::models::{Network, Solution, SrPath};
use roadef::evaluator::RoadefEvaluator;

const REWARD_FACTOR: f64 = 1.5;
const DECAY_FACTOR:  f64 = 0.9;
const DECAY_WINDOW:  usize = 5;
const MIN_WEIGHT:    f64 = 0.1;
const MAX_WEIGHT:    f64 = 10.0;
const NUM_OPERATORS: usize = 5;

struct Config { k: usize, iters: usize, seed: u64 }
impl Config {
    fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut k = 10usize; let mut iters = 50usize; let mut seed = 42u64;
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--k"     => { i += 1; if i < args.len() { k     = args[i].parse().unwrap_or(10); } }
                "--iters" => { i += 1; if i < args.len() { iters = args[i].parse().unwrap_or(50); } }
                "--seed"  => { i += 1; if i < args.len() { seed  = args[i].parse().unwrap_or(42); } }
                _ => {}
            }
            i += 1;
        }
        Config { k, iters, seed }
    }
}

struct Lcg { state: u64 }
impl Lcg {
    fn new(seed: u64) -> Self { Lcg { state: seed.wrapping_add(1) } }
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.state
    }
    fn next_usize(&mut self, n: usize) -> usize {
        if n == 0 { return 0; }
        (self.next_u64() % n as u64) as usize
    }
    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

fn dijkstra_path(net: &Network, src: u64, dst: u64, disabled: &HashSet<u64>, mults: &HashMap<u64,f64>) -> Option<Vec<u64>> {
    if src == dst { return Some(vec![src]); }
    let mut adj: HashMap<u64, Vec<(u64,f64)>> = HashMap::new();
    for l in &net.links {
        if disabled.contains(&l.id) { continue; }
        let m = mults.get(&l.id).copied().unwrap_or(1.0);
        adj.entry(l.from).or_default().push((l.to, l.metric * m));
    }
    let mut dist: HashMap<u64,u64> = HashMap::new();
    let mut prev: HashMap<u64,u64> = HashMap::new();
    let mut heap: BinaryHeap<(Reverse<u64>,u64)> = BinaryHeap::new();
    dist.insert(src, 0); heap.push((Reverse(0), src));
    while let Some((Reverse(cost), node)) = heap.pop() {
        if dist.get(&node).copied().unwrap_or(u64::MAX) < cost { continue; }
        if node == dst { break; }
        if let Some(nbrs) = adj.get(&node) {
            for &(next, metric) in nbrs {
                let nc = cost + (metric * 1000.0) as u64;
                if dist.get(&next).copied().unwrap_or(u64::MAX) > nc {
                    dist.insert(next, nc); prev.insert(next, node);
                    heap.push((Reverse(nc), next));
                }
            }
        }
    }
    if !dist.contains_key(&dst) { return None; }
    let mut path = vec![dst]; let mut cur = dst;
    while cur != src {
        if let Some(&p) = prev.get(&cur) { path.push(p); cur = p; } else { return None; }
    }
    path.reverse(); Some(path)
}

fn path_to_waypoints(fp: &[u64], max_seg: usize) -> Vec<u64> {
    if fp.len() <= 2 { return vec![]; }
    let wp: Vec<u64> = fp[1..fp.len()-1].to_vec();
    if max_seg > 0 && wp.len() + 1 > max_seg { wp[..max_seg-1].to_vec() } else { wp }
}

fn load_aware_path(net: &Network, src: u64, dst: u64, disabled: &HashSet<u64>, sat: &HashMap<u64,f64>, penalty: f64) -> Option<Vec<u64>> {
    if src == dst { return Some(vec![src]); }
    let mut adj: HashMap<u64, Vec<(u64,f64)>> = HashMap::new();
    for l in &net.links {
        if disabled.contains(&l.id) { continue; }
        let s = sat.get(&l.id).copied().unwrap_or(0.0);
        let p = if s >= 1.0 { 1e9 } else if s > 0.8 { penalty*(1.0/(1.0-s)-1.0)*10.0 } else { penalty*s };
        adj.entry(l.from).or_default().push((l.to, l.metric + p));
    }
    let mut dist: HashMap<u64,u64> = HashMap::new();
    let mut prev: HashMap<u64,u64> = HashMap::new();
    let mut heap: BinaryHeap<(Reverse<u64>,u64)> = BinaryHeap::new();
    dist.insert(src, 0); heap.push((Reverse(0), src));
    while let Some((Reverse(cost), node)) = heap.pop() {
        if dist.get(&node).copied().unwrap_or(u64::MAX) < cost { continue; }
        if node == dst { break; }
        if let Some(nbrs) = adj.get(&node) {
            for &(next, em) in nbrs {
                let nc = cost + (em * 1000.0) as u64;
                if dist.get(&next).copied().unwrap_or(u64::MAX) > nc {
                    dist.insert(next, nc); prev.insert(next, node);
                    heap.push((Reverse(nc), next));
                }
            }
        }
    }
    if !dist.contains_key(&dst) { return None; }
    let mut path = vec![dst]; let mut cur = dst;
    while cur != src {
        if let Some(&p) = prev.get(&cur) { path.push(p); cur = p; } else { return None; }
    }
    path.reverse(); Some(path)
}
fn load_rp403(set_dir: &str, inst: &str, ev: &RoadefEvaluator, nd: usize) -> Option<(Vec<SrPath>, f64)> {
    let path = format!("{}/setA-{}-srpaths-rp403.json", set_dir, inst);
    if !std::path::Path::new(&path).exists() { return None; }
    let content = std::fs::read_to_string(&path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let arr = json["srpaths"].as_array()?;
    let mut srpaths: Vec<SrPath> = Vec::new();
    for e in arr {
        let d = e["d"].as_u64()? as usize;
        let t = e["t"].as_u64()? as usize;
        let w: Vec<u64> = e["w"].as_array()?.iter().filter_map(|v| v.as_u64()).collect();
        if d < nd { srpaths.push(SrPath { d, t, w }); }
    }
    let sol = Solution { srpaths: srpaths.clone() };
    let r = ev.evaluate_solution(&sol);
    Some((srpaths, r.obj))
}

fn to_t0_map(srpaths: &[SrPath]) -> HashMap<usize, Vec<u64>> {
    let mut m = HashMap::new();
    for sp in srpaths { if sp.t == 0 { m.insert(sp.d, sp.w.clone()); } }
    m
}

fn build_srpaths(t0: &HashMap<usize,Vec<u64>>, t1: &HashMap<usize,Vec<u64>>, nd: usize, ns: usize) -> Vec<SrPath> {
    let mut v = Vec::new();
    for d in 0..nd {
        if let Some(w) = t0.get(&d) { if !w.is_empty() { v.push(SrPath { d, t: 0, w: w.clone() }); } }
        if ns > 1 { if let Some(w) = t1.get(&d) { if !w.is_empty() { v.push(SrPath { d, t: 1, w: w.clone() }); } } }
    }
    v
}

fn compute_sat(ev: &RoadefEvaluator, srpaths: &[SrPath], t: usize, cap: &HashMap<u64,f64>) -> HashMap<u64,f64> {
    let sol = Solution { srpaths: srpaths.to_vec() };
    let mut sat = HashMap::new();
    if let Some(loads) = ev.compute_loads(t, &sol) {
        for (id, flow) in &loads.arc_flows {
            let c = cap.get(id).copied().unwrap_or(1.0);
            sat.insert(*id, if c > 0.0 { flow/c } else { f64::INFINITY });
        }
    }
    sat
}

fn eval_full(ev: &RoadefEvaluator, t0: &HashMap<usize,Vec<u64>>, t1: &HashMap<usize,Vec<u64>>, nd: usize, ns: usize) -> (Vec<SrPath>, f64) {
    let sp = build_srpaths(t0, t1, nd, ns);
    let sol = Solution { srpaths: sp.clone() };
    let r = ev.evaluate_solution(&sol);
    (sp, r.obj)
}

fn is_better(a: f64, b: f64) -> bool {
    if a.is_finite() && !b.is_finite() { return true; }
    if !a.is_finite() && b.is_finite() { return false; }
    a < b
}

fn pad_random_seen(asgn: &HashMap<usize,Vec<u64>>, sel: &mut Vec<usize>, seen: &mut HashSet<usize>, k: usize, rng: &mut Lcg) {
    if sel.len() >= k { return; }
    let mut rem: Vec<usize> = asgn.keys().copied().filter(|d| !seen.contains(d)).collect();
    rem.sort();
    let need = k - sel.len();
    let actual = need.min(rem.len());
    for i in 0..actual { let j = i + rng.next_usize(rem.len()-i); rem.swap(i,j); }
    sel.extend_from_slice(&rem[..actual]);
}

fn pad_random(asgn: &HashMap<usize,Vec<u64>>, sel: &mut Vec<usize>, k: usize, rng: &mut Lcg) {
    if sel.len() >= k { return; }
    let mut seen: HashSet<usize> = sel.iter().copied().collect();
    pad_random_seen(asgn, sel, &mut seen, k, rng);
}
fn destroy_random(asgn: &HashMap<usize,Vec<u64>>, k: usize, rng: &mut Lcg) -> Vec<usize> {
    let mut all: Vec<usize> = asgn.keys().copied().collect();
    all.sort();
    let actual = k.min(all.len());
    for i in 0..actual { let j = i + rng.next_usize(all.len()-i); all.swap(i,j); }
    all[..actual].to_vec()
}

fn destroy_congestion(asgn: &HashMap<usize,Vec<u64>>, srpaths: &[SrPath], ev: &RoadefEvaluator, cap: &HashMap<u64,f64>, net: &Network, k: usize, rng: &mut Lcg) -> Vec<usize> {
    let sat = compute_sat(ev, srpaths, 0, cap);
    let mut node_sat: HashMap<u64,f64> = HashMap::new();
    for l in &net.links {
        let s = sat.get(&l.id).copied().unwrap_or(0.0);
        let e = node_sat.entry(l.from).or_insert(0.0); if s > *e { *e = s; }
        let e2 = node_sat.entry(l.to).or_insert(0.0); if s > *e2 { *e2 = s; }
    }
    let mut scores: Vec<(usize,f64)> = Vec::new();
    for sp in srpaths {
        if sp.t != 0 || !asgn.contains_key(&sp.d) { continue; }
        let score = sp.w.iter().map(|n| node_sat.get(n).copied().unwrap_or(0.0)).fold(0.0_f64, f64::max);
        scores.push((sp.d, score));
    }
    scores.sort_by(|a,b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut sel: Vec<usize> = scores.iter().take(k).map(|(d,_)| *d).collect();
    pad_random(asgn, &mut sel, k, rng);
    sel
}

fn destroy_highcost(asgn: &HashMap<usize,Vec<u64>>, srpaths: &[SrPath], demands: &[(usize,u64,u64,f64)], k: usize, rng: &mut Lcg) -> Vec<usize> {
    let mut vol_map: HashMap<usize,f64> = HashMap::new();
    for (d,_,_,v) in demands { vol_map.insert(*d, *v); }
    let mut scores: Vec<(usize,f64)> = Vec::new();
    let mut seen: HashSet<usize> = HashSet::new();
    for sp in srpaths {
        if sp.t != 0 || !asgn.contains_key(&sp.d) { continue; }
        let vol = vol_map.get(&sp.d).copied().unwrap_or(1.0);
        scores.push((sp.d, vol * (sp.w.len() as f64 + 1.0)));
        seen.insert(sp.d);
    }
    for (&d, _) in asgn {
        if !seen.contains(&d) {
            let vol = vol_map.get(&d).copied().unwrap_or(1.0);
            scores.push((d, vol));
        }
    }
    scores.sort_by(|a,b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut sel: Vec<usize> = scores.iter().take(k).map(|(d,_)| *d).collect();
    pad_random(asgn, &mut sel, k, rng);
    sel
}
fn destroy_bottleneck_link(asgn: &HashMap<usize,Vec<u64>>, srpaths: &[SrPath], ev: &RoadefEvaluator, cap: &HashMap<u64,f64>, net: &Network, demands: &[(usize,u64,u64,f64)], k: usize, rng: &mut Lcg) -> Vec<usize> {
    let sat = compute_sat(ev, srpaths, 0, cap);
    let mut links_sorted: Vec<(u64,u64,f64)> = net.links.iter()
        .map(|l| (l.from, l.to, sat.get(&l.id).copied().unwrap_or(0.0)))
        .collect();
    links_sorted.sort_by(|a,b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    let mut src_dst: HashMap<usize,(u64,u64)> = HashMap::new();
    for (d,s,t,_) in demands { src_dst.insert(*d, (*s,*t)); }
    let mut sel: Vec<usize> = Vec::new();
    let mut seen: HashSet<usize> = HashSet::new();
    'outer: for (from, to, _) in &links_sorted {
        for sp in srpaths {
            if sp.t != 0 || !asgn.contains_key(&sp.d) || seen.contains(&sp.d) { continue; }
            if let Some(&(src,dst)) = src_dst.get(&sp.d) {
                let mut full = vec![src]; full.extend_from_slice(&sp.w); full.push(dst);
                if full.windows(2).any(|w| w[0] == *from && w[1] == *to) {
                    sel.push(sp.d); seen.insert(sp.d);
                    if sel.len() >= k { break 'outer; }
                }
            }
        }
    }
    pad_random(asgn, &mut sel, k, rng);
    sel
}

fn destroy_ecmp_conflict(asgn: &HashMap<usize,Vec<u64>>, srpaths: &[SrPath], ev: &RoadefEvaluator, cap: &HashMap<u64,f64>, net: &Network, demands: &[(usize,u64,u64,f64)], k: usize, rng: &mut Lcg) -> Vec<usize> {
    let sat = compute_sat(ev, srpaths, 0, cap);
    let mut edge_to_lid: HashMap<(u64,u64),u64> = HashMap::new();
    for l in &net.links { edge_to_lid.insert((l.from, l.to), l.id); }
    let mut src_dst: HashMap<usize,(u64,u64)> = HashMap::new();
    for (d,s,t,_) in demands { src_dst.insert(*d, (*s,*t)); }
    let mut dls: HashMap<usize, HashSet<(u64,u64)>> = HashMap::new();
    for sp in srpaths {
        if sp.t != 0 || !asgn.contains_key(&sp.d) { continue; }
        if let Some(&(src,dst)) = src_dst.get(&sp.d) {
            let mut full = vec![src]; full.extend_from_slice(&sp.w); full.push(dst);
            let edges: HashSet<(u64,u64)> = full.windows(2).map(|w| (w[0],w[1])).collect();
            dls.insert(sp.d, edges);
        }
    }
    if dls.is_empty() { return destroy_random(asgn, k, rng); }
    let mut load_scores: Vec<(usize,f64)> = dls.iter().map(|(&d, edges)| {
        let s: f64 = edges.iter().map(|e| { let lid = edge_to_lid.get(e).copied().unwrap_or(0); sat.get(&lid).copied().unwrap_or(0.0) }).sum();
        (d, s)
    }).collect();
    load_scores.sort_by(|a,b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let pivot = load_scores[0].0;
    let pivot_edges = match dls.get(&pivot) { Some(e) => e.clone(), None => return destroy_random(asgn, k, rng) };
    let mut conflict: Vec<(usize,f64)> = dls.iter().filter(|(&d,_)| d != pivot).map(|(&d, edges)| {
        let s: f64 = edges.intersection(&pivot_edges).map(|e| { let lid = edge_to_lid.get(e).copied().unwrap_or(0); sat.get(&lid).copied().unwrap_or(0.0) }).sum();
        (d, s)
    }).collect();
    conflict.sort_by(|a,b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut sel = vec![pivot];
    let mut seen: HashSet<usize> = HashSet::from([pivot]);
    for (d, s) in &conflict {
        if sel.len() >= k { break; }
        if *s > 0.0 { sel.push(*d); seen.insert(*d); }
    }
    pad_random_seen(asgn, &mut sel, &mut seen, k, rng);
    sel
}
fn repair_rp401c(net: &Network, ev: &RoadefEvaluator, cur: &HashMap<usize,Vec<u64>>, destroyed: &[usize], demands: &[(usize,u64,u64,f64)], disabled: &HashSet<u64>, max_seg: usize, cap: &HashMap<u64,f64>, deadline: Instant) -> HashMap<usize,Vec<u64>> {
    let dset: HashSet<usize> = destroyed.iter().copied().collect();
    let mut asgn = cur.clone();
    for d in &dset { asgn.remove(d); }
    let mut partial: Vec<SrPath> = asgn.iter().filter(|(_,w)| !w.is_empty()).map(|(&d,w)| SrPath { d, t: 0, w: w.clone() }).collect();
    let mut sat = compute_sat(ev, &partial, 0, cap);
    for l in &net.links { sat.entry(l.id).or_insert(0.0); }
    let mut to_repair: Vec<(usize,u64,u64,f64)> = demands.iter().filter(|(d,_,_,_)| dset.contains(d)).copied().collect();
    to_repair.sort_by(|a,b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
    for (d, src, dst, _) in &to_repair {
        if Instant::now() >= deadline { break; }
        let fp = load_aware_path(net, *src, *dst, disabled, &sat, 100.0)
            .or_else(|| dijkstra_path(net, *src, *dst, disabled, &HashMap::new()));
        if let Some(fp) = fp {
            let wp = path_to_waypoints(&fp, max_seg);
            if !wp.is_empty() { partial.push(SrPath { d: *d, t: 0, w: wp.clone() }); }
            asgn.insert(*d, wp);
            let ps = Solution { srpaths: partial.clone() };
            if let Some(loads) = ev.compute_loads(0, &ps) {
                sat.clear();
                for (id, flow) in &loads.arc_flows {
                    let c = cap.get(id).copied().unwrap_or(1.0);
                    sat.insert(*id, if c > 0.0 { flow/c } else { f64::INFINITY });
                }
            }
        }
    }
    asgn
}

fn select_op(weights: &[f64; NUM_OPERATORS], rng: &mut Lcg) -> usize {
    let total: f64 = weights.iter().sum();
    let r = rng.next_f64() * total;
    let mut cum = 0.0;
    for (i, &w) in weights.iter().enumerate() {
        cum += w;
        if r <= cum { return i; }
    }
    NUM_OPERATORS - 1
}
fn main() -> anyhow::Result<()> {
    let cfg = Config::from_args();
    let set_dir = "adapters/roadef/repo/challenge-roadef-2026-main/setA";
    println!("RP-405 -- Adaptive Operator Selection (k={}, iters={}, seed={})", cfg.k, cfg.iters, cfg.seed);
    println!("Operators: [0=random, 1=congestion, 2=highcost, 3=bottleneck, 4=ecmp-conflict]");
    println!("{}", "=".repeat(110));
    println!("{:<10} {:>12} {:>12} {:>10} {:>8} {:>8} {:>30}", "Instance", "LNS obj", "RP-403 obj", "delta", "improved", "ms", "final_weights");
    println!("{}", "-".repeat(110));
    let mut improved_count = 0usize;
    let mut regressed_count = 0usize;
    let mut unchanged_count = 0usize;
    let mut finite_count = 0usize;
    let mut total_delta = 0.0f64;
    for instance_id in 1..=20 {
        let inst = format!("{:02}", instance_id);
        let net  = load_network(&format!("{}/setA-{}-net.json", set_dir, inst))?;
        let tm   = load_traffic_matrix(&format!("{}/setA-{}-tm.json", set_dir, inst))?;
        let sc   = load_scenario(&format!("{}/setA-{}-scenario.json", set_dir, inst))?;
        let nd = tm.demands.len();
        let ns = tm.num_time_slots;
        let max_seg = if sc.max_segments >= 0 { sc.max_segments as usize } else { 100 };
        let ev = RoadefEvaluator::new(&net, tm.clone(), sc.clone());
        let dis_t0: HashSet<u64> = sc.interventions.iter().filter(|i| i.t==0).flat_map(|i| i.links.iter().copied()).collect();
        let dis_t1: HashSet<u64> = sc.interventions.iter().filter(|i| i.t==1).flat_map(|i| i.links.iter().copied()).collect();
        let dis_both: HashSet<u64> = dis_t0.union(&dis_t1).copied().collect();
        let cap: HashMap<u64,f64> = net.links.iter().map(|l| (l.id, l.capacity)).collect();
        let demands: Vec<(usize,u64,u64,f64)> = tm.demands.iter().enumerate().map(|(i,d)| {
            let v0 = d.v[0]; let v1 = if d.v.len()>1 { d.v[1] } else { d.v[0] };
            (i, d.s, d.t, (v0+v1)/2.0)
        }).collect();
        let t_start = Instant::now();
        let deadline = t_start + std::time::Duration::from_secs(120);
        let (base_sp, base_obj) = match load_rp403(set_dir, &inst, &ev, nd) {
            Some(x) => x,
            None => {
                println!("{:<10} {:>12} {:>12} {:>10} {:>8} {:>8} {:>30}", format!("setA-{}", inst), "N/A","N/A","N/A","N/A",0,"N/A");
                continue;
            }
        };
        let mut best_t0 = to_t0_map(&base_sp);
        let mut best_t1: HashMap<usize,Vec<u64>> = HashMap::new();
        for sp in &base_sp { if sp.t==1 { best_t1.insert(sp.d, sp.w.clone()); } }
        let mut best_obj = base_obj;
        let mut iters_improved = 0usize;
        let mut rng = Lcg::new(cfg.seed.wrapping_add(instance_id as u64));
        let mut weights: [f64; NUM_OPERATORS] = [1.0; NUM_OPERATORS];
        for iter_idx in 0..cfg.iters {
            if Instant::now() >= deadline { break; }
            if iter_idx > 0 && iter_idx % DECAY_WINDOW == 0 {
                for w in weights.iter_mut() { *w = (*w * DECAY_FACTOR).max(MIN_WEIGHT); }
            }
            let current_srpaths = build_srpaths(&best_t0, &best_t1, nd, ns);
            let op_idx = select_op(&weights, &mut rng);
            let destroyed = match op_idx {
                0 => destroy_random(&best_t0, cfg.k, &mut rng),
                1 => destroy_congestion(&best_t0, &current_srpaths, &ev, &cap, &net, cfg.k, &mut rng),
                2 => destroy_highcost(&best_t0, &current_srpaths, &demands, cfg.k, &mut rng),
                3 => destroy_bottleneck_link(&best_t0, &current_srpaths, &ev, &cap, &net, &demands, cfg.k, &mut rng),
                4 => destroy_ecmp_conflict(&best_t0, &current_srpaths, &ev, &cap, &net, &demands, cfg.k, &mut rng),
                _ => destroy_random(&best_t0, cfg.k, &mut rng),
            };
            if destroyed.is_empty() { continue; }
            let repaired_t0 = repair_rp401c(&net, &ev, &best_t0, &destroyed, &demands, &dis_both, max_seg, &cap, deadline);
            let (_, repaired_obj) = eval_full(&ev, &repaired_t0, &best_t1, nd, ns);
            if is_better(repaired_obj, best_obj) {
                best_t0 = repaired_t0;
                best_obj = repaired_obj;
                iters_improved += 1;
                weights[op_idx] = (weights[op_idx] * REWARD_FACTOR).min(MAX_WEIGHT);
            }
        }
        let final_srpaths = build_srpaths(&best_t0, &best_t1, nd, ns);
        let result = ev.evaluate_solution(&Solution { srpaths: final_srpaths.clone() });
        let empty_result = ev.evaluate_solution(&Solution { srpaths: vec![] });
        let (output_srpaths, final_obj) = if !result.valid {
            (vec![], empty_result.obj)
        } else if result.obj.is_finite() && (empty_result.obj.is_infinite() || result.obj <= empty_result.obj) {
            (final_srpaths, result.obj)
        } else if result.obj.is_infinite() && empty_result.obj.is_finite() {
            (vec![], empty_result.obj)
        } else if result.obj.is_infinite() && empty_result.obj.is_infinite() {
            (final_srpaths, result.obj)
        } else {
            (vec![], empty_result.obj)
        };
        if final_obj.is_finite() { finite_count += 1; }
        let delta_str = if base_obj.is_infinite() && final_obj.is_infinite() {
            "both inf".to_string()
        } else if base_obj.is_finite() && final_obj.is_finite() {
            let d = final_obj - base_obj;
            total_delta += d;
            if d < -1e-6 { improved_count += 1; format!("{:.4}", d) }
            else if d > 1e-6 { regressed_count += 1; format!("+{:.4}", d) }
            else { unchanged_count += 1; "=".to_string() }
        } else { unchanged_count += 1; "mixed".to_string() };
        let elapsed_ms = t_start.elapsed().as_millis();
        let wstr = format!("[{:.2},{:.2},{:.2},{:.2},{:.2}]", weights[0], weights[1], weights[2], weights[3], weights[4]);
        println!("{:<10} {:>12.4} {:>12.4} {:>10} {:>8} {:>8} {:>30}",
            format!("setA-{}", inst),
            if final_obj.is_finite() { final_obj } else { f64::INFINITY },
            if base_obj.is_finite() { base_obj } else { f64::INFINITY },
            delta_str, if iters_improved > 0 { "yes" } else { "no" }, elapsed_ms, wstr);
        let out_path = format!("{}/setA-{}-srpaths-rp405-adaptive.json", set_dir, inst);
        let srpaths_json: Vec<serde_json::Value> = output_srpaths.iter().map(|sp| {
            serde_json::json!({ "d": sp.d, "t": sp.t, "w": sp.w })
        }).collect();
        let json_out = serde_json::json!({ "srpaths": srpaths_json });
        let mut f = File::create(&out_path)?;
        writeln!(f, "{}", serde_json::to_string_pretty(&json_out)?)?;
    }
    println!("{}", "=".repeat(110));
    println!("RP-405 vs RP-403: {} improved, {} regressed, {} unchanged", improved_count, regressed_count, unchanged_count);
    println!("Total improvement vs RP-403: {:.4}", total_delta);
    println!("Finite solutions: {}/20", finite_count);
    println!("Solution files written to {}/setA-*-srpaths-rp405-adaptive.json", set_dir);
    Ok(())
}
