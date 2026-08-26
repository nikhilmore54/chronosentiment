use coralys_matching::{AssignmentSolver, BipartiteMatchingSolver};
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::time::Instant;

#[test]
fn test_assignment_scalability_benchmarks() {
    let sizes = vec![100, 500, 1000, 5000];
    let mut rng = StdRng::seed_from_u64(42);

    println!("\n=======================================================");
    println!("     BIPARTITE MATCHING BENCHMARK (SCALABILITY)       ");
    println!("=======================================================");
    println!(" Workers | Demands | Matched | Time (ms) ");
    println!("---------|---------|---------|-----------");

    for size in sizes {
        // Generate synthetic workers with random skills
        let skills = vec![
            "HeadNurse".to_string(),
            "Nurse".to_string(),
            "Caretaker".to_string(),
            "Trainee".to_string(),
        ];
        let mut workers = Vec::new();
        for id in 0..size {
            let num_skills = rng.gen_range(1..=3);
            let mut worker_skills = Vec::new();
            for _ in 0..num_skills {
                let s = skills[rng.gen_range(0..skills.len())].clone();
                if !worker_skills.contains(&s) {
                    worker_skills.push(s);
                }
            }
            workers.push((id, worker_skills));
        }

        // Generate synthetic demands (equal to ~90% of size)
        let demand_size = (size as f64 * 0.9) as usize;
        let mut demands = Vec::new();
        let mut remaining = demand_size;
        for (i, skill) in skills.iter().enumerate() {
            let count = if i == skills.len() - 1 {
                remaining
            } else {
                let c = rng.gen_range(0..=remaining);
                remaining -= c;
                c
            };
            if count > 0 {
                demands.push((skill.clone(), count));
            }
        }

        // Measure time
        let start = Instant::now();
        let result = BipartiteMatchingSolver.assign(&workers, &demands);
        let duration = start.elapsed();

        println!(
            " {:<7} | {:<7} | {:<7} | {:.3} ms",
            size,
            demand_size,
            result.cardinality,
            duration.as_secs_f64() * 1000.0
        );
    }
    println!("=======================================================\n");
}
