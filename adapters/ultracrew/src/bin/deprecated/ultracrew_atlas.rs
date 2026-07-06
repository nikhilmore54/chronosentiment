use ultracrew::helpers::generate_scenario;
use ultracrew::optimization::{ScheduleGenome, ScheduleOptimizer};
use coralys_moga::traits::{FitnessEvaluator, GenomeFactory, CrossoverOperator, ObservedTransitionMetric, RegionIdentifier};
use coralys_moga::observatory::{ReachabilityProbe, ReachabilityObservation};
use std::collections::HashMap;
use rand::SeedableRng;
use rand::seq::SliceRandom;
use rand::Rng;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::io::Write;
use std::sync::Arc;

pub struct AssignmentEditDistance;

impl ObservedTransitionMetric<ScheduleGenome> for AssignmentEditDistance {
    fn magnitude(&self, source: &ScheduleGenome, result_after_repair: &ScheduleGenome) -> f64 {
        let mut diffs = 0;
        for (shift_id, worker_id) in &source.assignments {
            if result_after_repair.assignments.get(shift_id) != Some(worker_id) {
                diffs += 1;
            }
        }
        diffs as f64
    }
}

pub struct UltraCrewRegion;

impl RegionIdentifier<ScheduleGenome> for UltraCrewRegion {
    type RegionId = u64;

    fn region_of(&self, state: &ScheduleGenome) -> Self::RegionId {
        let mut sorted_keys: Vec<u64> = state.assignments.keys().cloned().collect();
        sorted_keys.sort();
        
        let mut hasher = DefaultHasher::new();
        for k in sorted_keys {
            hasher.write_u64(k);
            hasher.write_u64(*state.assignments.get(&k).unwrap());
        }
        hasher.finish()
    }
}

fn main() {
    let context = generate_scenario(40, 160, 5); // 40 workers, 160 shifts
    let optimizer = ScheduleOptimizer { context: context.clone() };
    
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    
    let metric = AssignmentEditDistance;
    let region_id = UltraCrewRegion;
    
    // We don't know the exact elite threshold yet, let's find the best in 10000 randoms
    println!("Collecting Parents...");
    let mut best_cand = optimizer.create(&mut rng);
    let mut best_fitness = f64::MIN;
    
    for _ in 0..10000 {
        let cand = optimizer.create(&mut rng);
        let eval = optimizer.evaluate(&cand);
        if eval.fitness > best_fitness {
            best_fitness = eval.fitness;
            best_cand = cand;
        }
    }
    
    println!("Root Fitness: {}", best_fitness);
    
    // Set elite threshold as 95% of the root fitness (since fitness is ~8000+ due to penalties)
    // Wait, fitness includes base 10000. So higher is better. 
    // Let's set elite_threshold slightly below the best_fitness to allow a small envelope.
    let elite_threshold = best_fitness - 50.0;
    
    let probe = ReachabilityProbe::new(
        &optimizer,
        &optimizer, // Using optimizer as local search (it does nothing)
        &metric,
        &region_id,
        elite_threshold,
    );
    
    let root_region = region_id.region_of(&best_cand);
    let num_samples = 500;

    let mut file = std::fs::File::create("ultracrew_atlas.csv").unwrap();
    writeln!(file, "transition_level,assignment_edit_distance,optimum_fitness,quality_delta,elite_retention,discovered_new_region").unwrap();

    let mut evaluate_child = |child: &mut ScheduleGenome, level: &str| {
        let obs = probe.evaluate_transition(&best_cand, child, best_fitness, &root_region);
        
        writeln!(
            file, "{},{},{},{},{},{}", 
            level, 
            obs.magnitude, 
            obs.target_fitness, 
            obs.fitness_delta, 
            obs.retained_elite,
            obs.discovered_new_region
        ).unwrap();
    };

    let shift_ids: Vec<u64> = context.shifts.iter().map(|s| s.id).collect();
    let worker_ids: Vec<u64> = context.workers.iter().map(|w| w.id).collect();

    let apply_k_changes = |child: &mut ScheduleGenome, k: usize, rng: &mut rand::rngs::StdRng| {
        let mut shifts_to_change = shift_ids.clone();
        shifts_to_change.shuffle(rng);
        for i in 0..k {
            let s_id = shifts_to_change[i];
            let new_w = *worker_ids.choose(rng).unwrap();
            child.assignments.insert(s_id, new_w);
        }
    };

    println!("Running L1: 1 Assignment");
    for _ in 0..num_samples {
        let mut child = best_cand.clone();
        apply_k_changes(&mut child, 1, &mut rng);
        evaluate_child(&mut child, "L1");
    }

    println!("Running L2: 2 Assignments");
    for _ in 0..num_samples {
        let mut child = best_cand.clone();
        apply_k_changes(&mut child, 2, &mut rng);
        evaluate_child(&mut child, "L2");
    }

    println!("Running L3: 5 Assignments");
    for _ in 0..num_samples {
        let mut child = best_cand.clone();
        apply_k_changes(&mut child, 5, &mut rng);
        evaluate_child(&mut child, "L3");
    }

    println!("Running L4: 10 Assignments");
    for _ in 0..num_samples {
        let mut child = best_cand.clone();
        apply_k_changes(&mut child, 10, &mut rng);
        evaluate_child(&mut child, "L4");
    }

    println!("Running L5: 20 Assignments");
    for _ in 0..num_samples {
        let mut child = best_cand.clone();
        apply_k_changes(&mut child, 20, &mut rng);
        evaluate_child(&mut child, "L5");
    }

    println!("Running L6: Random Crossover");
    for _ in 0..num_samples {
        let p2 = optimizer.create(&mut rng);
        let (mut c1, _) = optimizer.crossover(&best_cand, &p2, &mut rng);
        evaluate_child(&mut c1, "L6");
    }
    
    println!("UltraCrew Atlas Complete.");
}
