use std::fs::File;
use std::io::Write;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
struct BitGenome {
    bits: Vec<bool>,
    is_vault: bool,
}

fn dist(bits: &[bool], start: usize, end: usize) -> usize {
    let mut d = 0;
    for &b in &bits[start..end] { if !b { d += 1; } }
    d
}

fn dist_x(bits: &[bool]) -> usize { dist(bits, 10, 31) }
fn dist_y(bits: &[bool]) -> usize { dist(bits, 40, 61) }
fn dist_z(bits: &[bool]) -> usize { dist(bits, 70, 91) }

fn mutate(genome: &mut BitGenome, rng: &mut StdRng) {
    let rate = 1.0 / genome.bits.len() as f64;
    for b in genome.bits.iter_mut() { if rng.gen_bool(rate) { *b = !*b; } }
}

fn crossover(p1: &BitGenome, p2: &BitGenome, rng: &mut StdRng) -> (BitGenome, BitGenome) {
    let mut c1_bits = Vec::with_capacity(p1.bits.len());
    let mut c2_bits = Vec::with_capacity(p2.bits.len());
    for i in 0..p1.bits.len() {
        if rng.gen_bool(0.5) { c1_bits.push(p1.bits[i]); c2_bits.push(p2.bits[i]); }
        else { c1_bits.push(p2.bits[i]); c2_bits.push(p1.bits[i]); }
    }
    
    let is_v = p1.is_vault || p2.is_vault;
    (BitGenome { bits: c1_bits, is_vault: is_v }, BitGenome { bits: c2_bits, is_vault: is_v })
}

fn evaluate(genome: &BitGenome, gen: usize, penalty: f64, target: char) -> f64 {
    let mut base_score = 0.0;
    for i in 0..10 { if genome.bits[i] { base_score += 1.0; } }
    for i in 31..40 { if genome.bits[i] { base_score += 1.0; } }
    for i in 61..70 { if genome.bits[i] { base_score += 1.0; } }
    for i in 91..100 { if genome.bits[i] { base_score += 1.0; } }
    
    let hx = dist_x(&genome.bits) == 0; let hy = dist_y(&genome.bits) == 0; let hz = dist_z(&genome.bits) == 0;
    if gen <= 500 {
        if hx { base_score -= penalty; } if hy { base_score -= penalty; } if hz { base_score -= penalty; }
    } else {
        if hx { if target == 'X' { base_score += 1000.0; } else { base_score -= penalty; } }
        if hy { if target == 'Y' { base_score += 1000.0; } else { base_score -= penalty; } }
        if hz { if target == 'Z' { base_score += 1000.0; } else { base_score -= penalty; } }
    }
    base_score
}

fn tournament(pop: &[(BitGenome, f64)], k: usize, rng: &mut StdRng) -> usize {
    let mut best_idx = rng.gen_range(0..pop.len());
    for _ in 1..k {
        let idx = rng.gen_range(0..pop.len());
        if pop[idx].1 > pop[best_idx].1 { best_idx = idx; }
    }
    best_idx
}

struct RunMetrics {
    target: char, recovery_time: Option<usize>, vault_hit: bool, max_vault_overlap: f64,
    best_non_vault_fitness: f64,
}

fn step_ga(pop: &Vec<(BitGenome, f64)>, rng: &mut StdRng, k: usize) -> Vec<BitGenome> {
    let mut next_pop = Vec::with_capacity(pop.len());
    while next_pop.len() < pop.len() {
        if rng.gen_bool(0.9) { 
            let p1 = tournament(pop, k, rng); let p2 = tournament(pop, k, rng);
            let (mut c1, mut c2) = crossover(&pop[p1].0, &pop[p2].0, rng);
            mutate(&mut c1, rng); mutate(&mut c2, rng);
            next_pop.push(c1);
            if next_pop.len() < pop.len() { next_pop.push(c2); }
        } else {
            let p1 = tournament(pop, k, rng);
            let mut c1 = pop[p1].0.clone();
            mutate(&mut c1, rng);
            next_pop.push(c1);
        }
    }
    next_pop
}

fn step_sa(pop: &Vec<(BitGenome, f64)>, rng: &mut StdRng, gen: usize, penalty: f64, target: char) -> Vec<BitGenome> {
    let mut next_pop = Vec::with_capacity(pop.len());
    for (g, score) in pop {
        let mut best_cand = g.clone(); let mut best_fit = *score;
        let mut curr_cand = best_cand.clone(); let mut curr_fit = best_fit;
        let mut t = 100.0; let alpha = 0.95;
        for _ in 0..50 {
            let mut neighbor = curr_cand.clone();
            mutate(&mut neighbor, rng);
            let n_fit = evaluate(&neighbor, gen, penalty, target);
            if n_fit - curr_fit > 0.0 || rng.gen_range(0.0..1.0) < ((n_fit - curr_fit) / t).exp() {
                curr_cand = neighbor; curr_fit = n_fit;
                if curr_fit > best_fit { best_fit = curr_fit; best_cand = curr_cand.clone(); }
            }
            t *= alpha;
        }
        next_pop.push(best_cand);
    }
    next_pop
}

