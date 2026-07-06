use cvrp::{CvrpInstance, CvrpCandidate, RadiusPolicy, CvrpGenomeFactory};
use cvrp::moga_impl::{CvrpEvaluator, CvrpMutator, CvrpLocalSearch};
use coralys_moga::traits::{FitnessEvaluator, MutationOperator, LocalSearchOperator, GenomeFactory, RegionIdentifier};
use std::collections::{HashSet, HashMap};
use rand::SeedableRng;
use rand::Rng;
use rand::seq::SliceRandom;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::io::Write;
use std::f64::consts::PI;

fn decode_routes(cand: &CvrpCandidate, instance: &CvrpInstance) -> Vec<Vec<usize>> {
    let mut routes = Vec::new();
    let mut current_route = Vec::new();
    let mut current_load = 0;
    for &cust_idx in &cand.permutation {
        let customer = &instance.customers[cust_idx];
        if current_load + customer.demand > instance.capacity {
            routes.push(current_route);
            current_route = Vec::new();
            current_load = 0;
        }
        current_route.push(cust_idx);
        current_load += customer.demand;
    }
    if !current_route.is_empty() { routes.push(current_route); }
    routes
}

fn extract_edges(routes: &Vec<Vec<usize>>, instance: &CvrpInstance) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    let depot_id = instance.depot.id;
    for r in routes {
        let mut prev = depot_id;
        for &c_idx in r {
            let c_id = instance.customers[c_idx].id;
            let n1 = prev.min(c_id); let n2 = prev.max(c_id);
            edges.push((n1, n2));
            prev = c_id;
        }
        edges.push((prev.min(depot_id), prev.max(depot_id)));
    }
    edges
}

fn extract_triplets(routes: &Vec<Vec<usize>>, instance: &CvrpInstance) -> Vec<(usize, usize, usize)> {
    let mut triplets = Vec::new();
    let depot_id = instance.depot.id;
    for r in routes {
        let mut full_route = vec![depot_id];
        for &c_idx in r {
            full_route.push(instance.customers[c_idx].id);
        }
        full_route.push(depot_id);
        for w in full_route.windows(3) {
            let mut t = [w[0], w[1], w[2]];
            if t[0] > t[2] { t.swap(0, 2); }
            triplets.push((t[0], t[1], t[2]));
        }
    }
    triplets
}

pub struct CvrpRegionIdentifier {
    pub instance: CvrpInstance,
}

impl RegionIdentifier<CvrpCandidate> for CvrpRegionIdentifier {
    type RegionId = u64;

    fn region_of(&self, state: &CvrpCandidate) -> Self::RegionId {
        let routes = decode_routes(state, &self.instance);
        let mut canonical = routes.clone();
        for r in &mut canonical { r.sort_unstable(); }
        canonical.sort_unstable();
        let mut s = DefaultHasher::new();
        canonical.hash(&mut s);
        s.finish()
    }
}

// ---------------------------------------------------------
// Solvers
// ---------------------------------------------------------

fn random_ls(instance: &CvrpInstance, ls: &CvrpLocalSearch, rng: &mut rand::rngs::StdRng) -> CvrpCandidate {
    let mut cand = CvrpCandidate { permutation: (0..instance.customers.len()).collect(), last_mutation_op: None, last_mutation_radius: None, route_boundary_changes: None };
    cand.permutation.shuffle(rng);
    ls.search(&mut cand);
    cand
}

fn sweep_ls(instance: &CvrpInstance, ls: &CvrpLocalSearch, rng: &mut rand::rngs::StdRng) -> CvrpCandidate {
    let cx = instance.depot.x;
    let cy = instance.depot.y;
    let mut angles: Vec<(usize, f64)> = (0..instance.customers.len()).map(|i| {
        let dx = instance.customers[i].x - cx;
        let dy = instance.customers[i].y - cy;
        (i, dy.atan2(dx))
    }).collect();
    
    let offset = rng.gen_range(-PI..PI);
    angles.sort_by(|a, b| {
        let mut a_ang = a.1 + offset; if a_ang > PI { a_ang -= 2.0 * PI; }
        let mut b_ang = b.1 + offset; if b_ang > PI { b_ang -= 2.0 * PI; }
        a_ang.partial_cmp(&b_ang).unwrap()
    });
    
    let mut cand = CvrpCandidate { permutation: angles.into_iter().map(|(i, _)| i).collect(), last_mutation_op: None, last_mutation_radius: None, route_boundary_changes: None };
    ls.search(&mut cand);
    cand
}

fn clarke_wright(instance: &CvrpInstance, rng: &mut rand::rngs::StdRng) -> CvrpCandidate {
    let n = instance.customers.len();
    let mut savings = Vec::new();
    let noise_scale = 0.5; // High noise to generate variety
    
    for i in 0..n {
        for j in (i+1)..n {
            let d_i0 = instance.distance(&instance.customers[i], &instance.depot);
            let d_j0 = instance.distance(&instance.customers[j], &instance.depot);
            let d_ij = instance.distance(&instance.customers[i], &instance.customers[j]);
            let s = d_i0 + d_j0 - d_ij;
            let noisy_s = s * (1.0 + rng.gen_range(-noise_scale..noise_scale));
            savings.push((i, j, noisy_s));
        }
    }
    savings.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    
    let mut routes: Vec<Vec<usize>> = (0..n).map(|i| vec![i]).collect();
    let mut loads: Vec<i32> = (0..n).map(|i| instance.customers[i].demand).collect();
    let mut route_of: Vec<usize> = vec![0; n];
    for i in 0..n { route_of[i] = i; }
    
    for (i, j, _) in savings {
        let r_i = route_of[i];
        let r_j = route_of[j];
        if r_i != r_j {
            let is_i_end = routes[r_i].last() == Some(&i);
            let is_j_start = routes[r_j].first() == Some(&j);
            let is_i_start = routes[r_i].first() == Some(&i);
            let is_j_end = routes[r_j].last() == Some(&j);
            
            if loads[r_i] + loads[r_j] <= instance.capacity {
                if is_i_end && is_j_start {
                    let mut rj_clone = routes[r_j].clone();
                    routes[r_i].append(&mut rj_clone);
                    loads[r_i] += loads[r_j];
                    for &node in &routes[r_i] { route_of[node] = r_i; }
                    routes[r_j].clear();
                } else if is_j_end && is_i_start {
                    let mut ri_clone = routes[r_i].clone();
                    routes[r_j].append(&mut ri_clone);
                    loads[r_j] += loads[r_i];
                    for &node in &routes[r_j] { route_of[node] = r_j; }
                    routes[r_i].clear();
                } else if is_i_end && is_j_end {
                    let mut rj_clone = routes[r_j].clone();
                    rj_clone.reverse();
                    routes[r_i].append(&mut rj_clone);
                    loads[r_i] += loads[r_j];
                    for &node in &routes[r_i] { route_of[node] = r_i; }
                    routes[r_j].clear();
                } else if is_i_start && is_j_start {
                    let mut ri_clone = routes[r_i].clone();
                    ri_clone.reverse();
                    ri_clone.append(&mut routes[r_j].clone());
                    routes[r_i] = ri_clone;
                    loads[r_i] += loads[r_j];
                    for &node in &routes[r_i] { route_of[node] = r_i; }
                    routes[r_j].clear();
                }
            }
        }
    }
    
    let mut perm = Vec::new();
    for r in routes {
        perm.extend(r);
    }
    CvrpCandidate { permutation: perm, last_mutation_op: None, last_mutation_radius: None, route_boundary_changes: None }
}

fn moga_step(instance: &CvrpInstance, cand: &mut CvrpCandidate, ls: &CvrpLocalSearch, rng: &mut rand::rngs::StdRng) {
    let mutator = CvrpMutator::new(instance.clone(), RadiusPolicy::Control);
    mutator.mutate(cand, rng);
    ls.search(cand);
}

fn main() {
    let instance = CvrpInstance::a_n32_k5();
    let evaluator = CvrpEvaluator { instance: instance.clone() };
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let ls = CvrpLocalSearch { instance: instance.clone() };
    let region_id = CvrpRegionIdentifier { instance: instance.clone() };
    
    let mut file = std::fs::File::create("m18_structural_data.csv").unwrap();
    writeln!(file, "solver,basin_hash,distance,edges,triplets").unwrap();
    
    let target_samples = 1000;
    
    let mut process = |cand: CvrpCandidate, solver: &str| {
        let dist = evaluator.evaluate(&cand).eval.total_distance;
        let basin = region_id.region_of(&cand);
        let routes = decode_routes(&cand, &instance);
        let edges = extract_edges(&routes, &instance);
        let triplets = extract_triplets(&routes, &instance);
        
        let edge_str = edges.iter().map(|(u,v)| format!("{}-{}", u, v)).collect::<Vec<_>>().join("|");
        let trip_str = triplets.iter().map(|(u,v,w)| format!("{}-{}-{}", u, v, w)).collect::<Vec<_>>().join("|");
        
        writeln!(file, "{},{},{:.4},{},{}", solver, basin, dist, edge_str, trip_str).unwrap();
    };

    println!("Generating Random + LS...");
    for _ in 0..target_samples {
        let cand = random_ls(&instance, &ls, &mut rng);
        process(cand, "Random");
    }

    println!("Generating Sweep + LS...");
    for _ in 0..target_samples {
        let cand = sweep_ls(&instance, &ls, &mut rng);
        process(cand, "Sweep");
    }

    println!("Generating Clarke-Wright (No LS)...");
    for _ in 0..target_samples {
        let cand = clarke_wright(&instance, &mut rng);
        process(cand, "ClarkeWright");
    }

    println!("Generating MOGA (Evolutionary Walk)...");
    let mut current_cand = random_ls(&instance, &ls, &mut rng);
    let mut moga_samples = 0;
    while moga_samples < target_samples {
        moga_step(&instance, &mut current_cand, &ls, &mut rng);
        // We accept all mutations just to sample the MOGA landscape, or we can use Metropolis-Hastings.
        // Let's use simple SA to stay in elite regions.
        let dist = evaluator.evaluate(&current_cand).eval.total_distance;
        if dist < 850.0 || rng.gen_bool(0.1) {
            process(current_cand.clone(), "MOGA");
            moga_samples += 1;
        } else {
            // Revert
            current_cand = sweep_ls(&instance, &ls, &mut rng);
        }
    }

    println!("M18 Structural Audit Complete.");
}