fn run_simulation(seed: u64, mode: &str, penalty: f64) -> RunMetrics {
    let mut rng = StdRng::seed_from_u64(seed);
    let target = match rng.gen_range(0..3) { 0 => 'X', 1 => 'Y', _ => 'Z' };
    
    let pop_size = 100;
    let mut pop = Vec::new();
    for i in 0..pop_size {
        let mut bits = vec![false; 100];
        for b in bits.iter_mut() { *b = rng.gen_bool(0.5); }
        
        if i < 33 { for j in 10..31 { bits[j] = true; } }
        else if i < 66 { for j in 40..61 { bits[j] = true; } }
        else { for j in 70..91 { bits[j] = true; } }
        
        pop.push(BitGenome { bits, is_vault: false });
    }
    
    let mut vault = Vec::new();
    let mut injected_vault = Vec::new();
    let mut recovery_time = None;
    let mut vault_hit = false;
    let mut max_vault_overlap = 0.0;

    let use_vault = mode.contains("Vault");
    let is_random_vault = mode.contains("RandomVault");
    let is_ga = mode.contains("GA");
    let is_sa = mode.ends_with("SA"); 
    let k = 4; // Moderate selection pressure

    for gen in 0..=1000 {
        let mut evals = Vec::new();
        for g in &pop { evals.push((g.clone(), evaluate(g, gen, penalty, target))); }
        evals.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
        
        if use_vault && gen <= 500 {
            for (g, _) in &evals {
                if dist_x(&g.bits) <= 10 || dist_y(&g.bits) <= 10 || dist_z(&g.bits) <= 10 {
                    vault.push(g.clone());
                }
            }
        }
        
        if gen == 501 && use_vault && !vault.is_empty() {
            let mut selected = Vec::new();
            if is_random_vault {
                let mut temp_v = vault.clone();
                for _ in 0..10.min(temp_v.len()) {
                    let idx = rng.gen_range(0..temp_v.len());
                    selected.push(temp_v.remove(idx));
                }
            } else {
                vault.sort_by_key(|g| match target { 'X' => dist_x(&g.bits), 'Y' => dist_y(&g.bits), _ => dist_z(&g.bits) });
                for i in 0..10.min(vault.len()) { selected.push(vault[i].clone()); }
            }
            
            for i in 0..selected.len() {
                let mut v_genome = selected[i].clone();
                v_genome.is_vault = true;
                injected_vault.push(v_genome.clone());
                let worst_idx = pop_size - 1 - i;
                let new_fitness = evaluate(&v_genome, gen, penalty, target);
                evals[worst_idx] = (v_genome, new_fitness);
            }
            evals.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
        }
        
        if gen > 500 && recovery_time.is_none() && evals[0].1 > 1000.0 { 
            recovery_time = Some(gen - 500); 
            vault_hit = evals[0].0.is_vault;
            
            // Calc overlap
            let mut max_o = 0.0;
            for v in &injected_vault {
                let mut overlap = 0;
                let target_range = match target { 'X' => 10..31, 'Y' => 40..61, _ => 70..91 };
                for i in target_range {
                    if evals[0].0.bits[i] == v.bits[i] { overlap += 1; }
                }
                let o_pct = overlap as f64 / 21.0;
                if o_pct > max_o { max_o = o_pct; }
            }
            max_vault_overlap = max_o;
        }
        if gen == 1000 { break; }

        pop = step_ga(&evals, &mut rng, k);
    }
    
    // Calc BestNonVaultFitness
    let mut b_nv_f = -9999.0;
    for g in &pop {
        if !g.is_vault {
            let f = evaluate(g, 1000, penalty, target);
            if f > b_nv_f { b_nv_f = f; }
        }
    }

    RunMetrics { target, recovery_time, vault_hit, max_vault_overlap, best_non_vault_fitness: b_nv_f }
}

fn main() {
    let seeds = 100..150; // 50 seeds for good stats
    let modes = vec!["GA", "RandomVault+GA", "Vault+GA"];
    let penalty = 500.0;
    
    let mut res_file = File::create("m23a11_vault.csv").unwrap();
    writeln!(res_file, "mode,seed,target,recovery,vault_hit,vault_overlap,best_nv_fitness").unwrap();

    for mode in &modes {
        println!("    Mode: {}", mode);
        for seed in seeds.clone() {
            let metrics = run_simulation(seed, mode, penalty);
            writeln!(res_file, "{},{},{},{},{},{:.4},{:.2}",
                mode, seed, metrics.target,
                metrics.recovery_time.unwrap_or(999),
                metrics.vault_hit, metrics.max_vault_overlap,
                metrics.best_non_vault_fitness
            ).unwrap();
        }
    }
    println!("M23A.11 Benchmark Complete.");
}
